//! Transport dasar MOD-06 (TSK-M45, DEC-33): TCP yang juga menerima nama DNS, dan pengenal alamat WebSocket.
//!
//! `DnsTcp` mendial `/dns4/<nama>/tcp/<port>`, `/dns6/...`, atau `/dns/...` dengan me-resolve nama lewat resolver sistem
//! operasi (`tokio::net::lookup_host`) setiap kali mendial, tanpa `libp2p-dns`. Alamat lain diteruskan ke TCP libp2p apa
//! adanya. Transport WebSocket libp2p memakai `DnsTcp` di bawahnya, sehingga alamat seed
//! `/dns4/<nama>/tcp/443/tls/sni/<nama>/ws` mengirim nama itu di SNI TLS dan di kepala `Host` WebSocket (Cloudflare
//! memilih tujuan dari keduanya), sementara koneksi TCP-nya ke alamat IP hasil resolve. Jika nama di-resolve sebelum
//! sampai ke lapisan WebSocket, kepala `Host` berisi alamat IP dan Cloudflare menolaknya.

use std::io;
use std::net::IpAddr;
use std::pin::Pin;
use std::task::{Context, Poll};

use libp2p::core::Transport;
use libp2p::core::transport::{DialOpts, ListenerId, TransportError, TransportEvent};
use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, tcp};

type TcpTransport = tcp::tokio::Transport;

/// `true` jika `address` memakai WebSocket (`/ws` atau `/wss`).
#[must_use]
pub fn is_websocket(address: &Multiaddr) -> bool {
    address
        .iter()
        .any(|part| matches!(part, Protocol::Ws(_) | Protocol::Wss(_)))
}

/// Keluarga alamat IP yang diterima dari hasil resolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Family {
    /// `/dns`: IPv4 atau IPv6.
    Any,
    /// `/dns4`: hanya IPv4.
    V4,
    /// `/dns6`: hanya IPv6.
    V6,
}

impl Family {
    /// `true` jika `ip` termasuk keluarga ini.
    const fn accepts(self, ip: IpAddr) -> bool {
        match self {
            Self::Any => true,
            Self::V4 => ip.is_ipv4(),
            Self::V6 => ip.is_ipv6(),
        }
    }
}

/// Nama, port, dan keluarga alamat dari `/dns*/<nama>/tcp/<port>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DnsTarget {
    /// Nama host.
    pub(crate) host: String,
    /// Port TCP.
    pub(crate) port: u16,
    /// Keluarga alamat yang diterima.
    pub(crate) family: Family,
}

/// Membaca `address` sebagai `/dns*/<nama>/tcp/<port>`, boleh diakhiri `/p2p/<PeerId>`. Alamat lain (IP, WebSocket,
/// protokol tambahan) menghasilkan `None`.
pub(crate) fn dns_target(address: &Multiaddr) -> Option<DnsTarget> {
    let mut parts = address.iter();
    let (host, family) = match parts.next()? {
        Protocol::Dns(host) => (host, Family::Any),
        Protocol::Dns4(host) => (host, Family::V4),
        Protocol::Dns6(host) => (host, Family::V6),
        _ => return None,
    };
    let Some(Protocol::Tcp(port)) = parts.next() else {
        return None;
    };
    if parts.any(|part| !matches!(part, Protocol::P2p(_))) {
        return None;
    }
    Some(DnsTarget {
        host: host.into_owned(),
        port,
        family,
    })
}

/// Me-resolve `target` menjadi `/ip4/<ip>/tcp/<port>` atau `/ip6/<ip>/tcp/<port>` (alamat pertama yang cocok dengan
/// keluarganya).
async fn resolve(target: DnsTarget) -> io::Result<Multiaddr> {
    let found = tokio::net::lookup_host((target.host.as_str(), target.port)).await?;
    let ip = found
        .map(|socket| socket.ip())
        .find(|ip| target.family.accepts(*ip))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("nama {} tidak punya alamat IP yang cocok", target.host),
            )
        })?;
    Ok(Multiaddr::empty()
        .with(Protocol::from(ip))
        .with(Protocol::Tcp(target.port)))
}

/// TCP libp2p yang juga mendial `/dns*/<nama>/tcp/<port>`; nama di-resolve setiap kali mendial, sehingga perubahan alamat
/// IP seed ikut terpakai saat sambung ulang.
#[derive(Default)]
pub(crate) struct DnsTcp {
    inner: TcpTransport,
}

impl Transport for DnsTcp {
    type Output = <TcpTransport as Transport>::Output;
    type Error = io::Error;
    type ListenerUpgrade = <TcpTransport as Transport>::ListenerUpgrade;
    type Dial = <TcpTransport as Transport>::Dial;

    fn listen_on(
        &mut self,
        id: ListenerId,
        addr: Multiaddr,
    ) -> Result<(), TransportError<Self::Error>> {
        self.inner.listen_on(id, addr)
    }

    fn remove_listener(&mut self, id: ListenerId) -> bool {
        self.inner.remove_listener(id)
    }

    fn dial(
        &mut self,
        addr: Multiaddr,
        opts: DialOpts,
    ) -> Result<Self::Dial, TransportError<Self::Error>> {
        let Some(target) = dns_target(&addr) else {
            return self.inner.dial(addr, opts);
        };
        let dial: Self::Dial = Box::pin(async move {
            let resolved = resolve(target).await?;
            // TCP baru untuk satu dial ini (tanpa memakai ulang port alamat dengar).
            let pending =
                TcpTransport::default()
                    .dial(resolved, opts)
                    .map_err(|error| match error {
                        TransportError::MultiaddrNotSupported(address) => io::Error::new(
                            io::ErrorKind::InvalidInput,
                            format!("alamat hasil resolve tidak didukung: {address}"),
                        ),
                        TransportError::Other(error) => error,
                    })?;
            pending.await
        });
        Ok(dial)
    }

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<TransportEvent<Self::ListenerUpgrade, Self::Error>> {
        Pin::new(&mut self.inner).poll(cx)
    }
}

#[cfg(test)]
mod tests;
