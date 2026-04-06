use std::collections::{BTreeMap, BTreeSet};

use faero_types::PluginManifest;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PluginHostError {
    #[error("plugin `{0}` requests unknown permission `{1}`")]
    UnknownPermission(String, String),
    #[error("plugin `{0}` is already installed")]
    AlreadyInstalled(String),
    #[error("plugin `{0}` is not installed")]
    NotInstalled(String),
}

#[derive(Debug, Default)]
pub struct PluginRegistry {
    installed: BTreeMap<String, PluginManifest>,
    enabled: BTreeSet<String>,
}

impl PluginRegistry {
    pub fn install(&mut self, manifest: PluginManifest) -> Result<(), PluginHostError> {
        validate_manifest(&manifest)?;
        if self.installed.contains_key(&manifest.plugin_id) {
            return Err(PluginHostError::AlreadyInstalled(manifest.plugin_id));
        }
        self.installed.insert(manifest.plugin_id.clone(), manifest);
        Ok(())
    }

    pub fn enable(&mut self, plugin_id: &str) -> Result<(), PluginHostError> {
        if !self.installed.contains_key(plugin_id) {
            return Err(PluginHostError::NotInstalled(plugin_id.to_string()));
        }
        self.enabled.insert(plugin_id.to_string());
        Ok(())
    }

    pub fn disable(&mut self, plugin_id: &str) -> Result<(), PluginHostError> {
        if !self.installed.contains_key(plugin_id) {
            return Err(PluginHostError::NotInstalled(plugin_id.to_string()));
        }
        self.enabled.remove(plugin_id);
        Ok(())
    }

    pub fn is_enabled(&self, plugin_id: &str) -> bool {
        self.enabled.contains(plugin_id)
    }

    pub fn installed_count(&self) -> usize {
        self.installed.len()
    }
}

pub fn allowed_permissions() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "project.read",
        "project.write",
        "integration.observe",
        "integration.control",
        "plugin.ui.mount",
    ])
}

pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), PluginHostError> {
    let allowed = allowed_permissions();
    for permission in &manifest.permissions {
        if !allowed.contains(permission.as_str()) {
            return Err(PluginHostError::UnknownPermission(
                manifest.plugin_id.clone(),
                permission.clone(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest(permissions: Vec<&str>) -> PluginManifest {
        PluginManifest {
            id: "ent_plugin_001".to_string(),
            plugin_id: "plg.integration.viewer".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec!["panel".to_string()],
            permissions: permissions.into_iter().map(str::to_string).collect(),
            entrypoints: vec!["plugins/integration-viewer/index.js".to_string()],
            compatibility: vec!["faero-core@0.1".to_string()],
            status: "installed".to_string(),
        }
    }

    #[test]
    fn rejects_unknown_permissions() {
        let error = validate_manifest(&sample_manifest(vec!["project.read", "system.shell"]))
            .expect_err("unknown permission should be rejected");

        assert_eq!(
            error,
            PluginHostError::UnknownPermission(
                "plg.integration.viewer".to_string(),
                "system.shell".to_string()
            )
        );
    }

    #[test]
    fn install_enable_disable_round_trip() {
        let mut registry = PluginRegistry::default();
        registry
            .install(sample_manifest(vec!["project.read", "integration.observe"]))
            .expect("manifest should install");
        registry
            .enable("plg.integration.viewer")
            .expect("plugin should enable");
        assert!(registry.is_enabled("plg.integration.viewer"));

        registry
            .disable("plg.integration.viewer")
            .expect("plugin should disable");
        assert!(!registry.is_enabled("plg.integration.viewer"));
        assert_eq!(registry.installed_count(), 1);
    }
}
