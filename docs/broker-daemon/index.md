# Broker Daemon Documentation

The Broker Daemon is a runtime component that mediates communication between external clients and connected devices. It handles device connections, manages session lifecycles, and exposes an IPC interface for command execution. The documentation in this section covers the daemon's internal architecture, how it manages device actors and sessions, its shutdown behavior, and practical guidance for running and interacting with a broker instance.

---

## Understanding the Broker Daemon

Before diving into specific topics, it's helpful to understand the daemon's overall structure and the two modes in which it operates.

The **Architecture** documentation provides the foundational overview of the broker daemon runtime, covering abstract socket binding, the device actor lifecycle, IPC request handling, and the key distinction between ephemeral and persistent (daemon) modes. This is the place to start if you want to understand how the pieces fit together.

- [Broker Daemon Architecture](broker-daemon-overview.md) — Explains the runtime model, socket binding, IPC handling, and the differences between ephemeral and persistent modes.

---

## Device Actor and Session Management

Once you understand the overall architecture, the next area of focus is how the daemon manages individual device connections and their sessions.

The **Device Actor Lifecycle** documentation describes how the device actor handles MAP and PBAP sessions, including lazy connection establishment, bounded retry with exponential backoff, reconnection on recoverable drops, and the exit conditions that differ between ephemeral and persistent modes. This material is relevant when you need to understand connection reliability, retry behavior, or how the daemon decides when to give up on a device.

- [Device Actor Lifecycle](device-actor-lifecycle.md) — Covers session management, connection retry strategies, and lifecycle transitions for device actors.

---

## Daemon Operations

The daemon's operational behavior extends beyond individual device sessions to how the entire process starts, runs, and shuts down.

The **Graceful Shutdown** documentation explains how the persistent daemon mode handles termination, including how IPC Shutdown requests and system signals (SIGTERM/SIGINT) converge into a single drain sequence, how connection completion is bounded, and how actor teardown is coordinated. This is essential reading if you are integrating the daemon into a system where clean shutdown is important.

- [Daemon Graceful Shutdown](daemon-graceful-shutdown.md) — Describes the shutdown sequence, signal handling, and resource cleanup in persistent mode.

---

## Reference Material

For detailed type definitions and internal structures used throughout the daemon, consult the runtime types reference. This section is not a guide but a lookup resource.

- [Runtime Types Reference](runtime-types-reference.md) — Reference documentation for internal types including DeviceOp message variants, ConnState connection states, ConnectPolicy retry configuration, and Connector factory types.

---

## Getting Started

To see the daemon in action and understand the practical flow from starting a broker instance to executing commands against it, work through the walkthrough. This demonstrates the full request lifecycle from CLI through the actor to the connected device.

- [Running Broker Commands](running-broker-commands.md) — A walkthrough demonstrating how to start a broker instance and execute IPC commands, showing the complete request flow.