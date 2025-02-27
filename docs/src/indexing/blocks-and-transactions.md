# Blocks and Transactions

You can use the `BlockData` and `TransactionData` data structures to index important information about the Fuel network for your dApp.

## `BlockData`

```rust,ignore
pub struct BlockData {
    pub height: u64,
    pub id: Bytes32,
    pub producer: Option<Bytes32>,
    pub time: i64,
    pub transactions: Vec<TransactionData>,
}
```

The `BlockData` struct is how blocks are represented in the Fuel indexer. It contains metadata such as the ID, height, and time, as well as a list of the transactions it contains (represented by `TransactionData`). It also contains the public key hash of the block producer, if present.

## `TransactionData`

```rust,ignore
pub struct TransactionData {
    pub transaction: Transaction,
    pub status: ClientTransactionStatus,
    pub receipts: Vec<Receipt>,
    pub id: ClientTxId,
}
```

The `TransactionData` struct contains important information about a transaction in the Fuel network. The `id` field is the transaction hash, which is a 32-byte string. The `receipts` field contains a list of `Receipts`, which are generated by a Fuel node during the execution of a Sway smart contract; you can find more information in the [Receipts](./receipts.md) section.

### `Transaction`

```rust,ignore
pub enum Transaction {
    Script(Script),
    Create(Create),
    Mint(Mint),
}
```

`Transaction` refers to the Fuel transaction entity and can be one of three distinct types: `Script`, `Create`, or `Mint`. Explaining the differences between each of the types is out of scope for the Fuel indexer; however, you can find information about the `Transaction` type in the [Fuel specifications](https://specs.fuel.network/master/tx-format/transaction.html).

### `TransactionStatus`

```rust,ignore
pub enum TransactionStatus {
    Failure {
        block_id: String,
        time: DateTime<Utc>,
        reason: String,
    },
    SqueezedOut {
        reason: String,
    },
    Submitted {
        submitted_at: DateTime<Utc>,
    },
    Success {
        block_id: String,
        time: DateTime<Utc>,
    },
}
```

`TransactionStatus` refers to the status of a `Transaction` in the Fuel network.
