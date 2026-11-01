// Vektor resmi BIP39 (Trezor) diambil tanpa modifikasi dari:
// - URL: https://raw.githubusercontent.com/trezor/python-mnemonic/master/vectors.json (bagian "english")
// - Semua vektor memakai passphrase "TREZOR". Kolom master_secret dan master_chain_code adalah isi xprv resmi
//   (Base58Check didekode Quick: 32 byte kunci rahasia dan 32 byte chain code kunci induk BIP32).
// W1 sampai W3: Spesifikasi Wallet v0.1 Bagian 4.

use proptest::prelude::*;
use sha2::{Digest, Sha256};

use super::{
    ENGLISH, Mnemonic, MnemonicError, NEW_WALLET_ENTROPY_LEN, SEED_LEN, WORDLIST_LEN, words,
};
use crate::bip32::ExtendedPrivKey;

const ENGLISH_SHA256: &str = "2f5eed53a4727b4bf8880d8f3f199efc90e58503646d9ff8eff3a2ed3b24dbda";

const W1: &str = concat!(
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon ",
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
);
const W2: &str = concat!(
    "408b285c123836004f4b8842c89324c1f01382450c0d439af345ba7fc49acf70",
    "5489c6fc77dbd4e3dc1dd8cc6bc9f043db8ada1e243c4a0eafb290d399480840"
);
const W3: &str = concat!(
    "bda85446c68413707090a52022edd26a1c9462295029f2e60cd7c4f2bbd30971",
    "70af7a4d73245cafa9c3cca8d561a7c3de6f5d4a10be8ed2a5e608d68f92fcc8"
);

struct TrezorVector {
    entropy: &'static str,
    phrase: &'static str,
    seed: &'static str,
    master_secret: &'static str,
    master_chain_code: &'static str,
}

