// Titik k·G untuk k = 1, 2, 3, n − 1 dihitung Quick dengan implementasi referensi Python (alat/wallet_ref.py,
// aritmetika kurva bilangan bulat murni). x-only k = 3 dan kunci vektor BIP340 #1 sama dengan vektor resmi BIP340.

use proptest::prelude::*;

use super::{
    COMPRESSED_PUBLIC_KEY_LEN, TweakError, is_valid_public_key, public_key_compressed,
    public_tweak_add, secret_tweak_add,
};
use crate::schnorr::SecretKey;

const G1: &str = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const G2: &str = "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";
const G3: &str = "02f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9";
const G_N_MINUS_1: &str = "0379be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
const N: &str = "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141";
const N_MINUS_1: &str = "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140";
const FIELD_P: &str = "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f";
// Kunci rahasia vektor BIP340 #1 dan kunci publik SEC1 terkompresinya.
const BIP340_SK1: &str = "b7e151628aed2a6abf7158809cf4f3c762e7160f38b4da56a784d9045190cfef";
const BIP340_PK1: &str = "02dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659";

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

fn h32(hex: &str) -> [u8; 32] {
    decode_hex(hex).try_into().expect("32 byte")
}

fn h33(hex: &str) -> [u8; COMPRESSED_PUBLIC_KEY_LEN] {
    decode_hex(hex).try_into().expect("33 byte")
}

fn small(value: u8) -> [u8; 32] {
    let mut out = [0_u8; 32];
    if let Some(last) = out.last_mut() {
        *last = value;
    }
    out
}

fn with_prefix(prefix: u8, x: &[u8; 32]) -> [u8; COMPRESSED_PUBLIC_KEY_LEN] {
    let mut out = [0_u8; COMPRESSED_PUBLIC_KEY_LEN];
    let (first, rest) = out.split_first_mut().expect("33 byte");
    *first = prefix;
    rest.copy_from_slice(x);
    out
}

#[test]
fn public_key_of_known_scalars() {
    assert_eq!(public_key_compressed(&small(1)), Ok(h33(G1)));
    assert_eq!(public_key_compressed(&small(2)), Ok(h33(G2)));
    assert_eq!(public_key_compressed(&small(3)), Ok(h33(G3)));
    assert_eq!(public_key_compressed(&h32(N_MINUS_1)), Ok(h33(G_N_MINUS_1)));
    assert_eq!(public_key_compressed(&h32(BIP340_SK1)), Ok(h33(BIP340_PK1)));
}

#[test]
fn public_key_rejects_invalid_secret() {
    for secret in [[0_u8; 32], h32(N), [0xff_u8; 32]] {
        assert_eq!(
            public_key_compressed(&secret),
            Err(TweakError::InvalidSecretKey)
        );
    }
}

#[test]
fn x_coordinate_matches_bip340_public_key() {
    for secret in [small(3), h32(BIP340_SK1), h32(N_MINUS_1)] {
        let compressed = public_key_compressed(&secret).expect("kunci sah");
        let (_prefix, x) = compressed.split_first().expect("33 byte");
        let xonly = SecretKey::from_bytes(&secret)
            .expect("kunci sah")
            .public_key();
        assert_eq!(x, xonly.as_bytes().as_slice());
    }
}

#[test]
fn secret_tweak_add_vectors() {
    assert_eq!(secret_tweak_add(&small(1), &small(2)), Ok(small(3)));
    assert_eq!(secret_tweak_add(&small(5), &[0_u8; 32]), Ok(small(5)));
    assert_eq!(secret_tweak_add(&h32(N_MINUS_1), &small(2)), Ok(small(1)));
    assert_eq!(
        secret_tweak_add(&h32(N_MINUS_1), &small(1)),
        Err(TweakError::InvalidResult)
    );
    assert_eq!(
        secret_tweak_add(&small(1), &h32(N)),
        Err(TweakError::InvalidTweak)
    );
    assert_eq!(
        secret_tweak_add(&[0_u8; 32], &small(1)),
        Err(TweakError::InvalidSecretKey)
    );
    assert_eq!(
        secret_tweak_add(&h32(N), &small(1)),
        Err(TweakError::InvalidSecretKey)
    );
}

#[test]
fn public_tweak_add_vectors() {
    assert_eq!(public_tweak_add(&h33(G1), &small(2)), Ok(h33(G3)));
    assert_eq!(public_tweak_add(&h33(G1), &[0_u8; 32]), Ok(h33(G1)));
    assert_eq!(public_tweak_add(&h33(G_N_MINUS_1), &small(3)), Ok(h33(G2)));
    assert_eq!(
        public_tweak_add(&h33(G_N_MINUS_1), &small(1)),
        Err(TweakError::InvalidResult)
    );
    assert_eq!(
        public_tweak_add(&h33(G1), &h32(N)),
        Err(TweakError::InvalidTweak)
    );
}

#[test]
fn invalid_public_keys_are_rejected() {
    let g1_x = h32("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");
    let invalid = [
        // Awalan bukan 02/03.
        with_prefix(0x04, &g1_x),
        with_prefix(0x00, &g1_x),
        with_prefix(0x05, &g1_x),
        // x = 5 tidak punya y pada kurva (5^3 + 7 bukan residu kuadrat modulo p).
        with_prefix(0x02, &small(5)),
        // x = p di luar medan prima.
        with_prefix(0x02, &h32(FIELD_P)),
        [0_u8; COMPRESSED_PUBLIC_KEY_LEN],
    ];
    for point in invalid {
        assert!(!is_valid_public_key(&point));
        assert_eq!(
            public_tweak_add(&point, &small(1)),
            Err(TweakError::InvalidPublicKey)
        );
    }
    for valid in [G1, G2, G3, G_N_MINUS_1, BIP340_PK1] {
        assert!(is_valid_public_key(&h33(valid)));
    }
    // x = 1 berada pada kurva; kedua awalan sah.
    assert!(is_valid_public_key(&with_prefix(0x02, &small(1))));
    assert!(is_valid_public_key(&with_prefix(0x03, &small(1))));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_public_tweak_matches_secret_tweak(
        secret in any::<[u8; 32]>(),
        tweak in any::<[u8; 32]>(),
    ) {
        let Ok(child_secret) = secret_tweak_add(&secret, &tweak) else {
            // Skalar acak 32 byte hampir selalu sah (peluang gagal sekitar 2^-127).
            return Ok(());
        };
        let parent_point = public_key_compressed(&secret).expect("kunci induk sah");
        prop_assert_eq!(
            public_tweak_add(&parent_point, &tweak),
            public_key_compressed(&child_secret)
        );
    }
}
