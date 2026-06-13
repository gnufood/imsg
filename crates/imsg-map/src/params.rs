//! MAP application parameter tag constants, bitflags, and encoder functions.

use bitflags::bitflags;
use bytes::Bytes;

use crate::MapError;

bitflags! {
    /// `SupportedMessageTypes` bitmask from `GetMASInstanceInformation`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MessageTypes: u8 {
        /// GSM cellular SMS messages.
        const SMS_GSM  = 0x01;
        /// CDMA cellular SMS messages.
        const SMS_CDMA = 0x02;
        /// Email messages.
        const EMAIL    = 0x04;
        /// MMS multimedia messages.
        const MMS      = 0x08;
        /// Instant messages (e.g. RCS, IM).
        const IM       = 0x10;
    }
}

bitflags! {
    /// `FilterMessageType` application parameter bitmask for `GetMessagesListing`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FilterMessageType: u8 {
        /// Include GSM SMS messages in results.
        const SMS_GSM  = 0x01;
        /// Include CDMA SMS messages in results.
        const SMS_CDMA = 0x02;
        /// Include email messages in results.
        const EMAIL    = 0x04;
        /// Include MMS messages in results.
        const MMS      = 0x08;
    }
}

/// MAP Application Parameter tag constants.
pub mod tag {
    /// If set, message is not copied to the sent folder.
    pub const TRANSPARENT: u8 = 0x0A;
    /// If set, retry delivery on transient failure.
    pub const RETRY: u8 = 0x0B;
    /// Character set (0x01 = UTF-8).
    pub const CHARSET: u8 = 0x14;
    /// Bitmask — see `FilterMessageType`.
    pub const FILTER_MESSAGE_TYPE: u8 = 0x03;
    /// Response-only: total folder count available.
    pub const FOLDER_LISTING_SIZE: u8 = 0x07;
    pub(crate) const MAX_LIST_COUNT: u8 = 0x01;
    pub(crate) const LIST_START_OFFSET: u8 = 0x02;
    /// Response-only: 1 if unread messages exist.
    pub const NEW_MESSAGE: u8 = 0x0D;
    /// 1 = enable MNS event reports, 0 = disable.
    pub const NOTIFICATION_STATUS: u8 = 0x0E;
    pub(crate) const FILTER_PERIOD_BEGIN: u8 = 0x04;
    pub(crate) const FILTER_PERIOD_END: u8 = 0x05;
    pub(crate) const FILTER_READ_STATUS: u8 = 0x06;
    pub(crate) const FILTER_ORIGINATOR: u8 = 0x08;
    /// Response-only: total message count available (MAP 1.4 wire tag 0x12).
    pub const MESSAGES_LISTING_SIZE: u8 = 0x12;
    /// Selects which status property to update in `SetMessageStatus`.
    pub const STATUS_INDICATOR: u8 = 0x17;
    /// The new value for the selected status property in `SetMessageStatus`.
    pub const STATUS_VALUE: u8 = 0x18;
}

/// Charset value for MAP `PushMessage` (UTF-8).
pub const CHARSET_UTF8: u8 = 0x01;

/// `StatusIndicator` wire value selecting read/unread state.
pub const INDICATOR_READ_STATUS: u8 = 0x00;
/// `StatusIndicator` wire value selecting deleted/present state.
pub const INDICATOR_DELETED_STATUS: u8 = 0x01;

/// Always `[0x14, 0x01, 0x01]`.
#[must_use]
pub const fn push_message_params() -> Bytes {
    Bytes::from_static(b"\x14\x01\x01")
}

/// Always `[0x14, 0x01, 0x01]`.
#[must_use]
pub const fn get_message_params() -> Bytes {
    Bytes::from_static(b"\x14\x01\x01")
}

/// 6-byte APP-PARAMS: `StatusIndicator` (tag `0x17`, 1 byte) + `StatusValue` (tag `0x18`, 1 byte).
#[must_use]
pub fn set_message_status_params(indicator: u8, value: u8) -> Vec<u8> {
    let mut params = Vec::with_capacity(6);
    params.extend_from_slice(&[tag::STATUS_INDICATOR, 0x01, indicator]);
    params.extend_from_slice(&[tag::STATUS_VALUE, 0x01, value]);
    params
}

