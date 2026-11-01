use homeostasis_primitives::{BlockHeight, DecodeError, Encode, HEADER_SIZE, Hash32};

use super::{
    Ack, Blocks, GetBlocks, GetHeaders, Headers, MAX_BLOCK_IDS, MAX_HEADERS, MAX_LOCATOR,
    MessageError, STATUS_SIZE, STATUS_VERSION, Status, Wire,
};
use crate::test_support::{id, sample_block, sample_header, sample_tx};

fn trailing() -> MessageError {
    MessageError::Decode(DecodeError::TrailingBytes { remaining: 1 })
}

fn with_extra_byte(mut bytes: Vec<u8>) -> Vec<u8> {
    bytes.push(0);
    bytes
}

#[test]
fn status_is_105_bytes_and_round_trips() {
    let status = Status::new(id(0xb9), BlockHeight::new(7), id(5), [0x11; 32]);
    let bytes = status.to_wire();
    assert_eq!(bytes.len(), STATUS_SIZE);
    assert_eq!(bytes[0], STATUS_VERSION);
    assert_eq!(&bytes[1..33], &[0xb9; 32]);
    assert_eq!(&bytes[33..41], &7_u64.to_le_bytes());
    assert_eq!(&bytes[41..73], &[5; 32]);
    assert_eq!(&bytes[73..105], &[0x11; 32]);
    assert_eq!(Status::from_wire(&bytes), Ok(status.clone()));
    assert!(status.is_compatible(&id(0xb9)));
    assert!(!status.is_compatible(&id(1)));

    // Versi lain tetap terbaca (koneksi ditutup tanpa sanksi), tetapi tidak cocok.
    let other = Status {
        version: 2,
        ..status.clone()
    };
    let decoded = Status::from_wire(&other.to_wire()).unwrap();
    assert_eq!(decoded.version, 2);
    assert!(!decoded.is_compatible(&id(0xb9)));

    assert_eq!(
        Status::from_wire(&bytes[..104]),
        Err(MessageError::Decode(DecodeError::UnexpectedEof))
    );
}

#[test]
fn get_headers_needs_1_to_64_locator_ids() {
    for count in [1, MAX_LOCATOR] {
        let request = GetHeaders {
            locator: vec![id(1); count],
            stop: Hash32::default(),
        };
        assert_eq!(GetHeaders::from_wire(&request.to_wire()), Ok(request));
    }
    for count in [0, MAX_LOCATOR + 1] {
        let request = GetHeaders {
            locator: vec![id(1); count],
            stop: id(2),
        };
        assert_eq!(
            GetHeaders::from_wire(&request.to_wire()),
            Err(MessageError::LocatorCount { count })
        );
    }
    // Jumlah yang mustahil dipenuhi sisa byte ditolak sebelum alokasi.
    assert!(matches!(
        GetHeaders::from_wire(&[5, 0, 0]),
        Err(MessageError::Decode(
            DecodeError::ImpossibleElementCount { .. }
        ))
    ));
}

#[test]
fn headers_allow_at_most_2000() {
    assert_eq!(
        Headers::from_wire(&[0]),
        Ok(Headers {
            headers: Vec::new()
        })
    );
    let full = Headers {
        headers: vec![sample_header(1); MAX_HEADERS],
    };
    let bytes = full.to_wire();
    assert_eq!(bytes.len(), 2 + MAX_HEADERS * HEADER_SIZE);
    assert_eq!(Headers::from_wire(&bytes), Ok(full));
    let too_many = Headers {
        headers: vec![sample_header(1); MAX_HEADERS + 1],
    };
    assert_eq!(
        Headers::from_wire(&too_many.to_wire()),
        Err(MessageError::TooManyHeaders {
            count: MAX_HEADERS + 1
        })
    );
}

