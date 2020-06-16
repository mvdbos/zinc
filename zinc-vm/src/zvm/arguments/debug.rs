//!
//! The Zinc virtual machine binary `debug` command.
//!

use std::fs;
use std::path::PathBuf;

use structopt::StructOpt;

use franklin_crypto::bellman::pairing::bn256::Bn256;

use zinc_bytecode::Program as BytecodeProgram;
use zinc_bytecode::TemplateValue;

use zinc_vm::IFacade;

use crate::error::Error;
use crate::error::IoToError;

#[derive(Debug, StructOpt)]
#[structopt(name = "debug", about = "Executes circuit with additional checks")]
pub struct DebugCommand {
    #[structopt(long = "binary", help = "The bytecode file")]
    pub binary_path: PathBuf,

    #[structopt(long = "witness", help = "The witness JSON file")]
    pub witness_path: PathBuf,

    #[structopt(long = "public-data", help = "The public data JSON file")]
    pub public_data_path: PathBuf,
}

impl DebugCommand {
    pub fn execute(&self) -> Result<(), Error> {
        let bytes =
            fs::read(&self.binary_path).error_with_path(|| self.binary_path.to_string_lossy())?;
        let program =
            BytecodeProgram::from_bytes(bytes.as_slice()).map_err(Error::ProgramDecoding)?;

        let input_text = fs::read_to_string(&self.witness_path)
            .error_with_path(|| self.witness_path.to_string_lossy())?;
        let json = serde_json::from_str(&input_text)?;
        let input = TemplateValue::from_typed_json(&json, &program.input())?;

        let output = program.debug::<Bn256>(input)?;

        let output_json = serde_json::to_string_pretty(&output.to_json())? + "\n";
        fs::write(&self.public_data_path, &output_json)
            .error_with_path(|| self.public_data_path.to_string_lossy())?;

        println!("{}", output_json);

        Ok(())
    }
}
