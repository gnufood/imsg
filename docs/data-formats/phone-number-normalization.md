# Phone Number Normalization

Phone number normalization in this system transforms raw device-reported strings into a canonical E.164 form suitable for reliable matching and storage. The design deliberately uses regional metadata from the `phonenumber` crate rather than simple punctuation stripping, and treats normalization as a best-effort transformation that never discards the original value.

## Why Regional Metadata Matters

A naive approach to phone number normalization might strip all non-digit characters, keeping only the raw digits. This works for some cases but fails in important ways that matter for a system handling international contacts and messages.

Consider a UK number written as `+44 (0) 1753 866488`. The `(0)` is a national trunk prefix — it indicates that the caller is in the UK and should dial the area code without the country code. However, this prefix is only valid within the UK; removing it incorrectly when the number is already in international format produces a different, invalid number. Worse, some countries use trunk prefixes that vary by number type (mobile vs. fixed line), and the rules differ from country to country.

The `phonenumber` crate embeds the regional numbering plan metadata from Google's libphonenumber library. This metadata knows that `+44` is the UK country code, that `0` after it is not part of the subscriber number, and that the valid UK subscriber number length is 10 digits after the area code. It can therefore correctly parse the number and reformat it as the canonical `+441753866488`.

Without this regional knowledge, any hand-rolled rule either over-strips (breaking international numbers) or under-strips (leaving regional prefixes that prevent matching). The metadata approach handles all regions uniformly, including edge cases like shared country codes (the US and Canada both use `+1`, distinguished by the area code) and non-geographic numbers (toll-free, premium-rate).

## The Normalization Outcome Model

Normalization can fail for several reasons, and the system models each failure mode explicitly rather than raising an exception. The `Normalization` enum captures all possible outcomes:

- **E164**: The number was successfully parsed and validated against the relevant regional plan, then formatted as an E.164 string with a leading `+` and country code.
- **InvalidNumber**: The number has a valid structure but fails validation — it may have the right length for its region but uses an unassigned prefix.
- **MissingRegion**: The number has no country calling code (no leading `+` or `00`) and no fallback region was provided to interpret it.
- **InvalidCountryCode**: The number claims an international prefix (`+` or `00`) but the following digits do not correspond to any assigned country code.
- **TooShort** or **TooLong**: The digit count falls outside what any regional plan allows.
- **NotANumber**: The input is not recognizable as a phone number at all — it may be alphabetic or empty.

Every outcome except `E164` means the caller should continue using the original raw value. Normalization is best-effort, not a gate. This design choice reflects the reality of device-reported data: a phone may report a contact in any format, and the system must preserve that value for display even when canonicalization fails.

## The PhoneField Value Type

The `PhoneField` type encapsulates both the raw device-reported value and the normalized form when available. It provides several accessors:

- `raw()` always returns the original string exactly as received from the device.
- `e164()` returns the canonical form only when normalization succeeded.
- `canonical()` returns E.164 if available, otherwise falls back to the raw value — this is the value used for matching.
- `display()` returns the canonical form for default rendering, matching the canonical behavior.

This separation serves two purposes. First, it preserves the original data for features like `contacts --raw` that must show what the device reported. Second, it enables reliable matching: two phone fields representing the same real number compare equal via `canonical()` regardless of how differently they were formatted on the wire.

The type also provides `from_parts`, a constructor that trusts pre-existing stored values. This avoids re-parsing on read, which would reintroduce the dependency on the phone metadata and risk reclassifying a number differently if the metadata has changed between the original parse and the read.

## Handling Outcomes in Practice

When normalizing user input (such as a number typed into `imsg send` or `imsg list --from`), the system normalizes once at the command entry point, before the transport fork between store and live device. If normalization fails, the raw value still works — it may not match stored canonical forms, but it reaches the device as entered.

When normalizing incoming data from the device (contacts via PBAP, messages via MAP), normalization happens at the parse stage. The raw value is always retained; the canonical value is stored alongside it when available. This means existing rows written before normalization was added continue to work — they simply have no E.164 form, and matching falls back to the raw string.

The key insight is that normalization does not need to succeed for the system to function. It improves matching reliability when it works, but the design explicitly accounts for its absence. Every code path that uses phone numbers checks `canonical()` rather than assuming E.164, so the fallback to raw is transparent.

## Relationship to Other Components

Phone normalization lives in `imsg-formats`, the crate that parses wire formats. It is used by the PBAP contact sync path (where vCard phone fields are parsed), the MAP message ingress path (where message addresses are normalized on receipt), and the CLI dispatch layer (where user-typed numbers are normalized before any operation).

The region used for normalization can come from configuration, be derived from the device's own phonebook entry, or fall back to a default. This is tracked separately in the region planning documentation, but the normalization function itself accepts an optional `country::Id` parameter that serves as the assumed region when the input has no country code.

## Summary

Phone number normalization uses regional metadata because simple punctuation stripping cannot handle the variation in national trunk prefixes, area code formats, and number lengths across regions. The system models all failure modes explicitly and preserves the original value whenever normalization cannot produce a canonical form. The `PhoneField` type encapsulates both forms, enabling reliable matching while retaining the device-reported value for display and for features that require it.