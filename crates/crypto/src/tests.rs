use homeostasis_primitives::amount::Amount;
use homeostasis_primitives::block::{Block, Header};
use homeostasis_primitives::hash::Hash32;
use homeostasis_primitives::height::BlockHeight;
use homeostasis_primitives::tx::{Condition, Output, OutputKind, Transaction};

use crate::{
    Domain, DomainHasher, MODULE_ID, PublicKey, SchnorrError, SecretKey, Signature, block_id,
    block_merkle_root, condition_hash, hash, merkle_root, network_id, sign, txid, verify, wtxid,
};

#[test]
fn module_id_matches() {
    assert_eq!(MODULE_ID, "MOD-02");
}

#[test]
fn crate_root_reexports_are_accessible() {
    let digest = hash(Domain::Txid, b"abc");
    let mut stream = DomainHasher::new(Domain::Txid);
    stream.update(b"abc");
    assert_eq!(digest, stream.finalize());
    assert_eq!(
        merkle_root(&[Hash32::ZERO]),
        Some(hash(Domain::MerkleLeaf, &[0_u8; 32]))
    );

    let mut sk_bytes = [0_u8; 32];
    if let Some(last) = sk_bytes.last_mut() {
        *last = 3;
    }
    let sk = SecretKey::from_bytes(&sk_bytes).unwrap();
    let pk = PublicKey::from_bytes(sk.public_key().as_bytes()).unwrap();
    let sig = sign(&sk, digest.as_bytes(), &[0_u8; 32]).unwrap();
    let roundtrip_sig = Signature::from_bytes(sig.as_bytes());
    assert!(verify(&pk, digest.as_bytes(), &roundtrip_sig));
    assert_eq!(
        SecretKey::from_bytes(&[0_u8; 32]).unwrap_err(),
        SchnorrError::InvalidSecretKey
    );

    // Smoke test re-export identitas data
    let out = Output::new(Amount::from_quanta(1), OutputKind::SingleKey, [0x11; 32]).unwrap();
    let tx = Transaction::coinbase(BlockHeight::new(1), vec![], vec![out]).unwrap();
    assert_ne!(txid(&tx), Hash32::ZERO);
    assert_ne!(wtxid(&tx), Hash32::ZERO);

    let header = Header::new(
        BlockHeight::new(1),
        Hash32::ZERO,
        Hash32::ZERO,
        1_800_000_000,
        0x1d00_ffff,
        0,
    );
    assert_ne!(block_id(&header), Hash32::ZERO);
    assert_eq!(network_id(&header), block_id(&header));

    let cond = Condition::new(1, vec![[0x01; 32]], BlockHeight::GENESIS).unwrap();
    assert_ne!(condition_hash(&cond), Hash32::ZERO);

    let block = Block::new(header, vec![tx]).unwrap();
    assert!(block_merkle_root(&block).is_some());
}
