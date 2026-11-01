// T1: transaksi 2 input + kembalian, dihitung Quick dengan implementasi referensi Python (BIP340 Python yang lolos
// vektor resmi BIP340, BLAKE3 alat/blake3_ref.py, turunan kunci alat/wallet_ref.py). Wallet W1 (Spesifikasi Wallet
// v0.1 Bagian 4): koin di m/44'/1'/0'/0/0 (W5), 0/1 (W6), dan 1/0 (W7); kembalian ke 1/1. network_id contoh 5a × 32,
// aux_rand nol, tarif 1 quanta/byte, tinggi ujung 150. Kasus batas biaya (E1 sampai E6) dari simulasi Python yang
// sama.

use std::collections::BTreeMap;

use homeostasis_crypto::{PublicKey, Signature, sighash_v1, txid, verify};
use homeostasis_primitives::{
    Amount, Encode, Hash32, Network, OutPoint, Output, OutputKind, TxBody, Witness,
};
use proptest::prelude::*;

use super::{Coin, Plan, SendError, SendRequest, build_transaction, plan, sign_plan};
use crate::bip32::{CHANGE_BRANCH, ExtendedPrivKey, RECEIVE_BRANCH, account_key};
use crate::mnemonic::Mnemonic;

const HSIS: u128 = 1_000_000_000_000_000_000;
const THREE_HSIS: u128 = 3 * HSIS;
const FIVE_HSIS: u128 = 5 * HSIS;
const SIX_HSIS: u128 = 6 * HSIS;
const SEVEN_HSIS: u128 = 7 * HSIS;
/// Kembalian T1: 8 HSIS − 6 HSIS − biaya 284.
const T1_CHANGE: u128 = 2 * HSIS - 284;
const HSIS_MINUS_ONE: u128 = HSIS - 1;
const W5_XONLY: &str = "5a6f28253f35e2ff1ab0b086b8c87f731159661157e5be2ca98188369625dc32";
const W6_XONLY: &str = "8f33099c2cc3739e24c329d9e2d72ff7ce8bb4f4491b1763f69ca8693ea34806";
const W7_XONLY: &str = "c71cdd713224479fff24b56db524c2385acf09725605e7440de0c34272e2abf5";
const CHANGE_1_1_XONLY: &str = "730ad008ce8bd2f391b08b9c4838af732341e9704cba0dc03d83f977dfcf7d96";
const RECIPIENT: &str = "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9";
const T1_TX: &str = concat!(
    "010002a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a100b2b2b2b2b2b2b2b2b2b2b2b2",
    "b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b201028080e0e2de8692a25300f9308a019258c31049344f85f89d5229",
    "b531c845836f99b08601f113bce036f9e4fd9ff6f4acdbe01b00730ad008ce8bd2f391b08b9c4838af732341e9704cba",
    "0dc03d83f977dfcf7d9600f98278c858ef3dfca1d0bfea415e604a45681c23ff65f0040a110202af619e8ffccbe7b7b4",
    "34efd3f7d66cff1c5ba294d04803b2bd298b1ba5aae19dc09ba56f00d208bcda0d1e9ae93e1e68f9f298d75c4d86a649",
    "928be7d63a48765340a35ecd11acf3b31110d7b8eee5b491e1220be4d8a084e53632fd23f4f7fbd9d57157dd",
);
const T1_TXID: &str = "2dd7f50382bab1376d77991113bfde898a48d9f1594670dd88f74e61e2ea4d86";

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

fn outpoint(fill: u8, index: u32) -> OutPoint {
    OutPoint::new(Hash32::from_bytes([fill; 32]), index)
}

fn w1_account() -> ExtendedPrivKey {
    let seed = Mnemonic::from_entropy(&[0_u8; 32])
        .expect("entropi sah")
        .to_seed("")
        .expect("tanpa passphrase");
    account_key(&seed, Network::Regtest).expect("akun regtest")
}

