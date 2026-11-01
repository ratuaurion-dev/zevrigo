use std::io;

use libp2p::futures::executor::block_on;
use libp2p::futures::io::Cursor;

use super::{decode_length, encode_frame, read_frame, write_frame};
use crate::message::{MAX_MESSAGE_SIZE, MessageError};

fn read(bytes: Vec<u8>) -> Result<Vec<u8>, MessageError> {
    block_on(read_frame(&mut Cursor::new(bytes))).unwrap()
}

#[test]
fn frames_round_trip_with_minimal_length_prefix() {
    for (len, prefix_len) in [(0, 1), (1, 1), (127, 1), (128, 2), (300, 2)] {
        let payload = vec![0xab; len];
        let framed = encode_frame(&payload).unwrap();
        assert_eq!(framed.len(), prefix_len + len);
        let mut written = Vec::new();
        block_on(write_frame(&mut written, &payload)).unwrap();
        assert_eq!(written, framed);
        assert_eq!(read(framed), Ok(payload));
    }
    assert_eq!(encode_frame(&[7]).unwrap(), vec![1, 7]);
}

#[test]
fn length_is_limited_to_2_mib() {
    assert_eq!(
        decode_length(&[0x80, 0x80, 0x80, 0x01]),
        Ok(MAX_MESSAGE_SIZE)
    );
    let max = vec![0; MAX_MESSAGE_SIZE];
    let framed = encode_frame(&max).unwrap();
    assert_eq!(framed.get(..4), Some(&[0x80, 0x80, 0x80, 0x01][..]));
    assert_eq!(read(framed), Ok(max));

    let over = MAX_MESSAGE_SIZE + 1;
    assert_eq!(
        encode_frame(&vec![0; over]),
        Err(MessageError::FrameTooLarge { len: over })
    );
    let error = block_on(write_frame(&mut Vec::new(), &vec![0; over])).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(
        read(vec![0x81, 0x80, 0x80, 0x01]),
        Err(MessageError::FrameTooLarge { len: over })
    );
}

#[test]
fn bad_length_prefixes_are_rejected() {
    // Tidak minimal.
    assert_eq!(read(vec![0x80, 0x00]), Err(MessageError::FrameLength));
    // Melampaui u32.
    assert_eq!(
        read(vec![0xff, 0xff, 0xff, 0xff, 0x7f]),
        Err(MessageError::FrameLength)
    );
    // Lebih dari 5 byte.
    assert_eq!(read(vec![0xff; 6]), Err(MessageError::FrameLength));
}

#[test]
fn truncated_streams_are_rejected() {
    assert_eq!(read(Vec::new()), Err(MessageError::Truncated));
    assert_eq!(read(vec![0x80]), Err(MessageError::Truncated));
    assert_eq!(read(vec![3, 1, 2]), Err(MessageError::Truncated));
}
