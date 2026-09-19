//! # HPCA — Hybrid Peer Connectivity Architecture
//!
//! Research & reference implementation for transport-independent peer connectivity.

pub mod core;
pub mod protocol;
pub mod session;
pub mod transport;

pub use crate::core::{AddressError, CandidateAddress, PeerId, PeerIdError, PEER_ID_LENGTH};
pub use crate::protocol::{
    ProbeError, ProbeRequest, ProbeResponse, ProbeState, ProbeStateMachine, NONCE_LENGTH,
    PROBE_FRAME_LENGTH,
};
pub use crate::session::Session;
pub use crate::transport::{Connection, LoopbackTransport, Transport, TransportError};
