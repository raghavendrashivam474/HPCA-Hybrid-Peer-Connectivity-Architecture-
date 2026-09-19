# HPCA Research: Hybrid Peer Connectivity Architecture

## Central Research Question
> **Can HPCA develop a principled hybrid peer-connectivity architecture inspired by proven systems while minimizing their respective trade-offs and avoiding unnecessary complexity?**

## Systems Under Investigation

### Primary Systems
1. **libp2p**: Modularity, multiaddresses, peer IDs, stream multiplexing, transport swarms.
2. **WebRTC**: ICE candidate gathering, STUN/TURN, connectivity checks, multi-path candidate pairing.
3. **QUIC**: Connection IDs, streams, path migration, integrated TLS 1.3 security context.
4. **WireGuard / Tailscale**: Stable cryptographic identity decoupled from roaming IP endpoints.
5. **Bluetooth Connectivity Models**: Discovery, pairing lifecycles, platform constraints, energy profiles.

### Secondary Systems
6. **Matrix**: Decentralized federation, identity server separation.
7. **Signal**: Asynchronous identity key exchange and session establishment.
8. **ZeroMQ / NATS**: Lightweight messaging topology and socket abstractions.

## Structured Evaluation Framework
Each reference system is analyzed against 11 standardized criteria:
1. What problem does it solve?
2. What abstraction does it introduce?
3. What does that abstraction decouple?
4. What are its major benefits?
5. What are its costs / overheads?
6. What assumptions does it make?
7. Does HPCA actually have the same problem?
8. Can the useful property be achieved more simply?
9. **BORROW**: Specific concepts or techniques HPCA should adopt.
10. **AVOID**: Anti-patterns, excess complexity, or tight couplings HPCA should reject.
11. **EXPERIMENT**: Concepts requiring empirical prototyping before making architectural decisions.

## Detailed System Investigations

### 1. libp2p
- **1. What problem does it solve?** Decentralized peer-to-peer networking across arbitrary, heterogeneous physical networks with dynamic routing.
- **2. What abstraction does it introduce?** `PeerId` (cryptographic public key hash), `Multiaddr` (self-describing composable network addresses, e.g., `/ip4/1.2.3.4/tcp/1234/p2p/Qm...`), `Transport` upgrader, and `Swarm` (multi-connection stream multiplexer).
- **3. What does that abstraction decouple?** Decouples peer identity from network addresses, and transport protocols from application streams.
- **4. Major benefits**: Universal address formatting, transport pluggability, protocol negotiation (`multistream-select`), built-in NAT traversal (AutoNAT/Relay).
- **5. Costs & overheads**: Excessive layered abstraction overhead, high handshake round-trips during multistream negotiation, bloated dependency footprint, high cognitive complexity.
- **6. Assumptions**: Peers can handle cryptographic handshakes and maintain stream multiplexers; assumes composable protocol stacks.
- **7. Does HPCA have this problem?** Yes, HPCA needs identity-over-transport separation and transport pluggability.
- **8. Can it be achieved more simply?** Yes. Avoid multi-round-trip protocol negotiation by using unified binary frame headers and concrete typed addresses instead of arbitrary dynamic string-encoded multiaddrs.
- **9. BORROW**: Separation of `PeerId` from physical addresses; concept of transport upgrading / stream multiplexing.
- **10. AVOID**: Arbitrary dynamic multistream-select negotiation overhead; bloated modular abstractions for simple peer links.
- **11. EXPERIMENT**: Fixed-size cryptographic `PeerId` format and typed multi-transport address enums.

---

