use proptest::prelude::*;
use zevrigo_primitives::Hash32;

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

// printf '' | b3sum --derive-key "zevrigo 2026-10-28 txid" --no-names
const TXID_EMPTY_HEX: &str = "8bc30b9ba42086e8dfb75905912f3f58fc47ce8c277a060adbfce58839bde22b";
// printf 'abc' | b3sum --derive-key "zevrigo 2026-10-28 txid" --no-names
const TXID_ABC_HEX: &str = "ae1600253f094bf35357b3e79ecdff82313f449266011cf3a7e1d89faf968ac4";

// printf '' | b3sum --derive-key "zevrigo 2026-10-28 wtxid" --no-names
const WTXID_EMPTY_HEX: &str = "e1b2a320a39722886304e1e7f2365d8ef04be2557c9c0a48b9bc9defd2a87978";
// printf 'abc' | b3sum --derive-key "zevrigo 2026-10-28 wtxid" --no-names
const WTXID_ABC_HEX: &str = "c31e00671f29f0e11b12f474cccfa2f49cdeb93bae75b74843f4216e73c5a618";

// printf '' | b3sum --derive-key "zevrigo 2026-10-28 merkle leaf" --no-names
const MERKLE_LEAF_EMPTY_HEX: &str =
    "5d58e397cbc0769fe7fe303ad752217c6de08b70b351044b282de7e6c1968a06";
// printf 'abc' | b3sum --derive-key "zevrigo 2026-10-28 merkle leaf" --no-names
const MERKLE_LEAF_ABC_HEX: &str =
    "23720bccaad09e0580985bdbdba56d690b3affc6c0074119f8c8726f92c5fdc6";

// printf '' | b3sum --derive-key "zevrigo 2026-10-28 merkle node" --no-names
const MERKLE_NODE_EMPTY_HEX: &str =
    "9751edfe56a7ed85f9686d07d39a24fe26d5de8207f8974b425492fd079d0405";
// printf 'abc' | b3sum --derive-key "zevrigo 2026-10-28 merkle node" --no-names
const MERKLE_NODE_ABC_HEX: &str =
    "4e8d7ea4fa08dabbd625230b228d9224d17c26fd7536af1a85c31c0218c5b293";

// printf '' | b3sum --derive-key "zevrigo 2026-10-28 block id" --no-names
const BLOCK_ID_EMPTY_HEX: &str = "f0c94c17252f6496627712335c20a48abef6fa3f9e530a35a7f23259bfacc80c";
// printf 'abc' | b3sum --derive-key "zevrigo 2026-10-28 block id" --no-names
const BLOCK_ID_ABC_HEX: &str = "c36d376cbd22e83c603814a3300c4990436a18ea1036352d0b0888c630cef2da";

// printf '' | b3sum --derive-key "zevrigo 2026-10-28 condition" --no-names
const CONDITION_EMPTY_HEX: &str =
    "06fb9e15398f0810e5fff6a02fb18bb918cc8f2e3fa002c3ca909951b2086078";
// printf 'abc' | b3sum --derive-key "zevrigo 2026-10-28 condition" --no-names
const CONDITION_ABC_HEX: &str = "0ff93f61dd85d3317d2c4e2bfc4dc91f7fb1a74b21496c2c0f05f0adde5ddef5";

// printf '' | b3sum --derive-key "zevrigo 2026-10-28 sighash v1" --no-names
const SIGHASH_EMPTY_HEX: &str = "24624b7cb3bfb9184e57b7ac75a1b2ea07fb15a69b9253c6879637a7d9d48e7c";
// printf 'abc' | b3sum --derive-key "zevrigo 2026-10-28 sighash v1" --no-names
const SIGHASH_ABC_HEX: &str = "ef9da163b99259dc81231014fe770462ce8b66ab7e30db397fde2c03299b9bfa";

// printf '' | b3sum --derive-key "zevrigo 2026-10-28 regtest pow" --no-names
const REGTEST_POW_EMPTY_HEX: &str =
    "1aceb0cccceb4072e2d58c4b73b8d3e41ee15f6e8bf87808b7e2d39cb6b64126";
// printf 'abc' | b3sum --derive-key "zevrigo 2026-10-28 regtest pow" --no-names
const REGTEST_POW_ABC_HEX: &str =
    "8bbb3b67ab30c8fe27b0064b6b7abd24816f0324b35f93d68d106f2ef607809e";

#[test]
fn domain_context_strings_match_spec_and_are_unique() {
    let expected: [(Domain, &str); 8] = [
        (Domain::Txid, "zevrigo 2026-10-28 txid"),
        (Domain::Wtxid, "zevrigo 2026-10-28 wtxid"),
        (Domain::MerkleLeaf, "zevrigo 2026-10-28 merkle leaf"),
        (Domain::MerkleNode, "zevrigo 2026-10-28 merkle node"),
        (Domain::BlockId, "zevrigo 2026-10-28 block id"),
        (Domain::Condition, "zevrigo 2026-10-28 condition"),
        (Domain::Sighash, "zevrigo 2026-10-28 sighash v1"),
        (Domain::RegtestPow, "zevrigo 2026-10-28 regtest pow"),
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