/// Encodes `NotificationStatus` (tag `0x0E`, 1 byte) + `MasInstanceId` (tag `0x0F`, 1 byte).
#[must_use]
pub const fn set_notification_registration_params(enable: bool) -> Bytes {
    if enable {
        Bytes::from_static(b"\x0e\x01\x01\x0f\x01\x00")
    } else {
        Bytes::from_static(b"\x0e\x01\x00\x0f\x01\x00")
    }
}

/// Builds a MAP app-params blob for `ListMessages`.
///
/// Encodes `MaxListCount` and `ListStartOffset` unconditionally. Appends `FilterReadStatus` when
/// `read_status` is `Some`. String filters are encoded as null-terminated UTF-8 with a 1-byte
/// length prefix (`[tag][len = bytes+1][UTF-8][0x00]`).
///
/// # Errors
///
/// Returns [`MapError::InvalidInput`] if any string contains a null byte or exceeds 254 UTF-8 bytes.
pub fn list_messages_params(
    max_count: u16,
    offset: u16,
    read_status: Option<u8>,
    originating_address: Option<&str>,
    period_begin: Option<&str>,
    period_end: Option<&str>,
) -> Result<Vec<u8>, MapError> {
    let mut cap = 8_usize;
    if read_status.is_some() {
        cap = cap.saturating_add(3);
    }
    for s in [originating_address, period_begin, period_end].into_iter().flatten() {
        cap = cap.saturating_add(s.len().saturating_add(3));
    }
    let mut params = Vec::with_capacity(cap);
    params.extend_from_slice(&[tag::MAX_LIST_COUNT, 0x02]);
    params.extend_from_slice(&max_count.to_be_bytes());
    params.extend_from_slice(&[tag::LIST_START_OFFSET, 0x02]);
    params.extend_from_slice(&offset.to_be_bytes());
    if let Some(status) = read_status {
        params.extend_from_slice(&[tag::FILTER_READ_STATUS, 0x01, status]);
    }
    push_str_tlv(&mut params, tag::FILTER_ORIGINATOR, originating_address)?;
    push_str_tlv(&mut params, tag::FILTER_PERIOD_BEGIN, period_begin)?;
    push_str_tlv(&mut params, tag::FILTER_PERIOD_END, period_end)?;
    Ok(params)
}

fn push_str_tlv(out: &mut Vec<u8>, tag_byte: u8, value: Option<&str>) -> Result<(), MapError> {
    let Some(s) = value else {
        return Ok(());
    };
    if s.contains('\x00') {
        return Err(MapError::InvalidInput("filter string must not contain null bytes"));
    }
    let len = u8::try_from(s.len().saturating_add(1))
        .map_err(|_| MapError::InvalidInput("filter string exceeds 254 UTF-8 bytes"))?;
    out.push(tag_byte);
    out.push(len);
    out.extend_from_slice(s.as_bytes());
    out.push(0x00);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::list_messages_params;
    use crate::MapError;

    #[test]
    fn list_messages_params_with_originator() -> Result<(), MapError> {
        let bytes = list_messages_params(1024, 0, None, Some("5550001001"), None, None)?;
        let tlv: &[u8] = b"\x08\x0b5550001001\x00";
        assert!(bytes.windows(tlv.len()).any(|w| w == tlv));
        Ok(())
    }

    #[test]
    fn list_messages_params_with_period_filters() -> Result<(), MapError> {
        let bytes = list_messages_params(
            1024,
            0,
            None,
            None,
            Some("20260601T000000"),
            Some("20260605T235959"),
        )?;
        let begin_tlv: &[u8] = b"\x04\x1020260601T000000\x00";
        let end_tlv: &[u8] = b"\x05\x1020260605T235959\x00";
        assert!(bytes.windows(begin_tlv.len()).any(|w| w == begin_tlv));
        assert!(bytes.windows(end_tlv.len()).any(|w| w == end_tlv));
        Ok(())
    }

    #[test]
    fn list_messages_params_rejects_null_byte() {
        let result = list_messages_params(1024, 0, None, Some("abc\x00def"), None, None);
        assert!(matches!(result, Err(MapError::InvalidInput(_))));
    }

    #[test]
    fn list_messages_params_rejects_overlong() {
        let long: String = "a".repeat(255);
        let result = list_messages_params(1024, 0, None, Some(&long), None, None);
        assert!(matches!(result, Err(MapError::InvalidInput(_))));
    }
}
