# Transport Documentation

This section covers the transport layer implementations that enable communication between devices in the application. The documentation here describes both the Bluetooth Classic transport mechanisms and the iroh QUIC-based remote transport, providing the architectural context and API references needed to understand how data moves between components.

## Bluetooth Classic Transport

The Bluetooth Classic transport handles RFCOMM-based serial communication over Bluetooth, supporting profile registration and device connectivity for legacy Bluetooth devices.

**RFCOMM connections** provide the core Bluetooth serial communication channel, including MNS (Message Notification Service) profile registration and link state monitoring. This is the foundational transport for communicating with Bluetooth-enabled devices that use RFCOMM channels.

**Device discovery** covers paired device management and Service Discovery Protocol (SDP) resolution for locating MAP (Message Access Profile) and PBAP (Phone Book Access Profile) RFCOMM channels on remote devices.

- [RFCOMM Transport Reference](rfcomm-transport.md) — Bluetooth Classic RFCOMM connection setup, MNS profile registration, and link state monitoring
- [Device Discovery Reference](device-discovery.md) — Paired device listing and SDP-based MAP/PBAP RFCOMM channel resolution

## iroh QUIC Transport

The iroh QUIC transport implements a modern remote communication layer using the QUIC protocol, replacing the legacy TCP bridge. This transport is designed for efficient, secure communication between hub and spoke endpoints.

**Architecture and design** documentation covers the Hub/Spoke model, explaining how the QUIC-based transport is structured and how it differs from previous TCP-based implementations.

**API reference** provides the runtime details, spoke endpoints, and stream wrappers needed to work with the QUIC transport programmatically.

- [iroh QUIC Hub/Spoke Transport](iroh-quic-transport.md) — Architecture and design of the QUIC-based remote transport that replaces the TCP bridge
- [iroh API Reference](iroh-api-reference.md) — Hub runtime, spoke endpoints, and stream wrappers for QUIC transport