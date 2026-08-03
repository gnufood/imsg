use std::io;
use std::path::Path;

use obex_core::TransportError;
use zeroize::Zeroize;

use super::SecretKey;

/// Loads a persisted hub secret key from `path`, or generates, persists, and returns a new one.
///
/// The file holds exactly 32 raw ed25519 seed bytes. Parent directories are created when absent
/// and a freshly generated key is written with mode `0600` before returning. Does not verify
/// that `path` is on an encrypted volume — secure placement is the caller's responsibility.
///
/// # Errors
///
/// Returns [`TransportError::Io`] on directory creation, read, or write failure, and
/// [`TransportError::External`] if an existing key file is not exactly 32 bytes.
pub async fn load_or_create_key(path: &Path) -> Result<SecretKey, TransportError> {
    match tokio::fs::read(path).await {
        Ok(mut bytes) => {
            let seed_result = <[u8; 32]>::try_from(bytes.as_slice())
                .map_err(|_| TransportError::External("hub key file is not 32 bytes".into()));
            bytes.zeroize();
            let mut seed = seed_result?;
            let key = SecretKey::from_bytes(&seed);
            seed.zeroize();
            return Ok(key);
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let key = SecretKey::generate();
    let mut key_bytes = key.to_bytes();
    let write_result = tokio::fs::write(path, &key_bytes).await;
    key_bytes.zeroize();
    write_result?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).await?;
    }
    Ok(key)
}
