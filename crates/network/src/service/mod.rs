//! Layanan jaringan MOD-06 (Spesifikasi P2P Bagian 1 sampai 4, 6, dan 7).
//!
//! [`Network`] memegang swarm libp2p: TCP, Noise XX, Yamux; lima protokol request-response; paling banyak 8 koneksi
//! keluar, 32 masuk, dan satu per `PeerId`; koneksi menganggur ditutup setelah 300 detik; permintaan gagal setelah
//! 30 detik. Layanan ini sendiri menangani:
//! - sambung ulang ke alamat `--connect` setiap 10 detik jika putus;
//! - pertukaran `status` saat terhubung (versi atau jaringan lain: koneksi ditutup tanpa sanksi);
//! - sanksi untuk pesan rusak dan jawaban `Blocks` yang jumlah entrinya salah (putus dan blokir 24 jam);
//! - putus tanpa blokir setelah 3 permintaan berturut-turut tidak dijawab dalam 30 detik.
//!
//! Permintaan `headers`, `blocks`, `block`, dan `tx` dari rekan diteruskan sebagai [`NetEvent`] beserta [`Reply`]
//! yang wajib dijawab pemanggil (MOD-11). Validasi blok, transaksi, dan header, serta sanksi untuk isi yang tidak sah
//! ([`Network::punish`]), ada di pemanggil.

use std::collections::{BTreeMap, VecDeque};
use std::net::IpAddr;
use std::time::{Duration, Instant};

use homeostasis_primitives::{Block, Hash32, Transaction};
use libp2p::connection_limits::{self, ConnectionLimits};
use libp2p::core::ConnectedPoint;
use libp2p::core::upgrade;
use libp2p::futures::StreamExt;
use libp2p::identity::Keypair;
use libp2p::multiaddr::Protocol;
use libp2p::request_response::{
    self, OutboundFailure, OutboundRequestId, ProtocolSupport, ResponseChannel,
};
use libp2p::swarm::{ConnectionId, NetworkBehaviour, SwarmEvent};
use libp2p::{
    Multiaddr, PeerId, Swarm, Transport as _, allow_block_list, noise, swarm, tcp, yamux,
};
use thiserror::Error;
use tokio::time::{Interval, MissedTickBehavior};

use crate::ban::BanList;
use crate::codec::{Received, WireCodec};
use crate::message::{Ack, Blocks, GetBlocks, GetHeaders, Headers, MessageError, Status, Wire};
use crate::protocol::{ProtocolKind, stream_protocol};

/// Koneksi keluar paling banyak.
pub const MAX_OUTBOUND: u32 = 8;

/// Koneksi masuk paling banyak.
pub const MAX_INBOUND: u32 = 32;

/// Koneksi tanpa aliran terbuka ditutup setelah 300 detik.
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(300);

/// Permintaan yang tidak dijawab dalam 30 detik dianggap gagal.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Batas waktu membuka koneksi (TCP, Noise, Yamux), sama dengan bawaan `SwarmBuilder` libp2p (10 detik).
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Selang sambung ulang ke alamat `--connect` (10 detik).
pub const REDIAL_INTERVAL: Duration = Duration::from_secs(10);

/// Jumlah permintaan berturut-turut yang tidak dijawab sebelum koneksi diputus (tanpa blokir).
pub const MAX_TIMEOUTS: u8 = 3;

/// ID permintaan keluar. Unik per protokol, jadi selalu dibaca bersama jenis event atau [`ProtocolKind`].
pub type RequestId = OutboundRequestId;

type Rr<Req, Resp> = request_response::Behaviour<WireCodec<Req, Resp>>;

/// Gabungan behaviour libp2p. Urutan field = urutan pemeriksaan koneksi baru.
#[derive(NetworkBehaviour)]
struct Behaviour {
    limits: connection_limits::Behaviour,
    banned: allow_block_list::Behaviour<allow_block_list::BlockedPeers>,
    status: Rr<Status, Status>,
    headers: Rr<GetHeaders, Headers>,
    bodies: Rr<GetBlocks, Blocks>,
    block: Rr<Block, Ack>,
    tx: Rr<Transaction, Ack>,
}

