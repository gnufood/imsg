//! bMessage encoder: produces CRLF-terminated wire format.

use super::types::{BMessage, BVCard, MessageType};

impl BMessage {
    /// CRLF line endings throughout. LENGTH = byte count of the CRLF `BEGIN:MSG…END:MSG` block;
    /// iOS rejects LF-only lengths.
    #[must_use]
    pub fn encode(&self) -> String {
        let mut out = String::with_capacity(256);
        line(&mut out, "BEGIN:BMSG");
        line(&mut out, "VERSION:1.0");
        line(&mut out, &format!("STATUS:{}", self.status().as_str()));
        line(&mut out, &format!("TYPE:{}", MessageType::as_str()));
        line(&mut out, &format!("FOLDER:{}", self.folder()));
        if let Some(orig) = self.originator() {
            encode_vcard(&mut out, orig);
        }
        let env = self.envelope();
        line(&mut out, "BEGIN:BENV");
        for recip in &env.recipients {
            encode_vcard(&mut out, recip);
        }
        encode_bbody(
            &mut out,
            &env.body.encoding,
            &env.body.charset,
            &env.body.language,
            &env.body.text,
        );
        line(&mut out, "END:BENV");
        line(&mut out, "END:BMSG");
        out
    }
}

fn line(out: &mut String, s: &str) {
    out.push_str(s);
    out.push_str("\r\n");
}

fn encode_vcard(out: &mut String, vc: &BVCard) {
    line(out, "BEGIN:VCARD");
    line(out, "VERSION:3.0");
    line(out, &format!("N:{}", vc.name));
    line(out, &format!("TEL:{}", vc.tel));
    line(out, "END:VCARD");
}

fn encode_bbody(out: &mut String, encoding: &str, charset: &str, language: &str, text: &str) {
    line(out, "BEGIN:BBODY");
    line(out, &format!("ENCODING:{encoding}"));
    line(out, &format!("CHARSET:{charset}"));
    line(out, &format!("LANGUAGE:{language}"));
    line(out, &format!("LENGTH:{}", msg_block_length(text)));
    line(out, "BEGIN:MSG");
    for l in text.lines() {
        line(out, l);
    }
    line(out, "END:MSG");
    line(out, "END:BBODY");
}

// iOS computes LENGTH from the CRLF block; LF-only lengths cause send failures.
fn msg_block_length(text: &str) -> usize {
    let mut n: usize = 11;
    for l in text.lines() {
        n = n.saturating_add(l.len()).saturating_add(2);
    }
    n.saturating_add(9)
}
