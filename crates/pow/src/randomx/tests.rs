use homeostasis_primitives::block::Header;
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::height::BlockHeight;

use super::{
    RandomxError, RandomxService, RandomxVerifier, SERVICE_KEYS, hash_with, key_height, light_vm,
};
use crate::target::Target;

/// Kunci dan input vektor resmi RandomX (tevador/RandomX, src/tests/tests.cpp, "Hash test 1a" sampai "1f"). Hash yang
/// diharapkan adalah hash varian HSIS (salt `RandomHSIS\x01`, DEC-01), dihitung Q101 dengan program uji C++ terhadap
/// pustaka vendor/randomx-rs; program yang sama menghasilkan vektor resmi persis dari sumber asli (salt `RandomX\x03`),
/// dengan flag bawaan dan flag anjuran CPU (JIT).
const KEY_000: &[u8] = b"test key 000";
const KEY_001: &[u8] = b"test key 001";
/// Kunci test 1f: 31 byte (`initCache` membaca sampai byte nol).
const KEY_1F: &str = "7797373ea4633194640bf8d8c3b66724d6aa7bd2dc20e009df2f8f1710abe8";

const TEST_C_INPUT: &[u8] = b"sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
const TEST_E_INPUT: &str = "0b0b98bea7e805e0010a2126d287a2a0cc833d312cb786385a7c2f9de69d25537f584a9bc9977b00000000666fd8753bf61a8631f12984e3fd44f4014eca629276817b56f32e9b68bd82f416";
const TEST_F_INPUT: &str = "1010e1eaf8cf067b37b5f0ee031ab23ed1755e090a3af4415830145853e2be3e1f6821fed84dae58d00e00da5214d6c1f2d0622e0abd51f9373d04e0b0f8e6d6514d90689721c4aac5a9bb0d";

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("hex ASCII");
            u8::from_str_radix(text, 16).expect("hex sah")
        })
        .collect()
}

fn check(key: &[u8], cases: &[(Vec<u8>, &str)]) {
    let vm = light_vm(key).expect("cache dan VM RandomX");
    for (input, expected) in cases {
        let expected = Hash32::from_hex(expected).expect("hash hex");
        assert_eq!(hash_with(&vm, input), Ok(expected));
    }
}

#[test]
fn hsis_vectors_light_mode() {
    check(
        KEY_000,
        &[
            (
                b"This is a test".to_vec(),
                "bffb4d6af107280fc2bbc8d2ac409abda2514d163ea823e67c1f3497b0b7d543",
            ),
            (
                b"Lorem ipsum dolor sit amet".to_vec(),
                "2e2e6ff4bce9197701346efccac90331809b808095ddf9889245323062a25c68",
            ),
            (
                TEST_C_INPUT.to_vec(),
                "8e9204a0246ba7765f516e0ea8c93b1fc126b9198ebd80dc55b3dc9ed5bc2f27",
            ),
        ],
    );
    check(
        KEY_001,
        &[
            (
                TEST_C_INPUT.to_vec(),
                "5c72e1144bfcbc5882c4da50db8ee4cc12fbc9d875fe8d7f953fb994b1d6b630",
            ),
            (
                bytes(TEST_E_INPUT),
                "2f91f0fa8d4ab0a1e76ade33285ef58444725ee94c54ff248a2e0eb88199e35c",
            ),
        ],
    );
    let key_1f = bytes(KEY_1F);
    assert_eq!(key_1f.len(), 31);
    check(
        &key_1f,
        &[(
            bytes(TEST_F_INPUT),
            "988e0c3a9b01f9ca198ae7acbdf4ace714bd535ffdf298c46c756acbb5df9cd5",
        )],
    );
}

