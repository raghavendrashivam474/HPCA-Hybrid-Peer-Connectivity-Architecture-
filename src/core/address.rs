use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;

/// Error types encountered when handling or parsing candidate addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    /// Provided binary slice is empty or incomplete.
    UnexpectedEndOfBuffer,
    /// Unknown address type discriminator tag.
    UnknownAddressType(u8),
    /// Invalid textual address format.
    InvalidTextRepresentation(String),
    /// Invalid byte length for candidate address type.
    InvalidLength { expected: usize, actual: usize },
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEndOfBuffer => write!(f, "unexpected end of buffer"),
            Self::UnknownAddressType(tag) => write!(f, "unknown address type tag: {tag:#x}"),
            Self::InvalidTextRepresentation(err) => {
                write!(f, "invalid textual address representation: {err}")
            }
            Self::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "invalid address length: expected {expected}, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for AddressError {}

/// Binary protocol tag constants
const TAG_IPV4: u8 = 0x01;
const TAG_IPV6: u8 = 0x02;
const TAG_MEMORY: u8 = 0x03;
#[cfg(feature = "bluetooth")]
const TAG_BLUETOOTH: u8 = 0x04;

/// Strongly-typed representation of a candidate address.
///
/// An address indicates *where* a peer might be located. It does NOT imply
/// that the peer is currently reachable or connected.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum CandidateAddress {
    /// Standard IPv4 socket address (IP + port).
    Ipv4(SocketAddrV4),
    /// Standard IPv6 socket address (IP + port).
    Ipv6(SocketAddrV6),
    /// In-memory logical transport address (identifier).
    Memory(u64),
    /// Bluetooth MAC address (6 bytes), feature-gated.
    #[cfg(feature = "bluetooth")]
    Bluetooth([u8; 6]),
}

impl CandidateAddress {
    /// Creates an IPv4 candidate address.
    #[must_use]
    pub const fn from_ipv4(ip: Ipv4Addr, port: u16) -> Self {
        Self::Ipv4(SocketAddrV4::new(ip, port))
    }

    /// Creates an IPv6 candidate address.
    #[must_use]
    pub const fn from_ipv6(ip: Ipv6Addr, port: u16) -> Self {
        Self::Ipv6(SocketAddrV6::new(ip, port, 0, 0))
    }

    /// Creates a candidate address from a standard `SocketAddr`.
    #[must_use]
    pub const fn from_socket_addr(addr: SocketAddr) -> Self {
        match addr {
            SocketAddr::V4(v4) => Self::Ipv4(v4),
            SocketAddr::V6(v6) => Self::Ipv6(v6),
        }
    }

    /// Returns the corresponding `SocketAddr` if this is an IPv4 or IPv6 candidate.
    #[must_use]
    pub const fn as_socket_addr(&self) -> Option<SocketAddr> {
        match self {
            Self::Ipv4(v4) => Some(SocketAddr::V4(*v4)),
            Self::Ipv6(v6) => Some(SocketAddr::V6(*v6)),
            Self::Memory(_) => None,
            #[cfg(feature = "bluetooth")]
            Self::Bluetooth(_) => None,
        }
    }

