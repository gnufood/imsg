//! Stdout writing that exits cleanly on a closed pipe instead of panicking.

use std::io::{self, Write as _};

use anyhow::Result;

/// When the downstream reader has closed the pipe (e.g. `imsg … | head`), exits the process
/// with status 0 rather than panicking.
///
/// # Errors
///
/// Returns an error for any stdout write failure other than a broken pipe.
pub(crate) fn line(text: &str) -> Result<()> {
    match writeln!(io::stdout(), "{text}") {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => std::process::exit(0),
        Err(e) => Err(e.into()),
    }
}
