#![deny(unused_crate_dependencies)]
pub mod cli;
pub(crate) mod commands;
mod database;
pub mod executor;
pub mod ffi;
pub(crate) mod queries;
mod service;

pub use database::Database;
pub use executor::{Executor, IndexEnv, NativeIndexExecutor, WasmIndexExecutor};
pub use fuel_indexer_database::IndexerDatabaseError;
pub use fuel_indexer_lib::{
    config::IndexerConfig,
    manifest::{Manifest, ManifestError, Module},
};
pub use fuel_indexer_schema::{db::IndexerSchemaDbError, FtColumn};
pub use service::get_start_block;
pub use service::IndexerService;
use thiserror::Error;
use wasmer::{ExportError, InstantiationError, RuntimeError};

// Required for vendored openssl
use openssl as _;

pub mod prelude {
    pub use super::{
        Database, Executor, FtColumn, IndexEnv, IndexerConfig, IndexerError,
        IndexerResult, IndexerService, Manifest, Module, NativeIndexExecutor,
        WasmIndexExecutor,
    };
    pub use async_std::sync::{Arc, Mutex};
    pub use fuel_indexer_lib::config::{DatabaseConfig, FuelClientConfig, WebApiConfig};
    pub use fuel_indexer_types::*;
}

pub type IndexerResult<T> = core::result::Result<T, IndexerError>;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Compiler error: {0:#?}")]
    CompileError(#[from] wasmer::CompileError),
    #[error("Error from sqlx: {0:#?}")]
    SqlxError(#[from] sqlx::Error),
    #[error("Error instantiating wasm interpreter: {0:#?}")]
    InstantiationError(#[from] InstantiationError),
    #[error("Error finding exported symbol: {0:#?}")]
    ExportError(#[from] ExportError),
    #[error("Error executing function: {0:#?}")]
    RuntimeError(#[from] RuntimeError),
    #[error("Run time limit exceeded error")]
    RunTimeLimitExceededError,
    #[error("IO Error: {0:#?}")]
    IoError(#[from] std::io::Error),
    #[error("FFI Error {0:?}")]
    FFIError(#[from] ffi::FFIError),
    #[error("Missing handler")]
    MissingHandler,
    #[error("Database error {0:?}")]
    DatabaseError(#[from] IndexerDatabaseError),
    #[error("Invalid address {0:?}")]
    InvalidAddress(#[from] std::net::AddrParseError),
    #[error("Join Error {0:?}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Error initializing executor")]
    ExecutorInitError,
    #[error("Error executing handler")]
    HandlerError,
    #[error("Invalid port {0:?}")]
    InvalidPortNumber(#[from] core::num::ParseIntError),
    #[error("No open transaction for {0}. Was a transaction started?")]
    NoTransactionError(String),
    #[error("{0}.")]
    Unknown(String),
    #[error("Indexer schema error: {0:?}")]
    SchemaError(#[from] IndexerSchemaDbError),
    #[error("Manifest error: {0:?}")]
    ManifestError(#[from] ManifestError),
    #[error("Error creating wasm executor.")]
    WasmExecutionInstantiationError,
    #[error("Error creating native executor.")]
    NativeExecutionInstantiationError,
    #[error("Native execution runtime error.")]
    NativeExecutionRuntimeError,
    #[error("Tokio time error: {0:?}")]
    Elapsed(#[from] tokio::time::error::Elapsed),
    #[error("Indexer end block has been stopping execution.")]
    EndBlockMet,
    #[error("Invalid schema: {0:?}")]
    SchemaVersionMismatch(String),
}
