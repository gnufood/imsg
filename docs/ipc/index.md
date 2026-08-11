# IPC Documentation

This section covers the Inter-Process Communication (IPC) layer that connects the CLI client to the broker service. The documentation here describes both the wire protocol used to transmit messages and the data structures that compose those messages.

## Protocol & Message Format

These pages define how messages travel between the CLI and broker over the network or IPC channel. You'll need this information if you're implementing a new client, debugging communication issues, or extending the protocol.

- **[IPC Protocol](ipc-protocol.md)** — The wire-level contract between CLI and broker. Covers the request/response message format, frame boundaries, the `MAX_FRAME_LEN` constant, and how JSON payloads are encoded and transmitted.

## Data Structures

These pages document the objects that flow through the protocol. Reference these when working with message payloads, understanding what fields are available, or mapping between internal types and wire representations.

- **[IPC Data Types](ipc-data-types.md)** — The data transfer objects and state types used in IPC communication. Includes `MessageDto`, `ContactDto`, `SessionState`, `Reason`, and related row types that define the structure of information exchanged between client and broker.

---

**Not sure where to start?** If you're debugging a connection issue or implementing a new client, begin with the IPC Protocol to understand the message format. If you're working with the data itself, start with IPC Data Types to see what information is available.