fn rr<Req, Resp>(network_id: &Hash32, kind: ProtocolKind) -> Result<Rr<Req, Resp>, NetworkError>
where
    Req: Wire + Send + 'static,
    Resp: Wire + Send + 'static,
{
    let protocol = stream_protocol(network_id, kind)
        .map_err(|error| NetworkError::Protocol(error.to_string()))?;
    Ok(request_response::Behaviour::new(
        [(protocol, ProtocolSupport::Full)],
        request_response::Config::default().with_request_timeout(REQUEST_TIMEOUT),
    ))
}

impl Behaviour {
    fn new(network_id: &Hash32) -> Result<Self, NetworkError> {
        let limits = ConnectionLimits::default()
            .with_max_established_incoming(Some(MAX_INBOUND))
            .with_max_established_outgoing(Some(MAX_OUTBOUND))
            .with_max_established_per_peer(Some(1));
        Ok(Self {
            limits: connection_limits::Behaviour::new(limits),
            banned: allow_block_list::Behaviour::default(),
            status: rr(network_id, ProtocolKind::Status)?,
            headers: rr(network_id, ProtocolKind::Headers)?,
            bodies: rr(network_id, ProtocolKind::Blocks)?,
            block: rr(network_id, ProtocolKind::Block)?,
            tx: rr(network_id, ProtocolKind::Tx)?,
        })
    }
}

/// Galat saat membangun layanan jaringan.
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Nama protokol tidak diterima libp2p.
    #[error("nama protokol tidak sah: {0}")]
    Protocol(String),
    /// Transport Noise gagal dibangun.
    #[error("transport gagal dibangun: {0}")]
    Transport(String),
    /// Gagal mendengar di alamat.
    #[error("gagal mendengar di {address}: {reason}")]
    Listen {
        /// Alamat dengar.
        address: String,
        /// Pesan galat.
        reason: String,
    },
}

/// Konfigurasi layanan jaringan.
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    /// `network_id` (block_id genesis) jaringan node.
    pub network_id: Hash32,
    /// Alamat dengar, misalnya `/ip4/127.0.0.1/tcp/28800`.
    pub listen: Vec<Multiaddr>,
    /// Alamat rekan dari operator (`--connect`), boleh diakhiri `/p2p/<PeerId>`.
    pub connect: Vec<Multiaddr>,
    /// Selang sambung ulang; bawaan [`REDIAL_INTERVAL`] (test memakai selang lebih pendek).
    pub redial_interval: Duration,
}

impl NetworkConfig {
    /// Konfigurasi tanpa alamat dengar dan tanpa rekan, selang sambung ulang 10 detik.
    #[must_use]
    pub const fn new(network_id: Hash32) -> Self {
        Self {
            network_id,
            listen: Vec::new(),
            connect: Vec::new(),
            redial_interval: REDIAL_INTERVAL,
        }
    }
}

/// Alasan sanksi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Offense {
    /// Pesan rusak (Spesifikasi P2P Bagian 3).
    BadMessage(MessageError),
    /// Jawaban `Blocks` tidak berisi tepat satu entri per ID yang diminta.
    WrongEntryCount {
        /// Jumlah ID yang diminta.
        expected: usize,
        /// Jumlah entri yang diterima.
        actual: usize,
    },
    /// Sanksi dari pemanggil lewat [`Network::punish`] (misalnya blok atau transaksi tidak sah).
    Reported,
}

/// Saluran jawaban untuk satu permintaan masuk. Jika dibuang tanpa dijawab, rekan menerima kegagalan.
#[derive(Debug)]
pub struct Reply<T>(ResponseChannel<Received<T>>);

