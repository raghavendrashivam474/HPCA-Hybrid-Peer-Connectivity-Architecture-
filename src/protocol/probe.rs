use crate::core::PeerId;
use rand::RngCore;
use std::fmt;
use std::time::{Duration, Instant};

/// Length of the challenge nonce in bytes.
pub const NONCE_LENGTH: usize = 16;
/// Total wire byte length for ProbeRequest and ProbeResponse.
pub const PROBE_FRAME_LENGTH: usize = 1 + NONCE_LENGTH + 32;

const TAG_PROBE_REQUEST: u8 = 0x01;
const TAG_PROBE_RESPONSE: u8 = 0x02;

/// Errors arising during probe frame parsing or validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeError {
    /// Buffer does not match required frame length.
    InvalidFrameLength { expected: usize, actual: usize },
    /// Unknown frame tag identifier.
    UnknownTag(u8),
    /// Challenge nonce in response did not match the request.
    ChallengeMismatch,
    /// Responder identity did not match expected target peer.
    PeerIdMismatch { expected: PeerId, actual: PeerId },
    /// State machine was in an invalid state for the requested operation.
    InvalidStateTransition(String),
}

impl fmt::Display for ProbeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFrameLength { expected, actual } => {
                write!(
                    f,
                    "invalid probe frame length: expected {expected}, got {actual}"
                )
            }
            Self::UnknownTag(tag) => write!(f, "unknown probe tag: {tag:#x}"),
            Self::ChallengeMismatch => write!(f, "challenge nonce mismatch in probe response"),
            Self::PeerIdMismatch { expected, actual } => {
                write!(
                    f,
                    "responder peer id mismatch: expected {expected}, got {actual}"
                )
            }
            Self::InvalidStateTransition(msg) => {
                write!(f, "invalid probe state transition: {msg}")
            }
        }
    }
}

impl std::error::Error for ProbeError {}

/// Binary challenge probe request sent to verify reachability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeRequest {
    pub nonce: [u8; NONCE_LENGTH],
    pub target_peer_id: PeerId,
}

impl ProbeRequest {
    /// Generates a new `ProbeRequest` with a cryptographically secure random nonce.
    #[must_use]
    pub fn new(target_peer_id: PeerId) -> Self {
        let mut nonce = [0u8; NONCE_LENGTH];
        rand::thread_rng().fill_bytes(&mut nonce);
        Self {
            nonce,
            target_peer_id,
        }
    }

    /// Serializes the probe request to wire format.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; PROBE_FRAME_LENGTH] {
        let mut buf = [0u8; PROBE_FRAME_LENGTH];
        buf[0] = TAG_PROBE_REQUEST;
        buf[1..1 + NONCE_LENGTH].copy_from_slice(&self.nonce);
        buf[1 + NONCE_LENGTH..].copy_from_slice(self.target_peer_id.as_bytes());
        buf
    }

    /// Deserializes a probe request from wire bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProbeError> {
        if bytes.len() != PROBE_FRAME_LENGTH {
            return Err(ProbeError::InvalidFrameLength {
                expected: PROBE_FRAME_LENGTH,
                actual: bytes.len(),
            });
        }
        if bytes[0] != TAG_PROBE_REQUEST {
            return Err(ProbeError::UnknownTag(bytes[0]));
        }

        let mut nonce = [0u8; NONCE_LENGTH];
        nonce.copy_from_slice(&bytes[1..1 + NONCE_LENGTH]);

        let mut peer_bytes = [0u8; 32];
        peer_bytes.copy_from_slice(&bytes[1 + NONCE_LENGTH..]);
        let target_peer_id = PeerId::from_bytes(peer_bytes);

        Ok(Self {
            nonce,
            target_peer_id,
        })
    }
}

/// Binary challenge probe response validating reachability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeResponse {
    pub nonce: [u8; NONCE_LENGTH],
    pub responder_peer_id: PeerId,
}

impl ProbeResponse {
    /// Creates a corresponding response for an incoming `ProbeRequest`.
    #[must_use]
    pub fn from_request(request: &ProbeRequest, responder_peer_id: PeerId) -> Self {
        Self {
            nonce: request.nonce,
            responder_peer_id,
        }
    }

