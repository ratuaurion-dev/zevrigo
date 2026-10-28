use homeostasis_primitives::Hash32;

use crate::{Domain, DomainHasher, MODULE_ID, hash, merkle_root};

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
}