/// Event dari layanan jaringan.
#[derive(Debug)]
pub enum NetEvent {
    /// Node mendengar di alamat ini.
    Listening {
        /// Alamat dengar (tanpa `/p2p`).
        address: Multiaddr,
    },
    /// Koneksi dengan rekan terbentuk (sesudah batas koneksi dan blokir diperiksa).
    PeerConnected {
        /// Rekan.
        peer: PeerId,
        /// Alamat rekan.
        address: Multiaddr,
    },
    /// Koneksi terakhir dengan rekan tertutup.
    PeerDisconnected {
        /// Rekan.
        peer: PeerId,
    },
    /// `Status` rekan diterima (versi dan jaringan cocok). Bisa muncul dua kali per koneksi: dari permintaan rekan
    /// dan dari jawaban atas permintaan node.
    PeerStatus {
        /// Rekan.
        peer: PeerId,
        /// Ujung chain rekan.
        status: Status,
    },
    /// Rekan meminta header.
    GetHeaders {
        /// Rekan.
        peer: PeerId,
        /// Permintaan.
        request: GetHeaders,
        /// Saluran jawaban.
        reply: Reply<Headers>,
    },
    /// Rekan meminta badan blok.
    GetBlocks {
        /// Rekan.
        peer: PeerId,
        /// Permintaan.
        request: GetBlocks,
        /// Saluran jawaban.
        reply: Reply<Blocks>,
    },
    /// Rekan menyiarkan blok.
    Block {
        /// Rekan.
        peer: PeerId,
        /// Blok (encoding kanonis sudah diperiksa, aturan konsensus belum).
        block: Box<Block>,
        /// Saluran jawaban `Ack`.
        reply: Reply<Ack>,
    },
    /// Rekan menyiarkan transaksi.
    Tx {
        /// Rekan.
        peer: PeerId,
        /// Transaksi (encoding kanonis sudah diperiksa, aturan belum).
        tx: Transaction,
        /// Saluran jawaban `Ack`.
        reply: Reply<Ack>,
    },
    /// Jawaban atas [`Network::request_headers`].
    Headers {
        /// Rekan.
        peer: PeerId,
        /// ID permintaan.
        request: RequestId,
        /// Header dari rekan (belum divalidasi).
        headers: Headers,
    },
    /// Jawaban atas [`Network::request_blocks`], dengan jumlah entri sama dengan jumlah ID yang diminta.
    Blocks {
        /// Rekan.
        peer: PeerId,
        /// ID permintaan.
        request: RequestId,
        /// Entri per ID.
        blocks: Blocks,
    },
    /// Jawaban atas [`Network::send_block`].
    BlockAck {
        /// Rekan.
        peer: PeerId,
        /// ID permintaan.
        request: RequestId,
        /// Jawaban.
        ack: Ack,
    },
    /// Jawaban atas [`Network::send_tx`].
    TxAck {
        /// Rekan.
        peer: PeerId,
        /// ID permintaan.
        request: RequestId,
        /// Jawaban.
        ack: Ack,
    },
    /// Permintaan keluar gagal (tidak terhubung, koneksi tertutup, waktu habis, atau protokol tidak didukung).
    RequestFailed {
        /// Rekan.
        peer: PeerId,
        /// Protokol permintaan.
        kind: ProtocolKind,
        /// ID permintaan.
        request: RequestId,
        /// `true` jika gagal karena 30 detik tanpa jawaban.
        timed_out: bool,
    },
    /// Rekan disanksi: koneksi diputus dan rekan diblokir 24 jam.
    PeerPunished {
        /// Rekan.
        peer: PeerId,
        /// Alasan.
        offense: Offense,
    },
}

/// Data rekan yang sedang terhubung.
#[derive(Debug)]
struct PeerInfo {
    /// Alamat yang didial node (tanpa `/p2p`), jika koneksi ini keluar.
    dialed: Option<Multiaddr>,
    /// Alamat IP rekan.
    ip: Option<IpAddr>,
    /// Permintaan berturut-turut yang tidak dijawab.
    timeouts: u8,
}

/// Permintaan, jawaban, atau kegagalan dari satu behaviour request-response.
enum Rx<Req, Resp> {
    Request {
        peer: PeerId,
        request: Received<Req>,
        channel: ResponseChannel<Received<Resp>>,
    },
    Response {
        peer: PeerId,
        request: RequestId,
        response: Received<Resp>,
    },
    Failed {
        peer: PeerId,
        request: RequestId,
        error: OutboundFailure,
    },
}

fn split<Req, Resp>(
    event: request_response::Event<Received<Req>, Received<Resp>>,
) -> Option<Rx<Req, Resp>> {
    match event {
        request_response::Event::Message { peer, message, .. } => match message {
            request_response::Message::Request {
                request, channel, ..
            } => Some(Rx::Request {
                peer,
                request,
                channel,
            }),
            request_response::Message::Response {
                request_id,
                response,
            } => Some(Rx::Response {
                peer,
                request: request_id,
                response,
            }),
        },
        request_response::Event::OutboundFailure {
            peer,
            request_id,
            error,
            ..
        } => Some(Rx::Failed {
            peer,
            request: request_id,
            error,
        }),
        request_response::Event::InboundFailure { .. }
        | request_response::Event::ResponseSent { .. } => None,
    }
}

