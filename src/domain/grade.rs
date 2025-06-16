use rust_decimal::Decimal;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

use crate::app::{
    localization::LocalizedError,
    validation::{Validation, ValidationFailure},
};

const MIN_INTEGER: i64 = 0;
const MAX_INTEGER: i64 = 100;
const FRACTION_DIGITS: u32 = 2;

const MIN_FRACTION: i64 = 0;
const MAX_FRACTION: i64 = 10i64.pow(FRACTION_DIGITS) - 1;

#[serde_as]
#[derive(Debug, Clone, Copy, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct Grade(#[serde_as(as = "DisplayFromStr")] Decimal);

impl Grade {
    pub fn new(grade: String) -> Result<Self, ValidationFailure<String>> {
        match Self::parse(&grade) {
            Some(grade) => Ok(grade),
            None => Validation::new(grade)
                .error(
                    LocalizedError::new("GRADE_INVALID_FORMAT")
                        .with_number("min", MIN_INTEGER as f64)
                        .with_number("max", MAX_INTEGER as f64)
                        .with_number("fractionDigits", FRACTION_DIGITS as f64),
                )
                .finish()
                .map(|_| unreachable!()),
        }
    }

    fn parse(grade: &str) -> Option<Self> {
        let (integer, fraction) = match grade.split_once('.') {
            Some((integer, fraction)) => {
                let integer = integer.parse::<i64>();
                let fraction = if fraction.len() == 1 {
                    fraction.parse::<i64>().map(|x| x * 10)
                } else {
                    fraction.parse::<i64>()
                };
                integer.and_then(|integer| fraction.map(|fraction| (integer, fraction)))
            }
            None => {
                let integer = grade.parse::<i64>();
                integer.map(|integer| (integer, 0))
            }
        }
        .ok()?;
        let (integer, fraction) = match (integer, fraction) {
            (integer @ MAX_INTEGER, fraction @ MIN_FRACTION)
            | (integer @ MIN_INTEGER..MAX_INTEGER, fraction @ MIN_FRACTION..=MAX_FRACTION) => {
                Some((integer, fraction))
            }
            _ => None,
        }?;
        let shifted = integer * 10i64.pow(FRACTION_DIGITS) + fraction;
        let decimal = Decimal::new(shifted, FRACTION_DIGITS);
        Some(Self(decimal))
    }
}

impl From<Grade> for Decimal {
    fn from(value: Grade) -> Self {
        value.0
    }
}

impl TryFrom<String> for Grade {
    type Error = ValidationFailure<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
