# HPCA Architectural Vocabulary & Concept Definitions

## Purpose
This document establishes the precise, authoritative terminology and structural distinctions for the Hybrid Peer Connectivity Architecture (HPCA). Every abstraction in HPCA maps directly to a concept defined here.

---

## 1. Identity & Location Concepts

### Term: Peer
- **Definition**: An autonomous participant in the HPCA network capable of initiating or receiving communications.
- **Represents**: The logical entity/node as a whole.
- **Does NOT represent**: A single physical machine, a specific network card, or a specific IP address (a single peer may roam across devices or multi-home across multiple network interfaces).
- **Relationships**: A `Peer` has exactly one authoritative `PeerId` at any given time and may possess zero or more candidate `Address`es.
- **Why HPCA needs it**: HPCA routes messages between logical peers, not between ephemeral hardware interfaces.

### Term: PeerId
- **Definition**: A globally unique, cryptographically verifiable identifier for a `Peer`.
- **Represents**: The public cryptographic identity (e.g., Ed25519 public key or cryptographic hash thereof).
- **Does NOT represent**: An IP address, port number, DNS name, or transport protocol.
- **Relationships**: 1:1 mapping with a `Peer`. Invariant under network changes or physical roaming.
- **Why HPCA needs it**: Enables end-to-end authentication and peer verification regardless of how network paths change.

### Term: Address
- **Definition**: A location identifier specific to a particular transport medium.
- **Represents**: Concrete reachability coordinates (e.g., IPv4/IPv6 socket `192.168.1.50:8080`, Bluetooth MAC `00:1A:7D:DA:71:13`, domain name `node.local`).
- **Does NOT represent**: Identity. An address can be reassigned, shared, or spoofed.
- **Relationships**: A `Peer` can have many `Address`es across different transport types.
- **Why HPCA needs it**: Provides the physical routing parameters necessary for underlying transport drivers to establish sockets.

### Term: Endpoint
- **Definition**: A bound local or remote transport communication termination point that can send and receive frames.
- **Represents**: An active socket or radio binding paired with an `Address`.
- **Does NOT represent**: A logical peer identity or an active bidirectional verified connection.
- **Relationships**: Endpoints are owned by `Transport` instances and produce/consume raw frames.
- **Why HPCA needs it**: Differentiates passive network addresses from active, bound OS communication handles.

---

## 2. Connectivity & Transport Concepts

### Term: Discovery
- **Definition**: The mechanism or process of finding candidate `Address`es and `PeerId`s across local or remote networks.
- **Represents**: Passive or active beaconing, multicast DNS announcements, rendezvous lookups, or local peer exchange.
- **Does NOT represent**: Establishing a connection or verifying actual reachability.
- **Relationships**: Produces candidate `Address`es associated with potential `PeerId`s.
- **Why HPCA needs it**: Populates the set of candidate paths before reachability checks are performed.

### Term: Reachability
- **Definition**: The empirical state or measurement of whether bidirectional communication is currently viable over a specific candidate path.
- **Represents**: The outcome of active ping/pong or probe validation between endpoints.
- **Does NOT represent**: Discovery (a discovered address might not be reachable) or an active application session.
- **Relationships**: Evaluates candidate `Address`es to produce verified `Path`s.
- **Why HPCA needs it**: Prevents connection stalls by proving a medium is open and responsive before attempting full session negotiation.

### Term: Path
- **Definition**: A specific, concrete communication route between a local endpoint and a remote endpoint over a distinct physical or virtual network interface.
- **Represents**: The pairing of `(Local Endpoint, Remote Address, Transport Type)`.
- **Does NOT represent**: The transport mechanism itself, nor an application-level logical connection.
- **Relationships**: Multiple `Path`s may connect the same pair of `Peer`s simultaneously (e.g., Path 1: Wi-Fi LAN, Path 2: Bluetooth LE).
- **Why HPCA needs it**: Enables multi-path evaluation, failover, and path migration without tearing down peer identity or session state.

