use zevrigo_primitives::Hash32;

use super::{ProtocolKind, network_prefix, protocol_name, stream_protocol};

fn regtest_like() -> Hash32 {
    Hash32::from_hex(&format!("8b9eb3187250bbdf{}", "0".repeat(48))).unwrap()
}

#[test]
fn names_use_first_16_hex_of_network_id() {
    let network_id = regtest_like();
    assert_eq!(network_prefix(&network_id), "8b9eb3187250bbdf");
    let names: Vec<String> = ProtocolKind::ALL
        .iter()
        .map(|kind| protocol_name(&network_id, *kind))
        .collect();
    assert_eq!(
        names,
        vec![
            "/zvrg/8b9eb3187250bbdf/status/1",
            "/zvrg/8b9eb3187250bbdf/headers/1",
            "/zvrg/8b9eb3187250bbdf/blocks/1",
            "/zvrg/8b9eb3187250bbdf/block/1",
            "/zvrg/8b9eb3187250bbdf/tx/1",
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
