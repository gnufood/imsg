# Session Management

This section covers the session layer that powers communication between your application and Bluetooth devices. The documentation here explains how sessions are established, how data flows in both directions, and the different patterns you can use depending on your use case.

Whether you need to establish a connection, read data directly from a device, or manage ongoing synchronization, start here to understand the available approaches before diving into specific APIs.

## Understanding Session Fundamentals

Before working with the session layer, it's helpful to understand how connections are established and how the system decides which transport to use. This foundational knowledge helps you reason about connection behavior and error handling.

- **[Session Architecture](session-architecture.md)** — Explains how the session layer chooses between iroh hub and RFCOMM transports, manages the OBEX handshake process, and classifies errors to determine when retries are appropriate. Start here if you want to understand *why* sessions behave the way they do.

- **[Connection API](connection-api.md)** — Covers the functions for initiating MAP and PBAP sessions, handling connection errors, and implementing retry logic. Use this when you're ready to write code that establishes connections to devices.

## Data Access Patterns

Once you understand how sessions work, choose the access pattern that matches your application's needs. These pages cover the two primary ways to interact with device data.

- **[Live Query API](live-query-api.md)** — Describes lean models and functions for reading messages and contacts directly from the device without writing to your local store. Choose this when you need lightweight, on-demand reads without the overhead of persistent synchronization.

- **[Sending Messages and Continuous Sync](sending-and-sync.md)** — A practical walkthrough covering the outbox flow for sending messages and the backfill/watch loop for keeping your local store in sync with the device. Use this when building features that send messages or need ongoing bidirectional synchronization.