use zevrigo_primitives::block::Header;
use zevrigo_primitives::hash::Hash32;
use zevrigo_primitives::height::BlockHeight;

use super::{
    RandomxError, RandomxMiner, RandomxMode, RandomxService, RandomxVerifier, SERVICE_KEYS,
    hash_with, key_height, light_vm,
};
use crate::target::Target;

/// Kunci dan input vektor resmi RandomX (tevador/RandomX, src/tests/tests.cpp, "Hash test 1a" sampai "1f"). Hash yang
/// diharapkan adalah hash varian ZVRG (salt `RandomZVRG\x01`, DEC-01), dihitung ulang Q107c dengan program uji C++ Q101 terhadap
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
fn zvrg_vectors_light_mode() {
    check(
        KEY_000,
        &[
            (
                b"This is a test".to_vec(),
                "88c14322b7870ef03913afb8116d0bcf556199117b69b9d7c4b497f3aca8a216",
            ),
            (
                b"Lorem ipsum dolor sit amet".to_vec(),
                "cb859f8d5236f94368516370dc00be6c4fc5028c429761941e4cefa7343b4378",
            ),
            (
                TEST_C_INPUT.to_vec(),
                "bd936defa22e157aafc2356ad4be8f9196351981b18dd4c46ddde14687ddd012",
            ),
        ],
    );
    check(
        KEY_001,
        &[
            (
                TEST_C_INPUT.to_vec(),
                "f32eb66b01d0805a9efd4f30ee8fe872ad43892cd5cc9bd3232e575983ccba60",
            ),
            (
                bytes(TEST_E_INPUT),
                "2a81fcb9c1ab3ed8e28279c7d75721792a4d47a44a8d2e05c48051303c260792",
            ),
        ],
    );
    let key_1f = bytes(KEY_1F);
    assert_eq!(key_1f.len(), 31);
    check(
        &key_1f,
        &[(
            bytes(TEST_F_INPUT),
            "11b35792ba457f0253156f7617f25c468208fd43d524ced9abb0e1e728745caf",
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
    // Vektor h1 Q101 (header ini, kunci 0x33 × 32, salt ZVRG).
    let h1 = Hash32::from_hex("5262931966a4225f8044392c0d08592164070e4cb1db67fb8be93c556e6ad957")
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

/// Header vektor h1 (Q101) dan hash ZVRG-nya dengan kunci 0x33 × 32.
fn h1() -> (Header, Hash32) {
    let header = Header::new(
        BlockHeight::new(1),
        Hash32::from_bytes([0x11; 32]),
        Hash32::from_bytes([0x22; 32]),
        1_800_000_000,
        0x1d00_ffff,
        42,
    );
    let hash = Hash32::from_hex("5262931966a4225f8044392c0d08592164070e4cb1db67fb8be93c556e6ad957")
        .expect("h1");
    (header, hash)
}

#[test]
fn light_miner_hashes_and_searches_nonces() {
    let key = Hash32::from_bytes([0x33; 32]);
    let miner = RandomxMiner::new(key, RandomxMode::Light).expect("penambang ringan");
    assert_eq!(miner.key(), key);
    assert_eq!(miner.mode(), RandomxMode::Light);
    let (header, expected) = h1();
    assert_eq!(miner.hash(&header), Ok(expected));

    // Target maksimum: nonce pertama langsung lolos, header tidak berubah.
    assert_eq!(miner.mine(&header, &Target::MAX, 1), Ok(Some(header)));
    // Target nol tidak pernah lolos; nol percobaan dan nonce yang meluap berhenti tanpa hasil.
    assert_eq!(miner.mine(&header, &Target::ZERO, 3), Ok(None));
    assert_eq!(miner.mine(&header, &Target::MAX, 0), Ok(None));
    let last = Header::new(
        header.height(),
        header.prev_block_id(),
        header.merkle_root(),
        header.timestamp(),
        header.target(),
        u64::MAX,
    );
    assert_eq!(miner.mine(&last, &Target::ZERO, 3), Ok(None));

    // Target setengah ruang: nonce yang ditemukan lolos pemeriksa, dan semua nonce sebelumnya gagal.
    let mut half = [0xff_u8; 32];
    if let Some(first) = half.first_mut() {
        *first = 0x7f;
    }
    let half = Target::from_be_bytes(half);
    let found = miner
        .mine(&header, &half, 64)
        .expect("penambangan")
        .expect("nonce dalam 64 percobaan");
    let verifier = RandomxVerifier::new(key).expect("pemeriksa");
    assert_eq!(verifier.verify(&found, &half), Ok(()));
    for nonce in header.nonce()..found.nonce() {
        let earlier = Header::new(
            header.height(),
            header.prev_block_id(),
            header.merkle_root(),
            header.timestamp(),
            header.target(),
            nonce,
        );
        assert!(verifier.verify(&earlier, &half).is_err(), "nonce {nonce}");
    }
}

/// Mode cepat memakai dataset sekitar 2 GiB dan menyusunnya sekitar satu menit, jadi tidak ikut `make check-all`.
/// Dijalankan skrip gerbang dengan `cargo test -p zevrigo-pow --lib -- --ignored`.
#[test]
#[ignore = "dataset RandomX sekitar 2 GiB"]
fn fast_miner_matches_light_hash() {
    let key = Hash32::from_bytes([0x33; 32]);
    let miner = RandomxMiner::new(key, RandomxMode::Fast).expect("penambang cepat");
    assert_eq!(miner.mode(), RandomxMode::Fast);
    let (header, expected) = h1();
    assert_eq!(miner.hash(&header), Ok(expected));
    let found = miner
        .mine(&header, &Target::MAX, 1)
        .expect("penambangan cepat");
    assert_eq!(found, Some(header));
}
