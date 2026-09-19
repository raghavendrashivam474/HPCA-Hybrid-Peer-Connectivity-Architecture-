# ADR-0002: Peer Identity Cryptographic Primitive and Encoding

## Status
Accepted

## Context
Phase 0 architecture established that a `PeerId` is an immutable, location-independent 32-byte cryptographic identity decoupled from physical network addresses (`Peer ≠ PeerId`, `PeerId ≠ Address`). 

Sprint 1.1 requires selecting the exact cryptographic primitive for generation, byte representation, and human-readable encoding.

## Decision
1. **Primitive**: Ed25519 public key (verifying key) consisting of exactly 32 bytes (`[u8; 32]`).
2. **Generation**: `PeerId::random()` derives identity from an Ed25519 `SigningKey` generated using a cryptographically secure random number generator (`OsRng`).
3. **Serialization**: Deterministic raw 32-byte conversion (`to_bytes()` / `from_bytes()`).
4. **Human-readable Representation**: Lowercase hexadecimal string (64 characters). Hex is chosen for zero-dependency ambiguity, standard diagnostic readability, and strict 64-char length invariant.
5. **Crate Boundaries**: Identity primitives reside in `hpca::core::peer_id` (or `hpca::core::PeerId`).

## Consequences
- `PeerId` is guaranteed to be 32 bytes.
- Validated Ed25519 public keys can be directly used in future phases for mutual authentication / cryptographic handshakes without format migration.
- Dependencies added: `ed25519-dalek` (2.1), `rand` (0.8), `hex` (0.4).
