//! Recorders for testing hooks and permissions

mod hooks;
mod permissions;

pub use hooks::{HookInvocation, HookRecorder};
pub use permissions::{PermissionDecision, PermissionRecorder};
