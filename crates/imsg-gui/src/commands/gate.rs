//! `#[tauri::command]` shims over the managed [`GateState`] — the startup gate's entire
//! frontend surface.
//!
//! The frontend can read where the gate is and poke it to re-evaluate; sequencing itself lives
//! in `crate::gate::run`, spawned once by `main.rs`. Both shims take `State` by value —
//! `tauri::command`'s argument extraction requires it, so clippy's `needless_pass_by_value` is
//! allowed here, not fixable.

use tauri::State;

use crate::gate::{GateState, GateStatus};

/// Returns the startup gate's current status. Authoritative at any time — poll until
/// [`GateStatus::Ready`].
#[allow(clippy::needless_pass_by_value)]
#[must_use]
#[tauri::command]
#[specta::specta]
pub fn gate_status(state: State<'_, GateState>) -> GateStatus {
    state.status()
}

/// Wakes a parked startup gate — call after persisting device config, or as the retry action
/// on a [`GateStatus::Failed`]. Carries no sequencing power: the gate re-derives everything
/// from ground truth on each pass, so stray or repeated calls are harmless.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
#[specta::specta]
pub fn gate_proceed(state: State<'_, GateState>) {
    state.proceed();
}

#[cfg(test)]
mod tests;
