// W1 sampai W10: Spesifikasi Wallet v0.1 Bagian 4, diuji ujung ke ujung lewat re-export crate.

use zevrigo_primitives::{Network, OutputKind};

use crate::{
    CHANGE_BRANCH, ExtendedPubKey, MODULE_ID, Mnemonic, NEW_WALLET_ENTROPY_LEN, RECEIVE_BRANCH,
    account_key, decode_address, encode_address,
};

const W5: &str = "rhsis1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939msequa3ft3";
const W6: &str = "rhsis1q3uesn8pvcdeeufxr98v794e07l8ghd85fyd3wclknj5xj04rfqrqvpxaeu";
const W7: &str = "rhsis1qcuwd6ufjy3relleyk4km2fxz8pdv7ztj2cz7w3qdurp5yuhz406sx6fc5n";
const W8_PHRASE: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";
const W8: &str = "rhsis1qpq864wpnc3u8h246ct450aj2ye20j2smwr9gmshsjeu30xkrv2ss0hnt4g";
const W9: &str = "rhsis1q2x064qcn4zk84y9zjtpjt8e8ksjjja7peenap4lcse2d0vmw2xyqnjjhdv";
const W10: &str = "thsis1qtfhjsfflxh307x4skzrt3jrlwvg4jes32ljmut9fsxyrd939mseqa3ymmj";

fn receive_address(mnemonic: &Mnemonic, passphrase: &str, network: Network, index: u32) -> String {
    let seed = mnemonic.to_seed(passphrase).expect("passphrase ASCII");
    let account = account_key(&seed, network).expect("akun regtest/testnet");
    let key = account
        .derive_path(&[RECEIVE_BRANCH, index])
        .expect("turunan sah")
        .to_public()
        .expect("kunci publik sah");
    encode_address(network, OutputKind::SingleKey, &key.x_only())
}

#[test]
fn module_id_matches() {
    assert_eq!(MODULE_ID, "MOD-09");
}

#[test]
fn spec_wallet_vectors_w5_to_w10_end_to_end() {
    let w1 = Mnemonic::from_entropy(&[0_u8; NEW_WALLET_ENTROPY_LEN]).expect("entropi sah");
    assert_eq!(receive_address(&w1, "", Network::Regtest, 0), W5);
    assert_eq!(receive_address(&w1, "", Network::Regtest, 1), W6);
    let w8 = Mnemonic::parse(W8_PHRASE).expect("frasa sah");
    assert_eq!(receive_address(&w8, "", Network::Regtest, 0), W8);
    assert_eq!(receive_address(&w1, "TREZOR", Network::Regtest, 0), W9);
    assert_eq!(receive_address(&w1, "", Network::Testnet, 0), W10);

    // W7 lewat watch-only: kunci akun publik, cabang kembalian, tanpa kunci rahasia.
    let seed = w1.to_seed("").expect("tanpa passphrase");
    let account_hex = account_key(&seed, Network::Regtest)
        .expect("akun regtest")
        .to_public()
        .expect("kunci publik akun")
        .to_account_hex();
    let watch_only = ExtendedPubKey::from_account_hex(&account_hex).expect("kunci akun sah");
    let change = watch_only
        .derive_path(&[CHANGE_BRANCH, 0])
        .expect("turunan publik sah");
    assert_eq!(
        encode_address(Network::Regtest, OutputKind::SingleKey, &change.x_only()),
        W7
    );

    // Alamat dekode kembali ke x-only yang sama; regtest menolak alamat testnet.
    assert_eq!(
        decode_address(Network::Regtest, W7),
        Ok((OutputKind::SingleKey, change.x_only()))
    );
    assert!(decode_address(Network::Regtest, W10).is_err());
}
