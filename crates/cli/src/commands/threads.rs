//! `threads` subcommand: group inbox and sent messages into per-contact conversation threads.

use std::collections::HashMap;
use std::fmt::Write as _;

use anyhow::Result;
use config::Config;
use map_core::folders::Folder;
use map_core::messages::{ListMessagesFilter, MessageEntry};

use transport::iroh::Endpoint;

use crate::commands::conn;
use crate::fmt::fmt_datetime;

/// Connects to MAP, fetches inbox and sent messages, groups them into per-contact
/// conversation threads sorted most-recent-first; each thread shows at most five messages.
/// Messages with empty addressing are silently excluded.
///
/// # Errors
///
/// Returns an error if the MAP connection, folder navigation, or either listing request
/// fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
) -> Result<String> {
    let mut client = conn::connect_map(cfg, endpoint, device).await?;
    client.set_folder(Folder::Inbox).await?;
    let mut all = client.list_messages(&ListMessagesFilter::default()).await?;
    client.set_folder(Folder::Sent).await?;
    all.extend(client.list_messages(&ListMessagesFilter::default()).await?);
    let groups = group(&all);
    Ok(render_threads(&groups))
}

const fn key_for(msg: &MessageEntry) -> Option<&str> {
    let key =
        if msg.sent { msg.recipient_addressing.as_str() } else { msg.sender_addressing.as_str() };
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn group(entries: &[MessageEntry]) -> Vec<(String, Vec<&MessageEntry>)> {
    let mut map: HashMap<&str, Vec<&MessageEntry>> = HashMap::new();
    for entry in entries {
        if let Some(key) = key_for(entry) {
            map.entry(key).or_default().push(entry);
        }
    }
    let mut groups: Vec<(String, Vec<&MessageEntry>)> = map
        .into_iter()
        .map(|(key, mut msgs)| {
            msgs.sort_by(|a, b| a.datetime.as_str().cmp(b.datetime.as_str()));
            (key.to_owned(), msgs)
        })
        .collect();
    groups.sort_by(|a, b| {
        let max_a = a.1.iter().copied().map(|m| m.datetime.as_str()).max().unwrap_or("");
        let max_b = b.1.iter().copied().map(|m| m.datetime.as_str()).max().unwrap_or("");
        max_b.cmp(max_a)
    });
    groups
}

fn render_threads(groups: &[(String, Vec<&MessageEntry>)]) -> String {
    if groups.is_empty() {
        return "(no threads)".to_owned();
    }
    let mut out = String::with_capacity(groups.len().saturating_mul(128));
    for (key, msgs) in groups {
        let name = msgs
            .iter()
            .copied()
            .find_map(|m| {
                let n = if m.sent { m.recipient_name.as_str() } else { m.sender_name.as_str() };
                if n.is_empty() {
                    None
                } else {
                    Some(n)
                }
            })
            .unwrap_or(key.as_str());
        let unread = msgs.iter().copied().filter(|m| !m.read && !m.sent).count();
        let total = msgs.len();
        let _ = write!(out, "{name}");
        if name != key.as_str() {
            let _ = write!(out, "  {key}");
        }
        let _ = write!(out, "  {total} messages");
        if unread > 0 {
            let _ = write!(out, "  {unread} unread");
        }
        let _ = writeln!(out);
        let skip = msgs.len().saturating_sub(5);
        if skip > 0 {
            let _ = writeln!(out, "  \u{2026} {skip} earlier");
        }
        for msg in msgs.iter().copied().skip(skip) {
            let marker = if !msg.read && !msg.sent { '*' } else { ' ' };
            let dir = if msg.sent { '\u{2192}' } else { '\u{2190}' };
            let mut tail = msg.subject.chars();
            let head: String = tail.by_ref().take(72).collect();
            let dt = fmt_datetime(&msg.datetime);
            if tail.next().is_some() {
                let _ = writeln!(out, "  {marker} {dir} {dt}  {head}\u{2026}");
            } else {
                let _ = writeln!(out, "  {marker} {dir} {dt}  {head}");
            }
        }
        out.push('\n');
    }
    out
}
