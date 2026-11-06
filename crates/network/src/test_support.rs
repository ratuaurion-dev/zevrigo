//! Data contoh bersama untuk test MOD-06 (struktur sah menurut MOD-01, bukan blok yang lolos validasi konsensus).

use zevrigo_primitives::{
    Amount, Block, BlockHeight, Hash32, Header, Input, Output, OutputKind, Transaction, Witness,
};

/// Hash 32 byte yang semua byte-nya `byte`.
pub(crate) const fn id(byte: u8) -> Hash32 {
    Hash32::from_bytes([byte; 32])
}

/// Header contoh di tinggi `height`.
pub(crate) const fn sample_header(height: u64) -> Header {
    Header::new(
        BlockHeight::new(height),
        id(1),
        id(2),
        1_700_000_000,
        0x207f_ffff,
        9,
    )
}

fn sample_output() -> Output {
    Output::new(Amount::from_quanta(5), OutputKind::SingleKey, [7; 32]).unwrap()
}

/// Blok contoh di tinggi `height` (bukan genesis) berisi satu coinbase.
pub(crate) fn sample_block(height: u64) -> Block {
    let coinbase =
        Transaction::coinbase(BlockHeight::new(height), Vec::new(), vec![sample_output()]).unwrap();
    Block::new(sample_header(height), vec![coinbase]).unwrap()
}

/// Transaksi biasa contoh dengan satu input dan satu output.
pub(crate) fn sample_tx() -> Transaction {
    Transaction::regular(
        vec![Input::new(id(3), 0)],
        vec![sample_output()],
        vec![Witness::single_key([4; 64])],
    )
    .unwrap()
}
