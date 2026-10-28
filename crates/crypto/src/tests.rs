use homeostasis_primitives::Hash32;

use crate::{
    Domain, DomainHasher, MODULE_ID, PublicKey, SchnorrError, SecretKey, Signature, hash,
    merkle_root, sign, verify,
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
}
