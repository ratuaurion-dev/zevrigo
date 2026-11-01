// Vektor resmi BIP32 1, 2, dan 3 diambil dari https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki.
// Kolom chain_code, secret, dan public adalah isi ext prv dan ext pub resmi (Base58Check didekode Quick) untuk
// setiap jalur. W4 sampai W7: Spesifikasi Wallet v0.1 Bagian 4.

use homeostasis_primitives::Network;

use super::{
    ACCOUNT_KEY_HEX_LEN, Bip32Error, CHANGE_BRANCH, ExtendedPrivKey, ExtendedPubKey, HARDENED,
    RECEIVE_BRANCH, account_key, account_path, coin_type, hardened,
};
use crate::mnemonic::Mnemonic;

struct Bip32Step {
    path: &'static [u32],
    chain_code: &'static str,
    secret: &'static str,
    public: &'static str,
}

const fn h(index: u32) -> u32 {
    hardened(index)
}

const VECTOR1_SEED: &str = "000102030405060708090a0b0c0d0e0f";
const VECTOR1: [Bip32Step; 6] = [
    // m
    Bip32Step {
        path: &[],
        chain_code: "873dff81c02f525623fd1fe5167eac3a55a049de3d314bb42ee227ffed37d508",
        secret: "e8f32e723decf4051aefac8e2c93c9c5b214313817cdb01a1494b917c8436b35",
        public: "0339a36013301597daef41fbe593a02cc513d0b55527ec2df1050e2e8ff49c85c2",
    },
    // m/0'
    Bip32Step {
        path: &[h(0)],
        chain_code: "47fdacbd0f1097043b78c63c20c34ef4ed9a111d980047ad16282c7ae6236141",
        secret: "edb2e14f9ee77d26dd93b4ecede8d16ed408ce149b6cd80b0715a2d911a0afea",
        public: "035a784662a4a20a65bf6aab9ae98a6c068a81c52e4b032c0fb5400c706cfccc56",
    },
    // m/0'/1
    Bip32Step {
        path: &[h(0), 1],
        chain_code: "2a7857631386ba23dacac34180dd1983734e444fdbf774041578e9b6adb37c19",
        secret: "3c6cb8d0f6a264c91ea8b5030fadaa8e538b020f0a387421a12de9319dc93368",
        public: "03501e454bf00751f24b1b489aa925215d66af2234e3891c3b21a52bedb3cd711c",
    },
    // m/0'/1/2'
    Bip32Step {
        path: &[h(0), 1, h(2)],
        chain_code: "04466b9cc8e161e966409ca52986c584f07e9dc81f735db683c3ff6ec7b1503f",
        secret: "cbce0d719ecf7431d88e6a89fa1483e02e35092af60c042b1df2ff59fa424dca",
        public: "0357bfe1e341d01c69fe5654309956cbea516822fba8a601743a012a7896ee8dc2",
    },
    // m/0'/1/2'/2
    Bip32Step {
        path: &[h(0), 1, h(2), 2],
        chain_code: "cfb71883f01676f587d023cc53a35bc7f88f724b1f8c2892ac1275ac822a3edd",
        secret: "0f479245fb19a38a1954c5c7c0ebab2f9bdfd96a17563ef28a6a4b1a2a764ef4",
        public: "02e8445082a72f29b75ca48748a914df60622a609cacfce8ed0e35804560741d29",
    },
    // m/0'/1/2'/2/1000000000
    Bip32Step {
        path: &[h(0), 1, h(2), 2, 1_000_000_000],
        chain_code: "c783e67b921d2beb8f6b389cc646d7263b4145701dadd2161548a8b078e65e9e",
        secret: "471b76e389e528d6de6d816857e012c5455051cad6660850e58372a6c3e6e7c8",
        public: "022a471424da5e657499d1ff51cb43c47481a03b1e77f951fe64cec9f5a48f7011",
    },
];

