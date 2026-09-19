use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::fmt;
use std::str::FromStr;

/// Expected byte length of a cryptographic PeerId.
pub const PEER_ID_LENGTH: usize = 32;

/// Errors that can occur when parsing or constructing a `PeerId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerIdError {
    /// Provided byte slice does not match the required 32-byte length.
    InvalidLength { expected: usize, actual: usize },
    /// String is not a valid hex-encoded representation.
    InvalidHex(String),
}

impl fmt::Display for PeerIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "invalid peer ID length: expected {expected} bytes, got {actual}"
                )
            }
            Self::InvalidHex(err) => write!(f, "invalid hex string: {err}"),
        }
    }
}

impl std::error::Error for PeerIdError {}

/// Immutable, 32-byte cryptographic peer identity.
///
/// Decoupled from physical transport addresses (`Peer ≠ PeerId`, `PeerId ≠ Address`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerId([u8; PEER_ID_LENGTH]);

impl PeerId {
    /// Generates a new cryptographic `PeerId` from an Ed25519 keypair using OS entropy.
    #[must_use]
    pub fn random() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self(verifying_key.to_bytes())
    }

    /// Constructs a `PeerId` directly from a 32-byte array.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; PEER_ID_LENGTH]) -> Self {
        Self(bytes)
    }

    /// Returns the raw 32-byte representation.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; PEER_ID_LENGTH] {
        self.0
    }

    /// Returns a reference to the inner 32-byte array.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; PEER_ID_LENGTH] {
        &self.0
    }

    /// Formats the identity as a lowercase 64-character hexadecimal string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parses a `PeerId` from a 64-character hexadecimal string.
    pub fn from_hex(s: &str) -> Result<Self, PeerIdError> {
        if s.len() != PEER_ID_LENGTH * 2 {
            return Err(PeerIdError::InvalidLength {
                expected: PEER_ID_LENGTH * 2,
                actual: s.len(),
            });
        }
        let decoded = hex::decode(s).map_err(|e| PeerIdError::InvalidHex(e.to_string()))?;
        let mut bytes = [0u8; PEER_ID_LENGTH];
        bytes.copy_from_slice(&decoded);
        Ok(Self(bytes))
    }
}

impl AsRef<[u8]> for PeerId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8; PEER_ID_LENGTH]> for PeerId {
    fn as_ref(&self) -> &[u8; PEER_ID_LENGTH] {
        &self.0
    }
}

impl TryFrom<&[u8]> for PeerId {
    type Error = PeerIdError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != PEER_ID_LENGTH {
            return Err(PeerIdError::InvalidLength {
                expected: PEER_ID_LENGTH,
                actual: slice.len(),
            });
        }
        let mut bytes = [0u8; PEER_ID_LENGTH];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId({})", self.to_hex())
    }
}

impl FromStr for PeerId {
    type Err = PeerIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id_size_is_exactly_32_bytes() {
        let peer_id = PeerId::random();
        assert_eq!(std::mem::size_of::<PeerId>(), 32);
        assert_eq!(peer_id.to_bytes().len(), 32);
        assert_eq!(peer_id.as_bytes().len(), 32);

        let slice: &[u8] = peer_id.as_ref();
        assert_eq!(slice.len(), 32);

        let array_ref: &[u8; 32] = peer_id.as_ref();
        assert_eq!(array_ref.len(), 32);
    }

    #[test]
    fn test_peer_id_random_uniqueness() {
        let a = PeerId::random();
        let b = PeerId::random();
        assert_ne!(a, b);
        assert_ne!(a.to_bytes(), b.to_bytes());
    }

    #[test]
    fn test_peer_id_byte_roundtrip() {
        let original = PeerId::random();
        let raw = original.to_bytes();
        let reconstructed = PeerId::from_bytes(raw);
        assert_eq!(original, reconstructed);

        let from_slice = PeerId::try_from(raw.as_slice()).expect("slice should parse");
        assert_eq!(original, from_slice);
    }

    #[test]
    fn test_peer_id_hex_roundtrip() {
        let original = PeerId::random();
        let hex_str = original.to_hex();
        assert_eq!(hex_str.len(), 64);

        let from_hex_fn = PeerId::from_hex(&hex_str).expect("from_hex should parse");
        assert_eq!(original, from_hex_fn);

        let from_str_parsed: PeerId = hex_str.parse().expect("FromStr should parse");
        assert_eq!(original, from_str_parsed);
    }

    #[test]
    fn test_peer_id_invalid_byte_slice_rejected() {
        let short = [0u8; 31];
        let long = [0u8; 33];
        assert_eq!(
            PeerId::try_from(short.as_slice()),
            Err(PeerIdError::InvalidLength {
                expected: 32,
                actual: 31
            })
        );
        assert_eq!(
            PeerId::try_from(long.as_slice()),
            Err(PeerIdError::InvalidLength {
                expected: 32,
                actual: 33
            })
        );
    }

    #[test]
    fn test_peer_id_invalid_hex_rejected() {
        let short_hex = "0123456789abcdef";
        assert!(PeerId::from_hex(short_hex).is_err());

        let invalid_chars = "zz".repeat(32);
        assert!(PeerId::from_hex(&invalid_chars).is_err());
    }

    #[test]
    fn test_peer_id_display_and_debug_format() {
        let peer_id = PeerId::from_bytes([0xaa; 32]);
        let expected_hex = "aa".repeat(32);
        assert_eq!(format!("{peer_id}"), expected_hex);
        assert_eq!(format!("{peer_id:?}"), format!("PeerId({expected_hex})"));
    }
}