const TREZOR: [TrezorVector; 24] = [
    TrezorVector {
        entropy: "00000000000000000000000000000000",
        phrase: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        seed: concat!(
            "c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e5349553",
            "1f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04"
        ),
        master_secret: "cbedc75b0d6412c85c79bc13875112ef912fd1e756631b5a00330866f22ff184",
        master_chain_code: "a3fa8c983223306de0f0f65e74ebb1e98aba751633bf91d5fb56529aa5c132c1",
    },
    TrezorVector {
        entropy: "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
        phrase: "legal winner thank year wave sausage worth useful legal winner thank yellow",
        seed: concat!(
            "2e8905819b8723fe2c1d161860e5ee1830318dbf49a83bd451cfb8440c28bd6f",
            "a457fe1296106559a3c80937a1c1069be3a3a5bd381ee6260e8d9739fce1f607"
        ),
        master_secret: "dddda5cdef032caf0b966bb1c7d2a8836e827aaa6480e9067080a075656d3228",
        master_chain_code: "3dff8e4e898ecd7f09dd62023bd6ca129312216b427d4f6b650f456da06b543f",
    },
    TrezorVector {
        entropy: "80808080808080808080808080808080",
        phrase: "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
        seed: concat!(
            "d71de856f81a8acc65e6fc851a38d4d7ec216fd0796d0a6827a3ad6ed5511a30",
            "fa280f12eb2e47ed2ac03b5c462a0358d18d69fe4f985ec81778c1b370b652a8"
        ),
        master_secret: "2fd0c70b975d9a38f84fba956ed795075fe43a85a45336143fe855551f1356db",
        master_chain_code: "51fd417898af090d1a4dc040f4427a7e6a4ad5fd4024ace7f068a167861e8655",
    },
    TrezorVector {
        entropy: "ffffffffffffffffffffffffffffffff",
        phrase: "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
        seed: concat!(
            "ac27495480225222079d7be181583751e86f571027b0497b5b5d11218e0a8a13",
            "332572917f0f8e5a589620c6f15b11c61dee327651a14c34e18231052e48c069"
        ),
        master_secret: "e1330e46e88f1c65cc1e228a16e3f0b94a316ae4fcfda1df4996b85c70d7b909",
        master_chain_code: "2aca5330a87a7890f10c849ebf658c3cef30cc5bd9d3124bd236185d4cfcb107",
    },
    TrezorVector {
        entropy: "000000000000000000000000000000000000000000000000",
        phrase: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon agent",
        seed: concat!(
            "035895f2f481b1b0f01fcf8c289c794660b289981a78f8106447707fdd9666ca",
            "06da5a9a565181599b79f53b844d8a71dd9f439c52a3d7b3e8a79c906ac845fa"
        ),
        master_secret: "410bbd9109b987e9e078ec6035745fa05515fda018214013153a5e6ed671d1b2",
        master_chain_code: "ab389a07217fe4ebb57f0a3385a05455ecd351e3506c758bb7a15fc3f0bf0c56",
    },
    TrezorVector {
        entropy: "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
        phrase: "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal will",
        seed: concat!(
            "f2b94508732bcbacbcc020faefecfc89feafa6649a5491b8c952cede496c214a",
            "0c7b3c392d168748f2d4a612bada0753b52a1c7ac53c1e93abd5c6320b9e95dd"
        ),
        master_secret: "b3226e9690ec49ce0cefe1c28b340d129779f2933857186400d92e0e2449f398",
        master_chain_code: "811e89816556573e1565f142f3d86e7210ac3ae4708156abaa2076f4176ea228",
    },
    TrezorVector {
        entropy: "808080808080808080808080808080808080808080808080",
        phrase: "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter always",
        seed: concat!(
            "107d7c02a5aa6f38c58083ff74f04c607c2d2c0ecc55501dadd72d025b751bc2",
            "7fe913ffb796f841c49b1d33b610cf0e91d3aa239027f5e99fe4ce9e5088cd65"
        ),
        master_secret: "fa8d0cde12d29eebecac474e4927e52cb80613e009f4208dc1e887b7c00c4fcf",
        master_chain_code: "8fc7f88d98e5f8569f589609920a2c1dc0489993b19dcdbe7b3a4a53c0e85b66",
    },
    TrezorVector {
        entropy: "ffffffffffffffffffffffffffffffffffffffffffffffff",
        phrase: "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo when",
        seed: concat!(
            "0cd6e5d827bb62eb8fc1e262254223817fd068a74b5b449cc2f667c3f1f985a7",
            "6379b43348d952e2265b4cd129090758b3e3c2c49103b5051aac2eaeb890a528"
        ),
        master_secret: "2786d73ac9c0261e01880df13e2751c89d3ac9f0cbdfd93fff342f122f80d04a",
        master_chain_code: "6794973b75f4fd626030d7686b8617024bbb5c9df31ce43b9f48074b61e672eb",
    },
    TrezorVector {
        entropy: "0000000000000000000000000000000000000000000000000000000000000000",
        phrase: "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
        seed: concat!(
            "bda85446c68413707090a52022edd26a1c9462295029f2e60cd7c4f2bbd30971",
            "70af7a4d73245cafa9c3cca8d561a7c3de6f5d4a10be8ed2a5e608d68f92fcc8"
        ),
        master_secret: "c8b4073ccfcc63475c3d5202c6594484ee4e77b867cde3c3b46432fd71b467ae",
        master_chain_code: "61ccb2bbe7d2a4fccd5f418ee931db7cbac9a153ec43b0ae3759ff991e4d23d1",
    },
    TrezorVector {
        entropy: "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
        phrase: "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title",
        seed: concat!(
            "bc09fca1804f7e69da93c2f2028eb238c227f2e9dda30cd63699232578480a40",
            "21b146ad717fbb7e451ce9eb835f43620bf5c514db0f8add49f5d121449d3e87"
        ),
        master_secret: "fe743fd5f203af53ccd9568df27f6d09d7f756a21ae9cd6135506844a90f45cf",
        master_chain_code: "9456f08a27d495d27cc0742249b0066b6dfc9f16924ee713eecf544788c4a875",
    },
    TrezorVector {
        entropy: "8080808080808080808080808080808080808080808080808080808080808080",
        phrase: "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic bless",
        seed: concat!(
            "c0c519bd0e91a2ed54357d9d1ebef6f5af218a153624cf4f2da911a0ed8f7a09",
            "e2ef61af0aca007096df430022f7a2b6fb91661a9589097069720d015e4e982f"
        ),
        master_secret: "af8b72038eef1f2c1ab6ce2ddbb50d149f744396b04ced94e7bf38d2ff14ebd9",
        master_chain_code: "7272c5335d54c2b86a2e9e8be5b167f501f622ae0ca9710dbb12c90963113d1d",
    },
    TrezorVector {
        entropy: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        phrase: "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote",
        seed: concat!(
            "dd48c104698c30cfe2b6142103248622fb7bb0ff692eebb00089b32d22484e16",
            "13912f0a5b694407be899ffd31ed3992c456cdf60f5d4564b8ba3f05a69890ad"
        ),
        master_secret: "40c1cf7c7d5fcd6a4b1f8460efb62a47c3680e4c378d70e6ffa5b5baa310efac",
        master_chain_code: "2cd568d37c19eb04f6c8a06eb32c3058ab1dc8709d411afd259d6a82c967c395",
    },
    TrezorVector {
        entropy: "9e885d952ad362caeb4efe34a8e91bd2",
        phrase: "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic",
        seed: concat!(
            "274ddc525802f7c828d8ef7ddbcdc5304e87ac3535913611fbbfa986d0c9e547",
            "6c91689f9c8a54fd55bd38606aa6a8595ad213d4c9c9f9aca3fb217069a41028"
        ),
        master_secret: "ac0164daefba477910d503a14743787982a118ca53adbc0044561b0088bd2e53",
        master_chain_code: "4acee798a1f034cd5a253a68dd61ee26d195fb9c5beab3dfba78fb37617ad533",
    },
    TrezorVector {
        entropy: "6610b25967cdcca9d59875f5cb50b0ea75433311869e930b",
        phrase: "gravity machine north sort system female filter attitude volume fold club stay feature office ecology stable narrow fog",
        seed: concat!(
            "628c3827a8823298ee685db84f55caa34b5cc195a778e52d45f59bcf75aba68e",
            "4d7590e101dc414bc1bbd5737666fbbef35d1f1903953b66624f910feef245ac"
        ),
        master_secret: "0bf8f75e9eb03c1fe723da7e30cae8d267a9adf4091dc8140868cbbf16f650df",
        master_chain_code: "b975e8d7517ed618cbbebe87555e415874438b670c9e54e57f70ff02a15c4c10",
    },
    TrezorVector {
        entropy: "68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
        phrase: "hamster diagram private dutch cause delay private meat slide toddler razor book happy fancy gospel tennis maple dilemma loan word shrug inflict delay length",
        seed: concat!(
            "64c87cde7e12ecf6704ab95bb1408bef047c22db4cc7491c4271d170a1b213d2",
            "0b385bc1588d9c7b38f1b39d415665b8a9030c9ec653d75e65f847d8fc1fc440"
        ),
        master_secret: "f2a26d6881b5d3bbdc4e16f1339f345d688eb129b5f5a602cfb1ab9db09218a0",
        master_chain_code: "2eebe52a1cbe55cbb845c4f8f03bd4580322d7f4bf3c6dd6d2276949bcf80afd",
    },
    TrezorVector {
        entropy: "c0ba5a8e914111210f2bd131f3d5e08d",
        phrase: "scheme spot photo card baby mountain device kick cradle pact join borrow",
        seed: concat!(
            "ea725895aaae8d4c1cf682c1bfd2d358d52ed9f0f0591131b559e2724bb234fc",
            "a05aa9c02c57407e04ee9dc3b454aa63fbff483a8b11de949624b9f1831a9612"
        ),
        master_secret: "4ff56dc2209441bb4be12a366816727c343d5fd0e12c004505fe270ae3819e10",
        master_chain_code: "784bebd7edb5325fc560d025468801e3ef9a14229495b1a28a2bfa4222a6a84f",
    },
    TrezorVector {
        entropy: "6d9be1ee6ebd27a258115aad99b7317b9c8d28b6d76431c3",
        phrase: "horn tenant knee talent sponsor spell gate clip pulse soap slush warm silver nephew swap uncle crack brave",
        seed: concat!(
            "fd579828af3da1d32544ce4db5c73d53fc8acc4ddb1e3b251a31179cdb71e853",
            "c56d2fcb11aed39898ce6c34b10b5382772db8796e52837b54468aeb312cfc3d"
        ),
        master_secret: "489f1bafdb0c9468aa153f54d9326f3504919484323190dbd5dd32c10f4d7587",
        master_chain_code: "883459bc464518454d0a3e6f63478fcae6affb0dbc9787a8dc2e136205836900",
    },
    TrezorVector {
        entropy: "9f6a2878b2520799a44ef18bc7df394e7061a224d2c33cd015b157d746869863",
        phrase: "panda eyebrow bullet gorilla call smoke muffin taste mesh discover soft ostrich alcohol speed nation flash devote level hobby quick inner drive ghost inside",
        seed: concat!(
            "72be8e052fc4919d2adf28d5306b5474b0069df35b02303de8c1729c9538dbb6",
            "fc2d731d5f832193cd9fb6aeecbc469594a70e3dd50811b5067f3b88b28c3e8d"
        ),
        master_secret: "ea8667131075ad759ed74127dba8ab373da4a3f65f057ecd420ece1694e44330",
        master_chain_code: "2d0f0adebddf123038fea4685a29c92f11c23a99a4671498ed6e020fe9813036",
    },
    TrezorVector {
        entropy: "23db8160a31d3e0dca3688ed941adbf3",
        phrase: "cat swing flag economy stadium alone churn speed unique patch report train",
        seed: concat!(
            "deb5f45449e615feff5640f2e49f933ff51895de3b4381832b3139941c57b592",
            "05a42480c52175b6efcffaa58a2503887c1e8b363a707256bdd2b587b46541f5"
        ),
        master_secret: "430efb10b6d3c065239eca4bc81fb49180651aeb0d536d689432de2d2d48034f",
        master_chain_code: "dd14bdb036e58033fd3adac5bb157ea768ebad844e41d2cb0d7439f9bff95000",
    },
    TrezorVector {
        entropy: "8197a4a47f0425faeaa69deebc05ca29c0a5b5cc76ceacc0",
        phrase: "light rule cinnamon wrap drastic word pride squirrel upgrade then income fatal apart sustain crack supply proud access",
        seed: concat!(
            "4cbdff1ca2db800fd61cae72a57475fdc6bab03e441fd63f96dabd1f183ef5b7",
            "82925f00105f318309a7e9c3ea6967c7801e46c8a58082674c860a37b93eda02"
        ),
        master_secret: "ae24b4f0967f28aa39341834b79532c0079c0ed5557b592675516c98f2c18850",
        master_chain_code: "bdb15c2b471ba045958ba5c4f7ec6e9c74d68eda69f696ce88ce146e0b8da608",
    },
    TrezorVector {
        entropy: "066dca1a2bb7e8a1db2832148ce9933eea0f3ac9548d793112d9a95c9407efad",
        phrase: "all hour make first leader extend hole alien behind guard gospel lava path output census museum junior mass reopen famous sing advance salt reform",
        seed: concat!(
            "26e975ec644423f4a4c4f4215ef09b4bd7ef924e85d1d17c4cf3f136c2863cf6",
            "df0a475045652c57eb5fb41513ca2a2d67722b77e954b4b3fc11f7590449191d"
        ),
        master_secret: "b91171c99f7097fd754b48f2521133a1e0a694b86ea5de6bba60045b0f254c61",
        master_chain_code: "b3e4aae033ec305910466adfe54908542edc5546fefb75f6c9f9f85184b16b93",
    },
    TrezorVector {
        entropy: "f30f8c1da665478f49b001d94c5fc452",
        phrase: "vessel ladder alter error federal sibling chat ability sun glass valve picture",
        seed: concat!(
            "2aaa9242daafcee6aa9d7269f17d4efe271e1b9a529178d7dc139cd18747090b",
            "f9d60295d0ce74309a78852a9caadf0af48aae1c6253839624076224374bc63f"
        ),
        master_secret: "dd22914cf6b4086123a487e82abe565eba0b1c0f1ba794a945d974182812c793",
        master_chain_code: "22e5f4c2ebe0c62bb62d3d692f7e3c045ccaeddaa0a7167383ffab9378a0ad17",
    },
    TrezorVector {
        entropy: "c10ec20dc3cd9f652c7fac2f1230f7a3c828389a14392f05",
        phrase: "scissors invite lock maple supreme raw rapid void congress muscle digital elegant little brisk hair mango congress clump",
        seed: concat!(
            "7b4a10be9d98e6cba265566db7f136718e1398c71cb581e1b2f464cac1ceedf4",
            "f3e274dc270003c670ad8d02c4558b2f8e39edea2775c9e232c7cb798b069e88"
        ),
        master_secret: "6d7c8d14d5e949647c9a3bf85344f9e40e82bff96789965acbf83bb66a7f9e5d",
        master_chain_code: "fc9e966f7b063366c65d23c2b75cc0fab48a9c09e7812c6d99901f47bc79e0b9",
    },
    TrezorVector {
        entropy: "f585c11aec520db57dd353c69554b21a89b20fb0650966fa0a9d6f74fd989d8f",
        phrase: "void come effort suffer camp survey warrior heavy shoot primary clutch crush open amazing screen patrol group space point ten exist slush involve unfold",
        seed: concat!(
            "01f5bced59dec48e362f2c45b5de68b9fd6c92c6634f44d6d40aab69056506f0",
            "e35524a518034ddc1192e1dacd32c1ed3eaa3c3b131c88ed8e7e54c49a5d0998"
        ),
        master_secret: "679bf92c04cf16307053cbed33784f3c4266b362bf5f3d7ee13bed6f2719743c",
        master_chain_code: "6df83b12407419139884bd46497e9ffabf29ff997c84b00e5b518cd06108a427",
    },
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

#[test]
fn wordlist_is_official_bip39_english() {
    let digest = Sha256::digest(ENGLISH.as_bytes());
    let digest_bytes: &[u8] = &digest;
    assert_eq!(digest_bytes, decode_hex(ENGLISH_SHA256).as_slice());
    let list = words();
    assert_eq!(list.len(), WORDLIST_LEN);
    assert_eq!(list.first(), Some(&"abandon"));
    assert_eq!(list.last(), Some(&"zoo"));
    // Urut naik tanpa duplikat, sehingga pencarian biner sah.
    for (left, right) in list.iter().zip(list.iter().skip(1)) {
        assert!(left < right, "daftar kata tidak urut: {left} {right}");
    }
    assert!(
        list.iter()
            .all(|word| !word.is_empty() && word.bytes().all(|b| b.is_ascii_lowercase()))
    );
}

#[test]
fn trezor_vectors_phrase_seed_and_master_key() {
    for vector in &TREZOR {
        let entropy = decode_hex(vector.entropy);
        let from_entropy = Mnemonic::from_entropy(&entropy).expect("entropi sah");
        assert_eq!(from_entropy.phrase(), vector.phrase);
        let parsed = Mnemonic::parse(vector.phrase).expect("frasa sah");
        assert_eq!(parsed.entropy(), entropy.as_slice());
        assert_eq!(parsed, from_entropy);

        let seed = parsed.to_seed("TREZOR").expect("passphrase ASCII");
        assert_eq!(
            seed.as_bytes().as_slice(),
            decode_hex(vector.seed).as_slice()
        );

        let master = ExtendedPrivKey::master(seed.as_bytes()).expect("kunci induk sah");
        assert_eq!(
            master.secret_bytes().as_slice(),
            decode_hex(vector.master_secret).as_slice()
        );
        assert_eq!(
            master.chain_code().as_slice(),
            decode_hex(vector.master_chain_code).as_slice()
        );
    }
}

#[test]
fn spec_vectors_w1_to_w3() {
    let mnemonic = Mnemonic::from_entropy(&[0_u8; NEW_WALLET_ENTROPY_LEN]).expect("entropi sah");
    assert_eq!(mnemonic.phrase(), W1);
    assert_eq!(mnemonic.word_count(), 24);
    let seed = mnemonic.to_seed("").expect("tanpa passphrase");
    assert_eq!(seed.as_bytes().len(), SEED_LEN);
    assert_eq!(seed.as_bytes().as_slice(), decode_hex(W2).as_slice());
    let seed_trezor = mnemonic.to_seed("TREZOR").expect("passphrase ASCII");
    assert_eq!(seed_trezor.as_bytes().as_slice(), decode_hex(W3).as_slice());
}

#[test]
fn parse_normalizes_spaces_and_case() {
    let messy = concat!(
        "  ABANDON abandon\tabandon  abandon abandon abandon abandon abandon abandon abandon abandon abandon\n",
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon Abandon ART  "
    );
    let parsed = Mnemonic::parse(messy).expect("frasa sah setelah normalisasi");
    assert_eq!(parsed.phrase(), W1);
    assert_eq!(parsed.entropy(), [0_u8; 32].as_slice());
}

#[test]
fn parse_rejects_bad_count_unknown_word_and_checksum() {
    assert_eq!(
        Mnemonic::parse(""),
        Err(MnemonicError::InvalidWordCount { count: 0 })
    );
    let eleven = ["abandon"; 11].join(" ");
    assert_eq!(
        Mnemonic::parse(&eleven),
        Err(MnemonicError::InvalidWordCount { count: 11 })
    );
    let twenty_five = ["abandon"; 25].join(" ");
    assert_eq!(
        Mnemonic::parse(&twenty_five),
        Err(MnemonicError::InvalidWordCount { count: 25 })
    );

    let mut unknown = vec!["abandon"; 11];
    unknown.push("about");
    if let Some(slot) = unknown.get_mut(4) {
        *slot = "abandonx";
    }
    assert_eq!(
        Mnemonic::parse(&unknown.join(" ")),
        Err(MnemonicError::UnknownWord { position: 5 })
    );
    let non_ascii = W1.replacen("art", "\u{e4}rt", 1);
    assert_eq!(
        Mnemonic::parse(&non_ascii),
        Err(MnemonicError::UnknownWord { position: 24 })
    );

    // 24 kali "abandon": entropi nol, tetapi checksum yang benar adalah kata terakhir "art".
    assert_eq!(
        Mnemonic::parse(&["abandon"; 24].join(" ")),
        Err(MnemonicError::BadChecksum)
    );
    // 12 kata: kata terakhir "about" benar, "abandon" salah checksum.
    assert_eq!(
        Mnemonic::parse(&["abandon"; 12].join(" ")),
        Err(MnemonicError::BadChecksum)
    );
}

#[test]
fn from_entropy_rejects_bad_length() {
    for len in [0_usize, 12, 15, 17, 31, 33, 64] {
        assert_eq!(
            Mnemonic::from_entropy(&vec![0_u8; len]),
            Err(MnemonicError::InvalidEntropyLength { len })
        );
    }
}

#[test]
fn seed_rejects_non_printable_passphrase() {
    let mnemonic = Mnemonic::parse(W1).expect("frasa sah");
    for bad in ["\u{e9}", "a\nb", "\t", "\u{7f}", "\u{0}"] {
        assert_eq!(mnemonic.to_seed(bad), Err(MnemonicError::InvalidPassphrase));
    }
    for good in [" ", "~", "Kata Sandi 123!"] {
        assert!(mnemonic.to_seed(good).is_ok());
    }
}

#[test]
fn debug_never_prints_secrets() {
    let mnemonic = Mnemonic::parse(W1).expect("frasa sah");
    let text = format!("{mnemonic:?}");
    assert_eq!(text, "Mnemonic(<rahasia>, 24 kata)");
    let seed = mnemonic.to_seed("").expect("tanpa passphrase");
    assert_eq!(format!("{seed:?}"), "Seed(<rahasia>)");
}

proptest! {
    #[test]
    fn prop_entropy_round_trip(
        entropy in prop_oneof![Just(16_usize), Just(20), Just(24), Just(28), Just(32)]
            .prop_flat_map(|len| prop::collection::vec(any::<u8>(), len)),
    ) {
        let mnemonic = Mnemonic::from_entropy(&entropy).expect("panjang sah");
        let parsed = Mnemonic::parse(mnemonic.phrase()).expect("frasa sah");
        prop_assert_eq!(parsed.entropy(), entropy.as_slice());
        prop_assert_eq!(parsed.word_count(), entropy.len().checked_mul(3).and_then(|n| n.checked_div(4)).expect("kecil"));
    }
}
