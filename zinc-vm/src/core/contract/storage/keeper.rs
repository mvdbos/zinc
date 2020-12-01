//!
//! The contract storage keeper trait.
//!

use num::BigInt;

use crate::error::Error;

///
/// The contract storage keeper trait.
///
pub trait IKeeper: Sync + Send {
    ///
    /// Fetches the contract storage instance by its ETH address.
    ///
    fn fetch(
        &self,
        eth_address: BigInt,
        field_types: Vec<zinc_types::ContractFieldType>,
    ) -> Result<zinc_types::Value, Error>;
}

///
/// The dummy keeper for places where loading is not implemented yet.
///
#[derive(Default)]
pub struct DummyKeeper {}

impl IKeeper for DummyKeeper {
    fn fetch(
        &self,
        _eth_address: BigInt,
        field_types: Vec<zinc_types::ContractFieldType>,
    ) -> Result<zinc_types::Value, Error> {
        Ok(zinc_types::Value::Contract(
            field_types
                .into_iter()
                .map(zinc_types::ContractFieldValue::new_from_type)
                .collect(),
        ))
    }
}
