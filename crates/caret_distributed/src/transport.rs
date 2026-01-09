// Caret Distributed - Transport layer
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{codec::FrameDecoder, Error, Message, Result};
use parking_lot::Mutex;
use rustls::{ClientConfig, ServerConfig};
use rustls_pemfile::{certs, private_key};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;
use tokio_rustls::TlsAcceptor as RustlsAcceptor;
use tokio_rustls::TlsConnector as RustlsConnector;

/// Transport configuration
#[derive(Clone, Debug)]
pub struct TransportConfig {
    /// Bind address
    pub bind_addr: SocketAddr,
    /// Maximum message size
    pub max_message_size: usize,
    /// Send buffer size
    pub send_buffer_size: usize,
    /// Receive buffer size
    pub recv_buffer_size: usize,
    /// TLS configuration for server mode
    pub tls_server_config: Option<Arc<ServerConfig>>,
    /// TLS configuration for client mode
    pub tls_client_config: Option<Arc<ClientConfig>>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            bind_addr: format!("0.0.0.0:{}", crate::DEFAULT_PORT).parse().unwrap(),
            max_message_size: crate::MAX_MESSAGE_SIZE,
            send_buffer_size: 1024,
            recv_buffer_size: 1024,
            tls_server_config: None,
            tls_client_config: None,
        }
    }
}

impl TransportConfig {
    /// Create a new config with the given bind address
    pub fn with_bind_addr(addr: impl Into<SocketAddr>) -> Self {
        Self {
            bind_addr: addr.into(),
            ..Default::default()
        }
    }

    /// Set the maximum message size
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Set the send buffer size
    pub fn with_send_buffer_size(mut self, size: usize) -> Self {
        self.send_buffer_size = size;
        self
    }

    /// Set the receive buffer size
    pub fn with_recv_buffer_size(mut self, size: usize) -> Self {
        self.recv_buffer_size = size;
        self
    }

    /// Set the TLS server configuration
    pub fn with_tls_server_config(mut self, config: Arc<ServerConfig>) -> Self {
        self.tls_server_config = Some(config);
        self
    }

    /// Set the TLS client configuration
    pub fn with_tls_client_config(mut self, config: Arc<ClientConfig>) -> Self {
        self.tls_client_config = Some(config);
        self
    }

    /// Create a TLS server configuration from PEM files
    pub fn with_tls_server_pem(
        mut self,
        cert_path: impl AsRef<std::path::Path>,
        key_path: impl AsRef<std::path::Path>,
    ) -> Result<Self> {
        let cert_file = File::open(cert_path).map_err(|e| Error::Io(e))?;
        let key_file = File::open(key_path).map_err(|e| Error::Io(e))?;

        let cert_chain = certs(&mut BufReader::new(cert_file))
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        let key_der = private_key(&mut BufReader::new(key_file))
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(cert_chain, key_der)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        self.tls_server_config = Some(Arc::new(config));
        Ok(self)
    }
}

/// Transport event
#[derive(Clone, Debug)]
pub enum TransportEvent {
    /// Message received
    Message { from: SocketAddr, message: Message },
    /// New connection established
    Connected { addr: SocketAddr },
    /// Connection closed
    Disconnected { addr: SocketAddr },
    /// Transport error
    Error { addr: SocketAddr, error: String },
}

/// Transport trait for network communication
pub trait Transport: Send + Sync {
    /// Get the local address
    fn local_addr(&self) -> SocketAddr;

    /// Send a message to the given address
    fn send(&self, to: SocketAddr, message: Message) -> Result<()>;

    /// Broadcast a message to all known peers
    fn broadcast(&self, message: Message) -> Result<()>;

    /// Subscribe to transport events
    fn subscribe(&self) -> mpsc::Receiver<TransportEvent>;

    /// Start the transport
    fn start(&mut self) -> Result<()>;

    /// Stop the transport
    fn stop(&mut self) -> Result<()>;

    /// Check if the transport is running
    fn is_running(&self) -> bool;
}