    /// Encodes the candidate address to a binary byte buffer.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ipv4(v4) => {
                let mut buf = Vec::with_capacity(7);
                buf.push(TAG_IPV4);
                buf.extend_from_slice(&v4.ip().octets());
                buf.extend_from_slice(&v4.port().to_be_bytes());
                buf
            }
            Self::Ipv6(v6) => {
                let mut buf = Vec::with_capacity(19);
                buf.push(TAG_IPV6);
                buf.extend_from_slice(&v6.ip().octets());
                buf.extend_from_slice(&v6.port().to_be_bytes());
                buf
            }
            Self::Memory(id) => {
                let mut buf = Vec::with_capacity(9);
                buf.push(TAG_MEMORY);
                buf.extend_from_slice(&id.to_be_bytes());
                buf
            }
            #[cfg(feature = "bluetooth")]
            Self::Bluetooth(mac) => {
                let mut buf = Vec::with_capacity(7);
                buf.push(TAG_BLUETOOTH);
                buf.extend_from_slice(mac);
                buf
            }
        }
    }

    /// Decodes a candidate address from a binary slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AddressError> {
        if bytes.is_empty() {
            return Err(AddressError::UnexpectedEndOfBuffer);
        }

        let tag = bytes[0];
        let payload = &bytes[1..];

        match tag {
            TAG_IPV4 => {
                if payload.len() != 6 {
                    return Err(AddressError::InvalidLength {
                        expected: 6,
                        actual: payload.len(),
                    });
                }
                let mut ip_bytes = [0u8; 4];
                ip_bytes.copy_from_slice(&payload[0..4]);
                let port = u16::from_be_bytes([payload[4], payload[5]]);
                Ok(Self::from_ipv4(Ipv4Addr::from(ip_bytes), port))
            }
            TAG_IPV6 => {
                if payload.len() != 18 {
                    return Err(AddressError::InvalidLength {
                        expected: 18,
                        actual: payload.len(),
                    });
                }
                let mut ip_bytes = [0u8; 16];
                ip_bytes.copy_from_slice(&payload[0..16]);
                let port = u16::from_be_bytes([payload[16], payload[17]]);
                Ok(Self::from_ipv6(Ipv6Addr::from(ip_bytes), port))
            }
            TAG_MEMORY => {
                if payload.len() != 8 {
                    return Err(AddressError::InvalidLength {
                        expected: 8,
                        actual: payload.len(),
                    });
                }
                let mut id_bytes = [0u8; 8];
                id_bytes.copy_from_slice(&payload[0..8]);
                let id = u64::from_be_bytes(id_bytes);
                Ok(Self::Memory(id))
            }
            #[cfg(feature = "bluetooth")]
            TAG_BLUETOOTH => {
                if payload.len() != 6 {
                    return Err(AddressError::InvalidLength {
                        expected: 6,
                        actual: payload.len(),
                    });
                }
                let mut mac = [0u8; 6];
                mac.copy_from_slice(&payload[0..6]);
                Ok(Self::Bluetooth(mac))
            }
            other => Err(AddressError::UnknownAddressType(other)),
        }
    }
}

impl From<SocketAddr> for CandidateAddress {
    fn from(addr: SocketAddr) -> Self {
        Self::from_socket_addr(addr)
    }
}

impl From<SocketAddrV4> for CandidateAddress {
    fn from(addr: SocketAddrV4) -> Self {
        Self::Ipv4(addr)
    }
}

impl From<SocketAddrV6> for CandidateAddress {
    fn from(addr: SocketAddrV6) -> Self {
        Self::Ipv6(addr)
    }
}

impl fmt::Display for CandidateAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ipv4(v4) => write!(f, "ipv4:{v4}"),
            Self::Ipv6(v6) => write!(f, "ipv6:{v6}"),
            Self::Memory(id) => write!(f, "memory:{id}"),
            #[cfg(feature = "bluetooth")]
            Self::Bluetooth(mac) => {
                write!(
                    f,
                    "bt:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                )
            }
        }
    }
}

impl FromStr for CandidateAddress {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(rest) = s.strip_prefix("ipv4:") {
            let addr: SocketAddrV4 = rest
                .parse()
                .map_err(|e| AddressError::InvalidTextRepresentation(format!("{e}")))?;
            return Ok(Self::Ipv4(addr));
        }

        if let Some(rest) = s.strip_prefix("ipv6:") {
            let addr: SocketAddrV6 = rest
                .parse()
                .map_err(|e| AddressError::InvalidTextRepresentation(format!("{e}")))?;
            return Ok(Self::Ipv6(addr));
        }

        if let Some(rest) = s.strip_prefix("memory:") {
            let id: u64 = rest
                .parse()
                .map_err(|e| AddressError::InvalidTextRepresentation(format!("{e}")))?;
            return Ok(Self::Memory(id));
        }

