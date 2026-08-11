# Protocol Clients

This section covers the protocol client implementations available in this codebase. These clients enable communication with remote devices using standard Bluetooth profiles—MAP (Message Access Profile) for messaging and PBAP (Phonebook Access Profile) for contacts—over an OBEX transport layer.

The documentation here is organized to support different reader needs: those who want to understand the underlying OBEX protocol machinery, those ready to implement a specific profile client, and those consulting API details during development.

---

## OBEX Protocol Foundation

Before working with profile-specific clients, it helps to understand the OBEX layer that handles transport and protocol mechanics. OBEX (Object Exchange) is the session protocol used by both MAP and PBAP to structure requests and responses.

- **[OBEX Transport Layer](obex-transport.md)** — Explains how OBEX frames data into packets, handles connection establishment, and manages the server-side state machine. Useful if you need to debug transport-level behavior or understand how bytes flow between client and server.

- **[OBEX Client Reference](obex-client-reference.md)** — Documents the sans-IO client state machine with contracts for connect, setpath, get, put, and disconnect operations. Reference material for developers implementing or extending OBEX client logic.

---

## MAP Client

The Message Access Profile (MAP) client enables browsing and manipulating messages on a remote device, as well as receiving notifications when new messages arrive.

- **[Using the MAP Client](map-client-walkthrough.md)** — A practical walkthrough covering connection to a MAP device, folder navigation, message listing, and receiving notifications via the Message Notification Service (MNS). Start here when you're ready to implement MAP functionality.

---

## PBAP Client

The Phonebook Access Profile (PBAP) client provides access to phonebook data on a remote device, including contact listings and individual contact details.

- **[Using the PBAP Client](pbap-client-walkthrough.md)** — A practical walkthrough covering connection to a PBAP device, pulling phonebooks, fetching metadata, and retrieving individual contacts. Start here when you're ready to implement PBAP functionality.

---

## Profile API Reference

When you need to look up specific message types, filter options, contact fields, or search parameters, consult the unified reference for both profile clients.

- **[MAP and PBAP Client API Reference](profile-client-reference.md)** — Covers MAP message types, listing filters, status operations, and application parameters; also documents PBAP phonebook paths, contact entry structures, metadata fields, and search attributes. Use this as a lookup resource during implementation rather than as reading material.