/// Alamat IP pertama di `address`.
pub(crate) fn ip_of(address: &Multiaddr) -> Option<IpAddr> {
    address.iter().find_map(|part| match part {
        Protocol::Ip4(ip) => Some(IpAddr::V4(ip)),
        Protocol::Ip6(ip) => Some(IpAddr::V6(ip)),
        _ => None,
    })
}

/// `PeerId` di bagian `/p2p/...` dari `address`.
pub(crate) fn peer_of(address: &Multiaddr) -> Option<PeerId> {
    address.iter().find_map(|part| match part {
        Protocol::P2p(peer) => Some(peer),
        _ => None,
    })
}

/// `address` tanpa bagian `/p2p/...`.
pub(crate) fn strip_p2p(address: &Multiaddr) -> Multiaddr {
    let mut out = Multiaddr::empty();
    for part in address.iter() {
        if !matches!(part, Protocol::P2p(_)) {
            out.push(part);
        }
    }
    out
}

/// Layanan jaringan satu node.
pub struct Network {
    swarm: Swarm<Behaviour>,
    network_id: Hash32,
    local_status: Status,
    connect: Vec<Multiaddr>,
    peers: BTreeMap<PeerId, PeerInfo>,
    bans: BanList,
    pending_bodies: BTreeMap<RequestId, usize>,
    events: VecDeque<NetEvent>,
    redial: Interval,
}

