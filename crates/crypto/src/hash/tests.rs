use homeostasis_primitives::Hash32;
use proptest::prelude::*;

use super::{Domain, DomainHasher, hash};

const ALL_DOMAINS: [Domain; 8] = [
    Domain::Txid,
    Domain::Wtxid,
    Domain::MerkleLeaf,
    Domain::MerkleNode,
    Domain::BlockId,
    Domain::Condition,
    Domain::Sighash,
    Domain::RegtestPow,
];

// printf '' | b3sum --derive-key "homeostasis 2026-10-28 txid" --no-names
const TXID_EMPTY_HEX: &str = "8ba1ce3a2abbc0dfe734c703d2eaa3a9c9a144107b9c0be8208f64289b1ef66f";
// printf 'abc' | b3sum --derive-key "homeostasis 2026-10-28 txid" --no-names
const TXID_ABC_HEX: &str = "f9ff4780e515cd6d0f63210ddd34f5cd7d3ee0e1de0c601550bca3f8a5d0179e";

// printf '' | b3sum --derive-key "homeostasis 2026-10-28 wtxid" --no-names
const WTXID_EMPTY_HEX: &str = "a5a42ee2f4aab6da974a74d171350116bf638580c5c8a60f829f47f61f541d46";
// printf 'abc' | b3sum --derive-key "homeostasis 2026-10-28 wtxid" --no-names
const WTXID_ABC_HEX: &str = "bf940fb7b9915c1f959b606192f099cf5afcb02cb042e877ecb04cfc9fcf063d";

// printf '' | b3sum --derive-key "homeostasis 2026-10-28 merkle leaf" --no-names
const MERKLE_LEAF_EMPTY_HEX: &str =
    "3fec184c7d7ebff9a62c94d70811960e046b807326ec54ec84b2f617cb3361c1";
// printf 'abc' | b3sum --derive-key "homeostasis 2026-10-28 merkle leaf" --no-names
const MERKLE_LEAF_ABC_HEX: &str =
    "53207aebd3c202a6c2d9efa1d3ec83558f5d93bc28646d3ccf03a55948b6b74a";

// printf '' | b3sum --derive-key "homeostasis 2026-10-28 merkle node" --no-names
const MERKLE_NODE_EMPTY_HEX: &str =
    "20ac9e2d28ae66d1c336ea1cebe6a88e601d3c74b86cf95043f277904b0e7406";
// printf 'abc' | b3sum --derive-key "homeostasis 2026-10-28 merkle node" --no-names
const MERKLE_NODE_ABC_HEX: &str =
    "34120f928c772193e55bd0c5ac1155e2312c4def275de64f47a9b54547443be3";

// printf '' | b3sum --derive-key "homeostasis 2026-10-28 block id" --no-names
const BLOCK_ID_EMPTY_HEX: &str = "1d8d47c81549967c6ac644932bcb9459e76680ee5d49644edf2a76d76e1c944e";
// printf 'abc' | b3sum --derive-key "homeostasis 2026-10-28 block id" --no-names
const BLOCK_ID_ABC_HEX: &str = "7e5fede80be9f6938dc8a559e0acdbece92c6898b588dd7f79bdef8f08ab5c8a";

// printf '' | b3sum --derive-key "homeostasis 2026-10-28 condition" --no-names
const CONDITION_EMPTY_HEX: &str =
    "0d4129c2af924f6f06e538841c09c13e99cc0ca38a596f04e83477390b76b87a";
// printf 'abc' | b3sum --derive-key "homeostasis 2026-10-28 condition" --no-names
const CONDITION_ABC_HEX: &str = "babab9792a38d67e97865552ce6dd70d8c685a3ecf38c4797b49905d7512cb0c";

// printf '' | b3sum --derive-key "homeostasis 2026-10-28 sighash v1" --no-names
const SIGHASH_EMPTY_HEX: &str = "b82f298bb64ff0f7212b04af4ba0d10df60f63509c6272c6ec989802b8b75460";
// printf 'abc' | b3sum --derive-key "homeostasis 2026-10-28 sighash v1" --no-names
const SIGHASH_ABC_HEX: &str = "1c253a6dc26d68b8ce19062f2242292cabb9a54ebb55dcb3c0ad02b26c30db6a";

// printf '' | b3sum --derive-key "homeostasis 2026-10-28 regtest pow" --no-names
const REGTEST_POW_EMPTY_HEX: &str =
    "e23cdaa829593b6b3cc7a3851de25587bf9143626246f016cd28cb05f09e2b0e";
// printf 'abc' | b3sum --derive-key "homeostasis 2026-10-28 regtest pow" --no-names
const REGTEST_POW_ABC_HEX: &str =
    "bdcdd6f8411d12b2243b06a52f53bb7196ac7c120f81674d1d1381c7deb4d64e";