const VECTOR2_SEED: &str = concat!(
    "fffcf9f6f3f0edeae7e4e1dedbd8d5d2cfccc9c6c3c0bdbab7b4b1aeaba8a5a2",
    "9f9c999693908d8a8784817e7b7875726f6c696663605d5a5754514e4b484542"
);
const VECTOR2: [Bip32Step; 6] = [
    // m
    Bip32Step {
        path: &[],
        chain_code: "60499f801b896d83179a4374aeb7822aaeaceaa0db1f85ee3e904c4defbd9689",
        secret: "4b03d6fc340455b363f51020ad3ecca4f0850280cf436c70c727923f6db46c3e",
        public: "03cbcaa9c98c877a26977d00825c956a238e8dddfbd322cce4f74b0b5bd6ace4a7",
    },
    // m/0
    Bip32Step {
        path: &[0],
        chain_code: "f0909affaa7ee7abe5dd4e100598d4dc53cd709d5a5c2cac40e7412f232f7c9c",
        secret: "abe74a98f6c7eabee0428f53798f0ab8aa1bd37873999041703c742f15ac7e1e",
        public: "02fc9e5af0ac8d9b3cecfe2a888e2117ba3d089d8585886c9c826b6b22a98d12ea",
    },
    // m/0/2147483647'
    Bip32Step {
        path: &[0, h(2_147_483_647)],
        chain_code: "be17a268474a6bb9c61e1d720cf6215e2a88c5406c4aee7b38547f585c9a37d9",
        secret: "877c779ad9687164e9c2f4f0f4ff0340814392330693ce95a58fe18fd52e6e93",
        public: "03c01e7425647bdefa82b12d9bad5e3e6865bee0502694b94ca58b666abc0a5c3b",
    },
    // m/0/2147483647'/1
    Bip32Step {
        path: &[0, h(2_147_483_647), 1],
        chain_code: "f366f48f1ea9f2d1d3fe958c95ca84ea18e4c4ddb9366c336c927eb246fb38cb",
        secret: "704addf544a06e5ee4bea37098463c23613da32020d604506da8c0518e1da4b7",
        public: "03a7d1d856deb74c508e05031f9895dab54626251b3806e16b4bd12e781a7df5b9",
    },
    // m/0/2147483647'/1/2147483646'
    Bip32Step {
        path: &[0, h(2_147_483_647), 1, h(2_147_483_646)],
        chain_code: "637807030d55d01f9a0cb3a7839515d796bd07706386a6eddf06cc29a65a0e29",
        secret: "f1c7c871a54a804afe328b4c83a1c33b8e5ff48f5087273f04efa83b247d6a2d",
        public: "02d2b36900396c9282fa14628566582f206a5dd0bcc8d5e892611806cafb0301f0",
    },
    // m/0/2147483647'/1/2147483646'/2
    Bip32Step {
        path: &[0, h(2_147_483_647), 1, h(2_147_483_646), 2],
        chain_code: "9452b549be8cea3ecb7a84bec10dcfd94afe4d129ebfd3b3cb58eedf394ed271",
        secret: "bb7d39bdb83ecf58f2fd82b6d918341cbef428661ef01ab97c28a4842125ac23",
        public: "024d902e1a2fc7a8755ab5b694c575fce742c48d9ff192e63df5193e4c7afe1f9c",
    },
];

const VECTOR3_SEED: &str = concat!(
    "4b381541583be4423346c643850da4b320e46a87ae3d2a4e6da11eba819cd4ac",
    "ba45d239319ac14f863b8d5ab5a0d0c64d2e8a1e7d1457df2e5a3c51c73235be"
);
const VECTOR3: [Bip32Step; 2] = [
    // m
    Bip32Step {
        path: &[],
        chain_code: "01d28a3e53cffa419ec122c968b3259e16b65076495494d97cae10bbfec3c36f",
        secret: "00ddb80b067e0d4993197fe10f2657a844a384589847602d56f0c629c81aae32",
        public: "03683af1ba5743bdfc798cf814efeeab2735ec52d95eced528e692b8e34c4e5669",
    },
    // m/0'
    Bip32Step {
        path: &[h(0)],
        chain_code: "e5fea12a97b927fc9dc3d2cb0d1ea1cf50aa5a1fdc1f933e8906bb38df3377bd",
        secret: "491f7a2eebc7b57028e0d3faa0acda02e75c33b03c48fb288c41e2ea44e1daef",
        public: "026557fdda1d5d43d79611f784780471f086d58e8126b8c40acb82272a7712e7f2",
    },
];

