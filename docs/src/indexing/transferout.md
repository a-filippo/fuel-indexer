# TransferOut

```rust,ignore
use fuel_types::{ContractId, AssetId, Address};
pub struct TransferOut {
    pub contract_id: ContractId,
    pub to: Address,
    pub amount: u64,
    pub asset_id: AssetId,
    pub pc: u64,
    pub is: u64,
}
```

- A `TransferOut` receipt is generated when coins are transferred to an address rather than a contract.
- Every other field of the receipt works the same way as it does in the `Transfer` receipt.
- [Read more about `TransferOut` in the Fuel protocol ABI spec](https://specs.fuel.network/master/abi/receipts.html#transferout-receipt)

You can handle functions that produce a `TransferOut` receipt type by adding a parameter with the type `TransferOut`.

```rust, ignore
fn handle_transferout(transfer_out: TransferOut) {
  // handle the emitted TransferOut receipt
}
```