        // Direct SocketAddr fallback (e.g. "127.0.0.1:8080" or "[::1]:8080")
        if let Ok(socket_addr) = s.parse::<SocketAddr>() {
            return Ok(Self::from_socket_addr(socket_addr));
        }

        Err(AddressError::InvalidTextRepresentation(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_candidate_roundtrip() {
        let ip = Ipv4Addr::new(192, 168, 1, 100);
        let cand = CandidateAddress::from_ipv4(ip, 9000);

        // Binary roundtrip
        let bytes = cand.to_bytes();
        assert_eq!(bytes.len(), 7);
        assert_eq!(bytes[0], TAG_IPV4);
        let parsed = CandidateAddress::from_bytes(&bytes).expect("should parse");
        assert_eq!(cand, parsed);

        // Textual roundtrip
        let s = cand.to_string();
        assert_eq!(s, "ipv4:192.168.1.100:9000");
        let parsed_str: CandidateAddress = s.parse().expect("should parse str");
        assert_eq!(cand, parsed_str);

        // SocketAddr fallback parsing
        let plain_str: CandidateAddress = "192.168.1.100:9000".parse().expect("plain parse");
        assert_eq!(cand, plain_str);
    }

    #[test]
    fn test_ipv6_candidate_roundtrip() {
        let ip = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
        let cand = CandidateAddress::from_ipv6(ip, 8080);

        // Binary roundtrip
        let bytes = cand.to_bytes();
        assert_eq!(bytes.len(), 19);
        assert_eq!(bytes[0], TAG_IPV6);
        let parsed = CandidateAddress::from_bytes(&bytes).expect("should parse");
        assert_eq!(cand, parsed);

        // Textual roundtrip
        let s = cand.to_string();
        assert_eq!(s, "ipv6:[2001:db8::1]:8080");
        let parsed_str: CandidateAddress = s.parse().expect("should parse str");
        assert_eq!(cand, parsed_str);

        // SocketAddr fallback parsing
        let plain_str: CandidateAddress = "[2001:db8::1]:8080".parse().expect("plain parse");
        assert_eq!(cand, plain_str);
    }

    #[test]
    fn test_memory_candidate_roundtrip() {
        let cand = CandidateAddress::Memory(0x0102_0304_0506_0708);

        // Binary roundtrip
        let bytes = cand.to_bytes();
        assert_eq!(bytes.len(), 9);
        assert_eq!(bytes[0], TAG_MEMORY);
        let parsed = CandidateAddress::from_bytes(&bytes).expect("should parse");
        assert_eq!(cand, parsed);

        // Textual roundtrip
        let s = cand.to_string();
        assert_eq!(s, "memory:72623859790382856");
        let parsed_str: CandidateAddress = s.parse().expect("should parse str");
        assert_eq!(cand, parsed_str);
    }

    #[test]
    fn test_socket_addr_conversion() {
        let socket: SocketAddr = "10.0.0.1:443".parse().unwrap();
        let cand = CandidateAddress::from(socket);
        assert_eq!(cand.as_socket_addr(), Some(socket));

        let mem_cand = CandidateAddress::Memory(1);
        assert_eq!(mem_cand.as_socket_addr(), None);
    }

    #[test]
    fn test_invalid_binary_buffers() {
        // Empty
        assert_eq!(
            CandidateAddress::from_bytes(&[]),
            Err(AddressError::UnexpectedEndOfBuffer)
        );

        // Unknown tag
        assert_eq!(
            CandidateAddress::from_bytes(&[0xFF, 1, 2, 3]),
            Err(AddressError::UnknownAddressType(0xFF))
        );

        // Short IPv4
        assert_eq!(
            CandidateAddress::from_bytes(&[TAG_IPV4, 127, 0, 0]),
            Err(AddressError::InvalidLength {
                expected: 6,
                actual: 3
            })
        );
    }

    #[test]
    fn test_invalid_text_parsing() {
        assert!(CandidateAddress::from_str("invalid:address:xyz").is_err());
        assert!(CandidateAddress::from_str("ipv4:not_an_ip:80").is_err());
        assert!(CandidateAddress::from_str("memory:not_a_number").is_err());
    }
}
