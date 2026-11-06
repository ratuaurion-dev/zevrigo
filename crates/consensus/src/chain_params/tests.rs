use core::fmt::Write as _;

use zevrigo_crypto::{PublicKey, SecretKey, block_id, txid, wtxid};
use zevrigo_primitives::params::GENESIS_ALLOCATION;
use zevrigo_primitives::{BlockHeight, Encode, Hash32, Network, OutputKind};

use super::{
    ChainParams, ParamsError, REGTEST_GENESIS_KEY, REGTEST_GENESIS_TARGET_COMPACT,
    REGTEST_GENESIS_TIMESTAMP, REGTEST_POW_LIMIT_COMPACT, TESTNET_GENESIS_EXTRA,
    TESTNET_GENESIS_KEY, TESTNET_GENESIS_TARGET_COMPACT, TESTNET_GENESIS_TIMESTAMP,
    TESTNET_POW_LIMIT_COMPACT, known_genesis, pow_limit, pow_limit_compact, regtest_genesis,
    regtest_network_id, testnet_genesis, testnet_network_id,
};
use crate::difficulty::{DifficultyError, decode_compact};

/// Encoding lengkap blok genesis regtest, 186 byte (Spesifikasi Validasi Bagian 10).
const GENESIS_BLOCK_HEX: &str = concat!(
    "01000000",
    "0000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "ef801283525bb565114f5e2d0d10deddc9465d206abe2675ef524ccc61def782",
    "003be16a00000000",
    "ffff7f20",
    "0000000000000000",
    "01",
    "0101000000000000000020",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "01808080cccfadc9ccc680fe1c00",
    "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
);

const GENESIS_TXID: &str = "e48a0c67c87ee39224a5ed3b09f4c301d8fe9a9f623161bf5252f6dd7fdd2310";
const GENESIS_WTXID: &str = "80917b98cd9844da532df6b15c6258b600aac9cfd6e20bc142a288a03ea96749";
const GENESIS_MERKLE_ROOT: &str =
    "ef801283525bb565114f5e2d0d10deddc9465d206abe2675ef524ccc61def782";
const REGTEST_NETWORK_ID: &str = "b858e5ce2bf393d7f4c0be814152d66fecfaacda4b0add19e79c3c161f441b8f";

/// Encoding lengkap blok genesis testnet, 186 byte (Spesifikasi Validasi Bagian 10b), dihitung Quick dengan
/// `blake3_ref.py`. Cap waktu masih placeholder `0` (TSK-F06).
const TESTNET_BLOCK_HEX: &str = concat!(
    "01000000",
    "0000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "d7df2a78012153e2ce5e14042c35c67de35c72d9f8062ebfb7b68dcaefe45fb0",
    "0000000000000000",
    "64cf8b1e",
    "0000000000000000",
    "01",
    "0101000000000000000020",
    "7a65767269676f20746573746e65742031000000000000000000000000000000",
    "01808080cccfadc9ccc680fe1c00",
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"
);

const TESTNET_TXID: &str = "ad23ee415df1905c83a7c4b0de263e494b2edec9091e6a70c9be238068333ff1";
const TESTNET_WTXID: &str = "581b0bc6cc5c6e5c628baee17797f082cfd04b5def263c3a712712b8558e8a94";
const TESTNET_MERKLE_ROOT: &str =
    "d7df2a78012153e2ce5e14042c35c67de35c72d9f8062ebfb7b68dcaefe45fb0";
const TESTNET_NETWORK_ID: &str = "ec9d66dfe49d755e7759c90defcaa4d86a45b431ea83368c448651affa4a2ff5";

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
    assert_eq!(pow_limit_compact(Network::Testnet), 0x1f36_9d03);
    assert_eq!(TESTNET_POW_LIMIT_COMPACT, 0x1f36_9d03);
    assert_eq!(pow_limit(Network::Testnet), decode_compact(0x1f36_9d03));
    assert!(pow_limit(Network::Testnet).is_ok());
    let compact = pow_limit_compact(Network::Mainnet);
    assert_eq!(
        pow_limit(Network::Mainnet),
        Err(DifficultyError::InvalidCompact { compact })
    );
}

