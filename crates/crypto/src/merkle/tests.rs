use proptest::prelude::*;
use zevrigo_primitives::Hash32;

use crate::hash::{Domain, hash};

use super::merkle_root;

// Perhitungan independen daun dan simpul Merkle memakai b3sum 1.8.7 dan xxd 2025-11-26:
//
// LEAF_CTX="zevrigo 2026-10-28 merkle leaf"
// NODE_CTX="zevrigo 2026-10-28 merkle node"
// I_k = 32 byte bernilai k (01..01 untuk k=1, ..., 05..05 untuk k=5)
//
// L1 = printf '01%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = c5146123eff079f1ca04e4edaec33545697d4b94e8d1b2f4b8d05ed4d9de7627
// L2 = printf '02%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = 5198d3484237af471dd1468ab53a74298d3a8e6479d30a8feef4e90c2f32b6aa
// L3 = printf '03%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = e0e4558785ee10ff3c314b889584eeaf64e3a25e05ea1a18e2eaf628e00f5d25
// L4 = printf '04%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = 82e11e6ffaa82ede2f39f20295a0f6086edae1070f5d8e13fc574f22c3ffc00c
// L5 = printf '05%.0s' {1..32} | xxd -r -p | b3sum --derive-key "$LEAF_CTX" --no-names
//    = b2750c060f2751fc1a9f8f61822a1715452d0b80553b3289d287bb92b0fb7117
//
// N12   = printf '%s%s' "$L1" "$L2" | xxd -r -p | b3sum --derive-key "$NODE_CTX" --no-names
//       = 7385faf467f446d4c77dfd766cbbf7528340ad7c03a853440305c74d64a3dfbc
// N34   = printf '%s%s' "$L3" "$L4" | xxd -r -p | b3sum --derive-key "$NODE_CTX" --no-names
//       = 11255f14e6f3df50c9837f1a1fa7a6c9d29bdbd94996d0050882829ecbc1c65a
// N1234 = printf '%s%s' "$N12" "$N34" | xxd -r -p | b3sum --derive-key "$NODE_CTX" --no-names
//       = 7b4090fe1d80314c0471f1072916bd98a544cf9171740fc2c1171a5bccaaf770

// 1 item: root = L1
// printf '0101010101010101010101010101010101010101010101010101010101010101' | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 merkle leaf" --no-names
const MERKLE_ROOT_1_HEX: &str = "c5146123eff079f1ca04e4edaec33545697d4b94e8d1b2f4b8d05ed4d9de7627";

// 2 item: root = N12 = H_node(L1 || L2)
// printf '%s%s' "$L1" "$L2" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 merkle node" --no-names
const MERKLE_ROOT_2_HEX: &str = "7385faf467f446d4c77dfd766cbbf7528340ad7c03a853440305c74d64a3dfbc";

// 3 item: tingkat 0 = [L1, L2, L3] -> tingkat 1 = [N12, L3] (L3 dinaikkan apa adanya) -> root = H_node(N12 || L3)
// printf '%s%s' "$N12" "$L3" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 merkle node" --no-names
const MERKLE_ROOT_3_HEX: &str = "4a2a690b024a69ae693ccc7c37b2f39eb69222042affe19adcf66ce731141f7f";

// 5 item: tingkat 0 = [L1..L5] -> tingkat 1 = [N12, N34, L5] -> tingkat 2 = [N1234, L5] -> root = H_node(N1234 || L5)
// printf '%s%s' "$N1234" "$L5" | xxd -r -p | b3sum --derive-key "zevrigo 2026-10-28 merkle node" --no-names
const MERKLE_ROOT_5_HEX: &str = "350a035fd86f4a9f694b6b1204bdf2c5ff7760bf1e6a110d3f63380c664fae61";

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
