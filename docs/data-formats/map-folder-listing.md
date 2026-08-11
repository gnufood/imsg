# MAP Folder Listing XML

The MAP (Message Access Profile) protocol uses XML to represent folder listings returned by the `GetFolderListing` OBEX operation. This module parses that XML into structured `FolderEntry` values.

## XML Format

A folder listing is an XML document containing `<folder>` elements with a `name` attribute:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<folder-listing>
  <folder name="inbox"/>
  <folder name="outbox"/>
  <folder name="sent"/>
  <folder name="drafts"/>
  <folder name="telecom/msg"/>
</folder-listing>
```

The root element is `<folder-listing>`. Each `<folder>` element represents a single folder. Folder names are encoded in the `name` attribute as UTF-8 text.

## Structures

### FolderEntry

A single folder from the listing.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Folder name decoded from the XML `name` attribute, e.g. `inbox` or `telecom/msg`. |

**Constructor:** `FolderEntry::new(name: String) -> Self`

**Methods:**

- `name(&self) -> &str` — Returns the folder name. The name is always valid UTF-8; parsing fails with `XmlError::Utf8` if the attribute is not valid UTF-8.

### FolderListing

Parsed body of a MAP `GetFolderListing` OBEX response.

| Field | Type | Description |
|-------|------|-------------|
| `folders` | `Vec<FolderEntry>` | Folders in document order. |

**Methods:**

- `folders(&self) -> &[FolderEntry]` — Returns the folders in the order they appear in the XML document.

- `parse(xml: &[u8]) -> Result<Self, XmlError>` — Parses a `<folder-listing>` XML document from raw bytes.

### XmlError

Error type for folder listing parsing. Variants:

| Variant | From | Description |
|---------|------|-------------|
| `Parse` | `quick_xml::Error` | Malformed XML or unrecognised structure. |
| `Attr` | `quick_xml::events::attributes::AttrError` | Invalid or malformed attribute encoding. |
| `Utf8` | `std::string::FromUtf8Error` | Folder name attribute is not valid UTF-8. |

## Parsing Behavior

- The parser uses `quick-xml` with text trimming enabled.
- Both `<folder name="..."/>` (self-closing) and `<folder name="..."></folder>` (paired) syntax are accepted.
- The parser iterates through all events and collects any element named `folder` with a `name` attribute.
- Folders are returned in document order — the order they appear in the XML.
- Empty folder listings (no `<folder>` elements) produce an empty `FolderListing` with no error.
- The parser does not validate folder names against any particular set; any UTF-8 string is accepted.

## Error Conditions

| Condition | Error Variant |
|-----------|---------------|
| Malformed XML (e.g., unclosed tags) | `XmlError::Parse` |
| Invalid attribute encoding | `XmlError::Attr` |
| Non-UTF-8 folder name | `XmlError::Utf8` |

## Example

```rust
use imsg_formats::xml::FolderListing;

let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<folder-listing>
  <folder name="inbox"/>
  <folder name="sent"/>
</folder-listing>"#;

let listing = FolderListing::parse(xml).unwrap();
assert_eq!(listing.folders().len(), 2);
assert_eq!(listing.folders()[0].name(), "inbox");
assert_eq!(listing.folders()[1].name(), "sent");
```

## Related Types

- `imap::types::Mailbox` — IMAP mailbox representation, used for IMAP folder operations.
- `bmessage::types::BMessage` — bMessage structure for MAP message bodies.
- `phone::PhoneField` — Phone number handling for MAP address fields.

## See Also

- [MAP Protocol Overview](../map-overview.md) — High-level MAP protocol flow.
- [bMessage Encoding](../bmessage-encoding.md) — bMessage format for MAP message transfer.