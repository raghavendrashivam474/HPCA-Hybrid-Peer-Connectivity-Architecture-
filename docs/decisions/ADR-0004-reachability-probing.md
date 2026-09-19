# ADR-0004: Lightweight Binary Reachability Probing

## Status
Accepted

## Context
Phase 0 vocabulary defined `Reachability ≠ Connection` and `Endpoint ≠ Reachability`. Having a candidate address does not prove reachability. Before allocating transport or session state, HPCA performs lightweight binary probing.

## Decision
1. **Wire Format**:
   - `ProbeRequest` (49 bytes): `[0x01 (u8)]` + `[Nonce (16 bytes)]` + `[Target PeerId (32 bytes)]`
   - `ProbeResponse` (49 bytes): `[0x02 (u8)]` + `[Nonce (16 bytes)]` + `[Responder PeerId (32 bytes)]`
2. **Challenge Mechanism**:
   - Prober generates a cryptographically random 16-byte nonce.
   - Responder reflects the nonce and asserts its own `PeerId`.
   - Prober rejects any response with mismatched nonce, invalid responder identity, or corrupted length/tag.
3. **State Machine**:
   - `Created` -> `AwaitingResponse` -> `Succeeded(Duration)` | `Failed(ProbeFailure)`.

## Consequences
- Deterministic, zero-overhead reachability verification.
- Completely decoupled from transport drivers and long-lived session framing.
