use std::{ffi::NulError, fmt::Error, io::ErrorKind};

use fixed::{types::extra::U50};
type FixedU64 = fixed::FixedU64<U50>; 
struct TokenAmount(FixedU64);
struct StakedTokenAmount(FixedU64);
struct LpTokenAmount(FixedU64);
struct Price(FixedU64);
struct Percentage(FixedU64);

struct LpPool {
    price: Price,
    token_amount: TokenAmount,
    st_token_amount: StakedTokenAmount,
    lp_token_amount: LpTokenAmount,
    liquidity_target: TokenAmount,
    fee_min: Percentage,
    fee_max: Percentage
}
impl LpPool {
    pub fn init(price: Price, fee_min: Percentage, fee_max: Percentage, liquidity_target: TokenAmount)->Result<Self, Error> {
        //min_max fix
        Ok(LpPool{
            price,
            token_amount: TokenAmount(0.into()),
            st_token_amount: StakedTokenAmount(0.into()),
            lp_token_amount: LpTokenAmount(0.into()),
            liquidity_target,
            fee_min,
            fee_max
        })
    }
    pub fn add_lqiuidity(&mut self, token_amount: TokenAmount)->Result<LpTokenAmount, Error> {
        Ok(LpTokenAmount(0.into()))
    }
}
fn main() {
    let lp_pool = LpPool::init(Price(0.into()), Percentage(1.into()), Percentage(2.into()), TokenAmount(0.into()));
    let mut lp_pool = lp_pool.unwrap();
    lp_pool.add_lqiuidity(TokenAmount(1.into())).unwrap();

    println!("Hello, world!");
}