#[test]
fn key_height_follows_interval_and_lag() {
    let cases: [(u64, u64); 9] = [
        (0, 0),
        (1, 0),
        (2_048, 0),
        (2_112, 0),
        (2_113, 2_048),
        (4_160, 2_048),
        (4_161, 4_096),
        (6_209, 6_144),
        (u64::MAX, 0xffff_ffff_ffff_f800),
    ];
    for (height, expected) in cases {
        assert_eq!(
            key_height(BlockHeight::new(height)),
            BlockHeight::new(expected),
            "tinggi {height}"
        );
    }
}

#[test]
fn header_hash_and_target_rule() {
    let key = Hash32::from_bytes([0x33; 32]);
    let verifier = RandomxVerifier::new(key).expect("pemeriksa RandomX");
    assert_eq!(verifier.key(), key);

    let header = Header::new(
        BlockHeight::new(1),
        Hash32::from_bytes([0x11; 32]),
        Hash32::from_bytes([0x22; 32]),
        1_800_000_000,
        0x1d00_ffff,
        42,
    );
    let pow_hash = verifier.hash(&header).expect("hash header");
    // Vektor h1 Q101 (header ini, kunci 0x33 × 32, salt HSIS).
    let h1 = Hash32::from_hex("97a628c587d1c1a463da37f83a33a66c60cf304300392992f3993a708fd62e53")
        .expect("h1");
    assert_eq!(pow_hash, h1);
    let vm = light_vm(key.as_bytes()).expect("VM kedua");
    assert_eq!(hash_with(&vm, &header.to_array()), Ok(pow_hash));
    assert_ne!(pow_hash, Hash32::ZERO);

    let other = Header::new(
        BlockHeight::new(1),
        Hash32::from_bytes([0x11; 32]),
        Hash32::from_bytes([0x22; 32]),
        1_800_000_000,
        0x1d00_ffff,
        43,
    );
    assert_ne!(verifier.hash(&other), Ok(pow_hash));

    let exact = Target::from_be_bytes(*pow_hash.as_bytes());
    assert_eq!(verifier.verify(&header, &exact), Ok(()));
    assert_eq!(verifier.verify(&header, &Target::MAX), Ok(()));
    assert_eq!(
        verifier.verify(&header, &Target::ZERO),
        Err(RandomxError::AboveTarget {
            pow_hash,
            target: Target::ZERO,
        })
    );
}

#[test]
fn empty_key_and_input_are_rejected() {
    assert!(matches!(light_vm(&[]), Err(RandomxError::Library(_))));
    let vm = light_vm(KEY_000).expect("cache dan VM RandomX");
    assert!(matches!(hash_with(&vm, &[]), Err(RandomxError::Library(_))));
    assert_eq!(
        RandomxError::HashLength.to_string(),
        "hash RandomX tidak 32 byte"
    );
}

#[test]
fn service_matches_verifier_and_keeps_recent_keys() {
    fn shared<T: Send + Sync>(_: &T) {}
    let service = RandomxService::start().expect("layanan RandomX");
    shared(&service);
    assert_eq!(SERVICE_KEYS, 2);

    let header = Header::new(
        BlockHeight::new(7),
        Hash32::from_bytes([0x44; 32]),
        Hash32::from_bytes([0x55; 32]),
        1_800_000_000,
        0x1d00_ffff,
        9,
    );
    let key_a = Hash32::from_bytes([0x01; 32]);
    let key_b = Hash32::from_bytes([0x02; 32]);
    let expected_a = RandomxVerifier::new(key_a)
        .expect("pemeriksa A")
        .hash(&header)
        .expect("hash A");
    assert_eq!(service.hash(key_a, &header), Ok(expected_a));
    let from_b = service.hash(key_b, &header).expect("hash B");
    assert_ne!(from_b, expected_a);
    // Kunci A masih disimpan bersama kunci B; hasilnya tetap sama.
    assert_eq!(service.hash(key_a, &header), Ok(expected_a));
    assert_eq!(service.hash(key_b, &header), Ok(from_b));
    assert_eq!(
        RandomxError::Service("x".to_owned()).to_string(),
        "layanan RandomX gagal: x"
    );
}
