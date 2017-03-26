use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
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
    pub fn new<T: Into<Float>>(value: T) -> Money {
        Money { float: value.into() }
    }

    pub fn zero() -> Money {
        Money { float: Float::from((0, 64)) }
    }

    // TODO multiply by percentage
    pub fn mul_percentage(&mut self, percentage: f64) {
        self.float *= Float::from((percentage, 64));
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
        Money::new(self.float + rhs.float)
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
        Money::new(self.float - rhs.float)
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, other: Money) {
        self.float -= other.float;
    }
}

// conversion
impl From<f64> for Money {
    fn from(t: f64) -> Money {
        Money::new((t, 64))
    }
}

// iterators
impl Sum for Money {
    fn sum<I>(iter: I) -> Money
        where I: Iterator<Item=Money>
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
        Ok(Money::new((value, 64)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where E: de::Error
    {
        use std::i32;
        if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
            Ok(Money::new((value as i32, 64)))
        } else {
            Err(E::custom(format!("i32 out of range: {}", value)))
        }
    }

    // unsigned integer types
    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(Money::new((value, 64)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where E: de::Error
    {
        use std::u32;
        if value >= u32::MIN as u64 && value <= u32::MAX as u64 {
            Ok(Money::new((value as u32, 64)))
        } else {
            Err(E::custom(format!("u32 out of range: {}", value)))
        }
    }

    // float types
    fn visit_f32<E>(self, value: f32) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(Money::new((value, 64)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(Money::new((value, 64)))
    }
}

impl Deserialize for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_f64(MoneyVisitor)
    }
}