/// TCP transport for real network communication
pub struct TcpTransport {
    /// Local address
    local_addr: SocketAddr,
    /// Connection write handles (addr -> sender)
    senders: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>>>,
    /// Event channel
    events: Arc<Mutex<mpsc::Sender<TransportEvent>>>,
    /// Running state
    running: Arc<Mutex<bool>>,
    /// Max message size
    max_message_size: usize,
    /// Background tasks
    _tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl TcpTransport {
    /// Create a new TCP transport in server mode
    pub async fn bind(config: TransportConfig) -> Result<Self> {
        let listener = TcpListener::bind(config.bind_addr)
            .await
            .map_err(|e| Error::Io(e))?;

        let local_addr = listener.local_addr().map_err(|e| Error::Io(e))?;

        let (events, _rx): (mpsc::Sender<TransportEvent>, mpsc::Receiver<TransportEvent>) =
            mpsc::channel(config.recv_buffer_size);

        let senders: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(Mutex::new(false));
        let tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));

        // Start accept loop
        let conn_events = Arc::new(TokioMutex::new(events.clone()));
        let conn_senders = Arc::clone(&senders);
        let conn_running = Arc::clone(&running);
        let max_size = config.max_message_size;

        let task = tokio::spawn(async move {
            while *conn_running.lock() {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let stream = Arc::new(TokioMutex::new(stream));
                        let (tx, mut rx): (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) =
                            mpsc::channel(1024);
                        conn_senders.lock().insert(addr, tx.clone());

                        let _ = conn_events
                            .lock()
                            .await
                            .try_send(TransportEvent::Connected { addr });

                        // Spawn read task
                        let conn_events_clone = Arc::clone(&conn_events);
                        let conn_senders_clone = Arc::clone(&conn_senders);
                        let conn_running_clone = Arc::clone(&conn_running);
                        let stream_clone = Arc::clone(&stream);

                        tokio::spawn(async move {
                            let mut decoder = FrameDecoder::with_max_size(max_size);
                            let mut buf = [0u8; 8192];

                            while *conn_running_clone.lock() {
                                {
                                    let mut s = stream_clone.lock().await;
                                    match tokio::io::AsyncReadExt::read(&mut *s, &mut buf).await {
                                        Ok(0) => {
                                            conn_senders_clone.lock().remove(&addr);
                                            let _ = conn_events_clone
                                                .lock()
                                                .await
                                                .try_send(TransportEvent::Disconnected { addr });
                                            break;
                                        }
                                        Ok(n) => {
                                            decoder.feed(&buf[..n]);

                                            while let Ok(msg) = decoder.try_decode() {
                                                if let Some(decoded) = msg {
                                                    let _ = conn_events_clone
                                                        .lock()
                                                        .await
                                                        .try_send(TransportEvent::Message {
                                                            from: addr,
                                                            message: decoded,
                                                        });
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            conn_senders_clone.lock().remove(&addr);
                                            break;
                                        }
                                    }
                                }
                            }
                        });

                        // Spawn write task
                        let conn_events_clone = Arc::clone(&conn_events);
                        tokio::spawn(async move {
                            while let Some(data) = rx.recv().await {
                                let mut s = stream.lock().await;
                                if let Err(e) =
                                    tokio::io::AsyncWriteExt::write_all(&mut *s, &data).await
                                {
                                    let _ = conn_events_clone.lock().await.try_send(
                                        TransportEvent::Error {
                                            addr,
                                            error: e.to_string(),
                                        },
                                    );
                                    break;
                                }
                                let _ = tokio::io::AsyncWriteExt::flush(&mut *s).await;
                            }
                        });
                    }
                    Err(e) => {
                        if *conn_running.lock() {
                            let _ = conn_events.lock().await.try_send(TransportEvent::Error {
                                addr: local_addr,
                                error: e.to_string(),
                            });
                        }
                    }
                }
            }
        });

        tasks.lock().push(task);
        *running.lock() = true;

