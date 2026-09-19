use crate::core::PeerId;
use crate::transport::{Connection, TransportError};
use std::sync::atomic::{AtomicU64, Ordering};

static SESSION_ID_GEN: AtomicU64 = AtomicU64::new(1);

/// logical communication context that persists above raw physical connections.
pub struct Session {
    session_id: u64,
    local_peer: PeerId,
    remote_peer: PeerId,
    connection: Box<dyn Connection>,
}

impl Session {
    /// Binds an active connection into a stable conversational session.
    pub fn bind(local_peer: PeerId, remote_peer: PeerId, connection: Box<dyn Connection>) -> Self {
        let session_id = SESSION_ID_GEN.fetch_add(1, Ordering::SeqCst);
        Self {
            session_id,
            local_peer,
            remote_peer,
            connection,
        }
    }

    /// Returns the unique conversational identifier for this session.
    #[must_use]
    pub const fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Returns the local peer identity.
    #[must_use]
    pub const fn local_peer(&self) -> &PeerId {
        &self.local_peer
    }

    /// Returns the remote peer identity.
    #[must_use]
    pub const fn remote_peer(&self) -> &PeerId {
        &self.remote_peer
    }

    /// Sends application-level frames over the active physical bound link.
    pub fn send_frame(&self, frame: &[u8]) -> Result<(), TransportError> {
        self.connection.send(frame)
    }

    /// Receives application-level frames over the active physical bound link.
    pub fn recv_frame(&self) -> Result<Vec<u8>, TransportError> {
        self.connection.recv()
    }
}
