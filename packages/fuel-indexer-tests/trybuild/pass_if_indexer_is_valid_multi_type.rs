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

    fn function_two(_event: SomeEvent, event2: AnotherEvent) {
        let AnotherEvent { id, account, hash } = event2;

        assert_eq!(id, 9);
        assert_eq!(account, Bits256([48u8; 32]));
        assert_eq!(hash, Bits256([56u8; 32]));
    }
}

fn main() {
    use fuels::core::codec::ABIEncoder;

    let s1 = SomeEvent {
        id: 9,
        account: Bits256([48u8; 32]),
    };

    let s2 = AnotherEvent {
        id: 9,
        account: Bits256([48u8; 32]),
        hash: Bits256([56u8; 32]),
    };

    let encoded1 = ABIEncoder::encode(&[s1.into_token()]).expect("Failed compile test");
    let bytes1 = encoded1.resolve(0);
    let encoded2 = ABIEncoder::encode(&[s2.into_token()]).expect("Failed compile test");
    let bytes2 = encoded2.resolve(0);

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
            id: [0u8; 32].into(),
            status: fuel::TransactionStatus::default(),
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
                    len: bytes1.len() as u64,
                    digest: [0u8; 32].into(),
                    data: Some(bytes1),
                    pc: 0,
                    is: 0,
                },
                fuel::Receipt::Call {
                    id: [0u8; 32].into(),
                    to: [0u8; 32].into(),
                    amount: 400,
                    asset_id: [0u8; 32].into(),
                    gas: 4,
                    param1: 2379805026,
                    param2: 0,
                    pc: 0,
                    is: 0,
                },
                fuel::Receipt::ReturnData {
                    id: [0u8; 32].into(),
                    ptr: 2342143,
                    len: bytes2.len() as u64,
                    digest: [0u8; 32].into(),
                    data: Some(bytes2),
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
