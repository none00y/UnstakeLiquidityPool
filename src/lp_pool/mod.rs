use std::fmt;

pub use crate::calc::percentage::Percentage;
use crate::calc::fixed_dec_u64::{proportional, multiply, write_fixed_u64};

#[derive(Copy, Clone)]
pub struct TokenAmount(pub u64);
#[derive(Copy, Clone)]
pub struct StakedTokenAmount(pub u64);
#[derive(Copy, Clone)]
pub struct LpTokenAmount(pub u64);
#[derive(Copy, Clone)]
pub struct Price(pub u64);

impl fmt::Debug for TokenAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>)->fmt::Result {
        write_fixed_u64(self.0, f)
    }
}
impl fmt::Debug for StakedTokenAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>)->fmt::Result {
        write_fixed_u64(self.0, f)
    }
}
impl fmt::Debug for LpTokenAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>)->fmt::Result {
        write_fixed_u64(self.0, f)
    }
}
impl fmt::Debug for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>)->fmt::Result {
        write_fixed_u64(self.0, f)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum LpPoolError {
    FeeMaxLowerThanFeeMin{max: Percentage, min: Percentage},
    ExchangePriceIsZero,
    CalculationError,
    ValueTooLarge {val: u64, max: u64}
}

#[derive(Debug)]
pub struct LpPool {
    price: Price,
    token_amount: TokenAmount,
    st_token_amount: StakedTokenAmount,
    lp_token_amount: LpTokenAmount,
    liquidity_target: TokenAmount,
    fee_min: Percentage,
    fee_max: Percentage
}

impl LpPool {
    pub fn init(price: Price, fee_min: Percentage, fee_max: Percentage, liquidity_target: TokenAmount)->Result<Self, LpPoolError> {
        if fee_max < fee_min { Err(LpPoolError::FeeMaxLowerThanFeeMin{max: fee_max, min: fee_min})? }
        if price.0 == 0 { Err(LpPoolError::ExchangePriceIsZero)? }

        Ok(LpPool{
            price,
            token_amount: TokenAmount(0),
            st_token_amount: StakedTokenAmount(0),
            lp_token_amount: LpTokenAmount(0),
            liquidity_target,
            fee_min,
            fee_max
        })
    }
    pub fn add_liquidity(&mut self, token_amount: TokenAmount)->Result<LpTokenAmount, LpPoolError> {
        let minted_tokens = if self.current_liquidity()?.0 == 0 {
            LpTokenAmount(token_amount.0)
        } else {
            LpTokenAmount(proportional(self.lp_token_amount.0,
                token_amount.0,
                self.token_amount.0.checked_add(multiply(self.st_token_amount.0, self.price.0)?).ok_or(LpPoolError::CalculationError)?)?)
        };
        let new_token_amount = TokenAmount(self.token_amount.0.checked_add(token_amount.0).ok_or(LpPoolError::CalculationError)?);
        let new_lp_token_amount = LpTokenAmount(self.lp_token_amount.0.checked_add(minted_tokens.0).ok_or(LpPoolError::CalculationError)?);
        self.token_amount = new_token_amount;
        self.lp_token_amount = new_lp_token_amount;
        Ok(minted_tokens)
    }
    pub fn swap(&mut self, st_token_amount: StakedTokenAmount)->Result<TokenAmount, LpPoolError> {
        let token_amount = TokenAmount(multiply(st_token_amount.0, self.price.0)?);
        if token_amount.0 > self.token_amount.0 {};
        let fee_tokens_amount = TokenAmount(multiply(self.fee(token_amount)?.bits(), token_amount.0)?) ;
        let token_amount_to_swap = TokenAmount(token_amount.0.checked_sub(fee_tokens_amount.0).ok_or(LpPoolError::CalculationError)?);
        
        let new_token_amount = TokenAmount(self.token_amount.0.checked_sub(token_amount_to_swap.0).ok_or(LpPoolError::CalculationError)?);
        let new_st_token_amount = StakedTokenAmount(self.st_token_amount.0.checked_add(st_token_amount.0).ok_or(LpPoolError::CalculationError)?);
        self.token_amount = new_token_amount;
        self.st_token_amount = new_st_token_amount;
        Ok(token_amount_to_swap)
    }
    pub fn remove_liquidity(&mut self, lp_token_amount: LpTokenAmount)->Result<(StakedTokenAmount, TokenAmount), LpPoolError> {
        if lp_token_amount.0 > self.lp_token_amount.0 {Err(LpPoolError::ValueTooLarge{val: lp_token_amount.0, max: self.lp_token_amount.0})?}

        let st_token_amount = StakedTokenAmount(proportional(self.st_token_amount.0,
                                                                                lp_token_amount.0,
                                                                                self.lp_token_amount.0)?);
        let token_amount = TokenAmount(proportional(self.token_amount.0,
                                                                 lp_token_amount.0,
                                                                 self.lp_token_amount.0)?);
        let new_token_amount = TokenAmount(self.token_amount.0.checked_sub(token_amount.0).ok_or(LpPoolError::CalculationError)?);
        let new_st_token_amount = StakedTokenAmount(self.st_token_amount.0.checked_sub(st_token_amount.0).ok_or(LpPoolError::CalculationError)?);
        self.st_token_amount = new_st_token_amount;
        self.token_amount = new_token_amount;
        Ok((st_token_amount, token_amount))
    }
    fn fee(&self, taken_token_amount: TokenAmount)->Result<Percentage, LpPoolError> {
        if self.liquidity_target.0 < self.token_amount.0.checked_sub(taken_token_amount.0).ok_or(LpPoolError::CalculationError)? {
            return Ok(self.fee_min)
        }
        let fee = self.fee_max.bits().checked_sub(proportional(self.fee_delta()?.bits(),
                                                         self.token_amount.0.checked_sub(taken_token_amount.0).ok_or(LpPoolError::CalculationError)?,
                                                         self.liquidity_target.0).unwrap()).ok_or(LpPoolError::CalculationError)?;
        Percentage::try_from(fee)
    }
    fn current_liquidity(&self)->Result<TokenAmount, LpPoolError>{
        let liquidity = 
            self.token_amount.0.checked_add(multiply(self.st_token_amount.0, self.price.0)?)
                                            .ok_or(LpPoolError::CalculationError);
        
        Ok(TokenAmount(liquidity?))
    }
    fn fee_delta(&self)->Result<Percentage, LpPoolError> {
        Percentage::try_from(self.fee_max.bits() - self.fee_min.bits())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::calc::fixed_dec_u64::multiply;
    use crate::calc::fixed_dec_u64::fixed_u64_from_str_radix_10;
    fn seeded_pool()->LpPool {
        let price =Price(fixed_u64_from_str_radix_10("1.5").unwrap());
        let fee_min = Percentage::try_from(fixed_u64_from_str_radix_10("0.001").unwrap()).unwrap();
        let fee_max = Percentage::try_from(fixed_u64_from_str_radix_10("0.09").unwrap()).unwrap();
        let token_amount = TokenAmount(fixed_u64_from_str_radix_10("90.").unwrap());
        LpPool::init(price, fee_min, fee_max, token_amount).unwrap()
    }
    #[test]
    fn works() {
        let mut lp_pool = seeded_pool();
        let first_lp_tokens = lp_pool.add_liquidity(TokenAmount(fixed_u64_from_str_radix_10("100.").unwrap())).unwrap();
        assert_eq!(first_lp_tokens.0, fixed_u64_from_str_radix_10("100.").unwrap());
        let first_tokens = lp_pool.swap(StakedTokenAmount(fixed_u64_from_str_radix_10("6.").unwrap())).unwrap();
        assert_eq!(first_tokens.0, fixed_u64_from_str_radix_10("8.991").unwrap());
        let second_lp_tokens = lp_pool.add_liquidity(TokenAmount(fixed_u64_from_str_radix_10("10.").unwrap())).unwrap();
        assert_eq!(second_lp_tokens.0, fixed_u64_from_str_radix_10("9.9991").unwrap());
        let second_tokens = lp_pool.swap(StakedTokenAmount(fixed_u64_from_str_radix_10("30.").unwrap())).unwrap();
        assert_eq!(second_tokens.0, fixed_u64_from_str_radix_10("43.44237").unwrap());
        let (first_staked_tokens, third_tokens) = lp_pool.remove_liquidity(LpTokenAmount(fixed_u64_from_str_radix_10("109.9991").unwrap())).unwrap();
        assert_eq!(third_tokens.0, fixed_u64_from_str_radix_10("57.56663").unwrap());
        assert_eq!(first_staked_tokens.0, fixed_u64_from_str_radix_10("36.").unwrap());
    }
    #[test]
    fn init_works() {
        let price =Price(fixed_u64_from_str_radix_10("1.5").unwrap());
        let fee_min = Percentage::try_from(fixed_u64_from_str_radix_10("0.001").unwrap()).unwrap();
        let fee_max = Percentage::try_from(fixed_u64_from_str_radix_10("0.09").unwrap()).unwrap();
        let token_amount = TokenAmount(fixed_u64_from_str_radix_10("90.").unwrap());
        let _ = LpPool::init(price, fee_min, fee_max, token_amount).unwrap();
    }
    #[test]
    fn init_fails_with_incorrect_fee() {
        let price =Price(fixed_u64_from_str_radix_10("1.5").unwrap());

        let fee_min = Percentage::try_from(fixed_u64_from_str_radix_10("0.091").unwrap()).unwrap();
        let fee_max = Percentage::try_from(fixed_u64_from_str_radix_10("0.090").unwrap()).unwrap();

        let token_amount = TokenAmount(fixed_u64_from_str_radix_10("90.").unwrap());
        let lp_pool = LpPool::init(price, fee_min, fee_max, token_amount);
        assert_eq!(lp_pool.unwrap_err(), LpPoolError::FeeMaxLowerThanFeeMin{max: fee_max, min: fee_min})
    }
    #[test]
    fn init_fails_with_incorrect_price() {
        let price = Price(fixed_u64_from_str_radix_10("0.").unwrap());

        let fee_min = Percentage::try_from(fixed_u64_from_str_radix_10("0.090").unwrap()).unwrap();
        let fee_max = Percentage::try_from(fixed_u64_from_str_radix_10("0.091").unwrap()).unwrap();

        let token_amount = TokenAmount(fixed_u64_from_str_radix_10("90.").unwrap());
        let lp_pool = LpPool::init(price, fee_min, fee_max, token_amount);
        assert_eq!(lp_pool.unwrap_err(), LpPoolError::ExchangePriceIsZero)
    }
    #[test]
    fn add_lqiuidity_works() {
        let mut lp_pool = seeded_pool();
        let token_amount = fixed_u64_from_str_radix_10("100.").unwrap();
        let additional_tokens = fixed_u64_from_str_radix_10("10.").unwrap();
        assert_eq!(lp_pool.add_liquidity(TokenAmount(token_amount)).unwrap().0, token_amount);
        let result = proportional(token_amount, additional_tokens, token_amount).unwrap();
        assert_eq!(lp_pool.add_liquidity(TokenAmount(additional_tokens)).unwrap().0, result);
    }
    #[test]
    fn add_lqiuidity_fails() {
        let mut lp_pool = seeded_pool();
        let token_amount = u64::MAX;
        let additional_tokens = fixed_u64_from_str_radix_10("10.").unwrap();
        assert_eq!(lp_pool.add_liquidity(TokenAmount(token_amount)).unwrap().0, token_amount);
        assert_eq!(lp_pool.add_liquidity(TokenAmount(additional_tokens)).unwrap_err(), LpPoolError::CalculationError);
    }
    #[test]
    fn swap_works() {
        let mut lp_pool = seeded_pool();
        let token_amount = fixed_u64_from_str_radix_10("90.").unwrap();
        lp_pool.add_liquidity(TokenAmount(token_amount)).unwrap();

        let st_token_amount = fixed_u64_from_str_radix_10("30.").unwrap();
        let fee = fixed_u64_from_str_radix_10("0.0455").unwrap();
        let st_token_amount_taken_by_fee = multiply(fee, st_token_amount).unwrap();
        let expected_token_amount = multiply(st_token_amount - st_token_amount_taken_by_fee, lp_pool.price.0).unwrap();
        assert_eq!(lp_pool.swap(StakedTokenAmount(st_token_amount)).unwrap().0, expected_token_amount);
    }
    #[test]
    fn swap_fails() {
        let mut pool = seeded_pool();
        assert_eq!(pool.swap(StakedTokenAmount(1)).unwrap_err(), LpPoolError::CalculationError);
    }
    #[test]
    fn remove_liquidity_works() {
        let mut lp_pool = seeded_pool();
        let token_amount = fixed_u64_from_str_radix_10("90.").unwrap();
        lp_pool.add_liquidity(TokenAmount(token_amount)).unwrap();

        let st_token_amount = fixed_u64_from_str_radix_10("30.").unwrap();
        let fee = fixed_u64_from_str_radix_10("0.0455").unwrap();
        let st_token_amount_taken_by_fee = multiply(fee, st_token_amount).unwrap();
        let expected_token_amount = multiply(st_token_amount - st_token_amount_taken_by_fee, lp_pool.price.0).unwrap();
        lp_pool.swap(StakedTokenAmount(st_token_amount)).unwrap();
        let (removed_st_token_amount, removed_token_amount) =
            lp_pool.remove_liquidity(LpTokenAmount(fixed_u64_from_str_radix_10("90.").unwrap())).unwrap();
        assert_eq!(removed_token_amount.0, token_amount - expected_token_amount);
        assert_eq!(removed_st_token_amount.0, st_token_amount);
    }
    #[test]
    fn remove_liquidity_fails() {
        let mut lp_pool = seeded_pool();
        let token_amount = fixed_u64_from_str_radix_10("90.").unwrap();
        lp_pool.add_liquidity(TokenAmount(token_amount)).unwrap();

        let st_token_amount = fixed_u64_from_str_radix_10("30.").unwrap();
        lp_pool.swap(StakedTokenAmount(st_token_amount)).unwrap();
        let removed_lp_tokens = fixed_u64_from_str_radix_10("91.").unwrap();
        assert_eq!(lp_pool.remove_liquidity(LpTokenAmount(removed_lp_tokens)).unwrap_err(),
                    LpPoolError::ValueTooLarge { val: removed_lp_tokens, max: token_amount });
    }
    #[test]
    fn fee_works() {
        let mut lp_pool = seeded_pool();
        let token_amount = fixed_u64_from_str_radix_10("90.").unwrap();
        let half_token_amount = token_amount/2;
        assert_eq!(lp_pool.add_liquidity(TokenAmount(token_amount)).unwrap().0, token_amount);
        assert_eq!(lp_pool.fee(TokenAmount(0)).unwrap().bits(), fixed_u64_from_str_radix_10("0.001").unwrap());
        assert_eq!(lp_pool.fee(TokenAmount(half_token_amount)).unwrap().bits(), fixed_u64_from_str_radix_10("0.0455").unwrap());
        assert_eq!(lp_pool.fee(TokenAmount(token_amount)).unwrap().bits(), fixed_u64_from_str_radix_10("0.09").unwrap());
    }
}
