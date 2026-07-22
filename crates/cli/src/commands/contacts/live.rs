//! Live PBAP contact reads: local/non-hub via the broker (Stage 8/9 wire), hub via a direct
//! connection using the shared `session::contacts` functions.

use std::path::Path;

use anyhow::Result;
use config::Config;
use ipc::{BrokerRequest, BrokerResponse, CardEntryDto, ContactDto};
use transport::iroh::Endpoint;

use super::render::{
    offset_of, render_contact_view, render_contacts_view, render_entries_view, ContactView,
    EntryView,
};
use super::ContactsOpts;
use crate::cli::{path_name, path_of};
use crate::commands::{broker, conn, live_footer};

/// Reads live phonebook data from the device. Exactly one of `--lookup`/`--get`/`--list`/
/// pull-all (the default) runs, per `opts` (clap enforces the flags are mutually exclusive).
///
/// # Errors
///
/// Returns an error if the broker call or the PBAP connection/operation fails.
pub(crate) async fn run(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    opts: &ContactsOpts,
    config_path: Option<&Path>,
) -> Result<String> {
    let out = if let Some(number) = &opts.lookup {
        run_lookup(cfg, endpoint, device, opts, number, config_path).await?
    } else if let Some(handle) = &opts.get {
        run_get(cfg, endpoint, device, opts, handle, config_path).await?
    } else if opts.list {
        run_list(cfg, endpoint, device, opts, config_path).await?
    } else {
        run_pull_all(cfg, endpoint, device, opts, config_path).await?
    };
    Ok(live_footer(out))
}

async fn run_lookup(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    opts: &ContactsOpts,
    number: &str,
    config_path: Option<&Path>,
) -> Result<String> {
    let found = if endpoint.is_none() {
        lookup_via_broker(cfg, device, config_path, opts, number)
            .await?
            .map(|c| render_contact_view(&ContactView::from_dto(&c), opts.raw))
    } else {
        let mut client = conn::connect_pbap(cfg, endpoint, device).await?;
        let result = session::contacts::lookup(&mut client, path_of(opts.path), number).await;
        warn_on_disconnect(client.disconnect().await);
        result?.map(|c| render_contact_view(&ContactView::from_contact(&c), opts.raw))
    };
    Ok(found.unwrap_or_else(|| format!("no contact found for {number}")))
}

async fn run_get(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    opts: &ContactsOpts,
    handle: &str,
    config_path: Option<&Path>,
) -> Result<String> {
    if endpoint.is_none() {
        let contact = get_via_broker(cfg, device, config_path, opts, handle).await?;
        return Ok(render_contact_view(&ContactView::from_dto(&contact), opts.raw));
    }
    let mut client = conn::connect_pbap(cfg, endpoint, device).await?;
    let result = session::contacts::get(&mut client, path_of(opts.path), handle).await;
    warn_on_disconnect(client.disconnect().await);
    Ok(render_contact_view(&ContactView::from_contact(&result?), opts.raw))
}

async fn run_list(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    opts: &ContactsOpts,
    config_path: Option<&Path>,
) -> Result<String> {
    let offset = offset_of(opts.limit, opts.page);
    if endpoint.is_none() {
        let entries = list_via_broker(cfg, device, config_path, opts, offset).await?;
        let views: Vec<EntryView> = entries.iter().map(EntryView::from_dto).collect();
        return Ok(render_entries_view(&views));
    }
    let mut client = conn::connect_pbap(cfg, endpoint, device).await?;
    let result = session::contacts::list(&mut client, path_of(opts.path), opts.limit, offset).await;
    warn_on_disconnect(client.disconnect().await);
    let entries = result?;
    let views: Vec<EntryView> = entries.iter().map(EntryView::from_entry).collect();
    Ok(render_entries_view(&views))
}

