# HPCA Minimal Architecture Candidate & Gate Evaluation

## Phase 0 Gate Objective
Determine the smallest, most coherent architecture candidate for HPCA before writing Phase 1 connectivity code. The goal is to maximize decoupling and flexibility while ruthlessly rejecting speculative abstractions.

---

## Candidate Architecture Options Evaluated

### Option A: Monolithic Connectivity
```text
Peer → Transport → Connection / 
```

- **Description**: Direct mapping of peer identities to single transport sockets (similar to classic 
   client-server TCP or basic ZeroMQ).
- **Pros**: Minimal code, simple to understand, very low layer overhead.
- **Cons**: Cannot handle multi-interface roaming (Wi-Fi ↔ Cellular), cannot evaluate multiple candidate 
   addresses in parallel, tightly couples physical transport state to peer identity.
- **Verdict**: REJECTED. Fails the core requirement of heterogeneous multi-transport peer connectivity.

### Option B: Deeply Layered Dynamic Stack

```text
Peer → Discovery Protocol → Rendezvous → Reachability Engine → Multiaddr Resolver → Transport Swarm → Protocol Multiplexer → Substream Session
```

- **Description**: Highly modular, fully composable stack where every protocol negotiation and address resolution 
   is a distinct pluggable dynamic layer (similar to full libp2p stack).
- **Pros**: Maximum theoretical flexibility.
- **Cons**: Massive cognitive complexity, excessive handshake latency (multiple round-trips for protocol discovery),
   high memory overhead, hard to debug and optimize.
- **Verdict**: REJECTED. Violates the minimality principle. Introduces abstractions before proven necessary.

### Option C: The Minimal Hybrid Path Model (Candidate Selected)

```text
Peer Identity (PeerId: 32-byte public key)
      ↓
Discovery & Candidate Gathering (Typed Candidate Addresses)
      ↓
Reachability Probing (Lightweight binary probe validation)
      ↓
Pluggable Transport Layer (TCP, QUIC, Bluetooth drivers)
      ↓
Session State (ConnectionId & session continuity across path changes)
```

- **Description**: Decouples identity from physical addresses, uses strongly-typed candidate addresses, 
   isolates transport drivers behind a unified interface, and maintains logical session continuity across path switches without heavy dynamic protocol negotiation.
- **Pros**: Minimal abstraction overhead, supports multi-path roaming and fallback, language-agnostic, easily 
   testable in unit/integration environments.
- **Verdict**: ACCEPTED as the minimal candidate for Phase 1 prototyping.

---

## Minimal Architecture Layer Responsibilities (Option C)

```text
┌────────────────────────────────────────────────────────┐
│ 5. Session Layer                                       │
│    • Preserves conversational state via ConnectionId   │
│    • Handles roaming migration across path changes     │
├────────────────────────────────────────────────────────┤
│ 4. Transport Abstraction                               │
│    • Uniform frame transmission interface              │
│    • Concrete drivers: TCP (baseline), QUIC, BLE       │
├────────────────────────────────────────────────────────┤
│ 3. Reachability & Path Selection                       │
│    • Binary ping/pong reachability probes              │
│    • Validates candidate paths before session bind     │
├────────────────────────────────────────────────────────┤
│ 2. Candidate Discovery & Addressing                    │
│    • Strongly-typed CandidateAddress enum              │
│    • Gather local interfaces and discovered remotes    │
├────────────────────────────────────────────────────────┤
│ 1. Peer Identity                                       │
│    • Cryptographic PeerId (32-byte public key)         │
│    • Immutable, location-independent identity          │
└────────────────────────────────────────────────────────┘
```

## Minimality Test Justification

- **Peer Identity**: Kept because without identity decoupling, peer roaming breaks application context.
- **Candidate Discovery & Addressing**: Kept because peers have multiple physical paths (Wi-Fi, Cellular, Bluetooth).
- **Reachability & Path Selection**: Kept as a lightweight probing check to prevent connection hangs on stale addresses.
- **Transport Abstraction**: Kept as a minimal trait to allow multi-transport drivers without coupling business logic to sockets.
- **Session Layer**: Kept to provide ConnectionId continuity so reconnects don't require restarting application state machines.

## What Phase 1 Prototyping Will Prove

>Phase 1 (Core Connectivity Model) will empirically test this minimal architecture through 4 sequential capabilities:

1. S1.1 Peer Identity: Implement concrete 32-byte cryptographic PeerId generation, validation, and string encoding (Base58/Hex).
2. S1.2 Candidate Addressing: Implement strongly-typed multi-transport candidate address representations.
3. S1.3 Reachability Probing: Implement lightweight binary probe challenge/response state machine.
4. S1.4 Minimal Transport Trait & Loopback Driver: Prove the transport abstraction against an in-memory/loopback 
   implementation with automated integration tests.

---

## Formal Phase 0 Gate Evaluation

### Gate Decision: 🟢 PROCEED

**Justification**:
1. The problem scope is clearly defined and separated from Aryntra Pravah.
2. Primary reference systems (libp2p, WebRTC, QUIC, WireGuard, BLE) have been thoroughly investigated with explicit trade-offs documented in the Research Matrix (`S0.2`).
3. Core terminology and critical non-equivalence distinctions (`Peer ≠ PeerId`, `Address ≠ Endpoint`, `Connection ≠ Session`, etc.) are established in the Architecture Vocabulary (`S0.3`).
4. The Minimal Architecture candidate (Option C: Minimal Hybrid Path Model) avoids speculative abstractions and defines concrete, testable layers.
5. S1.1 through S1.4 goals are specific, measurable, and implementation-ready.

---

### Phase 1 Unlocked: Core Connectivity Model
- **S1.1**: Peer Identity (Cryptographic `PeerId` type, generation, byte serialization, string encoding)
- **S1.2**: Candidate Addressing (Typed multi-transport candidate addresses)
- **S1.3**: Reachability Probing (Binary reachability probe protocol & state machine)
- **S1.4**: Transport Abstraction & In-Memory Loopback Driver (Integration test gate)
