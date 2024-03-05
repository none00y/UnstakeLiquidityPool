mod calc;
mod lp_pool;

use calc::fixed_dec_u64::fixed_u64_from_str_radix_10 as fixed_from_str;
use lp_pool::{LpPool, LpTokenAmount, Percentage, Price, StakedTokenAmount, TokenAmount};


fn main() {
    let mut lp_pool = LpPool::init(Price(fixed_from_str("1.5").unwrap()),
                               Percentage::try_from(fixed_from_str("0.001").unwrap()).unwrap(),
                               Percentage::try_from(fixed_from_str("0.09").unwrap()).unwrap(),
                               TokenAmount(fixed_from_str("90.").unwrap())
                            ).expect("Failed to create LiquidityPool");
    dbg!(lp_pool.add_liquidity(TokenAmount(fixed_from_str("100.").unwrap())).unwrap());
    dbg!(lp_pool.swap(StakedTokenAmount(fixed_from_str("6.").unwrap())).unwrap());
    dbg!(lp_pool.add_liquidity(TokenAmount(fixed_from_str("10.").unwrap())).unwrap());
    dbg!(lp_pool.swap(StakedTokenAmount(fixed_from_str("30.").unwrap())).unwrap());
    dbg!(lp_pool.remove_liquidity(LpTokenAmount(fixed_from_str("109.9991").unwrap())).unwrap());
}