# Security Policy

## Reporting

Do not open a public GitHub issue for security vulnerabilities.

**Preferred:** Use the "Report a vulnerability" button on the [Security tab](../../security/advisories/new).

**Alternative:** Email `bugs@gnu.foo`. Reports are acknowledged within 72 hours; fixes within 30 days depending on severity.

## Scope

In scope:
- Memory-safety bugs, panics, or crashes triggered by attacker-controlled input (malformed OBEX/MAP/PBAP frames from a rogue Bluetooth device, crafted CLI input)
- Message content (SMS/MMS bodies, contact data) leaking into logs via `-v`/`-vv` output beyond documented behaviour
- Unauthenticated access to or MITM of the iroh hub/spoke connection
- Hub node key or device address exposure via config files or process environment
- Supply-chain compromise of crate dependencies

Out of scope:
- Bluetooth pairing security — that is the responsibility of BlueZ and the OS
- Attacks requiring physical access to the paired iPhone
- Issues with no security impact or theoretical bugs without a proof of concept

## GPG Key

`EA92 184C E5A3 4B0B C9EE  3A91 8E28 40A2 97D4 7681`

[Fetch from keys.openpgp.org](https://keys.openpgp.org/search?q=EA92184CE5A34B0BC9EE3A918E2840A297D47681) · [keys/EA92184CE5A34B0BC9EE3A918E2840A297D47681.asc](keys/EA92184CE5A34B0BC9EE3A918E2840A297D47681.asc)