        Ok(Self {
            local_addr,
            senders,
            events: Arc::new(Mutex::new(events)),
            running,
            max_message_size: config.max_message_size,
            _tasks: tasks,
        })
    }

    /// Connect to a remote TCP transport (client mode)
    pub async fn connect(remote_addr: SocketAddr, config: TransportConfig) -> Result<Self> {
        let stream = TcpStream::connect(remote_addr)
            .await
            .map_err(|e| Error::ConnectionRefused(e.to_string()))?;

        let local_addr = stream.local_addr().map_err(|e| Error::Io(e))?;

        let (events, _rx): (mpsc::Sender<TransportEvent>, mpsc::Receiver<TransportEvent>) =
            mpsc::channel(config.recv_buffer_size);
        let (tx, mut rx): (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) = mpsc::channel(1024);

        let senders: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        senders.lock().insert(remote_addr, tx);

        let running = Arc::new(Mutex::new(true));
        let tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));

        // Start read loop
        let conn_events = Arc::new(TokioMutex::new(events.clone()));
        let conn_events_clone = Arc::clone(&conn_events);
        let stream = Arc::new(TokioMutex::new(stream));
        let stream_clone = Arc::clone(&stream);
        let max_size = config.max_message_size;

        let read_task = tokio::spawn(async move {
            let mut decoder = FrameDecoder::with_max_size(max_size);
            let mut buf = [0u8; 8192];

            loop {
                {
                    let mut s = stream_clone.lock().await;
                    match tokio::io::AsyncReadExt::read(&mut *s, &mut buf).await {
                        Ok(0) => {
                            let _ = conn_events
                                .lock()
                                .await
                                .try_send(TransportEvent::Disconnected { addr: remote_addr });
                            break;
                        }
                        Ok(n) => {
                            decoder.feed(&buf[..n]);

                            while let Ok(msg) = decoder.try_decode() {
                                if let Some(decoded) = msg {
                                    let _ = conn_events.lock().await.try_send(
                                        TransportEvent::Message {
                                            from: remote_addr,
                                            message: decoded,
                                        },
                                    );
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
        });

        // Start write loop
        let write_task = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                let mut s = stream.lock().await;
                if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut *s, &data).await {
                    let _ = conn_events_clone
                        .lock()
                        .await
                        .try_send(TransportEvent::Error {
                            addr: remote_addr,
                            error: e.to_string(),
                        });
                    break;
                }
                let _ = tokio::io::AsyncWriteExt::flush(&mut *s).await;
            }
        });

        tasks.lock().push(read_task);
        tasks.lock().push(write_task);

        let _ = events.try_send(TransportEvent::Connected { addr: remote_addr });

        Ok(Self {
            local_addr,
            senders,
            events: Arc::new(Mutex::new(events)),
            running,
            max_message_size: config.max_message_size,
            _tasks: tasks,
        })
    }
}

#[async_trait::async_trait]
impl Transport for TcpTransport {
    fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    fn send(&self, to: SocketAddr, message: Message) -> Result<()> {
        if !*self.running.lock() {
            return Err(Error::Transport("Transport not running".into()));
        }

        let codec = crate::codec::FrameCodec::with_max_size(self.max_message_size);
        let encoded = codec.encode(&message)?;

        let sender = {
            let senders = self.senders.lock();
            senders.get(&to).cloned()
        };

        if let Some(sender) = sender {
            let _ = sender.try_send(encoded);
            Ok(())
        } else {
            Err(Error::Transport(format!("No connection to {}", to)))
        }
    }

    fn broadcast(&self, message: Message) -> Result<()> {
        let codec = crate::codec::FrameCodec::with_max_size(self.max_message_size);
        let encoded = codec.encode(&message)?;

        let senders = self.senders.lock();
        for sender in senders.values() {
            let _ = sender.try_send(encoded.clone());
        }

        Ok(())
    }

    fn subscribe(&self) -> mpsc::Receiver<TransportEvent> {
        let (tx, rx) = mpsc::channel(1024);
        *self.events.lock() = tx;
        rx
    }

    fn start(&mut self) -> Result<()> {
        *self.running.lock() = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        *self.running.lock() = false;
        self.senders.lock().clear();
        Ok(())
    }