    /// Serializes the probe response to wire format.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; PROBE_FRAME_LENGTH] {
        let mut buf = [0u8; PROBE_FRAME_LENGTH];
        buf[0] = TAG_PROBE_RESPONSE;
        buf[1..1 + NONCE_LENGTH].copy_from_slice(&self.nonce);
        buf[1 + NONCE_LENGTH..].copy_from_slice(self.responder_peer_id.as_bytes());
        buf
    }

    /// Deserializes a probe response from wire bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProbeError> {
        if bytes.len() != PROBE_FRAME_LENGTH {
            return Err(ProbeError::InvalidFrameLength {
                expected: PROBE_FRAME_LENGTH,
                actual: bytes.len(),
            });
        }
        if bytes[0] != TAG_PROBE_RESPONSE {
            return Err(ProbeError::UnknownTag(bytes[0]));
        }

        let mut nonce = [0u8; NONCE_LENGTH];
        nonce.copy_from_slice(&bytes[1..1 + NONCE_LENGTH]);

        let mut peer_bytes = [0u8; 32];
        peer_bytes.copy_from_slice(&bytes[1 + NONCE_LENGTH..]);
        let responder_peer_id = PeerId::from_bytes(peer_bytes);

        Ok(Self {
            nonce,
            responder_peer_id,
        })
    }
}

/// Lifecycle states of a reachability probe operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeState {
    /// Probe created with target peer and challenge.
    Created,
    /// Probe request sent; awaiting response.
    AwaitingResponse,
    /// Probe challenge validated successfully with measured RTT.
    Succeeded(Duration),
    /// Probe failed (challenge mismatch, identity mismatch, or parse error).
    Failed(ProbeError),
}

/// Explicit lifecycle state machine driving a reachability probe.
#[derive(Debug)]
pub struct ProbeStateMachine {
    request: ProbeRequest,
    state: ProbeState,
    sent_time: Option<Instant>,
}

impl ProbeStateMachine {
    /// Initiates a new probe state machine targeted at `target_peer_id`.
    #[must_use]
    pub fn new(target_peer_id: PeerId) -> Self {
        Self {
            request: ProbeRequest::new(target_peer_id),
            state: ProbeState::Created,
            sent_time: None,
        }
    }

    /// Returns the underlying probe request.
    #[must_use]
    pub const fn request(&self) -> &ProbeRequest {
        &self.request
    }

    /// Returns the current state of the probe.
    #[must_use]
    pub const fn state(&self) -> &ProbeState {
        &self.state
    }

    /// Transitions state from `Created` to `AwaitingResponse` upon frame transmission.
    pub fn on_sent(&mut self) -> Result<[u8; PROBE_FRAME_LENGTH], ProbeError> {
        if self.state != ProbeState::Created {
            return Err(ProbeError::InvalidStateTransition(
                "probe can only be sent from Created state".to_string(),
            ));
        }
        self.state = ProbeState::AwaitingResponse;
        self.sent_time = Some(Instant::now());
        Ok(self.request.to_bytes())
    }

