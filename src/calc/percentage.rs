use core::fmt;

use crate::calc::fixed_dec_u64::{FIXED_PRECISION, write_fixed_u64};
use crate::lp_pool::LpPoolError;
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Percentage(u64);
impl fmt::Debug for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>)->fmt::Result {
        write_fixed_u64(self.0, f)
    }
}

impl Percentage {
    const MAX: u64 = FIXED_PRECISION * 100;
    pub fn bits(&self)->u64 {
        self.0
    }
    pub fn check(&self)->Result<&Self,LpPoolError> {
        if (Self::MAX as u128 * 100 as u128) <= self.bits() as u128 {Err(LpPoolError::ValueTooLarge{val: self.bits(), max: Self::MAX})?}
        
        Ok(self)
    }
}
impl TryFrom<u64> for Percentage {
    type Error = LpPoolError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let percentage = Percentage(value);
        Ok(*percentage.check()?)
    }
}