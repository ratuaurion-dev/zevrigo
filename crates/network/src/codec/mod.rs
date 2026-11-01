//! Codec request-response libp2p untuk pesan [`Wire`]: satu bingkai per permintaan dan satu per jawaban.
//!
//! Pesan rusak dari rekan dikembalikan sebagai nilai `Err(MessageError)`, bukan galat I/O. Dengan begitu
//! `request_response` tetap melaporkan permintaan masuk yang rusak (galat baca permintaan tidak pernah dilaporkan),
//! dan layanan jaringan bisa memberi sanksi kepada pengirimnya.

use std::io;
use std::marker::PhantomData;

use libp2p::StreamProtocol;
use libp2p::futures::{AsyncRead, AsyncWrite};
use libp2p::request_response::Codec;

use crate::frame::{read_frame, write_frame};
use crate::message::{MessageError, Wire};

/// Pesan yang diterima: sah, atau rusak beserta alasannya.
pub(crate) type Received<T> = Result<T, MessageError>;

/// Codec untuk satu protokol dengan permintaan `Req` dan jawaban `Resp`.
pub(crate) struct WireCodec<Req, Resp>(PhantomData<fn() -> (Req, Resp)>);

impl<Req, Resp> Default for WireCodec<Req, Resp> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Req, Resp> Clone for WireCodec<Req, Resp> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

/// Membaca satu pesan `M` dari satu bingkai.
pub(crate) async fn read_message<M, T>(io: &mut T) -> io::Result<Received<M>>
where
    M: Wire + Send,
    T: AsyncRead + Unpin + Send,
{
    Ok(read_frame(io).await?.and_then(|bytes| M::from_wire(&bytes)))
}

/// Menulis satu pesan sebagai satu bingkai. Nilai `Err` tidak pernah dikirim (`InvalidInput`).
pub(crate) async fn write_message<M, T>(io: &mut T, message: Received<M>) -> io::Result<()>
where
    M: Wire + Send,
    T: AsyncWrite + Unpin + Send,
{
    let bytes = message
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?
        .to_wire();
    write_frame(io, &bytes).await
}

impl<Req, Resp> Codec for WireCodec<Req, Resp>
where
    Req: Wire + Send + 'static,
    Resp: Wire + Send + 'static,
{
    type Protocol = StreamProtocol;
    type Request = Received<Req>;
    type Response = Received<Resp>;

    async fn read_request<T>(&mut self, _: &StreamProtocol, io: &mut T) -> io::Result<Received<Req>>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_message(io).await
    }

    async fn read_response<T>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
    ) -> io::Result<Received<Resp>>
    where
        T: AsyncRead + Unpin + Send,
    {
        read_message(io).await
    }

    async fn write_request<T>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        request: Received<Req>,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_message(io, request).await
    }

    async fn write_response<T>(
        &mut self,
        _: &StreamProtocol,
        io: &mut T,
        response: Received<Resp>,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_message(io, response).await
    }
}

#[cfg(test)]
mod tests;
