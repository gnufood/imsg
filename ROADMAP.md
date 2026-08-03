# Roadmap

> Last updated: 2026-07-31
> 

---

## Tags

Tags represent broad, durable domains; prefixes may change, but each item retains its original number.

| Prefix | Domain |
| --- | --- |
| `CORE` | Shared architecture, runtime behavior, and engineering foundations |
| `CONN` | Network transports, communication protocols, and session management |
| `DEVC` | Device integration, discovery, synchronization, and interoperability |
| `MSG` | Messaging, contacts, search, and communication features |
| `DATA` | Persistence, data protection, and storage infrastructure |
| `CLNT` | User interfaces and interaction surfaces |
| `API` | Public interfaces, SDKs, and developer extensibility |
| `OPS` | Installation, deployment, and service lifecycle |
| `CONF` | Application settings, validation, and configuration management |
| `DIAG` | Logging, tracing, diagnostics, and operational visibility |

The numeric portion of an identifier is immutable. For example, an item previously identified as `STORE-01` may become `DATA-01`, but not `DATA-10`.

---

## Planned

Planned work has a defined outcome and is ready for implementation.

### Core

*Shared foundations used across application components and entrypoints.*

- [ ]  **CORE-01** — Centralize error conversion across the CLI and GUI
- [ ]  **CORE-02** — Provide a shared logging configuration for all entrypoints

### Connectivity

*Communication infrastructure for reliable network and protocol sessions.*

- [ ]  **CONN-01** — Add persistent session handling to the hub
- [ ]  **CONN-02** — Add persistent session handling to each spoke

### Messaging

*Capabilities for communicating with, organizing, and finding people and conversations.*

- [ ]  **MSG-01** — Export conversations 
- [ ]  **MSG-02** — Add typo-tolerant text search across locally stored message bodies
- [ ]  **MSG-03** — Add semantic similarity search across locally stored conversations
- [ ]  **MSG-04** — Detect links in message text and render them as interactive links
- [ ]  **MSG-05** — Synchronize contact photos alongside existing contact data
- [ ]  **MSG-06** — Derive the phone-number normalization region from the connected device with a safe fallback and user override

### Data

*Infrastructure for securely storing and protecting application data.*

- [ ]  **DATA-04** — Encrypt each stored message with an independently managed record key
- [ ]  **DATA-05** — Record message decryption access in an auditable log
- [ ]  **DATA-06** — Rotate database encryption keys without rewriting the complete database

### Clients

*Interactive interfaces for accessing application capabilities.*

- [ ]  **CLNT-01** — Build browser-based thread, message, compose, and reply views

### API

*Stable interfaces for building applications and language bindings on top of imsg.*

- [ ]  **API-01** — Publish a stable Rust API with documented semantic-versioning guarantees
- [ ]  **API-02** — Version the broker wire protocol and reject incompatible client–daemon connections
- [ ]  **API-03** — Provide a C FFI for cross-language integrations
- [ ]  **API-04** — Publish a Python SDK using PyO3 and maturin
- [ ]  **API-05** — Publish a Go SDK using cgo

### Operations

*System-level installation, deployment, and daemon lifecycle management.*

- [ ]  **OPS-01** — Install and control the system service from the GUI through native operating-system authorization

---

## Research

Research work produces a validated technical direction, prototype, or implementation plan.

### Connectivity

*Candidate transports and protocols for expanding communication capabilities.*

- [ ]  **CONN-03** — Validate ANCS notification delivery over BLE and document the supported message-sync scope
- [ ]  **CONN-04** — Complete a TCP transport prototype for same-machine and LAN communication and compare it with the existing iroh path
- [ ]  **CONN-05** — Prototype iAP communication with USB-connected iOS devices and measure latency and capability coverage
- [ ]  **CONN-06** — Validate HFP support for call history and call control
- [ ]  **CONN-07** — Validate OBEX FTP support for browsing and retrieving device files

---

## Known Issues

### Configuration

*Settings and configuration behavior that requires stronger validation or error handling.*

- [ ]  **CONF-01** — Prevent messaging and contact services from using the same channel, and save channel overrides atomically
- [ ]  **CONF-02** — Surface unreadable or invalid configuration files instead of launching first-time setup

### Device Integration

*Device workflows that require improved reliability, synchronization, or interoperability.*

- [ ]  **DEVC-02** — Retry automatic channel detection after transient connection failures
- [ ]  **DEVC-03** — Preserve successful contact-sync status and prevent malformed contact names from entering the cache
- [ ]  **DEVC-04** — File sent messages into the correct folder and target deletion using the resolved folder
- [ ]  **DEVC-05** — Resolve and display participant contact names in conversation views
- [ ]  **DEVC-06** — Run initial device discovery concurrently with the splash animation

---

## Completed

Completed work retains its original numeric identifier under its current broad domain.

- [x]  **DIAG-02** — Removed proxy teardown log noise — `70f9b44`
- [x]  **CORE-06** — Surfaced undelete errors — `a36219b`
- [x]  **DATA-01** — Added an encrypted SQLCipher message store — `68fef48`
- [x]  **DATA-02** — Secured the 256-bit database key through the system keyring — `3dd359e`
- [x]  **DEVC-01** — Added built-in device scanning, selection, and persistence — `1a584de`
- [x]  **DATA-03** — Replaced `watch` with the persistent `daemon` and added eager MNS body retrieval — `ca7d21c`, `91597b9`