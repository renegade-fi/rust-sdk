//! Fixed point implementation for the external match client
//!
//! This implementation internalizes a subset of the functionality available to
//! the fixed point type, to avoid depending on the relayer crates

use std::fmt::{self, Display};

use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Serialize};

use super::order_types::Amount;

/// The number of bits to use for the fixed point precision
const FIXED_POINT_PRECISION_BITS: u64 = 63;
/// The fixed point precision shift value
const FIXED_POINT_PRECISION_SHIFT: u64 = 1u64 << FIXED_POINT_PRECISION_BITS;
/// Get a `BigUint` representing the fixed point precision shift value
fn fixed_point_precision_shift() -> BigUint {
    BigUint::from(FIXED_POINT_PRECISION_SHIFT)
}

/// A fixed point number
#[derive(Clone, Debug)]
pub struct FixedPoint {
    /// The value of the fixed point number
    pub value: BigUint,
}

impl FixedPoint {
    /// Create a new fixed point number
    pub fn new(value: BigUint) -> Self {
        Self { value }
    }

    /// Multiply a fixed point number by an `Amount` and return the floor
    pub fn floor_mul_int(&self, amount: Amount) -> Amount {
        let product = self.value.clone() * amount;
        let floor = product / fixed_point_precision_shift();
        floor.try_into().expect("fixed point overflow")
    }

    /// Convert a fixed point number to an `f64`
    pub fn to_f64(&self) -> f64 {
        let value_bigdec = BigDecimal::from_biguint(self.value.clone(), 0);
        let result = &value_bigdec / FIXED_POINT_PRECISION_SHIFT;
        result.to_f64().expect("fixed point overflow")
    }
}

impl Display for FixedPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_f64())
    }
}

// Serialize and deserialize using a string representation as is done in the
// relayer api
impl Serialize for FixedPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.value.to_str_radix(10 /* radix */).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FixedPoint {
    fn deserialize<D>(deserializer: D) -> Result<FixedPoint, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let decimal_string = String::deserialize(deserializer)?;
        let value = BigUint::from_str_radix(&decimal_string, 10 /* radix */)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;

        Ok(Self { value })
    }
}
