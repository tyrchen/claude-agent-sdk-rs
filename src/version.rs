//! Version information for the Claude Agent SDK

use std::sync::OnceLock;

/// The version of this SDK
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cached Claude Code CLI version
static CLAUDE_CODE_VERSION: OnceLock<Option<String>> = OnceLock::new();

/// Get the Claude Code CLI version.
///
/// This function caches the result using `OnceLock`, so the CLI is only invoked once.
/// Returns `None` if the CLI is not found or the version cannot be determined.
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk_rs::version::get_claude_code_version;
///
/// if let Some(version) = get_claude_code_version() {
///     println!("Claude Code CLI version: {}", version);
/// } else {
///     println!("Claude Code CLI not found");
/// }
/// ```
pub fn get_claude_code_version() -> Option<&'static str> {
    CLAUDE_CODE_VERSION
        .get_or_init(|| {
            std::process::Command::new("claude")
                .arg("--version")
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    version_output
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().next())
                        .map(|v| v.trim().to_string())
                })
        })
        .as_deref()
}

/// Minimum required Claude Code CLI version
///
/// Version 2.1.30+ is required for proper stream-json protocol support.
/// Earlier versions (e.g., 2.1.19) have protocol compatibility issues that cause
/// initialization hangs when using the SDK.
pub const MIN_CLI_VERSION: &str = "2.1.30";

/// Environment variable to skip version check
pub const SKIP_VERSION_CHECK_ENV: &str = "CLAUDE_AGENT_SDK_SKIP_VERSION_CHECK";

/// Entrypoint identifier for subprocess
pub const ENTRYPOINT: &str = "sdk-rs";

/// Parse a semantic version string into (major, minor, patch)
pub fn parse_version(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.trim_start_matches('v').split('.').collect();
    if parts.len() < 3 {
        return None;
    }

    let major = parts[0].parse().ok()?;
    let minor = parts[1].parse().ok()?;
    let patch = parts[2].parse().ok()?;

    Some((major, minor, patch))
}

/// Check if the CLI version meets the minimum requirement
pub fn check_version(cli_version: &str) -> bool {
    let Some((cli_maj, cli_min, cli_patch)) = parse_version(cli_version) else {
        return false;
    };

    let Some((req_maj, req_min, req_patch)) = parse_version(MIN_CLI_VERSION) else {
        return false;
    };

    if cli_maj > req_maj {
        return true;
    }
    if cli_maj < req_maj {
        return false;
    }

    // Major versions are equal
    if cli_min > req_min {
        return true;
    }
    if cli_min < req_min {
        return false;
    }

    // Major and minor are equal
    cli_patch >= req_patch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("10.20.30"), Some((10, 20, 30)));
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("invalid"), None);
    }

    #[test]
    fn test_check_version() {
        // Versions at or above MIN_CLI_VERSION (2.1.30) should pass
        assert!(check_version("2.1.30"));
        assert!(check_version("2.1.31"));
        assert!(check_version("2.2.0"));
        assert!(check_version("3.0.0"));
        // Versions below MIN_CLI_VERSION should fail
        assert!(!check_version("2.1.29"));
        assert!(!check_version("2.1.20"));
        assert!(!check_version("2.1.19"));
        assert!(!check_version("2.0.0"));
        assert!(!check_version("1.9.9"));
    }
}
