use ff::Field;
use franklin_crypto::bellman::ConstraintSystem;
use franklin_crypto::circuit::Assignment;

use crate::error::RuntimeError;
use crate::gadgets::scalar::Scalar;
use crate::IEngine;

pub fn assert<E, CS>(
    mut cs: CS,
    element: Scalar<E>,
    message: Option<&str>,
) -> Result<(), RuntimeError>
where
    E: IEngine,
    CS: ConstraintSystem<E>,
{
    if let Some(value) = element.get_value() {
        if value.is_zero() {
            let s = message.unwrap_or("<no message>");
            return Err(RuntimeError::AssertionError(s.into()));
        }
    }

    let inverse_value = element
        .get_value()
        .map(|fr| fr.inverse().unwrap_or_else(E::Fr::zero));

    let inverse_variable = cs
        .alloc(|| "inverse", || inverse_value.grab())
        .map_err(RuntimeError::SynthesisError)?;

    cs.enforce(
        || "assertion",
        |lc| lc + &element.lc::<CS>(),
        |lc| lc + inverse_variable,
        |lc| lc + CS::one(),
    );

    Ok(())
}
