use std::fmt::{self, Debug};

use crate::lp_pool::LpPoolError;
pub const FIXED_PRECISION: u64 = 1_000_000;
#[derive(Debug)]
pub enum FixedU64Error {
    MissingDelimeter,
    IncorrectIntegerPart,
    IncorrectFractionalPart(usize, char),
}

pub fn fixed_u64_from_str_radix_10(str: &str)->Result<u64, FixedU64Error> {
    match str.split_once('.') {
        None => Err(FixedU64Error::MissingDelimeter),
        Some((int, frac)) => {
            let int_val = u64::from_str_radix(int, 10).map_err(|_| FixedU64Error::IncorrectIntegerPart)?;
            let mut frac_val = 0;
            for (i,c) in frac.chars().enumerate() {
                frac_val += c.to_digit(10).ok_or(FixedU64Error::IncorrectFractionalPart(i,c))? as u64 *
                            (FIXED_PRECISION/10u64.pow(i as u32 + 1));
            }
            Ok(int_val*FIXED_PRECISION + frac_val)
        }
    }
}
pub fn write_fixed_u64(val: u64, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f,
           "{}.{}",
            val/FIXED_PRECISION,
            val%FIXED_PRECISION
        )
} 

pub fn proportional(amount: u64, numerator: u64, denominator: u64) -> Result<u64, LpPoolError> {
    if denominator == 0 {
        return Ok(amount);
    }
    u64::try_from((amount as u128) * (numerator as u128) / (denominator as u128)).map_err(|_| LpPoolError::CalculationError)
}
pub fn multiply(a: u64, b: u64)->Result<u64, LpPoolError> {
    u64::try_from((a as u128 * b as u128)/FIXED_PRECISION as u128).map_err(|_| LpPoolError::CalculationError)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test] 
    fn parse_from_string() {
        let nine = 9 * FIXED_PRECISION;
        assert_eq!(fixed_u64_from_str_radix_10("9.").unwrap(), nine);
        let zero_point_nine = 9 * FIXED_PRECISION/10;
        assert_eq!(fixed_u64_from_str_radix_10("0.9").unwrap(), zero_point_nine);
        let zero_point_three_nines_one = 9991 * FIXED_PRECISION/10000;
        assert_eq!(fixed_u64_from_str_radix_10("0.9991").unwrap(), zero_point_three_nines_one);
    
    }
    #[test] 
    fn proportional_works() {
        let numerator = fixed_u64_from_str_radix_10("10.").unwrap();
        let denominator = fixed_u64_from_str_radix_10("90.009").unwrap() + numerator;         
        let amount = fixed_u64_from_str_radix_10("100.").unwrap();
        let res = proportional(amount, numerator, denominator);
        assert_eq!(res.unwrap(), fixed_u64_from_str_radix_10("9.9991").unwrap());
    }

    #[test] 
    fn proportional_fails() {
        let numerator = u64::MAX;
        let denominator = fixed_u64_from_str_radix_10("90.009").unwrap();         
        let amount = u64::MAX;
        let res = proportional(amount, numerator, denominator);
        assert_eq!(res.unwrap_err(), LpPoolError::CalculationError);
    }

    #[test] 
    fn multiply_fails() {
        assert_eq!(multiply(u64::MAX, u64::MAX).unwrap_err(), LpPoolError::CalculationError);
    }
    #[test] 
    fn multiply_works() {
        assert_eq!(multiply(fixed_u64_from_str_radix_10("0.1").unwrap(), fixed_u64_from_str_radix_10("0.1").unwrap()).unwrap(),
                   fixed_u64_from_str_radix_10("0.01").unwrap());
    }
}
