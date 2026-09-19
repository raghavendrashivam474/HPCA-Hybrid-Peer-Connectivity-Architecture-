# HPCA Architecture Overview

## Status
**Architecture under research.**

## The Working Hypothesis
HPCA models heterogeneous peer connectivity through a progressive, layered sequence of questions and capabilities:

```text
1. Peer Identity           (WHO is the peer?)
      ↓
2. Discovery / Addresses   (WHERE might the peer be located?)
      ↓
3. Reachability            (CAN I contact the peer over any medium?)
      ↓
4. Transport               (HOW do I transmit bytes across the boundary?)
      ↓
5. Path Selection          (WHICH path is optimal right now?)
      ↓
6. Communication / Session (EXCHANGE application-level data reliably)
```

## Explicit Architectural Guardrails

* This diagram is a working model to guide research and decomposition, not an approved API contract.
* Subsystems must not be created or abstracted in code until their responsibility boundaries are proven necessary 
  by research and  ADRs.

## Structural Boundaries

The directory structure represents **future architectural boundaries**, not active implementations:

- `core/`: Fundamental types, identifiers, shared state, and core abstractions.
- `protocol/`: Wire protocol definitions, frame encodings, and serialization logic.
- `transport/`: Pluggable physical and virtual transport implementations (TCP, QUIC, Bluetooth, etc.).
- `discovery/`: Peer discovery mechanisms (mDNS, rendezvous, address resolution).
- `session/`: Connection state management, multiplexing, and session lifecycles.
- `tests/`: Integration, interoperability, and conformance test suites.

## Architectural Policies

1. **Directories vs. Crates**: Directories establish structural boundaries; crates represent current compilation boundaries. HPCA begins as a single minimal library crate.
2. **No Speculative Abstractions**: No trait, generic, or interface shall be introduced without an immediate concrete caller and test verification.
3. **Change Discipline**: Any fundamental deviation from established architectural boundaries requires an Architectural Decision Record (ADR) prior to implementation.
