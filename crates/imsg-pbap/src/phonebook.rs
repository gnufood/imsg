//! PBAP phonebook path identifiers.

/// PBAP phonebook path for `PullPhoneBook` and vCard-listing GETs. Use [`pull_name`](Self::pull_name) for the OBEX `Name` header value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhonebookPath {
    /// Main phonebook (`telecom/pb`).
    Pb,
    /// Incoming call history (`telecom/ich`).
    Ich,
    /// Outgoing call history (`telecom/och`).
    Och,
    /// Missed call history (`telecom/mch`).
    Mch,
    /// Combined call history (`telecom/cch`).
    Cch,
    /// Speed dial (`telecom/spd`).
    Spd,
    /// Favourites (`telecom/fav`).
    Fav,
}

impl PhonebookPath {
    /// OBEX `Name` header value for a `PullPhoneBook` GET, e.g. `"telecom/pb.vcf"` for [`Pb`](Self::Pb).
    #[must_use]
    pub const fn pull_name(self) -> &'static str {
        match self {
            Self::Pb => "telecom/pb.vcf",
            Self::Ich => "telecom/ich.vcf",
            Self::Och => "telecom/och.vcf",
            Self::Mch => "telecom/mch.vcf",
            Self::Cch => "telecom/cch.vcf",
            Self::Spd => "telecom/spd.vcf",
            Self::Fav => "telecom/fav.vcf",
        }
    }

    /// OBEX `Name` header value for a `ListvCardObjects` GET, e.g. `"telecom/pb"` for [`Pb`](Self::Pb).
    ///
    /// Unlike [`pull_name`](Self::pull_name), no `.vcf` suffix is included.
    #[must_use]
    pub const fn list_name(self) -> &'static str {
        match self {
            Self::Pb => "telecom/pb",
            Self::Ich => "telecom/ich",
            Self::Och => "telecom/och",
            Self::Mch => "telecom/mch",
            Self::Cch => "telecom/cch",
            Self::Spd => "telecom/spd",
            Self::Fav => "telecom/fav",
        }
    }

    /// OBEX `Name` header value for a `PullvCardEntry` GET for the given handle,
    /// e.g. `"telecom/pb/41.vcf"` for [`Pb`](Self::Pb) with handle `"41.vcf"`.
    ///
    /// Does not validate `handle` — callers must ensure it is non-empty and contains no CR or LF.
    #[must_use]
    pub fn entry_name(self, handle: &str) -> String {
        format!("{}/{}", self.list_name(), handle)
    }
}
