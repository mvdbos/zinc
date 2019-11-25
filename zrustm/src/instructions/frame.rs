extern crate franklin_crypto;

use crate::element::{Element, ElementOperator};
use crate::vm::VMInstruction;
use crate::vm::{RuntimeError, VirtualMachine};
use zrust_bytecode::{FrameBegin, FrameEnd};

impl<E, O> VMInstruction<E, O> for FrameBegin
where
    E: Element,
    O: ElementOperator<E>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, O>) -> Result<(), RuntimeError> {
        vm.frame_push(None)
    }
}

impl<E, O> VMInstruction<E, O> for FrameEnd
where
    E: Element,
    O: ElementOperator<E>,
{
    fn execute(&self, vm: &mut VirtualMachine<E, O>) -> Result<(), RuntimeError> {
        vm.frame_pop(self.outputs_count)
    }
}

#[cfg(test)]
mod tests {
    use crate::instructions::testing_utils::{TestingError, VMTestRunner};
    use zrust_bytecode::*;

    #[test]
    fn test_frame() -> Result<(), TestingError> {
        let _ = env_logger::builder().is_test(true).try_init();

        VMTestRunner::new()
            .add(Push { value: 3.into() })
            .add(Push { value: 5.into() })
            .add(FrameBegin)
            .add(Push { value: 1.into() })
            .add(Push { value: 100.into() })
            .add(Push { value: 200.into() })
            .add(FrameEnd::new(2))
            .add(FrameBegin)
            .add(Copy::new(3))
            .add(Push { value: 1.into() })
            .add(Add)
            .add(FrameEnd::new(1))
            .test(&[201, 200, 100, 5, 3])
    }
}