### 2. WebRTC (ICE / STUN / TURN / DataChannel)
- **1. What problem does it solve?** Direct real-time browser-to-browser interactive communication across symmetric/asymmetric NATs and firewalls.
- **2. What abstraction does it introduce?** ICE Candidates (host, srflx, relay), Candidate Pairing & Connectivity Checks (STUN binding requests), SDP Offer/Answer signaling model, and `RTCDataChannel` (SCTP over DTLS over UDP).
- **3. What does that abstraction decouple?** Decouples candidate discovery from the out-of-band signaling plane, and abstracts network reachability testing from data transmission.
- **4. Major benefits**: Unmatched NAT traversal success rates, aggressive path probing, built-in encryption and congestion control.
- **5. Costs & overheads**: Complex and verbose SDP signaling, heavyweight native stack, high connection setup latency, reliance on external STUN/TURN infrastructure.
- **6. Assumptions**: An existing out-of-band signaling channel exists between peers to exchange SDP/candidates.
- **7. Does HPCA have this problem?** Yes, HPCA peers must probe reachability across multiple interfaces (Wi-Fi, LAN, Bluetooth) and select working paths.
- **8. Can it be achieved more simply?** Yes. Borrow the candidate gathering and reachability probing model, but use lightweight binary ping/pong probing instead of heavy SDP/STUN/SCTP stacks.
- **9. BORROW**: Candidate gathering, parallel reachability checks, prioritized path selection.
- **10. AVOID**: Heavyweight SDP signaling strings, complex SCTP-over-DTLS encapsulation overhead.
- **11. EXPERIMENT**: Binary reachability probe frames sent across candidate paths to measure latency and validate bidirectional paths.

---

### 3. QUIC (RFC 9000)
- **1. What problem does it solve?** Head-of-line blocking in TCP multiplexing, slow connection establishment, and connection breakage during network interface switching (e.g., Wi-Fi to cellular).
- **2. What abstraction does it introduce?** `ConnectionID` (opaque identifier independent of IP/port 4-tuple), independent multiplexed `Streams` over UDP, and Path Validation (`PATH_CHALLENGE` / `PATH_RESPONSE`).
- **3. What does that abstraction decouple?** Decouples the transport connection identity from the underlying network routing 4-tuple (IP:Port), enabling seamless Connection Migration.
- **4. Major benefits**: 0-RTT/1-RTT connection setup with integrated TLS 1.3, resilience to IP roaming, stream isolation preventing HOL blocking.
- **5. Costs & overheads**: High user-space UDP processing CPU overhead, UDP throttling/blocking by enterprise middleboxes/firewalls, complex loss recovery logic.
- **6. Assumptions**: UDP traffic is permitted by intervening network equipment.
- **7. Does HPCA have this problem?** Yes, mobile and roaming peers change IP addresses and switch network interfaces frequently.
- **8. Can it be achieved more simply?** Yes. Adopt the `ConnectionId` decoupling principle so sessions survive underlying transport reconnects, while retaining TCP as a fallback when UDP/QUIC is blocked.
- **9. BORROW**: Connection IDs independent of socket addresses, path challenge/response validation, native stream multiplexing.
- **10. AVOID**: Hard requirement on UDP-only transport; keep transport layer pluggable with TCP fallback.
- **11. EXPERIMENT**: Session token / ConnectionId layer that maintains logical session continuity across transport rebinds.

---

### 4. WireGuard & Tailscale
- **1. What problem does it solve?** Simple, secure, zero-config encrypted overlay networking across changing endpoints and NAT boundaries (DERP / disco).
- **2. What abstraction does it introduce?** WireGuard `Cryptokey Routing` (mapping static public keys directly to tunnel IPs) and Tailscale `disco` / DERP relays (NAT-traversal discovery and encrypted relay fallbacks).
- **3. What does that abstraction decouple?** Decouples static peer cryptographic identity from dynamic, roaming UDP endpoints; unifies routing table with cryptographic ACLs.
- **4. Major benefits**: Extremely minimal attack surface (Noise protocol), automatic roaming (updates peer endpoint upon receiving authenticated packet from new address), near-line-rate performance.
- **5. Costs & overheads**: Point-to-point tunnel interface model is kernel-centric (in pure WireGuard); requires coordination server for key distribution (Tailscale).
- **6. Assumptions**: Peers know each other's static public keys ahead of time or via a coordination plane.
- **7. Does HPCA have this problem?** Yes. HPCA needs roaming endpoint updates where receiving an authenticated packet from an existing peer on a new path can transparently update reachability.
- **8. Can it be achieved more simply?** Yes. Adopt WireGuard's stateless roaming rule: authenticated messages on a valid path update the peer's active endpoint.
- **9. BORROW**: Static Cryptographic Key as Identity; auto-updating endpoint roaming upon authenticated packet arrival; DERP-style lightweight relay fallback.
- **10. AVOID**: Kernel TUN/TAP device dependency; HPCA should operate cleanly in user-space application layers.
- **11. EXPERIMENT**: Endpoint update trigger on verified session frames received from alternative physical paths.

