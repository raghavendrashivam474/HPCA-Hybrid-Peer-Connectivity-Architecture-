# ADR-0005: Transport Abstraction, Loopback Driver, and Session Layers

## Status
Accepted

## Context
Phase 0 vocabulary demands strict non-equivalence boundaries: `Connection ≠ Session` and `Transport ≠ Path`. To validate this architecture, Phase 1 requires an abstract transport trait, a concrete in-memory loopback driver, and a lightweight logical session context.

## Decision
1. **Abstractions**:
   - `Connection` (trait): Synchronous/blocking raw stream reader and writer.
   - `Transport` (trait): Abstraction for resource binding (`listen`) and connection initiation (`dial`).
2. **In-Memory Loopback Transport**:
   - Uses `LoopbackSwitch`, a thread-safe registry backed by a `std::sync::Mutex` of `std::sync::mpsc::Sender` channels.
   - Operates over `CandidateAddress::Memory(id)`.
3. **Session Layer**:
   - Decoupled from physical connections.
   - Holds logical state: `local_peer_id`, `remote_peer_id`, and `session_id`.
   - Binds to an active `Connection` and implements application frame exchange.

## Consequences
- No network socket or OS dependencies required for architectural testing.
- Reliable, fast, and concurrent loopback implementation for multi-peer modeling.
- Clear decoupling of physical path lifecycle from application session lifecycle.
