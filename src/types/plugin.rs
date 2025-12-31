//! Plugin configuration types for Claude Agent SDK
//!
//! Plugins allow you to extend Claude Code functionality with custom features,
//! tools, and integrations. This module provides types for configuring and
//! loading plugins from local paths.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Plugin configuration for extending Claude Code functionality
///
/// Plugins can be loaded from local filesystem paths and provide additional
/// tools, features, or integrations that Claude can use during execution.
///
/// # Examples
///
/// ```
/// use claude_agent_sdk_rs::SdkPluginConfig;
/// use std::path::PathBuf;
///
/// // Load a local plugin
/// let plugin = SdkPluginConfig::Local {
///     path: PathBuf::from("./my-plugin"),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum SdkPluginConfig {
    /// Local filesystem plugin
    ///
    /// Loads a plugin from a local directory path. The path should point to
    /// a valid Claude Code plugin directory structure.
    ///
    /// # Example
    ///
    /// ```
    /// use claude_agent_sdk_rs::SdkPluginConfig;
    /// use std::path::PathBuf;
    ///
    /// let plugin = SdkPluginConfig::Local {
    ///     path: PathBuf::from("~/.claude/plugins/my-plugin"),
    /// };
    /// ```
    Local {
        /// Path to the plugin directory
        path: PathBuf,
    },
}

impl SdkPluginConfig {
    /// Create a new local plugin configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use claude_agent_sdk_rs::SdkPluginConfig;
    ///
    /// let plugin = SdkPluginConfig::local("./my-plugin");
    /// ```
    pub fn local(path: impl Into<PathBuf>) -> Self {
        SdkPluginConfig::Local { path: path.into() }
    }

    /// Get the path for a local plugin
    ///
    /// Returns `Some(&PathBuf)` for local plugins, `None` for other types.
    ///
    /// # Examples
    ///
    /// ```
    /// use claude_agent_sdk_rs::SdkPluginConfig;
    ///
    /// let plugin = SdkPluginConfig::local("./my-plugin");
    /// assert!(plugin.path().is_some());
    /// ```
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            SdkPluginConfig::Local { path } => Some(path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_local_creation() {
        let plugin = SdkPluginConfig::local("/path/to/plugin");
        assert!(matches!(plugin, SdkPluginConfig::Local { .. }));
    }

    #[test]
    fn test_plugin_path_getter() {
        let plugin = SdkPluginConfig::local("/path/to/plugin");
        assert_eq!(plugin.path(), Some(&PathBuf::from("/path/to/plugin")));
    }

    #[test]
    fn test_plugin_serialization() {
        let plugin = SdkPluginConfig::Local {
            path: PathBuf::from("/test/path"),
        };

        let json = serde_json::to_value(&plugin).unwrap();
        assert_eq!(json["type"], "local");
        assert_eq!(json["path"], "/test/path");
    }

    #[test]
    fn test_plugin_deserialization() {
        let json = serde_json::json!({
            "type": "local",
            "path": "/test/path"
        });

        let plugin: SdkPluginConfig = serde_json::from_value(json).unwrap();
        assert_eq!(
            plugin,
            SdkPluginConfig::Local {
                path: PathBuf::from("/test/path")
            }
        );
    }

    #[test]
    fn test_plugin_roundtrip() {
        let original = SdkPluginConfig::local("~/.claude/plugins/test");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SdkPluginConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_plugin_with_relative_path() {
        let plugin = SdkPluginConfig::local("./plugins/my-plugin");
        assert_eq!(
            plugin.path().unwrap().to_str().unwrap(),
            "./plugins/my-plugin"
        );
    }

    #[test]
    fn test_plugin_with_home_directory() {
        let plugin = SdkPluginConfig::local("~/my-plugin");
        assert!(
            plugin
                .path()
                .unwrap()
                .to_str()
                .unwrap()
                .contains("my-plugin")
        );
    }
}
