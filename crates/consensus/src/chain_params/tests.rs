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
    "0d66aa3b8336e2dd1d77a3ff0b30dd4b73d0c06ff93aa9221789a2f7e1429ab7",
    "003be16a00000000",
    "ffff7f20",
    "0000000000000000",
    "01",
    "0101000000000000000020",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "01808080d084a9808a96aa964200",
    "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
);

const GENESIS_TXID: &str = "50eae4b92788be240e57ea71f8552cd1e2816c095baf593c391f13945e1c1d4e";
const GENESIS_WTXID: &str = "e2363ae4035977610005afe0288ec1ee0b00460c40f8ada5856306ea45dea21f";
const GENESIS_MERKLE_ROOT: &str =
    "0d66aa3b8336e2dd1d77a3ff0b30dd4b73d0c06ff93aa9221789a2f7e1429ab7";
const REGTEST_NETWORK_ID: &str = "8b9eb3187250bbdfe1577ebcb668c30003213a9ee6a4424427b08e38da364dab";

/// Encoding lengkap blok genesis testnet, 186 byte (Spesifikasi Validasi Bagian 10b), dihitung Quick dengan
/// `blake3_ref.py`. Cap waktu masih placeholder `0` (TSK-F06).
const TESTNET_BLOCK_HEX: &str = concat!(
    "01000000",
    "0000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "c136cd4718d5663de09d28138ed3e85945b823f7cf37b8a0f9375af5c0f39b36",
    "0000000000000000",
    "64cf8b1e",
    "0000000000000000",
    "01",
    "0101000000000000000020",
    "7a65767269676f20746573746e65742031000000000000000000000000000000",
    "01808080d084a9808a96aa964200",
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"
);

const TESTNET_TXID: &str = "8d4ac6d11fa59e49ef46bff7207989ab65878a877c091bd145c5e650728a0892";
const TESTNET_WTXID: &str = "851da958564661c9af81346c2b58048b9b10d1fc25085b554e33d5d89c0decc0";
const TESTNET_MERKLE_ROOT: &str =
    "c136cd4718d5663de09d28138ed3e85945b823f7cf37b8a0f9375af5c0f39b36";
const TESTNET_NETWORK_ID: &str = "e4b9f2fb41c16ea54aa7ab3f76f92d4cb69fa501c5c95e1317f9fcd49cace50c";

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
