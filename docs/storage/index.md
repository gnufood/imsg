# Storage

This section covers how the application persists and retrieves data, including messages, contacts, and synchronization state. The documentation here explains the storage layer's architecture, the data models it uses, and the workflows for sending messages and managing contacts.

Begin here if you need to understand how data flows into and out of the local database, how the sync process tracks progress, or how to work with the storage APIs directly.

---

## Core Storage Infrastructure

These pages cover the foundational storage layer: how the encrypted database is opened and managed, and the data types that define the schema.

- **[Store API](store-api.md)** — Explains how to open the encrypted SQLCipher database, manage connection lifecycle, and handle the error types the storage layer can raise.
- **[Data Types](data-types.md)** — Reference for the row structures, status enums, and constants used across messages, outbox entries, sync cursors, and contacts.

---

## Message Storage and Retrieval

These pages describe how messages are stored, organized into folders, indexed for querying, and how the sync process tracks progress per folder.

- **[Message Storage Architecture](message-storage.md)** — Covers the schema design, folder organization, indexing strategy, and the read paths used to retrieve messages from the database.
- **[Sync Cursors](sync-cursors.md)** — Explains how per-folder sync progress is tracked, enabling independent folder syncs and providing the `latest_sync_at` freshness signal.

---

## Outgoing Messages

These pages cover the complete outgoing message lifecycle, from creating a speculative message to confirming it was sent and reconciling with the device's Sent folder.

- **[Outbox and Outgoing Lifecycle](outbox-lifecycle.md)** — Describes the state machines for outbox entries and outgoing messages, including speculative sends and reconciliation against the device Sent folder.
- **[Sending a Message](sending-a-message.md)** — A walkthrough of the end-to-end send flow: speculative message creation, outbox enqueue, push to the server, handle promotion, and final confirmation.

---

## Contact Management

This page explains how contacts are retrieved from the device and cached locally, including phone number normalization and cache invalidation.

- **[Contact Caching](contact-caching.md)** — Covers how PBAP contacts are cached, how phone numbers are normalized to E.164 format, and how the cache is invalidated when the device's contact version changes.