async fn run_pull_all(
    cfg: &Config,
    endpoint: Option<&Endpoint>,
    device: Option<&str>,
    opts: &ContactsOpts,
    config_path: Option<&Path>,
) -> Result<String> {
    let offset = offset_of(opts.limit, opts.page);
    let contacts = if endpoint.is_none() {
        pull_all_via_broker(cfg, device, config_path, opts, offset).await?
    } else {
        let mut client = conn::connect_pbap(cfg, endpoint, device).await?;
        let result =
            session::contacts::pull_all(&mut client, path_of(opts.path), opts.limit, offset).await;
        warn_on_disconnect(client.disconnect().await);
        return Ok(render_pulled(&result?, opts.raw));
    };
    Ok(render_dto_contacts(&contacts, opts.raw))
}

fn render_pulled(contacts: &[pbap_core::Contact], raw: bool) -> String {
    if contacts.is_empty() {
        return "(no contacts)".to_owned();
    }
    let views: Vec<ContactView> = contacts.iter().map(ContactView::from_contact).collect();
    render_contacts_view(&views, raw)
}

fn render_dto_contacts(contacts: &[ContactDto], raw: bool) -> String {
    if contacts.is_empty() {
        return "(no contacts)".to_owned();
    }
    let views: Vec<ContactView> = contacts.iter().map(ContactView::from_dto).collect();
    render_contacts_view(&views, raw)
}

/// Logs a PBAP disconnect failure as a warning; never propagated, matching every other live
/// command's teardown.
fn warn_on_disconnect(result: Result<(), pbap_core::PbapError>) {
    if let Err(e) = result {
        tracing::warn!("PBAP disconnect failed: {e}");
    }
}

/// Issues a [`BrokerRequest::LookupContact`] and unwraps the [`BrokerResponse::ContactLookup`]
/// frame.
async fn lookup_via_broker(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    opts: &ContactsOpts,
    number: &str,
) -> Result<Option<ContactDto>> {
    let req = BrokerRequest::LookupContact {
        path: Some(path_name(opts.path).to_owned()),
        number: number.to_owned(),
    };
    match broker::call(cfg, device, config_path, req).await? {
        BrokerResponse::ContactLookup(c) => Ok(c),
        BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
        BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
        other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
    }
}

/// Issues a [`BrokerRequest::GetContact`] and unwraps the [`BrokerResponse::Contact`] frame.
async fn get_via_broker(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    opts: &ContactsOpts,
    handle: &str,
) -> Result<ContactDto> {
    let req = BrokerRequest::GetContact {
        path: Some(path_name(opts.path).to_owned()),
        handle: handle.to_owned(),
    };
    match broker::call(cfg, device, config_path, req).await? {
        BrokerResponse::Contact(c) => Ok(c),
        BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
        BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
        other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
    }
}

/// Issues a [`BrokerRequest::ListContacts`] and unwraps the [`BrokerResponse::ContactEntries`]
/// frame.
async fn list_via_broker(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    opts: &ContactsOpts,
    offset: u16,
) -> Result<Vec<CardEntryDto>> {
    let req = BrokerRequest::ListContacts {
        path: Some(path_name(opts.path).to_owned()),
        limit: opts.limit,
        offset,
    };
    match broker::call(cfg, device, config_path, req).await? {
        BrokerResponse::ContactEntries(rows) => Ok(rows),
        BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
        BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
        other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
    }
}

/// Issues a [`BrokerRequest::PullAllContacts`] and unwraps the [`BrokerResponse::Contacts`]
/// frame.
async fn pull_all_via_broker(
    cfg: &Config,
    device: Option<&str>,
    config_path: Option<&Path>,
    opts: &ContactsOpts,
    offset: u16,
) -> Result<Vec<ContactDto>> {
    let req = BrokerRequest::PullAllContacts {
        path: Some(path_name(opts.path).to_owned()),
        limit: opts.limit,
        offset,
    };
    match broker::call(cfg, device, config_path, req).await? {
        BrokerResponse::Contacts(rows) => Ok(rows),
        BrokerResponse::Failed(reason) => Err(anyhow::anyhow!("{reason}")),
        BrokerResponse::Error(e) => Err(anyhow::anyhow!("{e}")),
        other => Err(anyhow::anyhow!("unexpected broker response: {other:?}")),
    }
}
