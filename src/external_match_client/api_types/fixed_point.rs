//! Fixed point implementation for the external match client
//!
//! This implementation internalizes a subset of the functionality available to
//! the fixed point type, to avoid depending on the relayer crates

use num_bigint::BigUint;
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
}
