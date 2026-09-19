# ADR-0003: Strongly Typed Candidate Addressing

## Status
Accepted

## Context
Phase 0 vocabulary defined `Address ≠ Endpoint` and `PeerId ≠ Address`. A peer may have zero or more candidate addresses across distinct physical/virtual networks. HPCA rejected generic dynamic string multi-addresses in favor of a strongly-typed enum.

## Decision
1. **Enum Structure**: `CandidateAddress` encapsulates:
   - `Ipv4(SocketAddrV4)`
   - `Ipv6(SocketAddrV6)`
   - `Memory(u64)` (used for in-memory and loopback transport drivers)
   - `Bluetooth([u8; 6])` (feature-gated behind `bluetooth`)
2. **Binary Protocol Tagging**:
   - `0x01`: IPv4 (`[u8; 4]` + `[u8; 2]`) -> 7 bytes
   - `0x02`: IPv6 (`[u8; 16]` + `[u8; 2]`) -> 19 bytes
   - `0x03`: Memory ID (`[u8; 8]`) -> 9 bytes
   - `0x04`: Bluetooth MAC (`[u8; 6]`) -> 7 bytes
3. **Decoupling**: A `CandidateAddress` contains no active socket descriptors, no connection state, and no reachability guarantee.

## Consequences
- Clean compile-time type safety over candidate addresses.
- Direct bidirectional conversion to/from standard `SocketAddr`.
- Zero runtime overhead from string multi-address parsers.
