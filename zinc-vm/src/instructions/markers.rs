//!
//! The `*Marker` instructions. TODO
//!

use zinc_bytecode::ColumnMarker;
use zinc_bytecode::FileMarker;
use zinc_bytecode::FunctionMarker;
use zinc_bytecode::LineMarker;

use crate::core::location::Location;
use crate::core::virtual_machine::IVirtualMachine;
use crate::error::RuntimeError;
use crate::instructions::IExecutable;

impl<VM: IVirtualMachine> IExecutable<VM> for FileMarker {
    fn execute(self, vm: &mut VM) -> Result<(), RuntimeError> {
        vm.set_location(Location {
            file: Some(self.file),
            function: None,
            line: None,
            column: None,
        });

        Ok(())
    }
}

impl<VM: IVirtualMachine> IExecutable<VM> for FunctionMarker {
    fn execute(self, vm: &mut VM) -> Result<(), RuntimeError> {
        let mut location = vm.get_location();
        location.function = Some(self.function);
        vm.set_location(location);
        Ok(())
    }
}

impl<VM: IVirtualMachine> IExecutable<VM> for LineMarker {
    fn execute(self, vm: &mut VM) -> Result<(), RuntimeError> {
        let mut location = vm.get_location();
        location.line = Some(self.line);
        vm.set_location(location);
        Ok(())
    }
}

impl<VM: IVirtualMachine> IExecutable<VM> for ColumnMarker {
    fn execute(self, vm: &mut VM) -> Result<(), RuntimeError> {
        let mut location = vm.get_location();
        location.column = Some(self.column);
        vm.set_location(location);
        Ok(())
    }
}
