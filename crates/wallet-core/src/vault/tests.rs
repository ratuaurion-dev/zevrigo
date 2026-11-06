// Vektor: RFC 9106 Bagian 5.3 (Argon2id), draft-irtf-cfrg-xchacha-03 Lampiran A.3.1 (XChaCha20-Poly1305), dan vektor
// V1 dan V2 modul ini yang dihitung dengan implementasi rujukan Python terpisah (lolos kedua vektor resmi di atas).

use argon2::{Algorithm, Argon2, AssociatedData, Block, ParamsBuilder, Version};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};

use super::{KdfParams, Sealed, VaultError, associated_data, derive_key, seal, unseal};

/// Frasa W1 Spesifikasi Wallet v0.1 Bagian 4.
const W1: &str = concat!(
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
);

/// V1: password `kata sandi uji`, salt 11 × 16, nonce 22 × 24, Argon2id 64 KiB, 2 putaran, 1 jalur, isi frasa W1.
const V1_PASSWORD: &[u8] = b"kata sandi uji";
const V1_KDF: KdfParams = KdfParams {
    memory_kib: 64,
    iterations: 2,
    parallelism: 1,
};
const V1_KEY: &str = "76ae64e6040d33b83f389ab9f996346bf06b78106de65c291ae3fe5e8d9e50d5";
const V1_AAD: &str = concat!(
    "7a65767269676f2077616c6c6574207661756c74207631",
    "000000400000000200000001",
    "11111111111111111111111111111111"
);
const V1_CIPHERTEXT: &str = concat!(
    "efa16af8f8ddf1cdbc1b3a18c7804a97debde5cdc1513996958a965a02489a2c1d2be9eb10cbfd5c87bbd5bf3d35c4f3",
    "38f86961d38dfe3216cad2e5fd78335ee80aa37d46e519966f00a162c6d23113d59d97ecf1e237d1ad5014a3ba3440c7",
    "eb7a84c7df4efe96f6317a3296a6ef03ab5b4ecbc231547bfb8441f4e0b86c549288d57333153863d7e23bf40eb008d5",
    "6f13cdf175276d6b693f1e2bc8dde36b04b3f982afc0b9f83f20ff9c97983e5b434bb99a4aaab7052022e7df08941b90",
    "8e03a74c7cf569dfd98384"
);

/// V2: kunci Argon2id dengan memori minimum (8 KiB, 1 putaran), password `password`, salt 00 × 8 ‖ 01 × 8.
const V2_KEY: &str = "973a5f6faf3473b4041eecfdff2a54fdf28463c76ce98cf4b3b55b16dc888c65";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Byte `start`, `start + 1`, … sepanjang `N` (berputar di 255).
fn counting<const N: usize>(start: u8) -> [u8; N] {
    core::array::from_fn(|index| u8::try_from(index).unwrap().wrapping_add(start))
}

fn v1() -> Sealed {
    seal(W1.as_bytes(), V1_PASSWORD, V1_KDF, [0x11; 16], [0x22; 24]).unwrap()
}

#[test]
fn argon2id_matches_rfc9106() {
    let data = AssociatedData::new(&[0x04; 12]).unwrap();
    let params = ParamsBuilder::new()
        .m_cost(32)
        .t_cost(3)
        .p_cost(4)
        .data(data)
        .output_len(32)
        .build()
        .unwrap();
    let argon =
        Argon2::new_with_secret(&[0x03; 8], Algorithm::Argon2id, Version::V0x13, params).unwrap();
    let mut memory = vec![Block::new(); 32];
    let mut out = [0_u8; 32];
    argon
        .hash_password_into_with_memory(&[0x01; 32], &[0x02; 16], &mut out, memory.as_mut_slice())
        .unwrap();
    assert_eq!(
        hex(&out),
        "0d640df58d78766c08c037a34a8b53c9d01ef0452d75b65eb52520e96b01e659"
    );
}