#[test]
fn get_blocks_needs_1_to_16_ids() {
    for count in [1, MAX_BLOCK_IDS] {
        let request = GetBlocks {
            ids: vec![id(9); count],
        };
        let bytes = request.to_wire();
        assert_eq!(bytes.len(), 1 + 32 * count);
        assert_eq!(GetBlocks::from_wire(&bytes), Ok(request));
    }
    for count in [0, MAX_BLOCK_IDS + 1] {
        let request = GetBlocks {
            ids: vec![id(9); count],
        };
        assert_eq!(
            GetBlocks::from_wire(&request.to_wire()),
            Err(MessageError::IdCount { count })
        );
    }
}

#[test]
fn blocks_entries_round_trip_and_reject_bad_tags() {
    let blocks = Blocks {
        entries: vec![Some(sample_block(1)), None],
    };
    let bytes = blocks.to_wire();
    assert_eq!(bytes.first(), Some(&2));
    assert_eq!(bytes.get(1), Some(&1));
    assert_eq!(bytes.last(), Some(&0));
    assert_eq!(Blocks::from_wire(&bytes), Ok(blocks));
    assert_eq!(
        Blocks::from_wire(&[1, 2]),
        Err(MessageError::UnknownEntryTag { tag: 2 })
    );
    let too_many = Blocks {
        entries: vec![None; MAX_BLOCK_IDS + 1],
    };
    assert_eq!(
        Blocks::from_wire(&too_many.to_wire()),
        Err(MessageError::TooManyEntries {
            count: MAX_BLOCK_IDS + 1
        })
    );
}

#[test]
fn ack_has_five_status_values() {
    let all = [
        (0, Ack::Accepted),
        (1, Ack::UnknownParent),
        (2, Ack::Deferred),
        (3, Ack::Invalid),
        (4, Ack::Rejected),
    ];
    for (byte, ack) in all {
        assert_eq!(ack.to_wire(), vec![byte]);
        assert_eq!(Ack::from_wire(&[byte]), Ok(ack));
    }
    assert_eq!(
        Ack::from_wire(&[5]),
        Err(MessageError::UnknownAckStatus { status: 5 })
    );
    assert_eq!(
        Ack::from_wire(&[]),
        Err(MessageError::Decode(DecodeError::UnexpectedEof))
    );
}

#[test]
fn block_and_transaction_use_canonical_encoding() {
    let block = sample_block(3);
    assert_eq!(block.to_wire(), block.to_bytes());
    assert_eq!(
        homeostasis_primitives::Block::from_wire(&block.to_wire()),
        Ok(block.clone())
    );
    assert_eq!(
        homeostasis_primitives::Block::from_wire(&with_extra_byte(block.to_wire())),
        Err(trailing())
    );
    let tx = sample_tx();
    assert_eq!(tx.to_wire(), tx.to_bytes());
    assert_eq!(
        homeostasis_primitives::Transaction::from_wire(&tx.to_wire()),
        Ok(tx.clone())
    );
    assert_eq!(
        homeostasis_primitives::Transaction::from_wire(&with_extra_byte(tx.to_wire())),
        Err(trailing())
    );
}

#[test]
fn every_message_rejects_trailing_bytes() {
    let status = Status::new(id(1), BlockHeight::new(0), id(2), [0; 32]);
    assert_eq!(
        Status::from_wire(&with_extra_byte(status.to_wire())),
        Err(trailing())
    );
    let get_headers = GetHeaders {
        locator: vec![id(1)],
        stop: id(2),
    };
    assert_eq!(
        GetHeaders::from_wire(&with_extra_byte(get_headers.to_wire())),
        Err(trailing())
    );
    let headers = Headers {
        headers: vec![sample_header(4)],
    };
    assert_eq!(
        Headers::from_wire(&with_extra_byte(headers.to_wire())),
        Err(trailing())
    );
    let get_blocks = GetBlocks { ids: vec![id(3)] };
    assert_eq!(
        GetBlocks::from_wire(&with_extra_byte(get_blocks.to_wire())),
        Err(trailing())
    );
    let blocks = Blocks {
        entries: vec![None],
    };
    assert_eq!(
        Blocks::from_wire(&with_extra_byte(blocks.to_wire())),
        Err(trailing())
    );
    assert_eq!(Ack::from_wire(&[0, 0]), Err(trailing()));
}
