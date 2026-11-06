// Vektor uji resmi BIP340 diambil tanpa modifikasi dari:
// - URL: https://raw.githubusercontent.com/bitcoin/bips/master/bip-0340/test-vectors.csv
// - Lokasi lokal: crates/crypto/tests/data/bip340-test-vectors.csv
// - SHA-256: 34c9d1d9c3a88d524bc80778540dc43f8306ec249a7485293063c376db851c2d

use proptest::prelude::*;

use super::{PublicKey, SchnorrError, SecretKey, Signature, sign, verify};

const BIP340_CSV: &str = include_str!("../../tests/data/bip340-test-vectors.csv");

const SECP256K1_ORDER_HEX: &str =
    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141";
const SECP256K1_ORDER_MINUS_ONE_HEX: &str =
    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364140";
const SECP256K1_ORDER_PLUS_ONE_HEX: &str =
    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364142";
const SECP256K1_FIELD_PRIME_HEX: &str =
    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F";
const SECP256K1_FIELD_PRIME_PLUS_ONE_HEX: &str =
    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC30";

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10),
        b'A'..=b'F' => byte.checked_sub(b'A')?.checked_add(10),
        _ => None,
    }
}

fn decode_hex_vec(hex: &str) -> Option<Vec<u8>> {
    let (pairs, rem) = hex.as_bytes().as_chunks::<2>();
    if !rem.is_empty() {
        return None;
    }
    let mut out = Vec::with_capacity(pairs.len());
    for &[hi_ch, lo_ch] in pairs {
        let hi = hex_nibble(hi_ch)?;
        let lo = hex_nibble(lo_ch)?;
        let combined = hi.checked_shl(4)?.checked_add(lo)?;
        out.push(combined);
    }
    Some(out)
}

fn decode_hex_32(hex: &str) -> Option<[u8; 32]> {
    decode_hex_vec(hex)?.try_into().ok()
}

fn decode_hex_64(hex: &str) -> Option<[u8; 64]> {
    decode_hex_vec(hex)?.try_into().ok()
}

fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields = Vec::with_capacity(8);
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    let _ = chars.next();
                    current.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(ch);
            }
        } else if ch == '"' {
            in_quotes = true;
        } else if ch == ',' {
            fields.push(core::mem::take(&mut current));
        } else {
            current.push(ch);
        }
    }
    fields.push(current);
    fields
}

fn flip_bit_32(bytes: &[u8; 32], bit_index: usize) -> [u8; 32] {
    let mut out = *bytes;
    if let (Some(byte_idx), Some(rem)) = (bit_index.checked_div(8), bit_index.checked_rem(8))
        && let Ok(shift) = u32::try_from(rem)
        && let Some(mask) = 1_u8.checked_shl(shift)
        && let Some(slot) = out.get_mut(byte_idx)
    {
        *slot ^= mask;
    }
    out
}

fn flip_bit_64(bytes: &[u8; 64], bit_index: usize) -> [u8; 64] {
    let mut out = *bytes;
    if let (Some(byte_idx), Some(rem)) = (bit_index.checked_div(8), bit_index.checked_rem(8))
        && let Ok(shift) = u32::try_from(rem)
        && let Some(mask) = 1_u8.checked_shl(shift)
        && let Some(slot) = out.get_mut(byte_idx)
    {
        *slot ^= mask;
    }
    out
}

