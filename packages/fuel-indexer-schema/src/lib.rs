//! # fuel_indexer_schema
//!
//! A collection of utilities used to create SQL-based objects that interact with
//! objects being pulled from, and being persisted to, the database backend.

// TODO: Deny `clippy::unused_crate_dependencies` when including feature-flagged dependency `itertools`

extern crate alloc;

use fuel_indexer_lib::MAX_ARRAY_LENGTH;
use fuel_indexer_types::{fuel::*, scalar::*, Identity};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "db-models")]
pub mod db;

pub mod join;

/// Placeholder value for SQL `NULL` values.
const NULL_VALUE: &str = "NULL";

/// Result type used by indexer schema operations.
pub type IndexerSchemaResult<T> = core::result::Result<T, IndexerSchemaError>;

/// Error type used by indexer schema operations.
#[derive(Error, Debug)]
pub enum IndexerSchemaError {
    #[error("Generic error")]
    Generic,
    #[error("GraphQL parser error: {0:?}")]
    ParseError(#[from] async_graphql_parser::Error),
    #[error("Could not build schema: {0:?}")]
    SchemaConstructionError(String),
    #[error("Unable to parse join directive: {0:?}")]
    JoinDirectiveError(String),
    #[error("Unable to build schema field and type map: {0:?}")]
    FieldAndTypeConstructionError(String),
    #[error("This TypeKind is unsupported.")]
    UnsupportedTypeKind,
    #[error("List types are unsupported.")]
    ListTypesUnsupported,
    #[error("Inconsistent use of virtual union types. {0:?}")]
    InconsistentVirtualUnion(String),
}

/// `FtColumn` is an abstraction that represents a sized type that can be persisted to, and
/// fetched from the database.
///
/// Each `FtColumn` corresponds to a Fuel-specific GraphQL scalar type.
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Clone, Hash)]
pub enum FtColumn {
    Address(Option<Address>),
    Array(Option<Vec<FtColumn>>),
    AssetId(Option<AssetId>),
    Blob(Option<Blob>),
    BlockHeight(Option<BlockHeight>),
    BlockId(Option<BlockId>),
    Boolean(Option<bool>),
    Bytes32(Option<Bytes32>),
    Bytes4(Option<Bytes4>),
    Bytes64(Option<Bytes64>),
    Bytes8(Option<Bytes8>),
    Charfield(Option<String>),
    ContractId(Option<ContractId>),
    Enum(Option<String>),
    HexString(Option<HexString>),
    ID(Option<UID>),
    Identity(Option<Identity>),
    Int1(Option<Int1>),
    Int16(Option<Int16>),
    Int4(Option<Int4>),
    Int8(Option<Int8>),
    Json(Option<Json>),
    MessageId(Option<MessageId>),
    Nonce(Option<Nonce>),
    Salt(Option<Salt>),
    Signature(Option<Signature>),
    Tai64Timestamp(Option<Tai64Timestamp>),
    Timestamp(Option<Int8>),
    TxId(Option<TxId>),
    UID(Option<UID>),
    UInt1(Option<UInt1>),
    UInt16(Option<UInt16>),
    UInt4(Option<UInt4>),
    UInt8(Option<UInt8>),
    Virtual(Option<Virtual>),
}

