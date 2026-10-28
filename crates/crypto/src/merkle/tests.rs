use homeostasis_primitives::Hash32;
use proptest::prelude::*;

use crate::hash::{Domain, hash};

use super::merkle_root;

// Perhitungan independen daun dan simpul Merkle memakai b3sum 1.8.7 dan xxd 2025-11-26:
//
// LEAF_CTX="homeostasis 2026-10-28 merkle leaf"
// NODE_CTX="homeostasis 2026-10-28 merkle node"
// I_k = 32 byte bernilai k (01..01 untuk k=1, ..., 05..05 untuk k=5)
//
// L1 = printf '01%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = b6415d9fb0cf8fdc5abb0cc223d2f94705b6b9a4f26d325477121c88c611ad68
// L2 = printf '02%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = 8acb95fba6a5705eddf20e37a420bb39e72a1c7ed3e9e4fbb5e61a46b474aa13
// L3 = printf '03%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = e8aae81dc05b4213caf2874867d04d2cd69b96953a41782ecb8053671e23d08f
// L4 = printf '04%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = 68db0fe1ec316885a16a13a25dc4250701d189ee1dfddcc6b6717a33cbfa5149
// L5 = printf '05%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = 6141b16f2d20fcbe64da37d19b32063ed9dc6676a6d2ba337ab50e1a52a90522
//
// N12   = printf '%s%s' "$L1" "$L2" | xxd -r -p | b3sum --derive-key "$NODE_CTX" --no-names
//       = 7ad28d062048aeb6d004a942a08253a22f5b418fa5ca97c58a8c9c928a6bc50d
// N34   = printf '%s%s' "$L3" "$L4" | xxd -r -p | b3sum --derive-key "$NODE_CTX" --no-names
//       = 57bb448ab067724c5216c6218590ca067e169be162329d87aecaa3d569683535
// N1234 = printf '%s%s' "$N12" "$N34" | xxd -r -p | b3sum --derive-key "$NODE_CTX" --no-names
//       = 6534e51cc382f3afdb7a28ab8033e6f0da7fa53913fc7e80a9eccf12668ee28c

// 1 item: root = L1
// printf '0101010101010101010101010101010101010101010101010101010101010101' | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 merkle leaf" --no-names
const MERKLE_ROOT_1_HEX: &str = "b6415d9fb0cf8fdc5abb0cc223d2f94705b6b9a4f26d325477121c88c611ad68";

// 2 item: root = N12 = H_node(L1 || L2)
// printf '%s%s' "$L1" "$L2" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 merkle node" --no-names
const MERKLE_ROOT_2_HEX: &str = "7ad28d062048aeb6d004a942a08253a22f5b418fa5ca97c58a8c9c928a6bc50d";

// 3 item: tingkat 0 = [L1, L2, L3] -> tingkat 1 = [N12, L3] (L3 dinaikkan apa adanya) -> root = H_node(N12 || L3)
// printf '%s%s' "$N12" "$L3" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 merkle node" --no-names
const MERKLE_ROOT_3_HEX: &str = "35f1f982675829affdb5269b53084c01fd9d2aae508074f7b391bb2eb21f7c80";

// 5 item: tingkat 0 = [L1..L5] -> tingkat 1 = [N12, N34, L5] -> tingkat 2 = [N1234, L5] -> root = H_node(N1234 || L5)
// printf '%s%s' "$N1234" "$L5" | xxd -r -p | b3sum --derive-key "homeostasis 2026-10-28 merkle node" --no-names
const MERKLE_ROOT_5_HEX: &str = "996fb40ce7b1b2137c1c4ef8800ce584e98cafefe9d68de4a5bf292c3a55d7e2";

fn item_repeated(val: u8) -> Hash32 {
    Hash32::from_bytes([val; 32])
}

#[test]
fn empty_items_returns_none() {
    assert_eq!(merkle_root(&[]), None);
}

#[test]
fn b3sum_merkle_vectors_for_1_2_3_and_5_items() {
    let i1 = item_repeated(1);
    let i2 = item_repeated(2);
    let i3 = item_repeated(3);
    let i4 = item_repeated(4);
    let i5 = item_repeated(5);

    assert_eq!(
        merkle_root(&[i1]),
        Some(Hash32::from_hex(MERKLE_ROOT_1_HEX).unwrap())
    );
    assert_eq!(
        merkle_root(&[i1, i2]),
        Some(Hash32::from_hex(MERKLE_ROOT_2_HEX).unwrap())
    );
    assert_eq!(
        merkle_root(&[i1, i2, i3]),
        Some(Hash32::from_hex(MERKLE_ROOT_3_HEX).unwrap())
    );
    assert_eq!(
        merkle_root(&[i1, i2, i3, i4, i5]),
        Some(Hash32::from_hex(MERKLE_ROOT_5_HEX).unwrap())
    );
}

#[test]
fn merkle_order_matters_and_odd_leaf_is_not_duplicated() {
    let a = item_repeated(0x0a);
    let b = item_repeated(0x0b);
    let c = item_repeated(0x0c);

    assert_ne!(merkle_root(&[a, b]), merkle_root(&[b, a]));
    assert_ne!(merkle_root(&[a, b, c]), merkle_root(&[a, c, b]));

    // Bukti eksplisit tidak ada penggandaan simpul ganjil (mencegah CVE-2012-2459).
    assert_ne!(merkle_root(&[a, b, c]), merkle_root(&[a, b, c, c]));

    // Untuk 2 item, merkle_root wajib sama persis dengan H(MerkleNode, H(MerkleLeaf, a) || H(MerkleLeaf, b)).
    let leaf_a = hash(Domain::MerkleLeaf, a.as_bytes());
    let leaf_b = hash(Domain::MerkleLeaf, b.as_bytes());
    let mut joined = Vec::with_capacity(64);
    joined.extend_from_slice(leaf_a.as_bytes());
    joined.extend_from_slice(leaf_b.as_bytes());
    assert_eq!(
        merkle_root(&[a, b]),
        Some(hash(Domain::MerkleNode, &joined))
    );
}

proptest! {
    #[test]
    fn prop_no_cve_2012_2459_odd_duplication(
        a in any::<[u8; 32]>(),
        b in any::<[u8; 32]>(),
        c in any::<[u8; 32]>(),
    ) {
        let ha = Hash32::from_bytes(a);
        let hb = Hash32::from_bytes(b);
        let hc = Hash32::from_bytes(c);
        prop_assert_ne!(merkle_root(&[ha, hb, hc]), merkle_root(&[ha, hb, hc, hc]));
    }
}
