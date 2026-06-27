//! OS keyring access for imsg via the Secret Service D-Bus API.
//!
//! Provides get-or-create of the 256-bit database encryption key, returned as
//! an in-memory secret. Isolated from `imsg-config` so the Secret Service and
//! D-Bus dependency tree does not leak into the common config crate.

use keyring_core::{set_default_store, Entry};
use secrecy::{ExposeSecret, SecretBox};
use thiserror::Error;
use zbus_secret_service_keyring_store::Store;

const SERVICE: &str = "imsg";
const ACCOUNT: &str = "database-encryption-key";

/// Proof that [`init_store`] completed successfully.
///
/// Required by [`get_or_create_db_key`] to enforce call ordering at compile time.
/// Cannot be constructed outside this crate.
pub struct KeyringReady(());

/// Secret Service init, keyring entry I/O, corrupt key length, or CSPRNG failure.
#[derive(Debug, Error)]
pub enum Error {
    /// The Secret Service store could not be initialized; D-Bus may be unavailable.
    #[error("Secret Service store init failed: {0}")]
    StoreInit(keyring_core::Error),
    /// Underlying Secret Service get/set on the credential failed.
    #[error("keyring entry error: {0}")]
    Entry(keyring_core::Error),
    /// The stored credential is not 32 bytes; it may be corrupt or from a different version.
    #[error("stored key is {0} bytes, expected 32")]
    BadLength(usize),
    /// OS CSPRNG unavailable during first-run key creation.
    #[error("key generation failed: {0}")]
    Generate(getrandom::Error),
}

/// Initializes the zbus Secret Service store as the process-wide keyring backend.
///
/// Must be called once before [`get_or_create_db_key`]. Requires a running
/// Secret Service daemon (GNOME Keyring or `KWallet`) accessible over D-Bus.
///
/// # Errors
///
/// Returns [`Error::StoreInit`] if the Secret Service D-Bus connection fails.
pub fn init_store() -> Result<KeyringReady, Error> {
    let store = Store::new().map_err(Error::StoreInit)?;
    set_default_store(store);
    Ok(KeyringReady(()))
}

/// Returns the 256-bit database encryption key, creating and persisting it if absent.
///
/// Requires a [`KeyringReady`] token produced by [`init_store`], enforcing correct
/// call ordering at compile time.
/// On first call the key is generated via the OS CSPRNG and written to the Secret Service.
/// On subsequent calls the stored bytes are retrieved and returned directly.
///
/// # Errors
///
/// Returns [`Error::Entry`] on keyring read/write failure, [`Error::BadLength`] if the
/// stored credential is not exactly 32 bytes, or [`Error::Generate`] if the OS CSPRNG
/// is unavailable during key creation.
pub fn get_or_create_db_key(_ready: &KeyringReady) -> Result<SecretBox<[u8; 32]>, Error> {
    let entry = Entry::new(SERVICE, ACCOUNT).map_err(Error::Entry)?;
    match entry.get_secret() {
        Ok(bytes) => bytes_to_secret(bytes),
        Err(keyring_core::Error::NoEntry) => create_and_store_key(&entry),
        Err(e) => Err(Error::Entry(e)),
    }
}

fn bytes_to_secret(bytes: Vec<u8>) -> Result<SecretBox<[u8; 32]>, Error> {
    let key: [u8; 32] = bytes.try_into().map_err(|v: Vec<u8>| Error::BadLength(v.len()))?;
    Ok(SecretBox::new(Box::new(key)))
}

fn create_and_store_key(entry: &Entry) -> Result<SecretBox<[u8; 32]>, Error> {
    let secret = SecretBox::<[u8; 32]>::try_init_with(|| -> Result<_, Error> {
        let mut key = [0u8; 32];
        getrandom::fill(&mut key).map_err(Error::Generate)?;
        Ok(key)
    })?;
    entry.set_secret(secret.expose_secret()).map_err(Error::Entry)?;
    Ok(secret)
}
