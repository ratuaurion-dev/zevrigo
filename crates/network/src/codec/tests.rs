use std::io;

use libp2p::StreamProtocol;
use libp2p::futures::executor::block_on;
use libp2p::futures::io::Cursor;
use libp2p::request_response::Codec as _;

use super::WireCodec;
use crate::message::{Ack, Blocks, GetBlocks, MessageError};
use crate::test_support::id;

#[test]
fn codec_round_trips_and_returns_bad_messages_as_values() {
    block_on(async {
        let protocol = StreamProtocol::new("/hsis/test/blocks/1");
        let mut codec = WireCodec::<GetBlocks, Blocks>::default();
        let request = GetBlocks {
            ids: vec![id(1), id(2)],
        };
        let mut wire = Vec::new();
        codec
            .write_request(&protocol, &mut wire, Ok(request.clone()))
            .await
            .unwrap();
        assert_eq!(wire.first(), Some(&65));
        let got = codec
            .read_request(&protocol, &mut Cursor::new(wire))
            .await
            .unwrap();
        assert_eq!(got, Ok(request));

        // GetBlocks tanpa ID: permintaan tetap sampai sebagai nilai rusak, agar pengirimnya bisa disanksi.
        let got = codec
            .read_request(&protocol, &mut Cursor::new(vec![1, 0]))
            .await
            .unwrap();
        assert_eq!(got, Err(MessageError::IdCount { count: 0 }));

        let mut wire = Vec::new();
        codec
            .write_response(
                &protocol,
                &mut wire,
                Ok(Blocks {
                    entries: vec![None],
                }),
            )
            .await
            .unwrap();
        assert_eq!(wire, vec![2, 1, 0]);

        let error = codec
            .write_request(&protocol, &mut Vec::new(), Err(MessageError::Truncated))
            .await
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);

        let mut ack_codec = WireCodec::<Ack, Ack>::default();
        let got = ack_codec
            .read_response(&protocol, &mut Cursor::new(vec![1, 9]))
            .await
            .unwrap();
        assert_eq!(got, Err(MessageError::UnknownAckStatus { status: 9 }));
    });
}
