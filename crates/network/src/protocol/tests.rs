use homeostasis_primitives::Hash32;

use super::{ProtocolKind, network_prefix, protocol_name, stream_protocol};

fn regtest_like() -> Hash32 {
    Hash32::from_hex(&format!("b933d7c548dc7845{}", "0".repeat(48))).unwrap()
}

#[test]
fn names_use_first_16_hex_of_network_id() {
    let network_id = regtest_like();
    assert_eq!(network_prefix(&network_id), "b933d7c548dc7845");
    let names: Vec<String> = ProtocolKind::ALL
        .iter()
        .map(|kind| protocol_name(&network_id, *kind))
        .collect();
    assert_eq!(
        names,
        vec![
            "/hsis/b933d7c548dc7845/status/1",
            "/hsis/b933d7c548dc7845/headers/1",
            "/hsis/b933d7c548dc7845/blocks/1",
            "/hsis/b933d7c548dc7845/block/1",
            "/hsis/b933d7c548dc7845/tx/1",
        ]
    );
}

#[test]
fn stream_protocols_match_names() {
    let network_id = regtest_like();
    for kind in ProtocolKind::ALL {
        let protocol = stream_protocol(&network_id, kind).unwrap();
        assert_eq!(protocol.as_ref(), protocol_name(&network_id, kind));
    }
}
