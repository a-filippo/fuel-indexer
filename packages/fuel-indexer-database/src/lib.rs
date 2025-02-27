#![deny(unused_crate_dependencies)]

pub mod queries;
pub mod types {
    pub use fuel_indexer_database_types::*;
}

pub use fuel_indexer_database_types::DbType;
use fuel_indexer_lib::utils::{attempt_database_connection, ServiceStatus};
use fuel_indexer_postgres as postgres;
use sqlx::{
    pool::PoolConnection, postgres::PgConnectOptions, ConnectOptions, Error as SqlxError,
};
use std::{cmp::Ordering, collections::HashMap, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexerDatabaseError {
    #[error("Invalid connection string: {0:?}")]
    InvalidConnectionString(String),
    #[error("Database backend not supported: {0:?}")]
    BackendNotSupported(String),
    #[error("No transaction is open.")]
    NoTransactionError,
    #[error("Error from sqlx: {0:#?}")]
    SqlxError(#[from] SqlxError),
    #[error("Unknown error")]
    Unknown,
    #[error("You don't own this indexer.")]
    NotYourIndexer,
    #[error("No table mapping exists for TypeId({0:?})")]
    TableMappingDoesNotExist(i64),
}

#[derive(Debug)]
pub enum IndexerConnection {
    Postgres(Box<PoolConnection<sqlx::Postgres>>),
}

#[derive(Clone, Debug)]
pub enum IndexerConnectionPool {
    Postgres(sqlx::Pool<sqlx::Postgres>),
}

impl IndexerConnectionPool {
    pub fn database_type(&self) -> DbType {
        match self {
            IndexerConnectionPool::Postgres(_) => DbType::Postgres,
        }
    }

    pub async fn connect(
        database_url: &str,
    ) -> Result<IndexerConnectionPool, IndexerDatabaseError> {
        let url = url::Url::parse(database_url);
        if url.is_err() {
            return Err(IndexerDatabaseError::InvalidConnectionString(
                database_url.into(),
            ));
        }
        let mut url = url.expect("Database URL should be correctly formed");
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();

        let query = query
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<String>>()
            .join("&");

        url.set_query(Some(&query));

        match url.scheme() {
            "postgres" => {
                let mut opts = PgConnectOptions::from_str(url.as_str())?;
                opts.disable_statement_logging();

                let pool = attempt_database_connection(|| {
                    sqlx::postgres::PgPoolOptions::new().connect_with(opts.clone())
                })
                .await;

                Ok(IndexerConnectionPool::Postgres(pool))
            }
            err => Err(IndexerDatabaseError::BackendNotSupported(err.into())),
        }
    }

    pub async fn is_connected(&self) -> sqlx::Result<ServiceStatus> {
        match self {
            IndexerConnectionPool::Postgres(p) => {
                let mut conn = p.acquire().await?;
                let result =
                    postgres::execute_query(&mut conn, "SELECT true;".to_string())
                        .await?;

                match result.cmp(&1) {
                    Ordering::Equal => Ok(ServiceStatus::OK),
                    _ => Ok(ServiceStatus::NotOk),
                }
            }
        }
    }

    pub async fn acquire(&self) -> sqlx::Result<IndexerConnection> {
        match self {
            IndexerConnectionPool::Postgres(p) => {
                Ok(IndexerConnection::Postgres(Box::new(p.acquire().await?)))
            }
        }
    }
}