impl FtColumn {
    /// Return query fragments for `INSERT` statements.
    ///
    /// Since `FtColumn` column is used when compiling indexers we can panic here. Anything that panics,
    /// will panic when compiling indexers, so will be caught before runtime.
    pub fn query_fragment(&self) -> String {
        match self {
            FtColumn::ID(value) => {
                if let Some(val) = value {
                    format!("'{val}'")
                } else {
                    panic!("Schema fields of type `ID` cannot be nullable.")
                }
            }
            FtColumn::UID(value) => match value {
                Some(val) => format!("'{val}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Address(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::AssetId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Bytes4(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Bytes8(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Bytes32(value) | FtColumn::BlockId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Nonce(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Bytes64(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::TxId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::HexString(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Signature(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::ContractId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Int4(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Int1(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::UInt1(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Int8(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Int16(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::UInt4(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::BlockHeight(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::UInt8(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::UInt16(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Timestamp(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Tai64Timestamp(value) => match value {
                Some(val) => {
                    let x = hex::encode(val.to_bytes());
                    format!("'{x}'")
                }
                None => String::from(NULL_VALUE),
            },
            FtColumn::Salt(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Json(value) | FtColumn::Virtual(value) => match value {
                Some(val) => {
                    let x = &val.0;
                    format!("'{x}'")
                }
                None => String::from(NULL_VALUE),
            },
            FtColumn::MessageId(value) => match value {
                Some(val) => format!("'{val:x}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Charfield(value) => match value {
                Some(val) => format!("'{val}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Identity(value) => match value {
                Some(val) => match val {
                    Identity::Address(v) => format!("'{v:x}'",),
                    Identity::ContractId(v) => format!("'{v:x}'",),
                },
                None => String::from(NULL_VALUE),
            },
            FtColumn::Boolean(value) => match value {
                Some(val) => format!("{val}"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Blob(value) => match value {
                Some(blob) => {
                    let x = hex::encode(blob.as_ref());
                    format!("'{x}'")
                }
                None => String::from(NULL_VALUE),
            },
            FtColumn::Enum(value) => match value {
                Some(val) => format!("'{val}'"),
                None => String::from(NULL_VALUE),
            },
            FtColumn::Array(arr) => match arr {
                Some(arr) => {
                    assert!(
                        arr.len() < MAX_ARRAY_LENGTH,
                        "Array length exceeds maximum allowed length."
                    );

                    // If the array has no items, then we have no `FtColumn`s from which to determine
                    // what type of PostgreSQL array this is. In this case, the user should be using a
                    // inner required (outer optional) array (e.g., [Foo!]) in their schema.
                    //
                    // Ideally we need a way to validate this in something like `fuel_indexer_lib::graphql::GraphQLSchemaValidator`.
                    if arr.is_empty() {
                        return String::from(NULL_VALUE);
                    }

                    let discriminant = std::mem::discriminant(&arr[0]);
                    let result = arr
                            .iter()
                            .map(|e| {
                                if std::mem::discriminant(e) != discriminant {
                                    panic!(
                                        "Array elements are not of the same column type. Expected {discriminant:#?} - Actual: {:#?}",
                                        std::mem::discriminant(e)
                                    )
                                } else {
                                    e.to_owned().query_fragment()
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(",");

                    // We have to force sqlx to see this as a JSON type else it will think this type
                    // should be TEXT
                    let suffix = match arr[0] {
                        FtColumn::Virtual(_) | FtColumn::Json(_) => "::json[]",
                        _ => "",
                    };

                    // Using ARRAY syntax vs curly braces so we can keep the single quotes used by
                    // `ColumnType::query_fragment`
                    format!("ARRAY [{result}]{suffix}")
                }
                None => String::from(NULL_VALUE),
            },
        }
    }
}

mod tests {
    #[test]
    fn test_fragments_some_types() {
        use super::*;

        let uid = FtColumn::ID(Some(
            UID::new(
                "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            )
            .unwrap(),
        ));
        let addr =
            FtColumn::Address(Some(Address::try_from([0x12; 32]).expect("Bad bytes")));
        let asset_id =
            FtColumn::AssetId(Some(AssetId::try_from([0xA5; 32]).expect("Bad bytes")));
        let bytes4 =
            FtColumn::Bytes4(Some(Bytes4::try_from([0xF0; 4]).expect("Bad bytes")));
        let bytes8 =
            FtColumn::Bytes8(Some(Bytes8::try_from([0x9D; 8]).expect("Bad bytes")));
        let bytes32 =
            FtColumn::Bytes32(Some(Bytes32::try_from([0xEE; 32]).expect("Bad bytes")));
        let nonce =
            FtColumn::Nonce(Some(Nonce::try_from([0x12; 32]).expect("Bad bytes")));
        let bytes64 =
            FtColumn::Bytes64(Some(Bytes64::try_from([0x12; 64]).expect("Bad bytes")));
        let txid = FtColumn::TxId(Some(TxId::try_from([0x12; 32]).expect("Bad bytes")));
        let hex = FtColumn::HexString(Some(
            HexString::try_from("this is a hexstring").expect("Bad bytes"),
        ));
        let sig = FtColumn::Signature(Some(
            Signature::try_from([0x12; 64]).expect("Bad bytes"),
        ));
        let contractid = FtColumn::ContractId(Some(
            ContractId::try_from([0x78; 32]).expect("Bad bytes"),
        ));
        let int4 = FtColumn::Int4(Some(i32::from_le_bytes([0x78; 4])));
        let int8 = FtColumn::Int8(Some(i64::from_le_bytes([0x78; 8])));
        let int16 = FtColumn::Int16(Some(i128::from_le_bytes([0x78; 16])));
        let uint4 = FtColumn::UInt4(Some(u32::from_le_bytes([0x78; 4])));
        let uint1 = FtColumn::UInt1(Some(u8::from_le_bytes([0x78; 1])));
        let uint8 = FtColumn::UInt8(Some(u64::from_le_bytes([0x78; 8])));
        let uint16 = FtColumn::UInt16(Some(u128::from_le_bytes([0x78; 16])));
        let int64 = FtColumn::Timestamp(Some(i64::from_le_bytes([0x78; 8])));
        let block_height =
            FtColumn::BlockHeight(Some(BlockHeight::try_from(0x78).unwrap()));
        let timestamp = FtColumn::Timestamp(Some(i64::from_le_bytes([0x78; 8])));
        let tai64_timestamp = FtColumn::Tai64Timestamp(Some(
            Tai64Timestamp::try_from([0u8; 8]).expect("Bad bytes"),
        ));
        let salt = FtColumn::Salt(Some(Salt::try_from([0x31; 32]).expect("Bad bytes")));
        let message_id = FtColumn::MessageId(Some(
            MessageId::try_from([0x0F; 32]).expect("Bad bytes"),
        ));
        let charfield = FtColumn::Charfield(Some(String::from("hello world")));
        let json = FtColumn::Json(Some(Json(r#"{"hello":"world"}"#.to_string())));
        let identity = FtColumn::Identity(Some(Identity::Address([0x12; 32].into())));
        let r#bool = FtColumn::Boolean(Some(true));
        let blob = FtColumn::Blob(Some(Blob::from(vec![0u8, 1, 2, 3, 4, 5])));
        let r#enum = FtColumn::Enum(Some(String::from("hello")));
        let array = FtColumn::Array(Some(vec![FtColumn::Int4(Some(1))]));

        insta::assert_yaml_snapshot!(uid.query_fragment());
        insta::assert_yaml_snapshot!(addr.query_fragment());
        insta::assert_yaml_snapshot!(asset_id.query_fragment());
        insta::assert_yaml_snapshot!(bytes4.query_fragment());
        insta::assert_yaml_snapshot!(bytes8.query_fragment());
        insta::assert_yaml_snapshot!(bytes32.query_fragment());
        insta::assert_yaml_snapshot!(nonce.query_fragment());
        insta::assert_yaml_snapshot!(bytes64.query_fragment());
        insta::assert_yaml_snapshot!(txid.query_fragment());
        insta::assert_yaml_snapshot!(hex.query_fragment());
        insta::assert_yaml_snapshot!(sig.query_fragment());
        insta::assert_yaml_snapshot!(contractid.query_fragment());
        insta::assert_yaml_snapshot!(salt.query_fragment());
        insta::assert_yaml_snapshot!(int4.query_fragment());
        insta::assert_yaml_snapshot!(int8.query_fragment());
        insta::assert_yaml_snapshot!(int16.query_fragment());
        insta::assert_yaml_snapshot!(uint4.query_fragment());
        insta::assert_yaml_snapshot!(uint1.query_fragment());
        insta::assert_yaml_snapshot!(uint8.query_fragment());
        insta::assert_yaml_snapshot!(uint16.query_fragment());
        insta::assert_yaml_snapshot!(int64.query_fragment());
        insta::assert_yaml_snapshot!(block_height.query_fragment());
        insta::assert_yaml_snapshot!(timestamp.query_fragment());
        insta::assert_yaml_snapshot!(tai64_timestamp.query_fragment());
        insta::assert_yaml_snapshot!(message_id.query_fragment());
        insta::assert_yaml_snapshot!(charfield.query_fragment());
        insta::assert_yaml_snapshot!(json.query_fragment());
        insta::assert_yaml_snapshot!(identity.query_fragment());
        insta::assert_yaml_snapshot!(r#bool.query_fragment());
        insta::assert_yaml_snapshot!(blob.query_fragment());
        insta::assert_yaml_snapshot!(r#enum.query_fragment());
        insta::assert_yaml_snapshot!(array.query_fragment());
    }

    #[test]
    fn test_fragments_none_types() {
        use super::*;

        let addr_none = FtColumn::Address(None);
        let asset_id_none = FtColumn::AssetId(None);
        let bytes4_none = FtColumn::Bytes4(None);
        let bytes8_none = FtColumn::Bytes8(None);
        let bytes32_none = FtColumn::Bytes32(None);
        let contractid_none = FtColumn::ContractId(None);
        let int4_none = FtColumn::Int4(None);
        let int8_none = FtColumn::Int8(None);
        let int16_none = FtColumn::Int8(None);
        let uint4_none = FtColumn::UInt4(None);
        let uint8_none = FtColumn::UInt8(None);
        let uint16_none = FtColumn::UInt8(None);
        let int64_none = FtColumn::Timestamp(None);
        let salt_none = FtColumn::Salt(None);
        let message_id_none = FtColumn::MessageId(None);
        let charfield_none = FtColumn::Charfield(None);
        let json_none = FtColumn::Json(None);
        let identity_none = FtColumn::Identity(None);
        let nonce = FtColumn::Nonce(None);
        let bytes64 = FtColumn::Bytes64(None);
        let txid = FtColumn::TxId(None);
        let hex = FtColumn::HexString(None);
        let sig = FtColumn::Signature(None);
        let block_height = FtColumn::BlockHeight(None);
        let timestamp = FtColumn::Timestamp(None);
        let tai64_timestamp = FtColumn::Tai64Timestamp(None);
        let r#bool = FtColumn::Boolean(None);
        let blob = FtColumn::Blob(None);
        let r#enum = FtColumn::Enum(None);
        let array = FtColumn::Array(None);

        insta::assert_yaml_snapshot!(addr_none.query_fragment());
        insta::assert_yaml_snapshot!(asset_id_none.query_fragment());
        insta::assert_yaml_snapshot!(bytes4_none.query_fragment());
        insta::assert_yaml_snapshot!(bytes8_none.query_fragment());
        insta::assert_yaml_snapshot!(bytes32_none.query_fragment());
        insta::assert_yaml_snapshot!(contractid_none.query_fragment());
        insta::assert_yaml_snapshot!(salt_none.query_fragment());
        insta::assert_yaml_snapshot!(int4_none.query_fragment());
        insta::assert_yaml_snapshot!(int8_none.query_fragment());
        insta::assert_yaml_snapshot!(int16_none.query_fragment());
        insta::assert_yaml_snapshot!(uint4_none.query_fragment());
        insta::assert_yaml_snapshot!(uint8_none.query_fragment());
        insta::assert_yaml_snapshot!(uint16_none.query_fragment());
        insta::assert_yaml_snapshot!(int64_none.query_fragment());
        insta::assert_yaml_snapshot!(message_id_none.query_fragment());
        insta::assert_yaml_snapshot!(charfield_none.query_fragment());
        insta::assert_yaml_snapshot!(json_none.query_fragment());
        insta::assert_yaml_snapshot!(identity_none.query_fragment());
        insta::assert_yaml_snapshot!(nonce.query_fragment());
        insta::assert_yaml_snapshot!(bytes64.query_fragment());
        insta::assert_yaml_snapshot!(txid.query_fragment());
        insta::assert_yaml_snapshot!(hex.query_fragment());
        insta::assert_yaml_snapshot!(sig.query_fragment());
        insta::assert_yaml_snapshot!(block_height.query_fragment());
        insta::assert_yaml_snapshot!(timestamp.query_fragment());
        insta::assert_yaml_snapshot!(tai64_timestamp.query_fragment());
        insta::assert_yaml_snapshot!(r#bool.query_fragment());
        insta::assert_yaml_snapshot!(blob.query_fragment());
        insta::assert_yaml_snapshot!(r#enum.query_fragment());
        insta::assert_yaml_snapshot!(array.query_fragment());
    }

    #[test]
    #[should_panic(expected = "Schema fields of type `ID` cannot be nullable.")]
    fn test_panic_on_none_id_fragment() {
        use super::*;

        let uid_none = FtColumn::ID(None);
        insta::assert_yaml_snapshot!(uid_none.query_fragment());
    }
}