const W4: &str = concat!(
    "03b9385a6fbee0caa6e79407f83b0053e2d864b6c7a8f32ce61c3f495b5dd5d027",
    "12ed17c3e72223beb38ea98ec1317581f02458f02b0ecf4abe02c6490eca170a"
);

// (branch, index, kunci rahasia, x-only) untuk W5, W6, W7.
const W5_TO_W7: [(u32, u32, &str, &str); 3] = [
    (
        RECEIVE_BRANCH,
        0,
        "11831b777e3fad99962ab773b23bad02812b85d8ac452e6c92434b4d7a79d9a3",
        "5a6f28253f35e2ff1ab0b086b8c87f731159661157e5be2ca98188369625dc32",
    ),
    (
        RECEIVE_BRANCH,
        1,
        "e5dde95cb4c071fe5cc1bb06b5d8183e2ae5bec60ae6fb45c53085aaee8aa7df",
        "8f33099c2cc3739e24c329d9e2d72ff7ce8bb4f4491b1763f69ca8693ea34806",
    ),
    (
        CHANGE_BRANCH,
        0,
        "f6615fe1ba1d1c685ac5eb2697b6b377aafc30a03c4b5aeb14d0bec5a7b26869",
        "c71cdd713224479fff24b56db524c2385acf09725605e7440de0c34272e2abf5",
    ),
];

fn decode_hex(hex: &str) -> Vec<u8> {
    let (pairs, rem) = hex.as_bytes().as_chunks::<2>();
    assert!(rem.is_empty(), "panjang string hex harus genap");
    let mut out = Vec::with_capacity(pairs.len());
    for &[hi, lo] in pairs {
        let hi_val = char::from(hi).to_digit(16).expect("hex nibble valid");
        let lo_val = char::from(lo).to_digit(16).expect("hex nibble valid");
        let byte = u8::try_from(hi_val.wrapping_shl(4) | lo_val).expect("byte muat u8");
        out.push(byte);
    }
    out
}

fn w1_seed() -> crate::mnemonic::Seed {
    Mnemonic::from_entropy(&[0_u8; 32])
        .expect("entropi sah")
        .to_seed("")
        .expect("tanpa passphrase")
}

fn check_vector(seed_hex: &str, steps: &[Bip32Step]) {
    let master = ExtendedPrivKey::master(&decode_hex(seed_hex)).expect("kunci induk sah");
    for step in steps {
        let key = master.derive_path(step.path).expect("turunan sah");
        assert_eq!(
            key.chain_code().as_slice(),
            decode_hex(step.chain_code).as_slice()
        );
        assert_eq!(
            key.secret_bytes().as_slice(),
            decode_hex(step.secret).as_slice()
        );
        let public = key.to_public().expect("kunci publik sah");
        assert_eq!(
            public.public_key().as_slice(),
            decode_hex(step.public).as_slice()
        );
        assert_eq!(public.chain_code(), key.chain_code());

        // CKDpub: jika langkah terakhir non-hardened, kunci publik induk menurunkan kunci publik yang sama.
        if let Some((&last, parent_path)) = step.path.split_last()
            && last < HARDENED
        {
            let parent = master
                .derive_path(parent_path)
                .expect("turunan induk sah")
                .to_public()
                .expect("kunci publik induk sah");
            assert_eq!(parent.derive_child(last), Ok(public));
        }
    }
}

#[test]
fn bip32_official_vector_1() {
    check_vector(VECTOR1_SEED, &VECTOR1);
}

#[test]
fn bip32_official_vector_2() {
    check_vector(VECTOR2_SEED, &VECTOR2);
}

#[test]
fn bip32_official_vector_3_keeps_leading_zeros() {
    check_vector(VECTOR3_SEED, &VECTOR3);
}

