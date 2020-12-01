//!
//! The Zandbox error.
//!

use std::fmt;

use actix_web::http::StatusCode;
use actix_web::ResponseError;

///
/// The Zandbox error.
///
#[derive(Debug)]
pub enum Error {
    /// The uploaded bytecode is malformed.
    InvalidBytecode(String),
    /// The uploaded application is not a contract.
    NotAContract,
    /// The contract has no constructor.
    ConstructorNotFound,

    /// The contract with the specified address is not found in the server cache.
    ContractNotFound(String),
    /// The specified method does not exist in the contract.
    MethodNotFound(String),
    /// The mutable method must be called via the `call` endpoint.
    MethodIsMutable(String),
    /// The immutable method must be called via the `query` endpoint.
    MethodIsImmutable(String),
    /// The `query` endpoint got the method name but the method arguments are missing.
    MethodArgumentsNotFound(String),
    /// Invalid contract method arguments.
    InvalidInput(anyhow::Error),
    /// The contract source code has changed, but the name and version are the same.
    ContractSourceCodeMismatch,

    /// Token cannot be resolved by zkSync.
    TokenNotFound(String),
    /// The contract method input transaction is invalid.
    Transaction(zinc_types::TransactionError),
    /// The ZkSync transfer errors.
    TransferFailure(String),
    /// Could not get the account ID.
    AccountIdNotFound,
    /// Failed to execute the change-pubkey transaction.
    ChangePubkey(String),

    /// The virtual machine contract method runtime error.
    VirtualMachine(zinc_vm::Error),
    /// The PostgreSQL database error.
    Database(sqlx::Error),
    /// The ZkSync server client error.
    ZkSyncClient(zksync::error::ClientError),
    /// The ZkSync server signer error.
    ZkSyncSigner(zksync_eth_signer::error::SignerError),
}

impl From<zinc_types::TransactionError> for Error {
    fn from(inner: zinc_types::TransactionError) -> Self {
        Self::Transaction(inner)
    }
}

impl From<zinc_vm::Error> for Error {
    fn from(inner: zinc_vm::Error) -> Self {
        Self::VirtualMachine(inner)
    }
}

impl From<sqlx::Error> for Error {
    fn from(inner: sqlx::Error) -> Self {
        Self::Database(inner)
    }
}

impl From<zksync::error::ClientError> for Error {
    fn from(inner: zksync::error::ClientError) -> Self {
        Self::ZkSyncClient(inner)
    }
}

impl From<zksync_eth_signer::error::SignerError> for Error {
    fn from(inner: zksync_eth_signer::error::SignerError) -> Self {
        Self::ZkSyncSigner(inner)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidBytecode(..) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::NotAContract => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ConstructorNotFound => StatusCode::UNPROCESSABLE_ENTITY,

            Self::ContractNotFound(..) => StatusCode::NOT_FOUND,
            Self::MethodNotFound(..) => StatusCode::BAD_REQUEST,
            Self::MethodIsMutable(..) => StatusCode::BAD_REQUEST,
            Self::MethodIsImmutable(..) => StatusCode::BAD_REQUEST,
            Self::MethodArgumentsNotFound(..) => StatusCode::BAD_REQUEST,
            Self::InvalidInput(..) => StatusCode::BAD_REQUEST,
            Self::ContractSourceCodeMismatch => StatusCode::BAD_REQUEST,

            Self::TokenNotFound(..) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::TransferFailure { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::AccountIdNotFound => StatusCode::UNPROCESSABLE_ENTITY,
            Self::ChangePubkey(..) => StatusCode::UNPROCESSABLE_ENTITY,

            Self::Transaction(..) => StatusCode::BAD_REQUEST,
            Self::VirtualMachine(..) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Database(..) => StatusCode::SERVICE_UNAVAILABLE,
            Self::ZkSyncClient(..) => StatusCode::SERVICE_UNAVAILABLE,
            Self::ZkSyncSigner(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            Self::InvalidBytecode(inner) => format!("Invalid bytecode: {}", inner),
            Self::NotAContract => "Not a contract".to_owned(),
            Self::ConstructorNotFound => "Constructor not found".to_owned(),

            Self::ContractNotFound(address) => {
                format!("Contract with address {} not found", address)
            }
            Self::MethodNotFound(name) => format!("Method `{}` not found", name),
            Self::MethodIsMutable(name) => {
                format!("Method `{}` is mutable: use 'call' instead", name)
            }
            Self::MethodIsImmutable(name) => {
                format!("Method `{}` is immutable: use 'query' instead", name)
            }
            Self::MethodArgumentsNotFound(name) => {
                format!("Method `{}` arguments are not specified", name)
            }
            Self::InvalidInput(inner) => format!("Input: {}", inner),
            Self::ContractSourceCodeMismatch => {
                "Contract source code mismatch, consider increasing the project version".to_owned()
            }

            Self::TokenNotFound(token_id) => format!("Token ID {} cannot be resolved", token_id),
            Self::Transaction(inner) => format!("Transaction: {}", inner),
            Self::TransferFailure(inner) => format!("Transfer failure: {}", inner),
            Self::AccountIdNotFound => "Could not get the contract account ID".to_owned(),
            Self::ChangePubkey(inner) => format!("Changing the contract public key: {}", inner),

            Self::VirtualMachine(inner) => format!("Runtime: {:?}", inner),
            Self::Database(inner) => format!("Database: {:?}", inner),
            Self::ZkSyncClient(inner) => format!("ZkSync: {:?}", inner),
            Self::ZkSyncSigner(inner) => format!("ZkSync: {:?}", inner),
        };

        log::warn!("{}", error);
        write!(f, "{}", error)
    }
}
