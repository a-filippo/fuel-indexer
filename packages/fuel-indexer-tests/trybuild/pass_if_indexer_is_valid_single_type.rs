extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[no_mangle]
fn ff_log_data(_inp: ()) {}

#[no_mangle]
fn ff_put_object(_inp: ()) {}

#[no_mangle]
fn ff_put_many_to_many_record(_inp: ()) {}

#[no_mangle]
fn ff_early_exit(_inp: ()) {}

#[indexer(manifest = "packages/fuel-indexer-tests/trybuild/simple_wasm.yaml")]
mod indexer {
    fn function_one(event: SomeEvent) {
        let SomeEvent { id, account } = event;

        assert_eq!(id, 9);
        assert_eq!(account, Bits256([48u8; 32]));
    }
}

fn main() {
    use fuels::core::codec::ABIEncoder;

    let s = SomeEvent {
        id: 9,
        account: Bits256([48u8; 32]),
    };

    let encoded = ABIEncoder::encode(&[s.into_token()]).expect("Failed compile test");
    let bytes = encoded.resolve(0);

    let data: Vec<BlockData> = vec![BlockData {
        id: [0u8; 32].into(),
        time: 1,
        producer: None,
        height: 0,
        consensus: fuel::Consensus::default(),
        header: fuel::Header {
            id: [0u8; 32].into(),
            da_height: 1,
            transactions_count: 1,
            message_receipt_count: 1,
            transactions_root: [0u8; 32].into(),
            height: 1,
            prev_root: [0u8; 32].into(),
            time: 1,
            application_hash: [0u8; 32].into(),
            message_receipt_root: [0u8; 32].into(),
        },
        transactions: vec![fuel::TransactionData {
            status: fuel::TransactionStatus::default(),
            id: [0u8; 32].into(),
            receipts: vec![
                fuel::Receipt::Call {
                    id: [0u8; 32].into(),
                    to: [0u8; 32].into(),
                    amount: 400,
                    asset_id: [0u8; 32].into(),
                    gas: 4,
                    param1: 2048508220,
                    param2: 0,
                    pc: 0,
                    is: 0,
                },
                fuel::Receipt::ReturnData {
                    id: [0u8; 32].into(),
                    ptr: 2342143,
                    len: bytes.len() as u64,
                    digest: [0u8; 32].into(),
                    data: Some(bytes),
                    pc: 0,
                    is: 0,
                },
            ],
            transaction: fuel::Transaction::default(),
        }],
    }];

    let mut bytes = serialize(&data);

    let ptr = bytes.as_mut_ptr();
    let len = bytes.len();

    handle_events(ptr, len);
}
