use winnow::ascii::{line_ending, till_line_ending};
use winnow::combinator::{preceded, terminated};
use winnow::prelude::*;
use winnow::token::{literal, take_till};

use super::types::{BBody, BEnvelope, BMessage, BMessageError, BVCard, MessageStatus, MessageType};

pub(super) fn parse_bmessage(input: &str) -> Result<BMessage, BMessageError> {
    let s = &mut &*input;
    expect_begin("BMSG", s)?;
    let (status, type_, folder, originator) = parse_bmsg_header(s)?;
    let envelope = parse_benv(s)?;
    expect_end("BMSG", s)?;
    Ok(BMessage::from_parts(status, type_, folder, originator, envelope))
}

fn kv_line<'i>(input: &mut &'i str) -> Result<(&'i str, &'i str), ()> {
    (
        take_till(1.., |c: char| c == ':' || c == '\r' || c == '\n'),
        preceded(':', terminated(till_line_ending, line_ending)),
    )
        .parse_next(input)
}

fn line_str<'i>(input: &mut &'i str) -> Result<&'i str, ()> {
    terminated(till_line_ending, line_ending).parse_next(input)
}

fn expect_begin(name: &'static str, input: &mut &str) -> Result<(), BMessageError> {
    (literal("BEGIN:"), literal(name), line_ending)
        .void()
        .parse_next(input)
        .map_err(|()| BMessageError::UnterminatedSection(name))
}

fn expect_end(name: &'static str, input: &mut &str) -> Result<(), BMessageError> {
    (literal("END:"), literal(name), line_ending)
        .void()
        .parse_next(input)
        .map_err(|()| BMessageError::UnterminatedSection(name))
}

fn parse_bmsg_header(
    input: &mut &str,
) -> Result<(MessageStatus, MessageType, String, Option<BVCard>), BMessageError> {
    let mut status: Option<MessageStatus> = None;
    let mut type_: Option<MessageType> = None;
    let mut folder: Option<String> = None;
    let mut originator: Option<BVCard> = None;

    loop {
        if input.starts_with("BEGIN:VCARD") {
            originator = Some(parse_vcard(input)?);
            continue;
        }
        if input.starts_with("BEGIN:BENV") {
            break;
        }
        if input.is_empty() {
            return Err(BMessageError::UnterminatedSection("BMSG"));
        }
        if input.starts_with("END:BMSG") {
            return Err(BMessageError::MissingField("BENV"));
        }
        let (k, v) = kv_line(input).map_err(|()| BMessageError::UnterminatedSection("BMSG"))?;
        match k {
            "STATUS" => status = Some(parse_status(v)?),
            "TYPE" => type_ = Some(parse_type(v)?),
            "FOLDER" => folder = Some(v.to_string()),
            _ => {}
        }
    }

    Ok((
        status.ok_or(BMessageError::MissingField("STATUS"))?,
        type_.ok_or(BMessageError::MissingField("TYPE"))?,
        folder.ok_or(BMessageError::MissingField("FOLDER"))?,
        originator,
    ))
}

fn parse_vcard(input: &mut &str) -> Result<BVCard, BMessageError> {
    expect_begin("VCARD", input)?;
    let mut name = String::new();
    let mut tel = String::new();

    loop {
        if input.starts_with("END:VCARD") {
            break;
        }
        if input.is_empty() {
            return Err(BMessageError::UnterminatedSection("VCARD"));
        }
        let (k, v) = kv_line(input).map_err(|()| BMessageError::UnterminatedSection("VCARD"))?;
        match k {
            "N" => name = v.to_string(),
            "TEL" => tel = v.to_string(),
            _ => {}
        }
    }

    expect_end("VCARD", input)?;
    Ok(BVCard { name, tel })
}

fn parse_benv(input: &mut &str) -> Result<BEnvelope, BMessageError> {
    expect_begin("BENV", input)?;
    let mut recipients = Vec::new();
    while input.starts_with("BEGIN:VCARD") {
        recipients.push(parse_vcard(input)?);
    }
    let body = parse_bbody(input)?;
    expect_end("BENV", input)?;
    Ok(BEnvelope { recipients, body })
}

fn parse_bbody(input: &mut &str) -> Result<BBody, BMessageError> {
    expect_begin("BBODY", input)?;
    let mut encoding = String::new();
    let mut charset = String::new();
    let mut language = String::new();
    let mut length: Option<usize> = None;

    loop {
        if input.starts_with("BEGIN:MSG") {
            break;
        }
        if input.is_empty() {
            return Err(BMessageError::UnterminatedSection("BBODY"));
        }
        let (k, v) = kv_line(input).map_err(|()| BMessageError::UnterminatedSection("BBODY"))?;
        match k {
            "ENCODING" => encoding = v.to_string(),
            "CHARSET" => charset = v.to_string(),
            "LANGUAGE" => language = v.to_string(),
            "LENGTH" => length = v.parse().ok(),
            _ => {}
        }
    }

    let len = length.ok_or(BMessageError::MissingField("LENGTH"))?;
    let text = parse_msg(input, len)?;
    expect_end("BBODY", input)?;
    Ok(BBody { encoding, charset, language, text })
}

// LENGTH is always CRLF-based (MAP spec); simulate +2 per line even on LF-only input
fn parse_msg(input: &mut &str, length: usize) -> Result<String, BMessageError> {
    expect_begin("MSG", input)?;
    let budget = length.saturating_sub(20);
    let mut body = String::new();
    let mut consumed: usize = 0;
    while consumed < budget {
        let line = line_str(input).map_err(|()| BMessageError::UnterminatedSection("MSG"))?;
        if !body.is_empty() {
            body.push('\n');
        }
        body.push_str(line);
        consumed = consumed.saturating_add(line.len().saturating_add(2));
    }
    expect_end("MSG", input)?;
    Ok(body)
}

fn parse_status(s: &str) -> Result<MessageStatus, BMessageError> {
    match s {
        "READ" => Ok(MessageStatus::Read),
        "UNREAD" => Ok(MessageStatus::Unread),
        _ => Err(BMessageError::UnknownStatus(s.to_string())),
    }
}

fn parse_type(s: &str) -> Result<MessageType, BMessageError> {
    match s {
        "SMS_GSM" => Ok(MessageType::SmsGsm),
        _ => Err(BMessageError::UnknownType(s.to_string())),
    }
}
