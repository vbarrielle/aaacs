use std::error::Error;
use std::num::ParseIntError;

use num_rational::Rational64;

#[derive(Debug, Clone)]
pub enum ParseRationalError {
    NumerError(ParseIntError),
    DenomError(ParseIntError),
    EmptyString,
}

impl std::fmt::Display for ParseRationalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseRationalError::NumerError(error) => {
                write!(f, "Error parsing numerator: {}.", error)
            }
            ParseRationalError::DenomError(error) => {
                write!(f, "Error parsing denominator: {}.", error)
            }
            ParseRationalError::EmptyString => {
                write!(f, "Could not parse empty string as a rational.")
            }
        }
    }
}

impl Error for ParseRationalError {}

/// Deserialize a rational number from its string representation
///
/// The string representation of a rational is `<integral_part>.<decimal_part>`
pub fn rational_from_str(
    rat_str: &str,
) -> Result<Rational64, ParseRationalError> {
    let mut parts_iter = rat_str.trim().split('.');
    let integral_part =
        parts_iter.next().ok_or(ParseRationalError::EmptyString)?;
    let integral_part: i64 = integral_part
        .parse()
        .map_err(|e| ParseRationalError::NumerError(e))?;
    let integral_part = Rational64::new(integral_part, 1);
    if let Some(decimal_part) = parts_iter.next() {
        let nb_decimals = decimal_part.len();
        Ok(integral_part
            + Rational64::new(
                decimal_part
                    .parse()
                    .map_err(|e| ParseRationalError::DenomError(e))?,
                10_i64.pow(nb_decimals as u32),
            ))
    } else {
        Ok(integral_part)
    }
}

/// Convert a rational number to a decimal string representation. Rounding
/// is performed to the closest decimal number with the required number of
/// decimals.
pub fn rational_to_string(rat: Rational64, nb_max_decimals: u8) -> String {
    let rat = rat.reduced();
    if rat.is_integer() {
        format!("{}", rat.to_integer())
    } else {
        let integral_part = rat.trunc();
        let mut fract_part = rat.fract();
        if fract_part < Rational64::new(0, 1) {
            fract_part = -fract_part;
        };
        // The fractional part is a fraction m / n where 0 < m < n.
        // We want to compute a decimal approximation, ie find two integers
        // p and q such that p / 10^q < m / n and the difference is minimal,
        // under the constraint that q <= nb_max_decimals.
        let mut q = nb_max_decimals;
        let mut p = (*fract_part.numer() as f64 * 10.0_f64.powi(q as i32)
            / *fract_part.denom() as f64)
            .round() as i64;
        while p % 10 == 0 {
            q -= 1;
            p = p / 10;
        }
        let integral_part = integral_part.to_integer();
        format!(
            "{integral_part}.{decimal_part:0width$}",
            integral_part = integral_part,
            decimal_part = p,
            width = q as usize,
        )
    }
}

#[cfg(test)]
mod test {
    use super::Rational64;

    #[test]
    fn rational_from_str() {
        assert_eq!(
            super::rational_from_str(&"10").unwrap(),
            Rational64::new(10, 1),
        );
        assert_eq!(
            super::rational_from_str(&"10.5").unwrap(),
            Rational64::new(105, 10),
        );
        assert_eq!(
            super::rational_from_str(&"0.5").unwrap(),
            Rational64::new(5, 10),
        );
        assert_eq!(
            super::rational_from_str(&"3.00523").unwrap(),
            Rational64::new(300523, 100000),
        );
    }

    #[test]
    fn rational_to_string() {
        let nb_max_decimals = 4;
        assert_eq!(
            &super::rational_to_string(Rational64::new(10, 1), nb_max_decimals),
            &"10",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(105, 10),
                nb_max_decimals
            ),
            &"10.5",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(105, 100),
                nb_max_decimals
            ),
            &"1.05",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(-105, 100),
                nb_max_decimals
            ),
            &"-1.05",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(99, 30),
                nb_max_decimals
            ),
            &"3.3",
        );
        assert_eq!(
            &super::rational_to_string(
                Rational64::new(99, 29),
                nb_max_decimals
            ),
            &"3.4138",
        );
    }
}
