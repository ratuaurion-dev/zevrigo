//! Bingkai pesan P2P (Spesifikasi P2P Bagian 3): `varint<32>(panjang) ‖ isi`, isi paling banyak 2 MiB.
//!
//! Panjang yang melebihi batas, varint tidak minimal, atau aliran yang berakhir sebelum isi lengkap dilaporkan sebagai
//! [`MessageError`] (pelanggaran rekan), terpisah dari galat I/O biasa.

use std::io;

use libp2p::futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use zevrigo_primitives::Reader;
use zevrigo_primitives::encoding::write_varint_u32;

use crate::message::{MAX_MESSAGE_SIZE, MessageError};

/// Panjang paling besar awalan `varint<32>` (5 byte).
const MAX_PREFIX_LEN: usize = 5;

/// Membentuk bingkai `varint<32>(panjang) ‖ payload`.
///
/// # Errors
///
/// [`MessageError::FrameTooLarge`] jika `payload` lebih dari [`MAX_MESSAGE_SIZE`] byte.
pub fn encode_frame(payload: &[u8]) -> Result<Vec<u8>, MessageError> {
    let len = payload.len();
    if len > MAX_MESSAGE_SIZE {
        return Err(MessageError::FrameTooLarge { len });
    }
    let len_u32 = u32::try_from(len).map_err(|_| MessageError::FrameTooLarge { len })?;
    let mut out = Vec::with_capacity(len.saturating_add(MAX_PREFIX_LEN));
    write_varint_u32(&mut out, len_u32);
    out.extend_from_slice(payload);
    Ok(out)
}

/// Mengurai awalan panjang lengkap (byte terakhir tanpa bit lanjutan).
fn decode_length(prefix: &[u8]) -> Result<usize, MessageError> {
    let mut r = Reader::new(prefix);
    let len = r.read_varint_u32().map_err(|_| MessageError::FrameLength)?;
    r.finish().map_err(|_| MessageError::FrameLength)?;
    let len = usize::try_from(len).map_err(|_| MessageError::FrameLength)?;
    if len > MAX_MESSAGE_SIZE {
        return Err(MessageError::FrameTooLarge { len });
    }
    Ok(len)
}

/// Membaca satu bingkai dari `io`.
///
/// Hasil luar `Err` adalah galat I/O biasa; hasil dalam `Err` adalah pesan rusak dari rekan.
///
/// # Errors
///
/// Galat I/O dari `io` selain akhir aliran.
pub async fn read_frame<T>(io: &mut T) -> io::Result<Result<Vec<u8>, MessageError>>
where
    T: AsyncRead + Unpin + Send,
{
    let mut prefix = Vec::with_capacity(MAX_PREFIX_LEN);
    loop {
        let mut one = [0_u8; 1];
        match io.read_exact(&mut one).await {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
                return Ok(Err(MessageError::Truncated));
            }
            Err(error) => return Err(error),
        }
        let [byte] = one;
        prefix.push(byte);
        if byte & 0x80 == 0 {
            break;
        }
        if prefix.len() >= MAX_PREFIX_LEN {
            return Ok(Err(MessageError::FrameLength));
        }
    }
    let len = match decode_length(&prefix) {
        Ok(len) => len,
        Err(error) => return Ok(Err(error)),
    };
    let mut payload = vec![0_u8; len];
    match io.read_exact(&mut payload).await {
        Ok(()) => Ok(Ok(payload)),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
            Ok(Err(MessageError::Truncated))
        }
        Err(error) => Err(error),
    }
}

/// Menulis satu bingkai berisi `payload` ke `io`.
///
/// # Errors
///
/// `io::ErrorKind::InvalidInput` jika `payload` lebih dari 2 MiB, atau galat I/O dari `io`.
pub async fn write_frame<T>(io: &mut T, payload: &[u8]) -> io::Result<()>
where
    T: AsyncWrite + Unpin + Send,
{
    let framed = encode_frame(payload)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    io.write_all(&framed).await?;
    io.flush().await
}

#[cfg(test)]
mod tests;
