//!
//! The full proof-check test runner.
//!

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use colored::Colorize;

use zinc_bytecode::Program as BytecodeProgram;

use zinc_vm::Bn256;
use zinc_vm::IFacade;

use crate::file::File;
use crate::metadata::Metadata;
use crate::program::Program;
use crate::runners::Runnable;
use crate::Summary;

pub struct Runner {
    pub verbosity: usize,
    pub filter: Option<String>,
}

impl Runner {
    pub fn new(verbosity: usize, filter: Option<String>) -> Self {
        Self { verbosity, filter }
    }
}

impl Runnable for Runner {
    fn run(&self, path: &PathBuf, file: &File, metadata: &Metadata, summary: Arc<Mutex<Summary>>) {
        let path = match path.strip_prefix(crate::TESTS_DIRECTORY) {
            Ok(path) => path,
            Err(_error) => path,
        };

        for case in metadata.cases.iter() {
            let case_name = format!("{}::{}", path.to_string_lossy(), case.case);
            if let Some(filter) = self.filter.as_ref() {
                if !case_name.contains(filter) {
                    continue;
                }
            }

            if metadata.ignore || case.ignore {
                summary.lock().expect(crate::panic::MUTEX_SYNC).ignored += 1;
                println!("[INTEGRATION] {} {}", "IGNORE".yellow(), case_name);
                continue;
            }

            let program = match Program::new(file.code.as_str(), &case.input, case.entry.as_str()) {
                Ok(program) => program,
                Err(error) => {
                    summary.lock().expect(crate::panic::MUTEX_SYNC).invalid += 1;
                    println!(
                        "[INTEGRATION] {} {} ({})",
                        "INVALID".red(),
                        case_name,
                        error
                    );
                    continue;
                }
            };

            let params = match program.bytecode.clone().setup::<Bn256>() {
                Ok(params) => params,
                Err(error) => {
                    summary.lock().expect(crate::panic::MUTEX_SYNC).invalid += 1;
                    println!(
                        "[INTEGRATION] {} {} (setup: {})",
                        "FAILED".red(),
                        path.to_string_lossy(),
                        error
                    );
                    continue;
                }
            };

            let (output, proof) = match program
                .bytecode
                .prove::<Bn256>(params.clone(), program.witness)
            {
                Ok((output, proof)) => {
                    let output_json = output.to_json();
                    if case.expect != output_json {
                        summary.lock().expect(crate::panic::MUTEX_SYNC).failed += 1;
                        println!(
                            "[INTEGRATION] {} {} (expected {}, but got {})",
                            "FAILED".bright_red(),
                            case_name,
                            case.expect,
                            output_json
                        );
                    }
                    (output, proof)
                }
                Err(error) => {
                    if case.should_panic {
                        summary.lock().expect(crate::panic::MUTEX_SYNC).passed += 1;
                        if self.verbosity > 0 {
                            println!(
                                "[INTEGRATION] {} {} (panicked)",
                                "PASSED".green(),
                                case_name
                            );
                        }
                    } else {
                        summary.lock().expect(crate::panic::MUTEX_SYNC).failed += 1;
                        println!(
                            "[INTEGRATION] {} {} (prove: {})",
                            "FAILED".bright_red(),
                            case_name,
                            error
                        );
                    }
                    continue;
                }
            };

            match BytecodeProgram::verify(params.vk, proof, output) {
                Ok(success) => {
                    if !success {
                        summary.lock().expect(crate::panic::MUTEX_SYNC).failed += 1;
                        println!(
                            "[INTEGRATION] {} {} (verification failed)",
                            "FAILED".bright_red(),
                            case_name
                        );
                    }
                }
                Err(error) => {
                    summary.lock().expect(crate::panic::MUTEX_SYNC).failed += 1;
                    println!(
                        "[INTEGRATION] {} {} (verify: {})",
                        "FAILED".bright_red(),
                        case_name,
                        error
                    );
                }
            }
        }
    }
}
