use std::cell::Cell;

use homeostasis_pow::{RandomxVerifier, key_height, mine_regtest};
use homeostasis_primitives::{BlockHeight, Hash32, Header, Network};

use super::{RandomxHasher, randomx_hasher, randomx_key_height};
use crate::chain_params::ChainParams;
use crate::difficulty::decode_compact;
use crate::validation::{BlockError, check_header_context_free, check_header_pow};

/// `pow_limit` dan target genesis regtest (dipakai juga oleh parameter uji testnet).
const LIMIT: u32 = 0x207f_ffff;

/// Penghitung uji: selalu mengembalikan `hash`, dan mencatat kunci terakhir yang diminta.
struct Fixed {
    hash: Option<Hash32>,
    seen: Cell<Option<Hash32>>,
}

impl Fixed {
    fn new(hash: Option<Hash32>) -> Self {
        Self {
            hash,
            seen: Cell::new(None),
        }
    }
}

impl RandomxHasher for Fixed {
    fn randomx_hash(&self, key: &Hash32, _header: &Header) -> Option<Hash32> {
        self.seen.set(Some(*key));
        self.hash
    }
}

/// Parameter regtest dengan jaringan `network`: target dan genesis sama, PoW mengikuti jaringan.
fn params_for(network: Network) -> ChainParams {
    let regtest = ChainParams::regtest().unwrap();
    ChainParams::new(
        network,
        regtest.pow_limit(),
        regtest.genesis_target(),
        regtest.genesis_timestamp(),
        regtest.network_id(),
    )
}

fn header(target: u32, nonce: u64) -> Header {
    Header::new(
        BlockHeight::new(1),
        Hash32::from_bytes([0x11; 32]),
        Hash32::from_bytes([0x22; 32]),
        1_800_000_000,
        target,
        nonce,
    )
}

#[test]
fn key_height_is_the_pow_rule() {
    for height in [0_u64, 2_048, 2_112, 2_113, 4_161, u64::MAX] {
        let height = BlockHeight::new(height);
        assert_eq!(randomx_key_height(height), key_height(height));
    }
    assert_eq!(
        randomx_key_height(BlockHeight::new(2_113)),
        BlockHeight::new(2_048)
    );
}

#[test]
fn testnet_and_mainnet_check_randomx_hash_with_key() {
    let key = Hash32::from_bytes([0x33; 32]);
    let low = Fixed::new(Some(Hash32::ZERO));
    let high = Fixed::new(Some(Hash32::from_bytes([0xff; 32])));
    let failed = Fixed::new(None);
    // Header ini lolos PoW regtest; itu tidak berarti apa pun di testnet dan mainnet (DEC-09).
    let regtest_pow = mine_regtest(&header(LIMIT, 0), &decode_compact(LIMIT).unwrap(), 64).unwrap();
    for network in [Network::Testnet, Network::Mainnet] {
        let params = params_for(network);
        assert_eq!(
            check_header_pow(&regtest_pow, &params, &key, Some(&low)),
            Ok(())
        );
        assert_eq!(low.seen.get(), Some(key));
        assert_eq!(
            check_header_pow(&regtest_pow, &params, &key, Some(&high)),
            Err(BlockError::HighHash)
        );
        assert_eq!(
            check_header_pow(&regtest_pow, &params, &key, Some(&failed)),
            Err(BlockError::PowHashFailed)
        );
        assert_eq!(
            check_header_pow(&regtest_pow, &params, &key, None),
            Err(BlockError::PowUnavailable { network })
        );
        assert_eq!(
            check_header_context_free(&regtest_pow, &params),
            Err(BlockError::PowUnavailable { network })
        );
    }
    // V2 lebih dulu: target di atas pow_limit ditolak sebelum hash dihitung.
    let unused = Fixed::new(Some(Hash32::ZERO));
    assert_eq!(
        check_header_pow(
            &header(0x20ff_ffff, 0),
            &params_for(Network::Testnet),
            &key,
            Some(&unused)
        ),
        Err(BlockError::TargetAboveLimit)
    );
    assert_eq!(unused.seen.get(), None);
    assert_eq!(
        BlockError::PowHashFailed.to_string(),
        "hash PoW RandomX gagal dihitung"
    );
}

#[test]
fn regtest_ignores_key_and_hasher() {
    let regtest = params_for(Network::Regtest);
    let mined = mine_regtest(&header(LIMIT, 0), &regtest.pow_limit(), 64).unwrap();
    let high = Fixed::new(Some(Hash32::from_bytes([0xff; 32])));
    assert_eq!(
        check_header_pow(&mined, &regtest, &Hash32::ZERO, Some(&high)),
        Ok(())
    );
    assert_eq!(high.seen.get(), None);
    assert_eq!(check_header_context_free(&mined, &regtest), Ok(()));
}

#[test]
fn service_hash_decides_testnet_pow() {
    let hasher = randomx_hasher().unwrap();
    let shared: &dyn RandomxHasher = hasher.as_ref();
    let key = Hash32::from_bytes([0x33; 32]);
    let verifier = RandomxVerifier::new(key).unwrap();
    let params = params_for(Network::Testnet);
    let target = decode_compact(LIMIT).unwrap();
    let mut passed = 0_u32;
    let mut failed = 0_u32;
    for nonce in 0..16_u64 {
        let candidate = header(LIMIT, nonce);
        let expected = verifier.hash(&candidate).unwrap();
        assert_eq!(shared.randomx_hash(&key, &candidate), Some(expected));
        let result = check_header_pow(&candidate, &params, &key, Some(shared));
        if target.is_met_by(&expected) {
            assert_eq!(result, Ok(()));
            passed = passed.saturating_add(1);
        } else {
            assert_eq!(result, Err(BlockError::HighHash));
            failed = failed.saturating_add(1);
        }
    }
    assert!(passed > 0 && failed > 0, "lolos {passed}, gagal {failed}");
}
