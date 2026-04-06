use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

const DEFAULT_FIXTURE_ID: &str = "pick-and-place-demo.faero";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackendStatus {
    runtime: String,
    fixture_id: String,
    project_name: String,
    entity_count: usize,
    endpoint_count: usize,
    plugin_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FixtureProject {
    id: String,
    project_name: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixtures_root() -> PathBuf {
    repo_root().join("examples/projects")
}

fn build_backend_status(project_id: &str) -> Result<BackendStatus, String> {
    let document = faero_storage::load_project(fixtures_root().join(project_id))
        .map_err(|error| format!("failed to load fixture `{project_id}`: {error}"))?;

    Ok(BackendStatus {
        runtime: "tauri-rust".to_string(),
        fixture_id: project_id.to_string(),
        project_name: document.metadata.name,
        entity_count: document.nodes.len(),
        endpoint_count: document.endpoints.len(),
        plugin_count: document.plugin_manifests.len(),
    })
}

#[tauri::command]
fn backend_status() -> BackendStatus {
    build_backend_status(DEFAULT_FIXTURE_ID).unwrap_or(BackendStatus {
        runtime: "tauri-rust".to_string(),
        fixture_id: DEFAULT_FIXTURE_ID.to_string(),
        project_name: "FutureAero Desktop Shell".to_string(),
        entity_count: 0,
        endpoint_count: 0,
        plugin_count: 0,
    })
}

#[tauri::command]
fn available_fixture_projects() -> Result<Vec<FixtureProject>, String> {
    let root = fixtures_root();
    let mut entries = fs::read_dir(&root)
        .map_err(|error| format!("failed to read fixture root `{}`: {error}", root.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "failed to enumerate fixture root `{}`: {error}",
                root.display()
            )
        })?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut fixtures = Vec::new();
    for entry in entries {
        let file_type = entry.file_type().map_err(|error| {
            format!(
                "failed to read fixture entry type `{}`: {error}",
                entry.path().display()
            )
        })?;
        if !file_type.is_dir() {
            continue;
        }

        let fixture_id = entry.file_name().to_string_lossy().to_string();
        let document = faero_storage::load_project(entry.path())
            .map_err(|error| format!("failed to index fixture `{fixture_id}`: {error}"))?;

        fixtures.push(FixtureProject {
            id: fixture_id,
            project_name: document.metadata.name,
        });
    }

    Ok(fixtures)
}

#[tauri::command]
fn load_fixture_project(project_id: String) -> Result<BackendStatus, String> {
    build_backend_status(&project_id)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            backend_status,
            available_fixture_projects,
            load_fixture_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running FutureAero desktop shell");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_fixture_projects_from_examples_directory() {
        let fixtures = available_fixture_projects().expect("fixtures should index");

        assert!(
            fixtures
                .iter()
                .any(|fixture| fixture.id == "empty-project.faero")
        );
        assert!(
            fixtures
                .iter()
                .any(|fixture| fixture.id == "pick-and-place-demo.faero")
        );
    }

    #[test]
    fn loads_pick_and_place_fixture_into_backend_status() {
        let status = build_backend_status(DEFAULT_FIXTURE_ID).expect("fixture should load");

        assert_eq!(status.runtime, "tauri-rust");
        assert_eq!(status.fixture_id, DEFAULT_FIXTURE_ID);
        assert_eq!(status.project_name, "Pick And Place Demo");
        assert_eq!(status.entity_count, 2);
        assert_eq!(status.endpoint_count, 1);
        assert_eq!(status.plugin_count, 1);
    }
}
