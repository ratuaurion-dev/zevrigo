use proptest::prelude::*;

use super::{OutPoint, UtxoEntry};
use crate::amount::Amount;
use crate::hash::Hash32;
use crate::height::BlockHeight;
use crate::tx::{Input, Output, OutputKind};

#[test]
fn outpoint_new_and_from_input_preserve_txid_and_index() {
    let txid1 = Hash32::from_bytes([0x11; 32]);
    let op1 = OutPoint::new(txid1, 7);
    assert_eq!(op1.txid(), txid1);
    assert_eq!(op1.index(), 7);

    let txid2 = Hash32::from_bytes([0x22; 32]);
    let input = Input::new(txid2, 3);
    let op2 = OutPoint::from_input(&input);
    assert_eq!(op2, OutPoint::new(txid2, 3));
}

proptest! {
    #[test]
    fn prop_outpoint_ord_is_txid_then_index(
        ta in any::<[u8; 32]>(),
        ia in any::<u32>(),
        tb in any::<[u8; 32]>(),
        ib in any::<u32>(),
    ) {
        prop_assert_eq!(
            OutPoint::new(Hash32::from_bytes(ta), ia).cmp(&OutPoint::new(Hash32::from_bytes(tb), ib)),
            (ta, ia).cmp(&(tb, ib))
        );
    }
}

#[test]
fn utxo_entry_getters_return_constructor_values() {
    let output =
        Output::new(Amount::from_quanta(10), OutputKind::SingleKey, [0x01; 32]).expect("output");

    let entry1 = UtxoEntry::new(output.clone(), BlockHeight::new(0), true);
    assert_eq!(entry1.output(), &output);
    assert_eq!(entry1.height(), BlockHeight::new(0));
    assert!(entry1.is_coinbase());

    let entry2 = UtxoEntry::new(output.clone(), BlockHeight::new(5), false);
    assert_eq!(entry2.output(), &output);
    assert_eq!(entry2.height(), BlockHeight::new(5));
    assert!(!entry2.is_coinbase());
}
