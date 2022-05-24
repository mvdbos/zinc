use bellman::ConstraintSystem;
use franklin_crypto::circuit::blake2s::blake2s;

use crate::core::EvaluationStack;
use crate::gadgets::Scalar;
use crate::stdlib::NativeFunction;
use crate::{Engine, MalformedBytecode, Result};

const BYTE_LENGTH: usize = 8;

pub struct Blake2sMultiInput {
    message_length: usize,
}

impl Blake2sMultiInput {
    pub fn new(message_length: usize) -> Result<Self> {
        if message_length % 8 == 0 {
            Ok(Self { message_length })
        } else {
            Err(MalformedBytecode::InvalidArguments(format!(
                "message length for blake2s must be a multiple of 8, got {}",
                message_length
            ))
            .into())
        }
    }
}

// Implementation of Blake2s multi-input gadget for Zinc.
// It uses blake2s implementation of the franklin_crypto library.

// IMPORTANT NOTE ABOUT THE GADGET:
// In ZKFlow, we generally use Blake2s hash to compute hash of two concatenated messages
// such as Hash(nonce || serialized_component). Thus, in circuit computations, to eliminate
// additional concatenation operations we designed blake2s_multi_input gadget, which handles
// concatenation under the hood. The gadget expects exactly two messages as input.  

// Similar to Blake2s gadget, the multi-input gadget also implements reverse_byte_bits()
// to assure correct ordering of the bits within each byte. Please check Blake2s gadget
// to learn more about this operation.

impl<E: Engine> NativeFunction<E> for Blake2sMultiInput {
    fn execute<CS: ConstraintSystem<E>>(
        &self,
        mut cs: CS,
        stack: &mut EvaluationStack<E>,
    ) -> Result {
        let mut bits = Vec::new();
        for i in 0..self.message_length {
            let bit = stack
                .pop()?
                .value()?
                .to_boolean(cs.namespace(|| format!("bit {}", i)))?;

            bits.push(bit);
        }
        bits.reverse();

        // This function reverses the bit order within each byte of the parameter: a list of bits
        let reverse_byte_bits =
            |input: &mut [_]| input.chunks_mut(BYTE_LENGTH).for_each(|p| p.reverse());

        //reverse preimage for compatibility with the original spec
        reverse_byte_bits(&mut bits);

        let mut digest_bits = blake2s(cs.namespace(|| "blake2s"), &bits, b"12345678")?;

        //reverse digest for compatibility with the original spec
        reverse_byte_bits(&mut digest_bits);

        assert_eq!(digest_bits.len(), 256);

        for bit in digest_bits {
            let scalar = Scalar::from_boolean(cs.namespace(|| "from_boolean"), bit)?;
            stack.push(scalar.into())?;
        }

        Ok(())
    }
}