#[test]
fn testnet_genesis_matches_spec_vectors() {
    let genesis = testnet_genesis().unwrap();
    let header = genesis.header();
    assert_eq!(header.height(), BlockHeight::GENESIS);
    assert_eq!(header.prev_block_id(), Hash32::ZERO);
    assert_eq!(header.timestamp(), TESTNET_GENESIS_TIMESTAMP);
    assert_eq!(header.timestamp(), 0);
    assert_eq!(header.target(), TESTNET_GENESIS_TARGET_COMPACT);
    assert_eq!(header.target(), 0x1e8b_cf64);
    assert_eq!(header.nonce(), 0);
    assert_eq!(header.merkle_root().to_string(), TESTNET_MERKLE_ROOT);

    let coinbase = genesis.txs().first().unwrap();
    assert_eq!(genesis.txs().len(), 1);
    assert_eq!(txid(coinbase).to_string(), TESTNET_TXID);
    assert_eq!(wtxid(coinbase).to_string(), TESTNET_WTXID);
    let allocation = coinbase.body().outputs().first().unwrap();
    assert_eq!(allocation.amount(), GENESIS_ALLOCATION);
    assert_eq!(allocation.kind(), OutputKind::SingleKey);
    assert_eq!(allocation.payload(), &TESTNET_GENESIS_KEY);
    let (text, padding) = TESTNET_GENESIS_EXTRA.split_at(17);
    assert_eq!(text, b"zevrigo testnet 1");
    assert!(padding.iter().all(|byte| *byte == 0));
    // Titik NUMS BIP341 adalah koordinat x sah di secp256k1, jadi output-nya kunci tunggal biasa tanpa kunci rahasia.
    assert!(PublicKey::from_bytes(&TESTNET_GENESIS_KEY).is_ok());

    let encoded = genesis.to_bytes();
    assert_eq!(encoded.len(), 186);
    assert_eq!(to_hex(&encoded), TESTNET_BLOCK_HEX);
    assert_eq!(block_id(header).to_string(), TESTNET_NETWORK_ID);
    assert_eq!(
        testnet_network_id().unwrap().to_string(),
        TESTNET_NETWORK_ID
    );
}

#[test]
fn testnet_params_and_known_genesis() {
    let testnet = ChainParams::testnet().unwrap();
    assert_eq!(testnet.network(), Network::Testnet);
    assert_eq!(Ok(testnet.pow_limit()), decode_compact(0x1f36_9d03));
    assert_eq!(Ok(testnet.genesis_target()), decode_compact(0x1e8b_cf64));
    assert!(testnet.genesis_target() <= testnet.pow_limit());
    assert_eq!(testnet.genesis_timestamp(), 0);
    assert_eq!(testnet.network_id().to_string(), TESTNET_NETWORK_ID);

    let regtest = ChainParams::regtest().unwrap();
    assert_eq!(ChainParams::for_network(Network::Regtest), Ok(regtest));
    assert_eq!(ChainParams::for_network(Network::Testnet), Ok(testnet));
    assert_eq!(
        ChainParams::for_network(Network::Mainnet),
        Err(ParamsError::Unavailable {
            network: Network::Mainnet
        })
    );
    assert_ne!(regtest.network_id(), testnet.network_id());
    assert_eq!(
        known_genesis(&regtest.network_id()).unwrap(),
        Some(regtest_genesis().unwrap())
    );
    assert_eq!(
        known_genesis(&testnet.network_id()).unwrap(),
        Some(testnet_genesis().unwrap())
    );
    assert_eq!(known_genesis(&Hash32::ZERO).unwrap(), None);
}

#[test]
fn regtest_chain_params_match_genesis() {
    let params = ChainParams::regtest().unwrap();
    assert_eq!(params.network(), Network::Regtest);
    assert_eq!(Ok(params.pow_limit()), decode_compact(0x207f_ffff));
    assert_eq!(Ok(params.genesis_target()), decode_compact(0x207f_ffff));
    assert_eq!(params.genesis_timestamp(), 1_793_145_600);
    assert_eq!(params.network_id().to_string(), REGTEST_NETWORK_ID);
    let same = ChainParams::new(
        Network::Regtest,
        params.pow_limit(),
        params.genesis_target(),
        params.genesis_timestamp(),
        params.network_id(),
    );
    assert_eq!(same, params);
}