fn t1_coins() -> Vec<Coin> {
    vec![
        Coin::new(
            outpoint(0xa1, 0),
            Amount::from_quanta(FIVE_HSIS),
            h32(W5_XONLY),
            101,
            RECEIVE_BRANCH,
            0,
        ),
        Coin::new(
            outpoint(0xb2, 1),
            Amount::from_quanta(THREE_HSIS),
            h32(W6_XONLY),
            50,
            RECEIVE_BRANCH,
            1,
        ),
        // Belum matang di tinggi ujung 150 (spendable_at 200), jadi tidak dipilih.
        Coin::new(
            outpoint(0xc3, 0),
            Amount::from_quanta(SEVEN_HSIS),
            h32(W7_XONLY),
            200,
            CHANGE_BRANCH,
            0,
        ),
    ]
}

fn t1_request() -> SendRequest {
    SendRequest {
        recipient_kind: OutputKind::SingleKey,
        recipient_payload: h32(RECIPIENT),
        amount: Amount::from_quanta(SIX_HSIS),
        fee_rate: 1,
        change_payload: h32(CHANGE_1_1_XONLY),
        tip_height: 150,
    }
}

/// Satu koin sederhana untuk uji pemilihan (payload tidak dipakai karena tidak ditandatangani).
fn coin(fill: u8, index: u32, quanta: u128, spendable_at: u64) -> Coin {
    Coin::new(
        outpoint(fill, index),
        Amount::from_quanta(quanta),
        [0_u8; 32],
        spendable_at,
        RECEIVE_BRANCH,
        0,
    )
}

fn request(quanta: u128, fee_rate: u128) -> SendRequest {
    SendRequest {
        recipient_kind: OutputKind::SingleKey,
        recipient_payload: [0x11; 32],
        amount: Amount::from_quanta(quanta),
        fee_rate,
        change_payload: [0x22; 32],
        tip_height: 10,
    }
}

fn picked(chosen: &Plan) -> Vec<OutPoint> {
    chosen.coins().iter().map(Coin::outpoint).collect()
}

#[test]
fn signed_vector_t1_two_inputs_with_change() {
    let (chosen, tx) = build_transaction(
        &t1_coins(),
        &t1_request(),
        &w1_account(),
        &Hash32::from_bytes([0x5a; 32]),
        &[0_u8; 32],
    )
    .expect("transaksi T1");
    assert_eq!(picked(&chosen), vec![outpoint(0xa1, 0), outpoint(0xb2, 1)]);
    assert_eq!(chosen.fee(), Amount::from_quanta(284));
    assert_eq!(chosen.change(), Some(Amount::from_quanta(T1_CHANGE)));
    assert_eq!(chosen.size(), 284);
    assert_eq!(tx.encoded_size(), Some(284));
    assert_eq!(tx.to_bytes(), decode_hex(T1_TX));
    assert_eq!(txid(&tx), Hash32::from_hex(T1_TXID).expect("hex"));
    let outputs = tx.body().outputs();
    assert_eq!(outputs, chosen.outputs());
    assert_eq!(outputs.len(), 2);
    assert!(
        outputs
            .first()
            .is_some_and(|out| out.payload() == &h32(RECIPIENT))
    );
    assert!(
        outputs
            .get(1)
            .is_some_and(|out| out.payload() == &h32(CHANGE_1_1_XONLY))
    );
}

#[test]
fn signatures_verify_and_commit_to_network_id() {
    let network_id = Hash32::from_bytes([0x5a; 32]);
    let (chosen, tx) = build_transaction(
        &t1_coins(),
        &t1_request(),
        &w1_account(),
        &network_id,
        &[0_u8; 32],
    )
    .expect("transaksi T1");
    let spent: Vec<Output> = chosen
        .coins()
        .iter()
        .map(|c| Output::new(c.amount(), OutputKind::SingleKey, *c.payload()).expect("output"))
        .collect();
    assert!(matches!(tx.body(), TxBody::Regular(_)));
    let other_network = Hash32::from_bytes([0x5b; 32]);
    for (position, (witness, coin)) in tx.witnesses().iter().zip(chosen.coins()).enumerate() {
        let Witness::SingleKey(sig) = witness else {
            unreachable!("wallet v0.1 hanya membuat witness jenis 0");
        };
        let index = u32::try_from(position).expect("indeks kecil");
        let key = PublicKey::from_bytes(coin.payload()).expect("kunci sah");
        let signature = Signature::from_bytes(sig);
        let msg = sighash_v1(&network_id, &tx, index, &spent).expect("sighash");
        assert!(verify(&key, msg.as_bytes(), &signature));
        let replay = sighash_v1(&other_network, &tx, index, &spent).expect("sighash");
        assert!(!verify(&key, replay.as_bytes(), &signature));
    }
}

