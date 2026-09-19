# HPCA — Hybrid Peer Connectivity Architecture

HPCA is a research-driven project investigating a reusable, transport-independent architecture for heterogeneous peer connectivity.

## Core Conceptual Chain (Working Hypothesis)

```text
WHO?
Peer Identity
      ↓
WHERE?
Discovery / Addresses
      ↓
CAN I REACH IT?
Reachability / Connectivity
      ↓
HOW?
Transport
      ↓
WHICH PATH?
Path Selection
      ↓
COMMUNICATE
Session / Data
```

>Note: This sequence is a working hypothesis under active research, not a finalized API or implementation contract.

## Technology & Implementation Direction

HPCA investigates and references modern networking primitives and standards:

- **Language**: Rust (reference implementation, C ABI / FFI / cross-language bindings)
- **Serialization**: Protobuf / wire format specifications
- **Transports**: TCP, QUIC, UDP, WebRTC / WebTransport, Bluetooth adapters
- **Discovery**: mDNS / DNS-SD, peer exchange, rendezvous
- **Security**: TLS 1.3 / QUIC TLS, authenticated key exchange
- **State**: SQLite where persistent state or caching is required

> **Important**: The above list represents our research and implementation direction. It does not imply that all or any of these capabilities are currently implemented.

## Project Independence & Boundaries

- **HPCA is an independent research and reference implementation.**
- **HPCA is not a dependency of Aryntra Pravah.**
- **HPCA must not modify or block Pravah.**
- Validated HPCA findings may later inform Pravah only through deliberate, evaluated architectural decision records (ADRs).