#[test]
fn spec_vectors_w4_to_w7_private_and_watch_only() {
    let seed = w1_seed();
    let account = account_key(&seed, Network::Regtest).expect("akun regtest");
    let account_public = account.to_public().expect("kunci publik akun");
    let hex = account_public.to_account_hex();
    assert_eq!(hex.len(), ACCOUNT_KEY_HEX_LEN);
    assert_eq!(hex, W4);
    assert_eq!(ExtendedPubKey::from_account_hex(W4), Ok(account_public));
    // Testnet memakai coin yang sama (1), jadi kunci akunnya sama.
    let testnet = account_key(&seed, Network::Testnet).expect("akun testnet");
    assert_eq!(testnet, account);

    let watch_only = ExtendedPubKey::from_account_hex(W4).expect("kunci akun sah");
    for (branch, index, secret, x_only) in W5_TO_W7 {
        let key = account.derive_path(&[branch, index]).expect("turunan sah");
        assert_eq!(key.secret_bytes().as_slice(), decode_hex(secret).as_slice());
        let public = key.to_public().expect("kunci publik sah");
        assert_eq!(public.x_only().as_slice(), decode_hex(x_only).as_slice());
        let watched = watch_only
            .derive_path(&[branch, index])
            .expect("turunan publik sah");
        assert_eq!(watched, public);
    }
}

#[test]
fn account_path_and_coin_type_per_network() {
    let expected = [h(44), h(1), h(0)];
    assert_eq!(account_path(Network::Regtest), Ok(expected));
    assert_eq!(account_path(Network::Testnet), Ok(expected));
    assert_eq!(coin_type(Network::Regtest), Ok(1));
    assert_eq!(
        coin_type(Network::Mainnet),
        Err(Bip32Error::MainnetCoinTypeUnset)
    );
    assert_eq!(
        account_path(Network::Mainnet),
        Err(Bip32Error::MainnetCoinTypeUnset)
    );
    assert_eq!(
        account_key(&w1_seed(), Network::Mainnet),
        Err(Bip32Error::MainnetCoinTypeUnset)
    );
}

#[test]
fn hardened_child_from_public_key_is_rejected() {
    let public = ExtendedPrivKey::master(&decode_hex(VECTOR1_SEED))
        .expect("kunci induk sah")
        .to_public()
        .expect("kunci publik sah");
    for index in [HARDENED, u32::MAX] {
        assert_eq!(
            public.derive_child(index),
            Err(Bip32Error::HardenedFromPublic { index })
        );
    }
    assert_eq!(
        public.derive_path(&[0, h(1)]),
        Err(Bip32Error::HardenedFromPublic { index: h(1) })
    );
}

#[test]
fn master_rejects_bad_seed_length() {
    for len in [0_usize, 15, 65, 128] {
        assert_eq!(
            ExtendedPrivKey::master(&vec![0x5a_u8; len]),
            Err(Bip32Error::InvalidSeedLength { len })
        );
    }
    assert!(ExtendedPrivKey::master(&[0x5a_u8; 16]).is_ok());
    assert!(ExtendedPrivKey::master(&[0x5a_u8; 64]).is_ok());
}

#[test]
fn account_hex_rejects_malformed_input() {
    let upper = W4.to_ascii_uppercase();
    let short = W4.get(..128).expect("panjang W4 130");
    let long = format!("{W4}00");
    let non_hex = W4.replacen("03", "0g", 1);
    for bad in [upper.as_str(), short, long.as_str(), non_hex.as_str(), ""] {
        assert_eq!(
            ExtendedPubKey::from_account_hex(bad),
            Err(Bip32Error::InvalidAccountKey)
        );
    }
    // Awalan 04 dan x di luar kurva: panjang dan karakter benar, tetapi bukan titik sah.
    let wrong_prefix = W4.replacen("03", "04", 1);
    // x = 5 tidak punya y pada kurva.
    let chain_code = W4.get(66..).expect("chain code W4");
    let off_curve = format!("02{}5{chain_code}", "0".repeat(63));
    for bad in [wrong_prefix, off_curve] {
        assert_eq!(
            ExtendedPubKey::from_account_hex(&bad),
            Err(Bip32Error::InvalidPublicKey)
        );
    }
}

#[test]
fn debug_never_prints_private_key() {
    let master = ExtendedPrivKey::master(&decode_hex(VECTOR1_SEED)).expect("kunci induk sah");
    assert_eq!(format!("{master:?}"), "ExtendedPrivKey(<rahasia>)");
}