#[test]
fn plan_orders_by_amount_then_outpoint() {
    let coins = [
        coin(3, 0, 500, 0),
        coin(1, 5, 500, 0),
        coin(1, 2, 500, 0),
        coin(2, 0, 900, 0),
    ];
    let chosen = plan(&coins, &request(1_500, 0)).expect("cukup");
    assert_eq!(
        picked(&chosen),
        vec![outpoint(2, 0), outpoint(1, 2), outpoint(1, 5)]
    );
    assert_eq!(chosen.fee(), Amount::ZERO);
    assert_eq!(chosen.change(), Some(Amount::from_quanta(400)));
}

#[test]
fn plan_respects_spendable_at_boundary() {
    // Tinggi ujung 10: spendable_at 11 boleh dipakai di blok berikutnya, 12 belum.
    let coins = [coin(1, 0, 5_000, 11), coin(2, 0, 4_000, 12)];
    let chosen = plan(&coins, &request(1_000, 1)).expect("cukup");
    assert_eq!(picked(&chosen), vec![outpoint(1, 0)]);
    assert_eq!(
        plan(&[coin(1, 0, 5_000, 12)], &request(1_000, 1)),
        Err(SendError::InsufficientFunds {
            available: Amount::ZERO,
            needed: Amount::from_quanta(1_000),
        })
    );
}

#[test]
fn plan_fee_and_change_edge_cases() {
    // Satu input, kirim 1_000 quanta, tarif 1: ukuran tanpa kembalian 137 byte, dengan kembalian 1 byte 171 byte.
    // (total koin, biaya, kembalian, ukuran)
    let cases = [
        // E1: pas tanpa sisa.
        (1_137_u128, 137_u128, None, 137_usize),
        // E2: sisa 30 tidak cukup untuk output kembalian, ikut menjadi biaya.
        (1_167, 167, None, 137),
        // E3: sisa 33 juga tidak cukup (output kembalian butuh 34 byte biaya + 1 quanta).
        (1_170, 170, None, 137),
        // E4: sisa 34 hanya cukup untuk biaya output kembalian, kembalian 0, jadi ikut menjadi biaya.
        (1_171, 171, None, 137),
        // E5: kembalian 1 quanta tetap sah.
        (1_172, 171, Some(1_u128), 171),
        // E6: batas varint. Dengan 1 byte, kembalian 128 butuh 2 byte; dengan 2 byte, kembalian 127 cukup 1 byte.
        (1_299, 172, Some(127), 171),
    ];
    for (total, fee, change, size) in cases {
        let chosen = plan(&[coin(1, 0, total, 0)], &request(1_000, 1)).expect("cukup");
        assert_eq!(chosen.fee(), Amount::from_quanta(fee), "total {total}");
        assert_eq!(
            chosen.change(),
            change.map(Amount::from_quanta),
            "total {total}"
        );
        assert_eq!(chosen.size(), size, "total {total}");
        let expected_outputs = if change.is_some() { 2 } else { 1 };
        assert_eq!(chosen.outputs().len(), expected_outputs, "total {total}");
    }
    // Tarif 0: seluruh sisa menjadi kembalian.
    let free = plan(&[coin(1, 0, HSIS, 0)], &request(1, 0)).expect("cukup");
    assert_eq!(free.fee(), Amount::ZERO);
    assert_eq!(free.change(), Some(Amount::from_quanta(HSIS_MINUS_ONE)));
    assert_eq!(free.size(), 178);
}