impl Network {
    /// Membangun swarm, mulai mendengar di `config.listen`, dan mendial `config.connect`.
    ///
    /// Wajib dipanggil di dalam runtime tokio.
    ///
    /// # Errors
    ///
    /// [`NetworkError`] jika nama protokol, transport, atau alamat dengar gagal.
    pub fn new(
        keypair: Keypair,
        config: NetworkConfig,
        local_status: Status,
    ) -> Result<Self, NetworkError> {
        let behaviour = Behaviour::new(&config.network_id)?;
        let noise = noise::Config::new(&keypair)
            .map_err(|error| NetworkError::Transport(error.to_string()))?;
        // Sama dengan `SwarmBuilder::with_tcp` libp2p 0.57.1: TCP, lalu Noise XX, lalu Yamux, batas waktu 10 detik.
        let transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise)
            .multiplex(yamux::Config::default())
            .timeout(CONNECTION_TIMEOUT)
            .boxed();
        let mut swarm = Swarm::new(
            transport,
            behaviour,
            keypair.public().to_peer_id(),
            swarm::Config::with_tokio_executor().with_idle_connection_timeout(IDLE_TIMEOUT),
        );
        for address in &config.listen {
            swarm
                .listen_on(address.clone())
                .map_err(|error| NetworkError::Listen {
                    address: address.to_string(),
                    reason: error.to_string(),
                })?;
        }
        for address in &config.connect {
            // Gagal dial (misalnya alamat tidak didukung) tidak fatal; dicoba lagi pada selang berikutnya.
            let _ = swarm.dial(address.clone());
        }
        let period = config.redial_interval.max(Duration::from_millis(1));
        let mut redial = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
        redial.set_missed_tick_behavior(MissedTickBehavior::Delay);
        Ok(Self {
            swarm,
            network_id: config.network_id,
            local_status,
            connect: config.connect,
            peers: BTreeMap::new(),
            bans: BanList::new(),
            pending_bodies: BTreeMap::new(),
            events: VecDeque::new(),
            redial,
        })
    }

    /// `PeerId` node ini.
    #[must_use]
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Alamat dengar yang aktif.
    #[must_use]
    pub fn listeners(&self) -> Vec<Multiaddr> {
        self.swarm.listeners().cloned().collect()
    }

    /// Rekan yang sedang terhubung, urut `PeerId`.
    #[must_use]
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.peers.keys().copied().collect()
    }

    /// Mengganti `Status` yang dikirim ke rekan (setiap kali ujung chain aktif berubah).
    pub fn set_local_status(&mut self, status: Status) {
        self.local_status = status;
    }

    /// `true` jika `peer` sedang diblokir.
    #[must_use]
    pub fn is_banned(&self, peer: &PeerId) -> bool {
        self.bans.is_peer_banned(peer, Instant::now())
    }

    /// Meminta header ke `peer`; jawabannya [`NetEvent::Headers`].
    pub fn request_headers(&mut self, peer: &PeerId, request: GetHeaders) -> RequestId {
        self.swarm
            .behaviour_mut()
            .headers
            .send_request(peer, Ok(request))
    }

    /// Meminta badan blok ke `peer`; jawabannya [`NetEvent::Blocks`].
    pub fn request_blocks(&mut self, peer: &PeerId, request: GetBlocks) -> RequestId {
        let expected = request.ids.len();
        let id = self
            .swarm
            .behaviour_mut()
            .bodies
            .send_request(peer, Ok(request));
        self.pending_bodies.insert(id, expected);
        id
    }

    /// Mengirim blok ke `peer`; jawabannya [`NetEvent::BlockAck`].
    pub fn send_block(&mut self, peer: &PeerId, block: Block) -> RequestId {
        self.swarm
            .behaviour_mut()
            .block
            .send_request(peer, Ok(block))
    }

    /// Mengirim transaksi ke `peer`; jawabannya [`NetEvent::TxAck`].
    pub fn send_tx(&mut self, peer: &PeerId, tx: Transaction) -> RequestId {
        self.swarm.behaviour_mut().tx.send_request(peer, Ok(tx))
    }

    /// Menjawab [`NetEvent::GetHeaders`]. `false` jika permintaan sudah tidak menunggu jawaban.
    pub fn reply_headers(&mut self, reply: Reply<Headers>, headers: Headers) -> bool {
        self.swarm
            .behaviour_mut()
            .headers
            .send_response(reply.0, Ok(headers))
            .is_ok()
    }

    /// Menjawab [`NetEvent::GetBlocks`] (tepat satu entri per ID, urutan sama).
    pub fn reply_blocks(&mut self, reply: Reply<Blocks>, blocks: Blocks) -> bool {
        self.swarm
            .behaviour_mut()
            .bodies
            .send_response(reply.0, Ok(blocks))
            .is_ok()
    }

    /// Menjawab [`NetEvent::Block`] atau [`NetEvent::Tx`].
    pub fn reply_ack(&mut self, reply: Reply<Ack>, ack: Ack) -> bool {
        // Saluran jawaban tidak terikat pada behaviour tertentu, jadi `block` juga dipakai untuk `tx`.
        self.swarm
            .behaviour_mut()
            .block
            .send_response(reply.0, Ok(ack))
            .is_ok()
    }

    /// Memutus koneksi dengan `peer` tanpa blokir.
    pub fn disconnect(&mut self, peer: &PeerId) {
        let _ = self.swarm.disconnect_peer_id(*peer);
    }

    /// Memberi sanksi kepada `peer`: putus dan blokir 24 jam (per `PeerId` dan per IP bukan loopback).
    pub fn punish(&mut self, peer: &PeerId) {
        self.apply_punishment(*peer, Offense::Reported);
    }

    /// Menunggu event berikutnya. Selama menunggu, layanan juga menjawab `status`, menyambung ulang, dan mencabut
    /// blokir yang sudah berakhir.
    pub async fn next_event(&mut self) -> NetEvent {
        loop {
            if let Some(event) = self.events.pop_front() {
                return event;
            }
            let swarm_event = tokio::select! {
                event = self.swarm.select_next_some() => Some(event),
                _ = self.redial.tick() => None,
            };
            match swarm_event {
                Some(event) => self.on_swarm_event(event),
                None => self.on_tick(),
            }
        }
    }

    fn on_swarm_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                self.events.push_back(NetEvent::Listening { address });
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                endpoint,
                ..
            } => self.on_connected(peer_id, connection_id, &endpoint),
            SwarmEvent::ConnectionClosed {
                peer_id,
                num_established,
                ..
            } => {
                if num_established == 0 && self.peers.remove(&peer_id).is_some() {
                    self.events
                        .push_back(NetEvent::PeerDisconnected { peer: peer_id });
                }
            }
            SwarmEvent::Behaviour(event) => self.on_behaviour(event),
            _ => {}
        }
    }

    fn on_connected(&mut self, peer: PeerId, connection: ConnectionId, endpoint: &ConnectedPoint) {
        let now = Instant::now();
        let address = endpoint.get_remote_address().clone();
        let ip = ip_of(&address);
        let ip_banned = ip.is_some_and(|ip| self.bans.is_ip_banned(&ip, now));
        if ip_banned || self.bans.is_peer_banned(&peer, now) {
            self.swarm.close_connection(connection);
            return;
        }
        let dialed = endpoint.is_dialer().then(|| strip_p2p(&address));
        self.peers.insert(
            peer,
            PeerInfo {
                dialed,
                ip,
                timeouts: 0,
            },
        );
        self.events
            .push_back(NetEvent::PeerConnected { peer, address });
        let status = self.local_status.clone();
        self.swarm
            .behaviour_mut()
            .status
            .send_request(&peer, Ok(status));
    }

    fn on_behaviour(&mut self, event: BehaviourEvent) {
        match event {
            BehaviourEvent::Limits(never) | BehaviourEvent::Banned(never) => match never {},
            BehaviourEvent::Status(event) => self.on_status(event),
            BehaviourEvent::Headers(event) => self.on_headers(event),
            BehaviourEvent::Bodies(event) => self.on_bodies(event),
            BehaviourEvent::Block(event) => self.on_block(event),
            BehaviourEvent::Tx(event) => self.on_tx(event),
        }
    }

    fn on_status(&mut self, event: request_response::Event<Received<Status>, Received<Status>>) {
        match split(event) {
            Some(Rx::Request {
                peer,
                request,
                channel,
            }) => match request {
                Ok(status) if status.is_compatible(&self.network_id) => {
                    let local = self.local_status.clone();
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .status
                        .send_response(channel, Ok(local));
                    self.events.push_back(NetEvent::PeerStatus { peer, status });
                }
                Ok(_) => self.disconnect(&peer),
                Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
            },
            Some(Rx::Response { peer, response, .. }) => {
                self.reset_timeouts(&peer);
                match response {
                    Ok(status) if status.is_compatible(&self.network_id) => {
                        self.events.push_back(NetEvent::PeerStatus { peer, status });
                    }
                    Ok(_) => self.disconnect(&peer),
                    Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
                }
            }
            Some(Rx::Failed {
                peer,
                request,
                error,
            }) => self.on_failure(peer, ProtocolKind::Status, request, &error),
            None => {}
        }
    }

    fn on_headers(
        &mut self,
        event: request_response::Event<Received<GetHeaders>, Received<Headers>>,
    ) {
        match split(event) {
            Some(Rx::Request {
                peer,
                request,
                channel,
            }) => match request {
                Ok(request) => self.events.push_back(NetEvent::GetHeaders {
                    peer,
                    request,
                    reply: Reply(channel),
                }),
                Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
            },
            Some(Rx::Response {
                peer,
                request,
                response,
            }) => {
                self.reset_timeouts(&peer);
                match response {
                    Ok(headers) => self.events.push_back(NetEvent::Headers {
                        peer,
                        request,
                        headers,
                    }),
                    Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
                }
            }
            Some(Rx::Failed {
                peer,
                request,
                error,
            }) => self.on_failure(peer, ProtocolKind::Headers, request, &error),
            None => {}
        }
    }

    fn on_bodies(&mut self, event: request_response::Event<Received<GetBlocks>, Received<Blocks>>) {
        match split(event) {
            Some(Rx::Request {
                peer,
                request,
                channel,
            }) => match request {
                Ok(request) => self.events.push_back(NetEvent::GetBlocks {
                    peer,
                    request,
                    reply: Reply(channel),
                }),
                Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
            },
            Some(Rx::Response {
                peer,
                request,
                response,
            }) => {
                self.reset_timeouts(&peer);
                let expected = self.pending_bodies.remove(&request);
                match response {
                    Ok(blocks) => {
                        let actual = blocks.entries.len();
                        match expected {
                            Some(expected) if expected != actual => self.apply_punishment(
                                peer,
                                Offense::WrongEntryCount { expected, actual },
                            ),
                            _ => self.events.push_back(NetEvent::Blocks {
                                peer,
                                request,
                                blocks,
                            }),
                        }
                    }
                    Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
                }
            }
            Some(Rx::Failed {
                peer,
                request,
                error,
            }) => {
                self.pending_bodies.remove(&request);
                self.on_failure(peer, ProtocolKind::Blocks, request, &error);
            }
            None => {}
        }
    }

    fn on_block(&mut self, event: request_response::Event<Received<Block>, Received<Ack>>) {
        match split(event) {
            Some(Rx::Request {
                peer,
                request,
                channel,
            }) => match request {
                Ok(block) => self.events.push_back(NetEvent::Block {
                    peer,
                    block: Box::new(block),
                    reply: Reply(channel),
                }),
                Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
            },
            Some(Rx::Response {
                peer,
                request,
                response,
            }) => {
                self.reset_timeouts(&peer);
                match response {
                    Ok(ack) => self
                        .events
                        .push_back(NetEvent::BlockAck { peer, request, ack }),
                    Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
                }
            }
            Some(Rx::Failed {
                peer,
                request,
                error,
            }) => self.on_failure(peer, ProtocolKind::Block, request, &error),
            None => {}
        }
    }

    fn on_tx(&mut self, event: request_response::Event<Received<Transaction>, Received<Ack>>) {
        match split(event) {
            Some(Rx::Request {
                peer,
                request,
                channel,
            }) => match request {
                Ok(tx) => self.events.push_back(NetEvent::Tx {
                    peer,
                    tx,
                    reply: Reply(channel),
                }),
                Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
            },
            Some(Rx::Response {
                peer,
                request,
                response,
            }) => {
                self.reset_timeouts(&peer);
                match response {
                    Ok(ack) => self
                        .events
                        .push_back(NetEvent::TxAck { peer, request, ack }),
                    Err(error) => self.apply_punishment(peer, Offense::BadMessage(error)),
                }
            }
            Some(Rx::Failed {
                peer,
                request,
                error,
            }) => self.on_failure(peer, ProtocolKind::Tx, request, &error),
            None => {}
        }
    }

    fn on_failure(
        &mut self,
        peer: PeerId,
        kind: ProtocolKind,
        request: RequestId,
        error: &OutboundFailure,
    ) {
        let timed_out = matches!(error, OutboundFailure::Timeout);
        match error {
            OutboundFailure::Timeout => {
                let mut too_many = false;
                if let Some(info) = self.peers.get_mut(&peer) {
                    info.timeouts = info.timeouts.saturating_add(1);
                    if info.timeouts >= MAX_TIMEOUTS {
                        info.timeouts = 0;
                        too_many = true;
                    }
                }
                if too_many {
                    self.disconnect(&peer);
                }
            }
            // Rekan tidak mendukung protokol jaringan ini (jaringan lain): putus tanpa sanksi.
            OutboundFailure::UnsupportedProtocols => self.disconnect(&peer),
            _ => {}
        }
        self.events.push_back(NetEvent::RequestFailed {
            peer,
            kind,
            request,
            timed_out,
        });
    }

    fn reset_timeouts(&mut self, peer: &PeerId) {
        if let Some(info) = self.peers.get_mut(peer) {
            info.timeouts = 0;
        }
    }

    fn apply_punishment(&mut self, peer: PeerId, offense: Offense) {
        let ip = self.peers.get(&peer).and_then(|info| info.ip);
        self.bans.ban(peer, ip, Instant::now());
        // Memutus semua koneksi rekan ini dan menolak koneksi barunya.
        self.swarm.behaviour_mut().banned.block_peer(peer);
        self.events
            .push_back(NetEvent::PeerPunished { peer, offense });
    }

    fn on_tick(&mut self) {
        let now = Instant::now();
        for peer in self.bans.expire(now) {
            self.swarm.behaviour_mut().banned.unblock_peer(peer);
        }
        let targets: Vec<Multiaddr> = self
            .connect
            .iter()
            .filter(|address| self.should_dial(address, now))
            .cloned()
            .collect();
        for address in targets {
            let _ = self.swarm.dial(address);
        }
    }

    fn should_dial(&self, address: &Multiaddr, now: Instant) -> bool {
        if let Some(peer) = peer_of(address)
            && (self.peers.contains_key(&peer) || self.bans.is_peer_banned(&peer, now))
        {
            return false;
        }
        if ip_of(address).is_some_and(|ip| self.bans.is_ip_banned(&ip, now)) {
            return false;
        }
        let plain = strip_p2p(address);
        !self
            .peers
            .values()
            .any(|info| info.dialed.as_ref() == Some(&plain))
    }
}

#[cfg(test)]
mod tests;
