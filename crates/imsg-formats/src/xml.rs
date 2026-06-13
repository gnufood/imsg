//! MAP XML structures: folder listing parser. Use [`FolderListing::parse`].

use quick_xml::{events::Event, Reader};
use thiserror::Error;

/// MAP XML parsing errors — quick-xml parse, attribute decode, and folder name UTF-8 validation.
#[derive(Debug, Error)]
pub enum XmlError {
    /// Malformed XML or unrecognised structure.
    #[error("XML error: {0}")]
    Parse(#[from] quick_xml::Error),
    /// Invalid or malformed attribute encoding.
    #[error("attribute error: {0}")]
    Attr(#[from] quick_xml::events::attributes::AttrError),
    /// Folder name attribute is not valid UTF-8.
    #[error("non-UTF-8 folder name")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// A single MAP folder from a `GetFolderListing` response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderEntry {
    name: String,
}

impl FolderEntry {
    pub(crate) const fn new(name: String) -> Self {
        Self { name }
    }

    /// Folder name decoded from the XML `name` attribute, e.g. `inbox`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Parsed body of a MAP `GetFolderListing` OBEX response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderListing {
    folders: Vec<FolderEntry>,
}

impl FolderListing {
    /// Folders in document order.
    #[must_use]
    pub fn folders(&self) -> &[FolderEntry] {
        &self.folders
    }

    /// Parses a `<folder-listing>` XML document from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns [`XmlError`] on malformed XML, undecodable attributes, or non-UTF-8 folder names.
    pub fn parse(xml: &[u8]) -> Result<Self, XmlError> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);
        let mut folders = Vec::new();
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Empty(e) | Event::Start(e) if e.name().as_ref() == b"folder" => {
                    for attr in e.attributes() {
                        let a = attr?;
                        if a.key.as_ref() == b"name" {
                            folders.push(FolderEntry::new(
                                String::from_utf8(a.value.into_owned()).map_err(XmlError::Utf8)?,
                            ));
                        }
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(Self { folders })
    }
}