#[test]
fn domain_context_strings_match_spec_and_are_unique() {
    let expected: [(Domain, &str); 8] = [
        (Domain::Txid, "homeostasis 2026-10-28 txid"),
        (Domain::Wtxid, "homeostasis 2026-10-28 wtxid"),
        (Domain::MerkleLeaf, "homeostasis 2026-10-28 merkle leaf"),
        (Domain::MerkleNode, "homeostasis 2026-10-28 merkle node"),
        (Domain::BlockId, "homeostasis 2026-10-28 block id"),
        (Domain::Condition, "homeostasis 2026-10-28 condition"),
        (Domain::Sighash, "homeostasis 2026-10-28 sighash v1"),
        (Domain::RegtestPow, "homeostasis 2026-10-28 regtest pow"),
    ];

    for (domain, text) in expected {
        assert_eq!(domain.context(), text);
    }

    for (i, d1) in ALL_DOMAINS.iter().enumerate() {
        for (j, d2) in ALL_DOMAINS.iter().enumerate() {
            if i != j {
                assert_ne!(d1.context(), d2.context());
            }
        }
    }
}

#[test]
fn b3sum_domain_vectors_match_empty_and_abc() {
    let vectors: [(Domain, &str, &str); 8] = [
        (Domain::Txid, TXID_EMPTY_HEX, TXID_ABC_HEX),
        (Domain::Wtxid, WTXID_EMPTY_HEX, WTXID_ABC_HEX),
        (
            Domain::MerkleLeaf,
            MERKLE_LEAF_EMPTY_HEX,
            MERKLE_LEAF_ABC_HEX,
        ),
        (
            Domain::MerkleNode,
            MERKLE_NODE_EMPTY_HEX,
            MERKLE_NODE_ABC_HEX,
        ),
        (Domain::BlockId, BLOCK_ID_EMPTY_HEX, BLOCK_ID_ABC_HEX),
        (Domain::Condition, CONDITION_EMPTY_HEX, CONDITION_ABC_HEX),
        (Domain::Sighash, SIGHASH_EMPTY_HEX, SIGHASH_ABC_HEX),
        (
            Domain::RegtestPow,
            REGTEST_POW_EMPTY_HEX,
            REGTEST_POW_ABC_HEX,
        ),
    ];

    for (domain, expected_empty, expected_abc) in vectors {
        let expected_empty_hash = Hash32::from_hex(expected_empty).unwrap();
        let expected_abc_hash = Hash32::from_hex(expected_abc).unwrap();

        assert_eq!(hash(domain, b""), expected_empty_hash);
        assert_eq!(hash(domain, b"abc"), expected_abc_hash);

        let empty_hasher = DomainHasher::new(domain);
        assert_eq!(empty_hasher.finalize(), expected_empty_hash);

        let mut abc_hasher = DomainHasher::new(domain);
        abc_hasher.update(b"a").update(b"bc");
        assert_eq!(abc_hasher.finalize(), expected_abc_hash);
    }
}

#[test]
fn different_domains_produce_different_hashes_on_same_input() {
    for sample in [b"".as_slice(), b"abc".as_slice(), &[0x42_u8; 64]] {
        for (i, &d1) in ALL_DOMAINS.iter().enumerate() {
            for (j, &d2) in ALL_DOMAINS.iter().enumerate() {
                if i != j {
                    assert_ne!(hash(d1, sample), hash(d2, sample));
                }
            }
        }
    }
}

fn domain_from_index(idx: usize) -> Domain {
    match idx {
        0 => Domain::Txid,
        1 => Domain::Wtxid,
        2 => Domain::MerkleLeaf,
        3 => Domain::MerkleNode,
        4 => Domain::BlockId,
        5 => Domain::Condition,
        6 => Domain::Sighash,
        _ => Domain::RegtestPow,
    }
}

proptest! {
    #[test]
    fn prop_domain_hasher_streaming_matches_one_shot(
        domain_idx in 0_usize..8_usize,
        data in proptest::collection::vec(any::<u8>(), 0..=512),
        mut cuts in proptest::collection::vec(0_usize..=512_usize, 0..=16),
    ) {
        let domain = domain_from_index(domain_idx);
        let len = data.len();

        for cut in &mut cuts {
            if *cut > len {
                *cut = len;
            }
        }
        cuts.sort_unstable();
        cuts.dedup();

        let mut hasher = DomainHasher::new(domain);
        let mut cursor: usize = 0;

        for cut in cuts {
            if let Some(piece) = data.get(cursor..cut) {
                hasher.update(piece);
            }
            cursor = cut;
        }
        if let Some(tail) = data.get(cursor..len) {
            hasher.update(tail);
        }

        prop_assert_eq!(hasher.finalize(), hash(domain, &data));
    }
}