---

### 5. Bluetooth Connectivity Models (BLE & RFCOMM)
- **1. What problem does it solve?** Short-range, low-power device-to-device communication without local area network infrastructure (no Wi-Fi/router needed).
- **2. What abstraction does it introduce?** BLE Advertisements & GATT services/characteristics; RFCOMM / L2CAP stream-oriented sockets; Device MAC & Pairing bonds.
- **3. What does that abstraction decouple?** Decouples discovery (unconnected broadcast advertisements) from point-to-point data connections.
- **4. Major benefits**: Works in fully off-grid scenarios; ultra-low power consumption in idle/discovery state.
- **5. Costs & overheads**: Low throughput (BLE ~1-2 Mbps), severe platform-specific permission/API fragmentation (iOS CoreBluetooth vs. Android BLE vs. Linux BlueZ vs. Windows WinRT), asymmetric Central/Peripheral roles.
- **6. Assumptions**: Physical proximity (< 10-30 meters); platform OS permissions granted.
- **7. Does HPCA have this problem?** Yes, for off-grid / local peer discovery and proximity-based data exchange.
- **8. Can it be achieved more simply?** Yes. Treat Bluetooth purely as an auxiliary discovery beacon and fallback transport adapter, rather than the primary backbone.
- **9. BORROW**: Unconnected advertising payloads for presence/discovery; asymmetric discovery role negotiation.
- **10. AVOID**: Exposing Bluetooth GATT/L2CAP-specific semantics into core session APIs; encapsulate behind a uniform Transport trait.
- **11. EXPERIMENT**: BLE advertisement beacon carrying HPCA truncated PeerId and ephemeral reachability hints.

---

### 6. Secondary Systems (Matrix, Signal, ZeroMQ, NATS)
- **Matrix**: Demonstrates identity server decoupling and decentralized federation. *Borrow*: Room/channel session separation. *Avoid*: Heavy JSON-over-HTTP federation latency and high database overhead.
- **Signal**: Demonstrates X3DH and Double Ratchet session security. *Borrow*: Forward-secret peer session key ratchet concepts for Phase 3 security model. *Avoid*: Centralized discovery server dependency.
- **ZeroMQ**: Demonstrates smart socket abstractions (`DEALER/ROUTER`, `PUB/SUB`) hiding reconnect and framing details. *Borrow*: Clean message framing and auto-reconnect abstractions. *Avoid*: Complex multipart messaging state machines with implicit blocking.
- **NATS**: Demonstrates ultra-lightweight text/binary control plane and high-throughput subject routing. *Borrow*: Simple, explicit control frame design. *Avoid*: Centralized server cluster assumption.

---

## Comparative Research Matrix