#[test]
fn xchacha20poly1305_matches_draft_vector() {
    let plaintext = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
    let key: [u8; 32] = counting(0x80);
    let nonce: [u8; 24] = counting(0x40);
    let aad = [
        0x50, 0x51, 0x52, 0x53, 0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
    ];
    let cipher = XChaCha20Poly1305::new_from_slice(&key).unwrap();
    let sealed = cipher
        .encrypt(
            &XNonce::from(nonce),
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .unwrap();
    assert_eq!(
        hex(&sealed),
        concat!(
            "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb731c7f1b0b4aa6440bf3a82f4eda7e39",
            "ae64c6708c54c216cb96b72e1213b4522f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff9",
            "21f9664c97637da9768812f615c68b13b52ec0875924c1c7987947deafd8780acf49"
        )
    );
}

#[test]
fn seal_matches_reference_vectors() {
    let key = derive_key(V1_PASSWORD, &[0x11; 16], V1_KDF).unwrap();
    assert_eq!(hex(key.as_slice()), V1_KEY);
    assert_eq!(hex(&associated_data(V1_KDF, &[0x11; 16])), V1_AAD);
    let sealed = v1();
    assert_eq!(hex(&sealed.ciphertext), V1_CIPHERTEXT);
    assert_eq!(Some(sealed.ciphertext.len()), W1.len().checked_add(16));
    assert_eq!(sealed.kdf, V1_KDF);
    assert_eq!(sealed.salt, [0x11; 16]);
    assert_eq!(sealed.nonce, [0x22; 24]);
    let minimum = KdfParams {
        memory_kib: 8,
        iterations: 1,
        parallelism: 1,
    };
    let mut salt = [0x01; 16];
    salt.iter_mut().take(8).for_each(|byte| *byte = 0);
    assert_eq!(
        hex(derive_key(b"password", &salt, minimum).unwrap().as_slice()),
        V2_KEY
    );
}

#[test]
fn unseal_round_trips_and_rejects_wrong_password_or_tampering() {
    let sealed = v1();
    assert_eq!(
        unseal(&sealed, V1_PASSWORD).unwrap().as_slice(),
        W1.as_bytes()
    );
    assert_eq!(unseal(&sealed, b"kata sandi ujI"), Err(VaultError::Decrypt));
    assert_eq!(unseal(&sealed, b""), Err(VaultError::Decrypt));

    let mut flipped = sealed.clone();
    if let Some(byte) = flipped.ciphertext.last_mut() {
        *byte ^= 0x01;
    }
    let mut salt = sealed.clone();
    if let Some(byte) = salt.salt.first_mut() {
        *byte = 0x12;
    }
    let mut nonce = sealed.clone();
    nonce.nonce = [0x23; 24];
    let mut kdf = sealed.clone();
    kdf.kdf.iterations = 3;
    let mut short = sealed.clone();
    short.ciphertext.truncate(15);
    let mut empty = sealed.clone();
    empty.ciphertext.clear();
    for bad in [flipped, salt, nonce, kdf, short, empty] {
        assert_eq!(
            unseal(&bad, V1_PASSWORD),
            Err(VaultError::Decrypt),
            "{bad:?}"
        );
    }

    // Nonce lain: ciphertext berbeda, tetap bisa dibuka.
    let other = seal(W1.as_bytes(), V1_PASSWORD, V1_KDF, [0x11; 16], [0x23; 24]).unwrap();
    assert_ne!(other.ciphertext, sealed.ciphertext);
    assert_eq!(
        unseal(&other, V1_PASSWORD).unwrap().as_slice(),
        W1.as_bytes()
    );
}

#[test]
fn kdf_params_are_bounded() {
    assert_eq!(
        KdfParams::DEFAULT,
        KdfParams {
            memory_kib: 65_536,
            iterations: 3,
            parallelism: 1,
        }
    );
    assert_eq!(KdfParams::default(), KdfParams::DEFAULT);
    let with = |memory_kib, iterations, parallelism| KdfParams {
        memory_kib,
        iterations,
        parallelism,
    };
    let bad = [
        with(7, 1, 1),
        with(15, 1, 2),
        with(8, 0, 1),
        with(8, 65, 1),
        with(8, 1, 0),
        with(1_024, 1, 17),
        with(1_048_577, 1, 1),
    ];
    for kdf in bad {
        assert_eq!(
            seal(b"x", b"y", kdf, [0; 16], [0; 24]),
            Err(VaultError::InvalidParams),
            "{kdf:?}"
        );
        let sealed = Sealed {
            kdf,
            salt: [0; 16],
            nonce: [0; 24],
            ciphertext: vec![0; 32],
        };
        assert_eq!(
            unseal(&sealed, b"y"),
            Err(VaultError::InvalidParams),
            "{kdf:?}"
        );
    }
    let edge = seal(b"x", b"y", with(16, 1, 2), [0; 16], [0; 24]).unwrap();
    assert_eq!(unseal(&edge, b"y").unwrap().as_slice(), b"x");
}
