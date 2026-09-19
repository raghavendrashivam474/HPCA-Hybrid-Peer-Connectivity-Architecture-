# ADR-0001: Project Independence and Architectural Boundaries

## Status
Accepted

## Date
2025-03-30

## Context
Aryntra Pravah is a stable, production-oriented Java 21 system with a protected TCP-first architecture, comprehensive test suite (271/271 passing), and established operational constraints.

Researching heterogeneous, multi-transport peer connectivity (including QUIC, WebRTC, Bluetooth, and roaming path migration) introduces architectural unknowns and exploratory complexity that must not destabilize or block Pravah.

## Decision
We establish **HPCA (Hybrid Peer Connectivity Architecture)** as an entirely independent research and reference implementation repository.

1. **Independent Lifecycle**: HPCA is developed in its own repository with its own build, test, and release lifecycle.
2. **Language Choice**: Rust is chosen for the reference implementation for memory safety, performance, zero-cost abstractions, and clean C ABI / FFI cross-language bindings.
3. **Protocol Independence**: Architectural patterns, state machines, and wire protocols developed in HPCA must remain language- and framework-agnostic.

## Consequences

### Positive
- **Isolation**: HPCA research can move fast, prototype alternative transports, and discard unworkable designs without endangering Pravah.
- **Cross-platform**: Rust allows compiling native static libraries for Linux, macOS, Windows, Android (NDK), and iOS.
- **Clarity**: Separation of concerns forces all design decisions to be self-contained and documented on their own merits.

### Negative / Trade-offs
- Two separate repositories must be maintained.
- Learnings cannot be automatically injected into Pravah without explicit adaptation work.

## Pravah Separation Guardrails
1. **No Shared Code**: HPCA must not copy or import Pravah source code.
2. **No Direct Dependencies**: Neither repository may list the other as a dependency.
3. **No Breaking Pressure**: Pravah's protected architecture is not bound by HPCA's current state.
4. **Deliberate Migration Only**: Any future adoption of HPCA concepts into Pravah requires an ADR in Pravah evaluating the change against its operational requirements.