| System | Primary Problem Solved | Key Abstraction | Major Benefit | Major Cost / Trade-off | HPCA Decision |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **libp2p** | Generic P2P across heterogeneous nets | `PeerId`, `Multiaddr`, `Swarm` | Universal addressing & modular transport | Complex negotiation round-trips & bloated stack | **BORROW** PeerId/Address split; **AVOID** string Multiaddrs & multistream overhead |
| **WebRTC** | Real-time direct browser connectivity | ICE Candidates, Connectivity Checks | High NAT traversal success & parallel path probing | Complex SDP signaling & heavy DTLS/SCTP stack | **BORROW** Parallel reachability checks & path ranking; **AVOID** SDP syntax |
| **QUIC** | Head-of-line blocking & roaming connections | `ConnectionID`, Multiplexed Streams | 0-RTT handshakes & seamless connection migration | UDP-only restriction & CPU overhead | **BORROW** Connection ID decoupled from IP 4-tuple; **AVOID** UDP exclusivity |
| **WireGuard / Tailscale** | Simple, secure overlay network & roaming | Cryptokey Routing, DERP relays | Zero-overhead roaming on packet arrival, Noise security | Coordination server reliance (Tailscale) | **BORROW** Roaming endpoint updates on valid frames; **AVOID** Kernel TUN dependency |
| **Bluetooth** | Off-grid, local-range direct communication | BLE Beacons, RFCOMM / L2CAP | Works without LAN/WAN infrastructure, ultra-low power | Fragmented OS APIs, asymmetric roles, low bandwidth | **BORROW** Unconnected beacon discovery; **AVOID** Leaking GATT into core session APIs |
| **ZeroMQ / NATS** | High-throughput distributed messaging | Smart framed sockets, lightweight control frames | Clean message boundaries & reconnect ergonomics | Implicit queue bloat or centralized topology assumption | **BORROW** Explicit binary framing; **AVOID** Centralized broker assumptions |

---

## Architectural Synthesis

### What HPCA Will BORROW
1. **Identity Decoupling**: Static cryptographic `PeerId` distinct from any physical or logical network address (libp2p, WireGuard).
2. **Path-Agnostic Connection Identity**: Logical `ConnectionId` / session tokens that remain valid across transport reconnects or interface hops (QUIC).
3. **Parallel Reachability Probing**: Gathering multi-transport candidate addresses and probing reachability with lightweight binary ping/pong frames (WebRTC ICE model).
4. **Stateless Roaming Endpoint Updates**: Automatically updating the active path for a peer session when an authenticated frame arrives over a new valid path (WireGuard).
5. **Auxiliary Beacon Discovery**: Using broadcast/multicast mechanisms (mDNS, BLE beacons) purely for discovery without coupling them to session state (Bluetooth/mDNS).

### What HPCA Will AVOID
1. **Dynamic String Address Parsing**: Avoid text-based multiaddrs and SDP strings; use compact, strongly-typed binary enums for addresses and candidates.
2. **Multi-Round-Trip Protocol Negotiation**: Avoid `multistream-select` overhead; use fixed binary protocol framing headers.
3. **Transport Exclusivity**: Avoid assuming UDP-only (QUIC limitation) or TCP-only; maintain transport abstraction for heterogeneous environments.
4. **Kernel-Level Dependencies**: Avoid requiring TUN/TAP devices (WireGuard) or OS-level elevated network drivers.
5. **Over-Engineering Abstraction Layers**: Avoid creating nested generic wrappers around sockets before a concrete requirement demands them.

### What HPCA Will EXPERIMENT With in Phase 1 & 2
- Fixed 32-byte Ed25519/BLAKE3 cryptographic `PeerId`.
- Typed candidate address structures (`SocketAddr`, Bluetooth MAC, etc.).
- Lightweight binary reachability challenge/response protocol.
- Minimal session token state machine decoupling physical connections from logical sessions.

---

## Open Questions & Known Unknowns
- *Unknown 1*: Can Bluetooth LE discovery beacons fit sufficient cryptographic identity data without exceeding standard BLE advertisement payload limits (31 bytes legacy)?
  - *Investigation plan*: Test compact truncated key hashes in Phase 2.
- *Unknown 2*: What is the minimum CPU and memory overhead of running parallel reachability checks over cellular + Wi-Fi simultaneously on resource-constrained embedded/mobile devices?
  - *Investigation plan*: Profile lightweight probe latency in Phase 1 / Phase 2.