#[test]
fn bip340_anchor_vectors_0_and_1_match_exact() {
    // Jangkar Vektor 0 (persis dari kartu TSK-M04)
    let sk0_bytes =
        decode_hex_32("0000000000000000000000000000000000000000000000000000000000000003").unwrap();
    let aux0_bytes =
        decode_hex_32("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let msg0_bytes =
        decode_hex_32("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    let expected_pk0 =
        decode_hex_32("F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9").unwrap();
    let expected_sig0 = decode_hex_64(concat!(
        "E907831F80848D1069A5371B402410364BDF1C5F8307B0084C55F1CE2DCA8215",
        "25F66A4A85EA8B71E482A74F382D2CE5EBEEE8FDB2172F477DF4900D310536C0"
    ))
    .unwrap();

    let sk0 = SecretKey::from_bytes(&sk0_bytes).unwrap();
    let pk0 = sk0.public_key();
    assert_eq!(pk0.as_bytes(), &expected_pk0);
    assert_eq!(PublicKey::from_bytes(&expected_pk0).unwrap(), pk0);

    let sig0 = sign(&sk0, &msg0_bytes, &aux0_bytes).unwrap();
    assert_eq!(sig0.as_bytes(), &expected_sig0);
    assert!(verify(&pk0, &msg0_bytes, &sig0));

    // Jangkar Vektor 1 (persis dari kartu TSK-M04)
    let sk1_bytes =
        decode_hex_32("B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF").unwrap();
    let aux1_bytes =
        decode_hex_32("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
    let msg1_bytes =
        decode_hex_32("243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89").unwrap();
    let expected_pk1 =
        decode_hex_32("DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659").unwrap();
    let expected_sig1 = decode_hex_64(concat!(
        "6896BD60EEAE296DB48A229FF71DFE071BDE413E6D43F917DC8DCF8C78DE3341",
        "8906D11AC976ABCCB20B091292BFF4EA897EFCB639EA871CFA95F6DE339E4B0A"
    ))
    .unwrap();

    let sk1 = SecretKey::from_bytes(&sk1_bytes).unwrap();
    let pk1 = sk1.public_key();
    assert_eq!(pk1.as_bytes(), &expected_pk1);
    assert_eq!(PublicKey::from_bytes(&expected_pk1).unwrap(), pk1);

    let sig1 = sign(&sk1, &msg1_bytes, &aux1_bytes).unwrap();
    assert_eq!(sig1.as_bytes(), &expected_sig1);
    assert!(verify(&pk1, &msg1_bytes, &sig1));
}

#[test]
fn bip340_official_csv_vectors_all_32_byte_messages_pass() {
    let mut lines = BIP340_CSV.lines();
    let header = lines.next().unwrap();
    assert_eq!(
        header,
        "index,secret key,public key,aux_rand,message,signature,verification result,comment"
    );

    let mut tested_count: usize = 0;
    let mut signed_count: usize = 0;
    let mut skipped_non_32_byte_msgs: Vec<(u32, usize)> = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cols = parse_csv_row(line);
        assert_eq!(cols.len(), 8, "jumlah kolom CSV tidak sesuai: {line}");

        let index: u32 = cols.first().unwrap().parse().unwrap();
        let sk_hex = cols.get(1).unwrap();
        let pk_hex = cols.get(2).unwrap();
        let aux_hex = cols.get(3).unwrap();
        let msg_hex = cols.get(4).unwrap();
        let sig_hex = cols.get(5).unwrap();
        let verify_str = cols.get(6).unwrap();

        let msg_vec = decode_hex_vec(msg_hex).unwrap();
        if msg_vec.len() != 32 {
            // API Zevrigo hanya menerima pesan tepat 32 byte (sighash).
            skipped_non_32_byte_msgs.push((index, msg_vec.len()));
            continue;
        }

        let msg_bytes: [u8; 32] = msg_vec.try_into().unwrap();
        let pk_bytes = decode_hex_32(pk_hex).unwrap();
        let sig_bytes = decode_hex_64(sig_hex).unwrap();
        let expected_valid = match verify_str.as_str() {
            "TRUE" => true,
            "FALSE" => false,
            other => {
                assert_eq!(
                    other, "TRUE",
                    "nilai kolom verification result tidak dikenal"
                );
                false
            }
        };

        if !sk_hex.is_empty() {
            let sk_bytes = decode_hex_32(sk_hex).unwrap();
            let aux_bytes = decode_hex_32(aux_hex).unwrap();

            let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
            assert_eq!(
                sk.public_key().as_bytes(),
                &pk_bytes,
                "kunci publik turunan tidak cocok pada vektor #{index}"
            );

            let produced = sign(&sk, &msg_bytes, &aux_bytes).unwrap();
            assert_eq!(
                produced.as_bytes(),
                &sig_bytes,
                "tanda tangan tidak cocok pada vektor #{index}"
            );
            signed_count = signed_count.checked_add(1).unwrap();
        }

        // Jika PublicKey::from_bytes menolak kunci (mis. bukan titik di kurva), dianggap verifikasi gagal.
        let candidate_sig = Signature::from_bytes(&sig_bytes);
        let actual_valid = PublicKey::from_bytes(&pk_bytes)
            .is_ok_and(|pk| verify(&pk, &msg_bytes, &candidate_sig));

        assert_eq!(
            actual_valid, expected_valid,
            "hasil verifikasi tidak cocok pada vektor #{index}"
        );
        tested_count = tested_count.checked_add(1).unwrap();
    }

    // 15 vektor (indeks 0..=14) berpesan 32 byte diuji penuh (4 sign+verify, 11 verify-only);
    // 4 vektor tambahan Desember 2022 (indeks 15..=18) berpesan 0, 1, 17, dan 100 byte dilewati.
    assert_eq!(tested_count, 15);
    assert_eq!(signed_count, 4);
    assert_eq!(
        skipped_non_32_byte_msgs,
        vec![(15, 0), (16, 1), (17, 17), (18, 100)]
    );
}

#[test]
fn secret_key_rejects_zero_and_curve_order_and_accepts_boundaries() {
    assert_eq!(
        SecretKey::from_bytes(&[0_u8; 32]).unwrap_err(),
        SchnorrError::InvalidSecretKey
    );

    let mut one = [0_u8; 32];
    if let Some(last) = one.last_mut() {
        *last = 1;
    }
    assert!(SecretKey::from_bytes(&one).is_ok());

    let order_minus_one = decode_hex_32(SECP256K1_ORDER_MINUS_ONE_HEX).unwrap();
    assert!(SecretKey::from_bytes(&order_minus_one).is_ok());

    let order = decode_hex_32(SECP256K1_ORDER_HEX).unwrap();
    assert_eq!(
        SecretKey::from_bytes(&order).unwrap_err(),
        SchnorrError::InvalidSecretKey
    );

    let order_plus_one = decode_hex_32(SECP256K1_ORDER_PLUS_ONE_HEX).unwrap();
    assert_eq!(
        SecretKey::from_bytes(&order_plus_one).unwrap_err(),
        SchnorrError::InvalidSecretKey
    );

    assert_eq!(
        SecretKey::from_bytes(&[0xFF_u8; 32]).unwrap_err(),
        SchnorrError::InvalidSecretKey
    );
}

#[test]
fn public_key_rejects_off_curve_and_field_overflow() {
    // x = 0 bukan residu kuadratik pada y^2 = x^3 + 7 (mod p)
    assert_eq!(
        PublicKey::from_bytes(&[0_u8; 32]),
        Err(SchnorrError::InvalidPublicKey)
    );

    // Vektor BIP340 #5: x < p tetapi tidak berada di kurva secp256k1
    let off_curve =
        decode_hex_32("EEFDEA4CDB677750A420FEE807EACF21EB9898AE79B9768766E4FAA04A2D4A34").unwrap();
    assert_eq!(
        PublicKey::from_bytes(&off_curve),
        Err(SchnorrError::InvalidPublicKey)
    );

    // x == p dan x == p + 1 (di luar medan prima)
    let field_prime = decode_hex_32(SECP256K1_FIELD_PRIME_HEX).unwrap();
    assert_eq!(
        PublicKey::from_bytes(&field_prime),
        Err(SchnorrError::InvalidPublicKey)
    );

    let field_prime_plus_one = decode_hex_32(SECP256K1_FIELD_PRIME_PLUS_ONE_HEX).unwrap();
    assert_eq!(
        PublicKey::from_bytes(&field_prime_plus_one),
        Err(SchnorrError::InvalidPublicKey)
    );
}

#[test]
fn secret_key_debug_redacts_secret_and_sign_is_deterministic() {
    let sk_bytes =
        decode_hex_32("B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF").unwrap();
    let sk = SecretKey::from_bytes(&sk_bytes).unwrap();

    let dbg_str = format!("{sk:?}");
    assert_eq!(dbg_str, "SecretKey(<rahasia>)");
    assert!(!dbg_str.to_ascii_lowercase().contains("b7e15162"));

    let msg = [0x42_u8; 32];
    let aux_a = [0x11_u8; 32];
    let aux_b = [0x22_u8; 32];

    let sig_a1 = sign(&sk, &msg, &aux_a).unwrap();
    let sig_a2 = sign(&sk, &msg, &aux_a).unwrap();
    assert_eq!(sig_a1, sig_a2);

    let sig_b = sign(&sk, &msg, &aux_b).unwrap();
    assert_ne!(sig_a1, sig_b);

    let pk = sk.public_key();
    assert!(verify(&pk, &msg, &sig_a1));
    assert!(verify(&pk, &msg, &sig_b));
}

fn normalize_valid_secret_key(raw: [u8; 32]) -> SecretKey {
    if let Ok(sk) = SecretKey::from_bytes(&raw) {
        return sk;
    }
    let mut fallback = raw;
    if let Some(first) = fallback.first_mut() {
        *first = 0;
    }
    if let Some(last) = fallback.last_mut() {
        *last |= 1;
    }
    SecretKey::from_bytes(&fallback).unwrap()
}

proptest! {
    #[test]
    fn prop_single_bit_flip_in_msg_sig_or_pk_invalidates_verification(
        raw_sk in any::<[u8; 32]>(),
        msg in any::<[u8; 32]>(),
        aux in any::<[u8; 32]>(),
        msg_bit in 0_usize..256_usize,
        sig_bit in 0_usize..512_usize,
        pk_bit in 0_usize..256_usize,
    ) {
        let sk = normalize_valid_secret_key(raw_sk);
        let pk = sk.public_key();
        let sig = sign(&sk, &msg, &aux).unwrap();

        prop_assert!(verify(&pk, &msg, &sig));

        // 1. Membalik satu bit pada pesan wajib menggagalkan verifikasi
        let mutated_msg = flip_bit_32(&msg, msg_bit);
        prop_assert!(!verify(&pk, &mutated_msg, &sig));

        // 2. Membalik satu bit pada tanda tangan wajib menggagalkan verifikasi
        let mutated_sig = Signature::from_bytes(&flip_bit_64(sig.as_bytes(), sig_bit));
        prop_assert!(!verify(&pk, &msg, &mutated_sig));

        // 3. Membalik satu bit pada kunci publik wajib ditolak from_bytes atau gagal verify
        let mutated_pk_bytes = flip_bit_32(pk.as_bytes(), pk_bit);
        let mutated_pk_valid = PublicKey::from_bytes(&mutated_pk_bytes)
            .is_ok_and(|bad_pk| verify(&bad_pk, &msg, &sig));
        prop_assert!(!mutated_pk_valid);
    }
}
