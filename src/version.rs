//! Version information for the Claude Agent SDK

/// The version of this SDK
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Minimum required Claude Code CLI version
pub const MIN_CLI_VERSION: &str = "2.0.0";

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
        assert!(check_version("2.0.0"));
        assert!(check_version("2.0.1"));
        assert!(check_version("2.1.0"));
        assert!(check_version("3.0.0"));
        assert!(!check_version("1.9.9"));
        assert!(!check_version("1.99.99"));
    }
}
