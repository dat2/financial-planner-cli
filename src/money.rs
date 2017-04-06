use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign, Neg};
use std::iter::Sum;
use std::convert::From;

use rugflo::Float;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Money {
    float: Float,
}

// money
impl Money {
    pub fn zero() -> Money {
        Money { float: Float::from((0, 64)) }
    }

    pub fn mul_percent(self, percentage: Float) -> Money {
        Money { float: self.float * percentage }
    }
}

impl From<f32> for Money {
    fn from(value: f32) -> Money {
        Money { float: Float::from((value, 64)) }
    }
}

impl From<f64> for Money {
    fn from(value: f64) -> Money {
        Money { float: Float::from((value, 64)) }
    }
}

impl From<u32> for Money {
    fn from(value: u32) -> Money {
        Money { float: Float::from((value, 64)) }
    }
}

impl From<i32> for Money {
    fn from(value: i32) -> Money {
        Money { float: Float::from((value, 64)) }
    }
}

// display
impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let precision = f.precision().unwrap_or(2);
        write!(f, "${:.*}", precision, self.float.to_f64())
    }
}

// math
impl Add for Money {
    type Output = Money;

    fn add(self, rhs: Money) -> Self::Output {
        Money { float: self.float + rhs.float }
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Money) {
        self.float += other.float;
    }
}

impl Sub for Money {
    type Output = Money;

    fn sub(self, rhs: Money) -> Self::Output {
        Money { float: self.float - rhs.float }
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, other: Money) {
        self.float -= other.float;
    }
}

impl Neg for Money {
    type Output = Money;

    fn neg(self) -> Self::Output {
        Money { float: -self.float }
    }
}

// iterators
impl Sum for Money {
    fn sum<I>(iter: I) -> Money
        where I: Iterator<Item = Money>
    {
        iter.fold(Money::zero(), Add::add)
    }
}

// serde
impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_f64(self.float.to_f64())
    }
}

struct MoneyVisitor;

impl de::Visitor for MoneyVisitor {
    type Value = Money;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("expected i32, u32, f32, f64")
    }

    // integer types
    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(Money::from(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where E: de::Error
    {
        use std::i32;
        if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
            Ok(Money::from(value as i32))
        } else {
            Err(E::custom(format!("i32 out of range: {}", value)))
        }
    }

    // unsigned integer types
    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(Money::from(value))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where E: de::Error
    {
        use std::u32;
        if value >= u32::MIN as u64 && value <= u32::MAX as u64 {
            Ok(Money::from(value as u32))
        } else {
            Err(E::custom(format!("u32 out of range: {}", value)))
        }
    }

    // float types
    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(Money::from(value))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(Money::from(value))
    }
}

impl Deserialize for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_f64(MoneyVisitor)
    }
}
