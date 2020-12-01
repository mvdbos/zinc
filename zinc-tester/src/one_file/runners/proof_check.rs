//!
//! The full proof-check test runner.
//!

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use colored::Colorize;
use num::BigInt;
use num::Zero;

use zinc_types::TransactionMsg;
use zinc_vm::Bn256;
use zinc_vm::CircuitFacade;
use zinc_vm::ContractFacade;
use zinc_vm::ContractInput;
use zinc_vm::Facade;

use crate::error::Error;
use crate::one_file::file::File;
use crate::one_file::instance::Instance;
use crate::one_file::metadata::Metadata;
use crate::one_file::runners::IRunnable;
use crate::summary::Summary;

///
/// The proof-check runner.
///
/// Computes the result, makes the trusted setup and proof verification.
///
#[derive(Clone)]
pub struct Runner {
    /// If zero, does not print the successful tests.
    pub verbosity: usize,
    /// If set, runs only the tests whose full names contain the string.
    pub filter: Option<String>,
}

impl Runner {
    ///
    /// Creates a runner instance.
    ///
    pub fn new(verbosity: usize, filter: Option<String>) -> Self {
        Self { verbosity, filter }
    }
}

impl IRunnable for Runner {
    fn run(
        self,
        path: PathBuf,
        file: File,
        metadata: Metadata,
        summary: Arc<Mutex<Summary>>,
    ) -> anyhow::Result<()> {
        let path = match path.strip_prefix(crate::ONE_FILE_TESTS_DIRECTORY) {
            Ok(path) => path,
            Err(_error) => &path,
        };

        for case in metadata.cases.into_iter() {
            let case_name = format!("{}::{}", path.to_string_lossy(), case.case);
            if let Some(filter) = self.filter.as_ref() {
                if !case_name.contains(filter) {
                    continue;
                }
            }

            if metadata.ignore || case.ignore {
                summary
                    .lock()
                    .expect(zinc_const::panic::SYNCHRONIZATION)
                    .ignored += 1;
                println!("[INTEGRATION] {} {}", "IGNORE".yellow(), case_name);
                continue;
            }

            let mut instance = match Instance::new(
                case_name.clone(),
                file.code.as_str(),
                path.to_owned(),
                case.method.clone(),
                case.input,
            ) {
                Ok(application) => application,
                Err(error) => {
                    summary
                        .lock()
                        .expect(zinc_const::panic::SYNCHRONIZATION)
                        .invalid += 1;
                    println!(
                        "[INTEGRATION] {} {} ({})",
                        "INVALID".red(),
                        case_name,
                        error
                    );
                    continue;
                }
            };

            let params = match match instance.application.clone() {
                zinc_types::Application::Circuit(circuit) => {
                    CircuitFacade::new(circuit).setup::<Bn256>()
                }
                zinc_types::Application::Contract(contract) => ContractFacade::new(contract)
                    .setup::<Bn256>(case.method.clone().unwrap_or_else(|| {
                        zinc_const::source::FUNCTION_MAIN_IDENTIFIER.to_owned()
                    })),
                zinc_types::Application::Library(_library) => {
                    anyhow::bail!(Error::CannotRunLibrary);
                }
            } {
                Ok(params) => params,
                Err(error) => {
                    summary
                        .lock()
                        .expect(zinc_const::panic::SYNCHRONIZATION)
                        .failed += 1;
                    println!(
                        "[INTEGRATION] {} {} (setup: {})",
                        "FAILED".bright_red(),
                        path.to_string_lossy(),
                        error
                    );
                    continue;
                }
            };

            let (output, proof) = match instance.application.clone() {
                zinc_types::Application::Circuit(circuit) => {
                    let result =
                        CircuitFacade::new(circuit).prove::<Bn256>(params.clone(), instance.input);

                    match result {
                        Ok((result, proof)) => {
                            let result_json = result.clone().into_json();

                            if case.output != result_json {
                                summary
                                    .lock()
                                    .expect(zinc_const::panic::SYNCHRONIZATION)
                                    .failed += 1;
                                println!(
                                    "[INTEGRATION] {} {} (expected {}, but got {})",
                                    "FAILED".bright_red(),
                                    case_name,
                                    case.output,
                                    result_json
                                );
                            }
                            (result, proof)
                        }
                        Err(error) => {
                            if case.should_panic {
                                summary
                                    .lock()
                                    .expect(zinc_const::panic::SYNCHRONIZATION)
                                    .passed += 1;
                                if self.verbosity > 0 {
                                    println!(
                                        "[INTEGRATION] {} {} (panicked)",
                                        "PASSED".green(),
                                        case_name
                                    );
                                }
                            } else {
                                summary
                                    .lock()
                                    .expect(zinc_const::panic::SYNCHRONIZATION)
                                    .failed += 1;
                                println!(
                                    "[INTEGRATION] {} {} (prove: {})",
                                    "FAILED".bright_red(),
                                    case_name,
                                    error
                                );
                            }
                            continue;
                        }
                    }
                }
                zinc_types::Application::Contract(contract) => {
                    let storage: Vec<zinc_types::ContractFieldValue> = contract
                        .storage
                        .clone()
                        .into_iter()
                        .map(zinc_types::ContractFieldValue::new_from_type)
                        .collect();

                    instance.input.insert_contract_instance(BigInt::zero());
                    let result = ContractFacade::new(contract).prove::<Bn256>(
                        params.clone(),
                        ContractInput::new(
                            instance.input,
                            zinc_types::Value::Contract(storage),
                            case.method.unwrap_or_else(|| {
                                zinc_const::source::FUNCTION_MAIN_IDENTIFIER.to_owned()
                            }),
                            TransactionMsg::default(),
                        ),
                    );

                    match result {
                        Ok((result, proof)) => {
                            let result_json = result.clone().into_json();

                            if case.output != result_json {
                                summary
                                    .lock()
                                    .expect(zinc_const::panic::SYNCHRONIZATION)
                                    .failed += 1;
                                println!(
                                    "[INTEGRATION] {} {} (expected {}, but got {})",
                                    "FAILED".bright_red(),
                                    case_name,
                                    case.output,
                                    result_json
                                );
                            }
                            (result, proof)
                        }
                        Err(error) => {
                            if case.should_panic {
                                summary
                                    .lock()
                                    .expect(zinc_const::panic::SYNCHRONIZATION)
                                    .passed += 1;
                                if self.verbosity > 0 {
                                    println!(
                                        "[INTEGRATION] {} {} (panicked)",
                                        "PASSED".green(),
                                        case_name
                                    );
                                }
                            } else {
                                summary
                                    .lock()
                                    .expect(zinc_const::panic::SYNCHRONIZATION)
                                    .failed += 1;
                                println!(
                                    "[INTEGRATION] {} {} (prove: {})",
                                    "FAILED".bright_red(),
                                    case_name,
                                    error
                                );
                            }
                            continue;
                        }
                    }
                }
                zinc_types::Application::Library(_library) => {
                    anyhow::bail!(Error::CannotRunLibrary);
                }
            };

            match Facade::verify(params.vk, proof, output) {
                Ok(success) => {
                    if success {
                        summary
                            .lock()
                            .expect(zinc_const::panic::SYNCHRONIZATION)
                            .passed += 1;
                        if self.verbosity > 0 {
                            println!("[INTEGRATION] {} {}", "PASSED".green(), case_name);
                        }
                    } else {
                        summary
                            .lock()
                            .expect(zinc_const::panic::SYNCHRONIZATION)
                            .failed += 1;
                        println!(
                            "[INTEGRATION] {} {} (verification failed)",
                            "FAILED".bright_red(),
                            case_name
                        );
                    }
                }
                Err(error) => {
                    summary
                        .lock()
                        .expect(zinc_const::panic::SYNCHRONIZATION)
                        .failed += 1;
                    println!(
                        "[INTEGRATION] {} {} (verify: {})",
                        "FAILED".bright_red(),
                        case_name,
                        error
                    );
                }
            }
        }

        Ok(())
    }
}
