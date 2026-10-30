use core::fmt::Write as _;

use homeostasis_crypto::{SecretKey, block_id, txid, wtxid};
use homeostasis_primitives::params::GENESIS_ALLOCATION;
use homeostasis_primitives::{BlockHeight, Encode, Hash32, Network, OutputKind};

use super::{
    REGTEST_GENESIS_KEY, REGTEST_GENESIS_TARGET_COMPACT, REGTEST_GENESIS_TIMESTAMP,
    REGTEST_POW_LIMIT_COMPACT, pow_limit, pow_limit_compact, regtest_genesis, regtest_network_id,
};
use crate::difficulty::{DifficultyError, decode_compact};

/// Encoding lengkap blok genesis regtest, 186 byte (Spesifikasi Validasi Bagian 10).
const GENESIS_BLOCK_HEX: &str = concat!(
    "01000000",
    "0000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "66255b2a4e2a87346a7ce7f57078c1325e444a50738f75ef8a6e6246c5c088d6",
    "003be16a00000000",
    "ffff7f20",
    "0000000000000000",
    "01",
    "0101000000000000000020",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "01808080cccfadc9ccc680fe1c00",
    "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
);

const GENESIS_TXID: &str = "d95c2ed4614a2eacece319bf129078c55a74b63fe555b94eeb0b3d1320814875";
const GENESIS_WTXID: &str = "f0ea325014f4a3f4609f890c2379a1bed228d137959d795806fc136ea5bda614";
const GENESIS_MERKLE_ROOT: &str =
    "66255b2a4e2a87346a7ce7f57078c1325e444a50738f75ef8a6e6246c5c088d6";
const REGTEST_NETWORK_ID: &str = "b933d7c548dc78459ac7376499682cdb9b21ef93f732842ccc89c5d9a16a04c5";

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::new();
    for byte in bytes {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

#[test]
fn regtest_genesis_matches_spec_vectors() {
    let genesis = regtest_genesis().unwrap();
    let header = genesis.header();
    assert_eq!(header.height(), BlockHeight::GENESIS);
    assert_eq!(header.prev_block_id(), Hash32::ZERO);
    assert_eq!(header.timestamp(), REGTEST_GENESIS_TIMESTAMP);
    assert_eq!(header.timestamp(), 1_793_145_600);
    assert_eq!(header.target(), REGTEST_GENESIS_TARGET_COMPACT);
    assert_eq!(header.nonce(), 0);
    assert_eq!(header.merkle_root().to_string(), GENESIS_MERKLE_ROOT);

    assert_eq!(genesis.txs().len(), 1);
    let coinbase = genesis.txs().first().unwrap();
    assert!(coinbase.is_coinbase());
    assert_eq!(txid(coinbase).to_string(), GENESIS_TXID);
    assert_eq!(wtxid(coinbase).to_string(), GENESIS_WTXID);
    let outputs = coinbase.body().outputs();
    assert_eq!(outputs.len(), 1);
    let allocation = outputs.first().unwrap();
    assert_eq!(allocation.amount(), GENESIS_ALLOCATION);
    assert_eq!(allocation.kind(), OutputKind::SingleKey);
    assert_eq!(allocation.payload(), &REGTEST_GENESIS_KEY);

    let encoded = genesis.to_bytes();
    assert_eq!(encoded.len(), 186);
    assert_eq!(to_hex(&encoded), GENESIS_BLOCK_HEX);
    assert_eq!(block_id(header).to_string(), REGTEST_NETWORK_ID);
    assert_eq!(
        regtest_network_id().unwrap().to_string(),
        REGTEST_NETWORK_ID
    );
}

#[test]
fn regtest_genesis_key_is_public_key_of_secret_one() {
    let mut secret = [0_u8; 32];
    if let Some(last) = secret.last_mut() {
        *last = 1;
    }
    let key = SecretKey::from_bytes(&secret).unwrap();
    assert_eq!(key.public_key().as_bytes(), &REGTEST_GENESIS_KEY);
}

#[test]
fn pow_limit_per_network() {
    assert_eq!(REGTEST_GENESIS_TARGET_COMPACT, REGTEST_POW_LIMIT_COMPACT);
    assert_eq!(pow_limit_compact(Network::Regtest), 0x207f_ffff);
    assert_eq!(pow_limit(Network::Regtest), decode_compact(0x207f_ffff));
    assert!(pow_limit(Network::Regtest).is_ok());
    for network in [Network::Testnet, Network::Mainnet] {
        let compact = pow_limit_compact(network);
        assert_eq!(
            pow_limit(network),
            Err(DifficultyError::InvalidCompact { compact })
        );
    }
}