    fn is_running(&self) -> bool {
        *self.running.lock()
    }
}

/// TLS transport for secure TCP communication
///
/// Wraps TcpTransport with TLS encryption using rustls.
pub struct TlsTransport {
    /// Wrapped TCP transport
    inner: TcpTransport,
}

impl TlsTransport {
    /// Create a new TLS transport in server mode
    pub async fn bind_tls(config: TransportConfig) -> Result<Self> {
        let tls_config = config
            .tls_server_config
            .clone()
            .ok_or_else(|| Error::Transport("TLS server config required for TLS mode".into()))?;

        let listener = TcpListener::bind(config.bind_addr)
            .await
            .map_err(|e| Error::Io(e))?;

        let local_addr = listener.local_addr().map_err(|e| Error::Io(e))?;

        let acceptor = RustlsAcceptor::new(tls_config)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        let (events, _rx): (mpsc::Sender<TransportEvent>, mpsc::Receiver<TransportEvent>) =
            mpsc::channel(config.recv_buffer_size);

        let senders: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(Mutex::new(false));
        let tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));

        // Start accept loop with TLS
        let conn_events = Arc::new(TokioMutex::new(events.clone()));
        let conn_senders = Arc::clone(&senders);
        let conn_running = Arc::clone(&running);
        let max_size = config.max_message_size;
        let acceptor = Arc::new(acceptor);

        let task = tokio::spawn(async move {
            while *conn_running.lock() {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        let tls_stream = match acceptor.accept(stream).await {
                            Ok(s) => s,
                            Err(e) => {
                                let _ = conn_events.lock().await.try_send(TransportEvent::Error {
                                    addr: local_addr,
                                    error: format!("TLS handshake failed: {}", e),
                                });
                                continue;
                            }
                        };

                        let tls_stream = Arc::new(TokioMutex::new(tls_stream));
                        let (tx, mut rx): (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) =
                            mpsc::channel(1024);
                        conn_senders.lock().insert(addr, tx.clone());

                        let _ = conn_events
                            .lock()
                            .await
                            .try_send(TransportEvent::Connected { addr });

                        // Spawn read task
                        let conn_events_clone = Arc::clone(&conn_events);
                        let conn_senders_clone = Arc::clone(&conn_senders);
                        let conn_running_clone = Arc::clone(&conn_running);
                        let tls_stream_clone = Arc::clone(&tls_stream);

                        tokio::spawn(async move {
                            let mut decoder = FrameDecoder::with_max_size(max_size);
                            let mut buf = [0u8; 8192];

                            while *conn_running_clone.lock() {
                                {
                                    let mut s = tls_stream_clone.lock().await;
                                    match tokio::io::AsyncReadExt::read(&mut *s, &mut buf).await {
                                        Ok(0) => {
                                            conn_senders_clone.lock().remove(&addr);
                                            let _ = conn_events_clone
                                                .lock()
                                                .await
                                                .try_send(TransportEvent::Disconnected { addr });
                                            break;
                                        }
                                        Ok(n) => {
                                            decoder.feed(&buf[..n]);

                                            while let Ok(msg) = decoder.try_decode() {
                                                if let Some(decoded) = msg {
                                                    let _ = conn_events_clone
                                                        .lock()
                                                        .await
                                                        .try_send(TransportEvent::Message {
                                                            from: addr,
                                                            message: decoded,
                                                        });
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            conn_senders_clone.lock().remove(&addr);
                                            break;
                                        }
                                    }
                                }
                            }
                        });

                        // Spawn write task
                        let conn_events_clone = Arc::clone(&conn_events);
                        tokio::spawn(async move {
                            while let Some(data) = rx.recv().await {
                                let mut s = tls_stream.lock().await;
                                if let Err(e) =
                                    tokio::io::AsyncWriteExt::write_all(&mut *s, &data).await
                                {
                                    let _ = conn_events_clone.lock().await.try_send(
                                        TransportEvent::Error {
                                            addr,
                                            error: e.to_string(),
                                        },
                                    );
                                    break;
                                }
                                let _ = tokio::io::AsyncWriteExt::flush(&mut *s).await;
                            }
                        });
                    }
                    Err(e) => {
                        if *conn_running.lock() {
                            let _ = conn_events.lock().await.try_send(TransportEvent::Error {
                                addr: local_addr,
                                error: e.to_string(),
                            });
                        }
                    }
                }
            }
        });

        tasks.lock().push(task);
        *running.lock() = true;

        Ok(Self {
            inner: TcpTransport {
                local_addr,
                senders,
                events: Arc::new(Mutex::new(events)),
                running,
                max_message_size: config.max_message_size,
                _tasks: tasks,
            },
        })
    }

    /// Connect to a remote TLS transport (client mode)
    pub async fn connect_tls(remote_addr: SocketAddr, config: TransportConfig) -> Result<Self> {
        let tls_config = config
            .tls_client_config
            .clone()
            .ok_or_else(|| Error::Transport("TLS client config required for TLS mode".into()))?;

        let connector = RustlsConnector::from(Arc::clone(tls_config));
        let tcp_stream = TcpStream::connect(remote_addr)
            .await
            .map_err(|e| Error::ConnectionRefused(e.to_string()))?;
        let tls_stream = connector
            .connect(tcp_stream, "caret.local")
            .await
            .map_err(|e| Error::Transport(format!("TLS handshake failed: {}", e)))?;

        let local_addr = tls_stream
            .get_ref()
            .local_addr()
            .map_err(|e| Error::Io(e))?;

        let (events, _rx): (mpsc::Sender<TransportEvent>, mpsc::Receiver<TransportEvent>) =
            mpsc::channel(config.recv_buffer_size);
        let (tx, mut rx): (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) = mpsc::channel(1024);

        let senders: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<Vec<u8>>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        senders.lock().insert(remote_addr, tx);

        let running = Arc::new(Mutex::new(true));
        let tasks: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));

        // Start read loop
        let conn_events = Arc::new(TokioMutex::new(events.clone()));
        let conn_events_clone = Arc::clone(&conn_events);
        let tls_stream = Arc::new(TokioMutex::new(tls_stream));
        let tls_stream_clone = Arc::clone(&tls_stream);
        let max_size = config.max_message_size;

        let read_task = tokio::spawn(async move {
            let mut decoder = FrameDecoder::with_max_size(max_size);
            let mut buf = [0u8; 8192];

            loop {
                let mut s = tls_stream_clone.lock().await;
                match tokio::io::AsyncReadExt::read(&mut s, &mut buf).await {
                        Ok(0) => {
                            let _ = conn_events
                                .lock()
                                .await
                                .try_send(TransportEvent::Disconnected { addr: remote_addr });
                            break;
                        }
                        Ok(n) => {
                            decoder.feed(&buf[..n]);

                            while let Ok(msg) = decoder.try_decode() {
                                if let Some(decoded) = msg {
                                    let _ = conn_events.lock().await.try_send(
                                        TransportEvent::Message {
                                            from: remote_addr,
                                            message: decoded,
                                        },
                                    );
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
        });

        // Start write loop
        let write_task = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                let mut s = tls_stream.lock().await;
                if let Err(e) = tokio::io::AsyncWriteExt::write_all(&mut *s, &data).await {
                    let _ = conn_events_clone
                        .lock()
                        .await
                        .try_send(TransportEvent::Error {
                            addr: remote_addr,
                            error: e.to_string(),
                        });
                    break;
                }
                let _ = tokio::io::AsyncWriteExt::flush(&mut *s).await;
            }
        });

        tasks.lock().push(read_task);
        tasks.lock().push(write_task);

        let _ = events.try_send(TransportEvent::Connected { addr: remote_addr });

        Ok(Self {
            inner: TcpTransport {
                local_addr,
                senders,
                events: Arc::new(Mutex::new(events)),
                running,
                max_message_size: config.max_message_size,
                _tasks: tasks,
            },
        })
    }
}

