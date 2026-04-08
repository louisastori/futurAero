use std::collections::{BTreeMap, BTreeSet};

use faero_types::{PluginContribution, PluginManifest};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PluginHostError {
    #[error("plugin `{0}` requests unknown permission `{1}`")]
    UnknownPermission(String, String),
    #[error("plugin `{0}` contributes unknown kind `{1}`")]
    UnknownContributionKind(String, String),
    #[error("plugin `{0}` is already installed")]
    AlreadyInstalled(String),
    #[error("plugin `{0}` is not installed")]
    NotInstalled(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginAuditStatus {
    pub plugin_id: String,
    pub enabled: bool,
    pub release_channel: String,
    pub permission_count: usize,
    pub contribution_count: usize,
    pub signature_present: bool,
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

    pub fn audit_statuses(&self) -> Vec<PluginAuditStatus> {
        self.installed
            .values()
            .map(|manifest| PluginAuditStatus {
                plugin_id: manifest.plugin_id.clone(),
                enabled: self.enabled.contains(&manifest.plugin_id),
                release_channel: manifest.release_channel.clone(),
                permission_count: manifest.permissions.len(),
                contribution_count: manifest.contributions.len(),
                signature_present: manifest.signature.is_some(),
            })
            .collect()
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

pub fn allowed_contribution_kinds() -> BTreeSet<&'static str> {
    BTreeSet::from(["panel", "command", "inspector_section", "report_view"])
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
    let allowed_contributions = allowed_contribution_kinds();
    for contribution in &manifest.contributions {
        validate_contribution(manifest, contribution, &allowed_contributions)?;
    }
    Ok(())
}

fn validate_contribution(
    manifest: &PluginManifest,
    contribution: &PluginContribution,
    allowed_contributions: &BTreeSet<&'static str>,
) -> Result<(), PluginHostError> {
    if !allowed_contributions.contains(contribution.kind.as_str()) {
        return Err(PluginHostError::UnknownContributionKind(
            manifest.plugin_id.clone(),
            contribution.kind.clone(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest(permissions: Vec<&str>) -> PluginManifest {
        let mut permission_values = Vec::with_capacity(permissions.len());
        for permission in permissions {
            permission_values.push(permission.to_string());
        }

        PluginManifest {
            id: "ent_plugin_001".to_string(),
            plugin_id: "plg.integration.viewer".to_string(),
            version: "0.1.0".to_string(),
            release_channel: "stable".to_string(),
            capabilities: vec!["panel".to_string()],
            permissions: permission_values,
            contributions: vec![PluginContribution {
                kind: "panel".to_string(),
                target: "workspace.right".to_string(),
                title: "Integration Viewer".to_string(),
            }],
            entrypoints: vec!["plugins/integration-viewer/index.js".to_string()],
            compatibility: vec!["faero-core@0.1".to_string()],
            signature: Some("sha256:demo".to_string()),
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

    #[test]
    fn rejects_duplicate_install_and_missing_enable_disable() {
        let mut registry = PluginRegistry::default();
        let manifest = sample_manifest(vec!["project.read"]);

        registry
            .install(manifest.clone())
            .expect("first install should succeed");

        let duplicate = registry
            .install(manifest)
            .expect_err("duplicate install should fail");
        assert_eq!(
            duplicate,
            PluginHostError::AlreadyInstalled("plg.integration.viewer".to_string())
        );

        let missing_enable = PluginRegistry::default()
            .enable("missing.plugin")
            .expect_err("missing plugin enable should fail");
        assert_eq!(
            missing_enable,
            PluginHostError::NotInstalled("missing.plugin".to_string())
        );

        let missing_disable = PluginRegistry::default()
            .disable("missing.plugin")
            .expect_err("missing plugin disable should fail");
        assert_eq!(
            missing_disable,
            PluginHostError::NotInstalled("missing.plugin".to_string())
        );
    }

    #[test]
    fn allowed_permissions_contains_expected_runtime_scope() {
        let permissions = allowed_permissions();

        assert!(permissions.contains("project.read"));
        assert!(permissions.contains("project.write"));
        assert!(permissions.contains("integration.observe"));
        assert!(permissions.contains("integration.control"));
        assert!(permissions.contains("plugin.ui.mount"));
    }

    #[test]
    fn allowed_contributions_contains_expected_kinds() {
        let contributions = allowed_contribution_kinds();

        assert!(contributions.contains("panel"));
        assert!(contributions.contains("command"));
        assert!(contributions.contains("inspector_section"));
        assert!(contributions.contains("report_view"));
    }

    #[test]
    fn accepts_manifests_without_explicit_permissions() {
        let manifest = sample_manifest(Vec::new());
        assert_eq!(validate_manifest(&manifest), Ok(()));
    }

    #[test]
    fn install_rejects_invalid_permissions_before_registering_plugin() {
        let mut registry = PluginRegistry::default();
        let invalid_manifest = sample_manifest(vec!["system.shell"]);

        let _error = registry
            .install(invalid_manifest)
            .expect_err("invalid permissions should block installation");
        assert_eq!(registry.installed_count(), 0);
    }

    #[test]
    fn rejects_unknown_contribution_kind_and_reports_audit_status() {
        let invalid = PluginManifest {
            contributions: vec![PluginContribution {
                kind: "webhook".to_string(),
                target: "workspace.right".to_string(),
                title: "Webhook".to_string(),
            }],
            ..sample_manifest(vec!["project.read"])
        };
        let error = validate_manifest(&invalid).expect_err("invalid contribution should fail");
        assert_eq!(
            error,
            PluginHostError::UnknownContributionKind(
                "plg.integration.viewer".to_string(),
                "webhook".to_string()
            )
        );

        let mut registry = PluginRegistry::default();
        registry
            .install(sample_manifest(vec!["project.read", "plugin.ui.mount"]))
            .expect("manifest should install");
        registry
            .enable("plg.integration.viewer")
            .expect("plugin should enable");
        let audits = registry.audit_statuses();
        assert_eq!(audits.len(), 1);
        assert!(audits[0].enabled);
        assert_eq!(audits[0].release_channel, "stable");
        assert_eq!(audits[0].contribution_count, 1);
        assert!(audits[0].signature_present);
    }
}
