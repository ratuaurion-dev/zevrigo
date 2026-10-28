use crate::{
    Amount, Block, BlockHeight, Decode, Encode, GENESIS_EXTRA_LEN, HEADER_SIZE, HEADER_VERSION,
    Hash32, Header, MODULE_ID, Output, OutputKind, QUANTA_PER_HSIS, Reader, Transaction, params,
};

#[test]
fn module_id_matches() {
    assert_eq!(MODULE_ID, "MOD-01");
}

#[test]
fn crate_root_reexports_are_accessible() {
    assert_eq!(Amount::from_quanta(QUANTA_PER_HSIS), params::TAIL_REWARD);
    assert_eq!(BlockHeight::GENESIS.get(), 0);
    assert_eq!(Hash32::ZERO.as_bytes().len(), 32);
    assert_eq!(HEADER_SIZE, 96);
    assert_eq!(HEADER_VERSION, 1);
    assert_eq!(GENESIS_EXTRA_LEN, 32);

    let out = Output::new(Amount::from_quanta(1), OutputKind::SingleKey, [0x11; 32])
        .expect("output valid");
    let tx = Transaction::coinbase(BlockHeight::new(1), vec![], vec![out]).expect("tx valid");
    let header = Header::new(
        BlockHeight::new(1),
        Hash32::ZERO,
        Hash32::ZERO,
        1_800_000_000,
        0x1d00_ffff,
        0,
    );
    let block = Block::new(header, vec![tx]).expect("block valid");
    let encoded = block.to_bytes();
    let mut reader = Reader::new(&encoded);
    let decoded = Block::decode_from(&mut reader).expect("decode valid");
    assert_eq!(decoded, block);
}
