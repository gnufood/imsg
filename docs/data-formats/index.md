# Data Formats

This section documents the data formats, encoding schemes, and normalization approaches used throughout the system. Whether you're working with wire protocols for Bluetooth messaging, parsing XML responses from MAP servers, or handling phone number data, these pages provide the context you need to understand how data is structured and processed.

---

## Message and Protocol Formats

These pages cover the wire formats and encoding rules for communicating with external systems via Bluetooth MAP and related protocols.

- **[bMessage Format Reference](bmessage-format.md)** — The bMessage wire format used by iOS for SMS over Bluetooth MAP. Covers data structures, status and type values, encoding rules, and error conditions. Start here when implementing or debugging Bluetooth SMS transmission.

- **[MAP Folder Listing XML](map-folder-listing.md)** — Parsing MAP protocol `GetFolderListing` XML responses into `FolderEntry` structures. Documents the XML schema and how to handle the parsed data in your application.

---

## Data Normalization

This page addresses how raw input data is transformed into consistent, usable formats.

- **[Phone Number Normalization](phone-number-normalization.md)** — Explains why phone number normalization uses regional metadata from libphonenumber rather than simple punctuation stripping, and how to handle normalization outcomes. Essential reading when dealing with user-provided phone numbers that may arrive in varied formats.

---

## Choosing a Starting Point

- **Implementing Bluetooth SMS handling** → start with **bMessage Format Reference**
- **Working with MAP folder operations** → start with **MAP Folder Listing XML**
- **Processing user-provided phone numbers** → start with **Phone Number Normalization**