    /// Processes an incoming raw response frame and transitions to `Succeeded` or `Failed`.
    pub fn handle_response(&mut self, response_bytes: &[u8]) -> Result<Duration, ProbeError> {
        if self.state != ProbeState::AwaitingResponse {
            return Err(ProbeError::InvalidStateTransition(
                "cannot handle response when not in AwaitingResponse state".to_string(),
            ));
        }

        let response = match ProbeResponse::from_bytes(response_bytes) {
            Ok(resp) => resp,
            Err(err) => {
                self.state = ProbeState::Failed(err.clone());
                return Err(err);
            }
        };

        if response.nonce != self.request.nonce {
            let err = ProbeError::ChallengeMismatch;
            self.state = ProbeState::Failed(err.clone());
            return Err(err);
        }

        if response.responder_peer_id != self.request.target_peer_id {
            let err = ProbeError::PeerIdMismatch {
                expected: self.request.target_peer_id,
                actual: response.responder_peer_id,
            };
            self.state = ProbeState::Failed(err.clone());
            return Err(err);
        }

        let rtt = self
            .sent_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::from_millis(0));
        self.state = ProbeState::Succeeded(rtt);
        Ok(rtt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_request_and_response_roundtrip() {
        let peer_target = PeerId::random();

        let req = ProbeRequest::new(peer_target);
        let req_bytes = req.to_bytes();
        let parsed_req = ProbeRequest::from_bytes(&req_bytes).expect("parse req");
        assert_eq!(req, parsed_req);

        let resp = ProbeResponse::from_request(&req, peer_target);
        let resp_bytes = resp.to_bytes();
        let parsed_resp = ProbeResponse::from_bytes(&resp_bytes).expect("parse resp");
        assert_eq!(resp, parsed_resp);
    }

    #[test]
    fn test_successful_probe_state_machine_flow() {
        let peer_target = PeerId::random();
        let mut sm = ProbeStateMachine::new(peer_target);
        assert_eq!(*sm.state(), ProbeState::Created);

        let req_bytes = sm.on_sent().expect("on_sent");
        assert_eq!(*sm.state(), ProbeState::AwaitingResponse);

        let req = ProbeRequest::from_bytes(&req_bytes).expect("parse request");
        let resp = ProbeResponse::from_request(&req, peer_target);

        let rtt = sm
            .handle_response(&resp.to_bytes())
            .expect("response verified");
        match sm.state() {
            ProbeState::Succeeded(recorded_rtt) => assert_eq!(*recorded_rtt, rtt),
            other => panic!("expected Succeeded, got {other:?}"),
        }
    }

    #[test]
    fn test_probe_mismatched_challenge_rejected() {
        let peer_target = PeerId::random();
        let mut sm = ProbeStateMachine::new(peer_target);
        sm.on_sent().expect("on_sent");

        // Responder responds with different nonce
        let wrong_resp = ProbeResponse {
            nonce: [0xFF; NONCE_LENGTH],
            responder_peer_id: peer_target,
        };

        let res = sm.handle_response(&wrong_resp.to_bytes());
        assert_eq!(res, Err(ProbeError::ChallengeMismatch));
        assert_eq!(
            *sm.state(),
            ProbeState::Failed(ProbeError::ChallengeMismatch)
        );
    }

    #[test]
    fn test_probe_mismatched_responder_peer_id_rejected() {
        let peer_target = PeerId::random();
        let impostor = PeerId::random();

        let mut sm = ProbeStateMachine::new(peer_target);
        let req_bytes = sm.on_sent().expect("on_sent");
        let req = ProbeRequest::from_bytes(&req_bytes).expect("parse");

        // Impostor responds with correct nonce but its own identity
        let impostor_resp = ProbeResponse::from_request(&req, impostor);

        let res = sm.handle_response(&impostor_resp.to_bytes());
        let expected_err = ProbeError::PeerIdMismatch {
            expected: peer_target,
            actual: impostor,
        };
        assert_eq!(res, Err(expected_err.clone()));
        assert_eq!(*sm.state(), ProbeState::Failed(expected_err));
    }

    #[test]
    fn test_invalid_probe_frames_rejected() {
        assert_eq!(
            ProbeRequest::from_bytes(&[1, 2, 3]),
            Err(ProbeError::InvalidFrameLength {
                expected: PROBE_FRAME_LENGTH,
                actual: 3
            })
        );
        assert_eq!(
            ProbeResponse::from_bytes(&[1, 2, 3]),
            Err(ProbeError::InvalidFrameLength {
                expected: PROBE_FRAME_LENGTH,
                actual: 3
            })
        );

        let mut bad_tag_bytes = [0u8; PROBE_FRAME_LENGTH];
        bad_tag_bytes[0] = 0x99;
        assert_eq!(
            ProbeRequest::from_bytes(&bad_tag_bytes),
            Err(ProbeError::UnknownTag(0x99))
        );
        assert_eq!(
            ProbeResponse::from_bytes(&bad_tag_bytes),
            Err(ProbeError::UnknownTag(0x99))
        );
    }
}
