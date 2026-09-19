use crate::core::CandidateAddress;
use crate::core::PeerId;
use std::collections::HashMap;
use std::fmt;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

/// Errors encountered in transport operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// Port/Address already in use.
    AddressInUse(CandidateAddress),
    /// Target destination unreachable.
    Unreachable(CandidateAddress),
    /// General underlying channel or IO failure.
    ConnectionReset(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AddressInUse(addr) => write!(f, "address already in use: {addr}"),
            Self::Unreachable(addr) => write!(f, "unreachable address: {addr}"),
            Self::ConnectionReset(msg) => write!(f, "connection reset: {msg}"),
        }
    }
}

impl std::error::Error for TransportError {}

/// Represents a stateful, active connection between a local and remote address.
pub trait Connection: Send + Sync {
    /// Sends a raw frame of bytes across the connection.
    fn send(&self, data: &[u8]) -> Result<(), TransportError>;
    /// Receives a raw frame of bytes from the connection.
    fn recv(&self) -> Result<Vec<u8>, TransportError>;
    /// Returns the bound local address.
    fn local_addr(&self) -> CandidateAddress;
    /// Returns the target remote address.
    fn remote_addr(&self) -> CandidateAddress;
}

/// Pluggable physical or virtual transport driver.
pub trait Transport {
    /// Binds to a local address and prepares to accept connections.
    fn listen(&mut self, local: CandidateAddress) -> Result<(), TransportError>;
    /// Initiates an outbound connection to a target remote address.
    fn dial(
        &mut self,
        peer_id: PeerId,
        remote: CandidateAddress,
    ) -> Result<Box<dyn Connection>, TransportError>;
    /// Accepts an incoming connection (blocking).
    fn accept(&mut self) -> Result<Box<dyn Connection>, TransportError>;
}

/// Type alias for loopback handshake payloads: (source_addr, remote_tx, local_rx).
type LoopbackHandshake = (CandidateAddress, Sender<Vec<u8>>, Receiver<Vec<u8>>);
type SwitchRegistry = Mutex<HashMap<u64, Sender<LoopbackHandshake>>>;

// Global virtual switch routing messages between in-memory endpoints.
lazy_static::lazy_static! {
    static ref LOOPBACK_SWITCH: SwitchRegistry = Mutex::new(HashMap::new());
}

/// Concrete in-memory Loopback Connection.
pub struct LoopbackConnection {
    local: CandidateAddress,
    remote: CandidateAddress,
    tx: Sender<Vec<u8>>,
    rx: Mutex<Receiver<Vec<u8>>>,
}

impl Connection for LoopbackConnection {
    fn send(&self, data: &[u8]) -> Result<(), TransportError> {
        self.tx
            .send(data.to_vec())
            .map_err(|e| TransportError::ConnectionReset(e.to_string()))
    }

    fn recv(&self) -> Result<Vec<u8>, TransportError> {
        let rx = self.rx.lock().map_err(|_| {
            TransportError::ConnectionReset("failed to lock loopback receiver mutex".to_string())
        })?;
        rx.recv()
            .map_err(|e| TransportError::ConnectionReset(e.to_string()))
    }

    fn local_addr(&self) -> CandidateAddress {
        self.local
    }

    fn remote_addr(&self) -> CandidateAddress {
        self.remote
    }
}

/// Concrete virtual Loopback Transport implementation.
pub struct LoopbackTransport {
    local_addr: Option<CandidateAddress>,
    incoming_rx: Option<Receiver<LoopbackHandshake>>,
}

impl Default for LoopbackTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl LoopbackTransport {
    /// Creates a new unconfigured `LoopbackTransport`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            local_addr: None,
            incoming_rx: None,
        }
    }
}

impl Transport for LoopbackTransport {
    fn listen(&mut self, local: CandidateAddress) -> Result<(), TransportError> {
        let id = match local {
            CandidateAddress::Memory(id) => id,
            other => return Err(TransportError::Unreachable(other)),
        };

        let mut switch = LOOPBACK_SWITCH.lock().unwrap();
        if switch.contains_key(&id) {
            return Err(TransportError::AddressInUse(local));
        }

        let (tx, rx) = channel();
        switch.insert(id, tx);

        self.local_addr = Some(local);
        self.incoming_rx = Some(rx);
        Ok(())
    }

    fn dial(
        &mut self,
        _peer_id: PeerId,
        remote: CandidateAddress,
    ) -> Result<Box<dyn Connection>, TransportError> {
        let remote_id = match remote {
            CandidateAddress::Memory(id) => id,
            other => return Err(TransportError::Unreachable(other)),
        };

        let local = self.local_addr.unwrap_or(CandidateAddress::Memory(0));

        let target_tx = {
            let switch = LOOPBACK_SWITCH.lock().unwrap();
            switch.get(&remote_id).cloned()
        };

        let target_tx = target_tx.ok_or(TransportError::Unreachable(remote))?;

        // Channel pairs for bidirectional transfer
        let (local_tx, remote_rx) = channel();
        let (remote_tx, local_rx) = channel();

        // Notify listener of incoming connection
        target_tx
            .send((local, remote_tx, remote_rx))
            .map_err(|e| TransportError::ConnectionReset(e.to_string()))?;

        Ok(Box::new(LoopbackConnection {
            local,
            remote,
            tx: local_tx,
            rx: Mutex::new(local_rx),
        }))
    }

    fn accept(&mut self) -> Result<Box<dyn Connection>, TransportError> {
        let rx = self.incoming_rx.as_ref().ok_or_else(|| {
            TransportError::ConnectionReset("transport not listening".to_string())
        })?;

        let (remote_addr, tx, rx) = rx
            .recv()
            .map_err(|e| TransportError::ConnectionReset(e.to_string()))?;

        let local = self.local_addr.unwrap_or(CandidateAddress::Memory(0));

        Ok(Box::new(LoopbackConnection {
            local,
            remote: remote_addr,
            tx,
            rx: Mutex::new(rx),
        }))
    }
}

impl Drop for LoopbackTransport {
    fn drop(&mut self) {
        if let Some(CandidateAddress::Memory(id)) = self.local_addr {
            if let Ok(mut switch) = LOOPBACK_SWITCH.lock() {
                switch.remove(&id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopback_transport_bidirectional_flow() {
        let addr_a = CandidateAddress::Memory(100);
        let addr_b = CandidateAddress::Memory(200);
        let peer_b = PeerId::random();

        let mut transport_a = LoopbackTransport::new();
        transport_a.listen(addr_a).unwrap();

        let mut transport_b = LoopbackTransport::new();
        transport_b.listen(addr_b).unwrap();

        // Threaded acceptor
        let handle = std::thread::spawn(move || {
            let conn = transport_b.accept().unwrap();
            assert_eq!(conn.remote_addr(), addr_a);
            let msg = conn.recv().unwrap();
            assert_eq!(msg, b"ping");
            conn.send(b"pong").unwrap();
        });

        let conn_a = transport_a.dial(peer_b, addr_b).unwrap();
        conn_a.send(b"ping").unwrap();
        let resp = conn_a.recv().unwrap();
        assert_eq!(resp, b"pong");

        handle.join().unwrap();
    }
}