### Term: Transport
- **Definition**: The physical or virtual driver and framing abstraction capable of transmitting bytes over a specific medium (TCP, UDP, QUIC, Bluetooth RFCOMM, etc.).
- **Represents**: The driver implementation and protocol-specific socket lifecycle.
- **Does NOT represent**: An individual route (`Path`) or an end-to-end user session (`Session`).
- **Relationships**: Transports manage `Endpoint`s and provide raw frame delivery across `Path`s.
- **Why HPCA needs it**: Pluggable transport architecture allows adding new media (e.g. QUIC, Bluetooth) without altering higher layers.

---

## 3. Session & State Concepts

### Term: Connection
- **Definition**: An active, bidirectional transport-level channel established over a specific `Path`.
- **Represents**: The operational state of a socket/link (open, half-open, closed, errored).
- **Does NOT represent**: The long-lived logical association between peers (`Session`).
- **Relationships**: Bound to a specific `Path` and managed by a `Transport`.
- **Why HPCA needs it**: Encapsulates raw I/O streaming, flow control, and low-level framing.

### Term: ConnectionId
- **Definition**: An opaque identifier uniquely identifying a logical connection instance across transport lifecycles.
- **Represents**: The continuity token for a communication stream.
- **Does NOT represent**: The physical socket descriptor or IP 4-tuple.
- **Relationships**: Stays invariant even if the underlying physical connection or path migrates.
- **Why HPCA needs it**: Enables seamless path migration and transport reconnects without breaking application-level streams.

### Term: Session
- **Definition**: The long-lived logical association and authenticated security/state context between two `Peer`s.
- **Represents**: Cryptographic session keys, sequence numbers, channel state, and application message dispatch.
- **Does NOT represent**: A physical TCP connection or single network path.
- **Relationships**: A `Session` spans across transport reconnects, path migrations, and temporary physical disconnections.
- **Why HPCA needs it**: Applications interact with the `Session`, insulating application logic from physical network volatility.

---

## 4. Critical Architectural Distinctions

To prevent architectural blurring and premature coupling during implementation, the following distinctions are strictly enforced:

### `Peer ≠ PeerId`
- A **Peer** is the living autonomous entity. A **PeerId** is the cryptographic identifier that authenticates it. A Peer holds state and configuration; a PeerId is an immutable identity token.

### `PeerId ≠ Address`
- A **PeerId** specifies *who* the peer is (cryptographic identity). An **Address** specifies *where* the peer might be reached (IP:port, BLE MAC). Identity never changes when a peer roams to a new network address.

### `Address ≠ Endpoint`
- An **Address** is a passive descriptor or target coordinate. An **Endpoint** is an active, bound OS handle/socket instantiated on a local interface.

### `Endpoint ≠ Reachability`
- An **Endpoint** exists locally as a bound resource. **Reachability** is the empirical validation that traffic can actually transit to and return from a remote target via that endpoint.

### `Reachability ≠ Connection`
- **Reachability** confirms that a path is viable (e.g. ping/probe answered). A **Connection** is an established, framed, stateful link ready for continuous payload transfer.

### `Connection ≠ Session`
- A **Connection** is a physical or virtual transport link (ephemeral; may drop on Wi-Fi disconnect). A **Session** is the logical, authenticated conversational context (durable; survives reconnects and migrations).

### `Transport ≠ Path`
- A **Transport** is the driver/protocol engine (TCP driver, QUIC driver). A **Path** is a specific concrete route pairing `(Local Interface, Remote Address, Transport)`. Multiple paths can share the same transport driver.

---

## Architectural Guardrails for Phase 1 Developers
1. Do not define Rust structs for these concepts prematurely before their specific sprint in Phase 1.
2. When creating APIs in Phase 1, ensure types strictly adhere to these definitions without conflating identity, address, connection, and session.