impl Transport for TlsTransport {
    fn local_addr(&self) -> SocketAddr {
        self.inner.local_addr()
    }

    fn send(&self, to: SocketAddr, message: Message) -> Result<()> {
        self.inner.send(to, message)
    }

    fn broadcast(&self, message: Message) -> Result<()> {
        self.inner.broadcast(message)
    }

    fn subscribe(&self) -> mpsc::Receiver<TransportEvent> {
        self.inner.subscribe()
    }

    fn start(&mut self) -> Result<()> {
        self.inner.start()
    }

    fn stop(&mut self) -> Result<()> {
        self.inner.stop()
    }

    fn is_running(&self) -> bool {
        self.inner.is_running()
    }
}

/// In-memory transport for testing
pub struct MemoryTransport {
    local_addr: SocketAddr,
    peers: Arc<Mutex<Vec<SocketAddr>>>,
    events: Arc<Mutex<mpsc::Sender<TransportEvent>>>,
    running: Arc<Mutex<bool>>,
    _event_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl MemoryTransport {
    /// Create a new in-memory transport
    pub fn new(config: TransportConfig) -> Self {
        let (tx, _rx) = mpsc::channel(config.recv_buffer_size);

        Self {
            local_addr: config.bind_addr,
            peers: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(Mutex::new(tx)),
            running: Arc::new(Mutex::new(false)),
            _event_task: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to another in-memory transport
    pub fn connect_to(&mut self, addr: SocketAddr) {
        self.peers.lock().push(addr);
    }
}

impl Transport for MemoryTransport {
    fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    fn send(&self, _to: SocketAddr, message: Message) -> Result<()> {
        if !*self.running.lock() {
            return Err(Error::Transport("Transport not running".into()));
        }

        // In a real implementation, this would send to the peer
        // For now, we just emit a sent event
        let _ = self.events.lock().try_send(TransportEvent::Message {
            from: self.local_addr,
            message,
        });

        Ok(())
    }

    fn broadcast(&self, message: Message) -> Result<()> {
        if !*self.running.lock() {
            return Err(Error::Transport("Transport not running".into()));
        }

        let peers = self.peers.lock().clone();
        for peer in peers {
            self.send(peer, message.clone())?;
        }

        Ok(())
    }

    fn subscribe(&self) -> mpsc::Receiver<TransportEvent> {
        let (tx, rx) = mpsc::channel(1024);
        *self.events.lock() = tx;
        rx
    }

    fn start(&mut self) -> Result<()> {
        *self.running.lock() = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        *self.running.lock() = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        *self.running.lock()
    }
}

/// Clone the event sender for a memory transport
impl Clone for MemoryTransport {
    fn clone(&self) -> Self {
        Self {
            local_addr: self.local_addr,
            peers: Arc::clone(&self.peers),
            events: Arc::clone(&self.events),
            running: Arc::clone(&self.running),
            _event_task: Arc::clone(&self._event_task),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.max_message_size, crate::MAX_MESSAGE_SIZE);
        assert_eq!(config.send_buffer_size, 1024);
        assert_eq!(config.recv_buffer_size, 1024);
    }

    #[test]
    fn test_transport_config_builder() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let config = TransportConfig::with_bind_addr(addr)
            .with_max_message_size(1024)
            .with_send_buffer_size(512)
            .with_recv_buffer_size(256);

        assert_eq!(config.bind_addr, addr);
        assert_eq!(config.max_message_size, 1024);
        assert_eq!(config.send_buffer_size, 512);
        assert_eq!(config.recv_buffer_size, 256);
    }

    #[test]
    fn test_memory_transport() {
        let addr: SocketAddr = "127.0.0.1:9234".parse().unwrap();
        let mut transport = MemoryTransport::new(TransportConfig::with_bind_addr(addr));

        assert_eq!(transport.local_addr(), addr);
        assert!(!transport.is_running());

        transport.start().unwrap();
        assert!(transport.is_running());

        transport.stop().unwrap();
        assert!(!transport.is_running());
    }
}