#[test]
fn plan_rejects_zero_duplicate_and_insufficient() {
    assert_eq!(
        plan(&[coin(1, 0, 5_000, 0)], &request(0, 1)),
        Err(SendError::ZeroAmount)
    );
    assert_eq!(
        plan(
            &[coin(1, 0, 5_000, 0), coin(1, 0, 7_000, 0)],
            &request(1_000, 1)
        ),
        Err(SendError::DuplicateCoin {
            txid: Hash32::from_bytes([1; 32]),
            index: 0,
        })
    );
    // Dua input: ukuran tanpa kembalian 235 byte, kebutuhan 1_000 + 235.
    assert_eq!(
        plan(
            &[coin(1, 0, 500, 0), coin(2, 0, 400, 0)],
            &request(1_000, 1)
        ),
        Err(SendError::InsufficientFunds {
            available: Amount::from_quanta(900),
            needed: Amount::from_quanta(1_235),
        })
    );
    assert_eq!(
        plan(&[coin(1, 0, 5_000, 0)], &request(1_000, u128::MAX)),
        Err(SendError::AmountOverflow)
    );
}

#[test]
fn plan_rejects_transaction_larger_than_block() {
    // 1_100 koin 1 quanta: 1_020 input masih 100_000 byte, input ke-1_021 membuatnya 100_098 byte.
    let coins: Vec<Coin> = (0_u32..1_100)
        .map(|n| {
            let mut txid = [0_u8; 32];
            txid.get_mut(..4)
                .expect("32 byte")
                .copy_from_slice(&n.to_be_bytes());
            Coin::new(
                OutPoint::new(Hash32::from_bytes(txid), 0),
                Amount::from_quanta(1),
                [0_u8; 32],
                0,
                RECEIVE_BRANCH,
                0,
            )
        })
        .collect();
    assert_eq!(
        plan(&coins, &request(1_100, 0)),
        Err(SendError::TooLarge { size: 100_098 })
    );
}

#[test]
fn sign_rejects_key_mismatch() {
    // Payload W5 (0/0) dengan jalur 0/1: kunci turunan tidak cocok.
    let wrong = [Coin::new(
        outpoint(0xa1, 0),
        Amount::from_quanta(FIVE_HSIS),
        h32(W5_XONLY),
        0,
        RECEIVE_BRANCH,
        1,
    )];
    let mut one_hsis = t1_request();
    one_hsis.amount = Amount::from_quanta(HSIS);
    let chosen = plan(&wrong, &one_hsis).expect("cukup");
    assert_eq!(
        sign_plan(
            &chosen,
            &w1_account(),
            &Hash32::from_bytes([0x5a; 32]),
            &[0_u8; 32]
        ),
        Err(SendError::KeyMismatch {
            txid: Hash32::from_bytes([0xa1; 32]),
            index: 0,
        })
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn prop_plan_balances_and_pays_at_least_rate(
        raw in prop::collection::vec(
            (0_u8..4, prop_oneof![Just(0_u32), Just(1), Just(127), Just(128), Just(300)], 1_u128..=10_000_000_000_000_000_000_000, 0_u64..=12),
            1..6,
        ),
        amount in 1_u128..=1_000_000_000_000_000_000_000,
        fee_rate in prop_oneof![Just(0_u128), Just(1), Just(2), Just(1_000), Just(1_000_000)],
    ) {
        let mut unique = BTreeMap::new();
        for (fill, index, quanta, spendable_at) in raw {
            unique.entry((fill, index)).or_insert(coin(fill, index, quanta, spendable_at));
        }
        let coins: Vec<Coin> = unique.into_values().collect();
        let Ok(chosen) = plan(&coins, &request(amount, fee_rate)) else {
            return Ok(());
        };
        let inputs = Amount::checked_sum(chosen.coins().iter().map(Coin::amount)).expect("tidak meluap");
        let outputs = Amount::checked_sum(chosen.outputs().iter().map(Output::amount)).expect("tidak meluap");
        prop_assert_eq!(inputs, outputs.checked_add(chosen.fee()).expect("tidak meluap"));
        let size = u128::try_from(chosen.size()).expect("muat u128");
        prop_assert!(chosen.fee().as_quanta() >= fee_rate.checked_mul(size).expect("tidak meluap"));
        prop_assert!(chosen.coins().iter().all(|c| c.spendable_at() <= 11));
        prop_assert_eq!(chosen.outputs().first().map(Output::amount), Some(Amount::from_quanta(amount)));
    }
}
