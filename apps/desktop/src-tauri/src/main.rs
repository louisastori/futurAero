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
    stream_count: usize,
    plugin_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FixtureProject {
    id: String,
    project_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDetails {
    project_id: String,
    format_version: String,
    default_frame: String,
    root_scene_id: Option<String>,
    active_configuration_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EntitySummary {
    id: String,
    entity_type: String,
    name: String,
    revision: String,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EndpointSummary {
    id: String,
    name: String,
    endpoint_type: String,
    transport_kind: String,
    mode: String,
    address: String,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamSummary {
    id: String,
    name: String,
    endpoint_id: String,
    stream_type: String,
    direction: String,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginSummary {
    plugin_id: String,
    version: String,
    enabled: bool,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivityEntry {
    id: String,
    channel: String,
    kind: String,
    timestamp: String,
    target_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectSnapshot {
    status: BackendStatus,
    details: ProjectDetails,
    entities: Vec<EntitySummary>,
    endpoints: Vec<EndpointSummary>,
    streams: Vec<StreamSummary>,
    plugins: Vec<PluginSummary>,
    recent_activity: Vec<ActivityEntry>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixtures_root() -> PathBuf {
    repo_root().join("examples/projects")
}

fn serialized_variant<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|json| json.as_str().map(str::to_owned))
        .unwrap_or_else(|| "unknown".to_string())
}

fn endpoint_address_label(
    host: Option<&str>,
    port: Option<u16>,
    path: Option<&str>,
    device_id: Option<&str>,
) -> String {
    if let Some(host) = host {
        return match (port, path) {
            (Some(port), Some(path)) => format!("{host}:{port}{path}"),
            (Some(port), None) => format!("{host}:{port}"),
            (None, Some(path)) => format!("{host}{path}"),
            (None, None) => host.to_string(),
        };
    }

    if let Some(device_id) = device_id {
        return device_id.to_string();
    }

    "n/a".to_string()
}

fn load_project_document(project_id: &str) -> Result<faero_types::ProjectDocument, String> {
    faero_storage::load_project(fixtures_root().join(project_id))
        .map_err(|error| format!("failed to load fixture `{project_id}`: {error}"))
}

fn status_from_document(
    project_id: &str,
    document: &faero_types::ProjectDocument,
) -> BackendStatus {
    BackendStatus {
        runtime: "tauri-rust".to_string(),
        fixture_id: project_id.to_string(),
        project_name: document.metadata.name.clone(),
        entity_count: document.nodes.len(),
        endpoint_count: document.endpoints.len(),
        stream_count: document.streams.len(),
        plugin_count: document.plugin_manifests.len(),
    }
}

fn build_backend_status(project_id: &str) -> Result<BackendStatus, String> {
    let document = load_project_document(project_id)?;
    Ok(status_from_document(project_id, &document))
}

fn build_project_snapshot(project_id: &str) -> Result<ProjectSnapshot, String> {
    let document = load_project_document(project_id)?;
    let status = status_from_document(project_id, &document);

    let details = ProjectDetails {
        project_id: document.metadata.project_id.clone(),
        format_version: document.metadata.format_version.clone(),
        default_frame: document.metadata.default_frame.clone(),
        root_scene_id: document.metadata.root_scene_id.clone(),
        active_configuration_id: document.metadata.active_configuration_id.clone(),
    };

    let entities = document
        .nodes
        .values()
        .map(|entity| EntitySummary {
            id: entity.id.clone(),
            entity_type: entity.entity_type.clone(),
            name: entity.name.clone(),
            revision: entity.revision.clone(),
            status: entity.status.clone(),
        })
        .collect::<Vec<_>>();

    let endpoints = document
        .endpoints
        .values()
        .map(|endpoint| EndpointSummary {
            id: endpoint.id.clone(),
            name: endpoint.name.clone(),
            endpoint_type: serialized_variant(&endpoint.endpoint_type),
            transport_kind: endpoint.transport_profile.transport_kind.clone(),
            mode: serialized_variant(&endpoint.mode),
            address: endpoint_address_label(
                endpoint.addressing.host.as_deref(),
                endpoint.addressing.port,
                endpoint.addressing.path.as_deref(),
                endpoint.addressing.device_id.as_deref(),
            ),
            status: endpoint.status.clone(),
        })
        .collect::<Vec<_>>();

    let streams = document
        .streams
        .values()
        .map(|stream| StreamSummary {
            id: stream.id.clone(),
            name: stream.name.clone(),
            endpoint_id: stream.endpoint_id.clone(),
            stream_type: stream.stream_type.clone(),
            direction: serialized_variant(&stream.direction),
            status: stream.status.clone(),
        })
        .collect::<Vec<_>>();

    let plugins = document
        .plugin_manifests
        .values()
        .map(|plugin| PluginSummary {
            plugin_id: plugin.plugin_id.clone(),
            version: plugin.version.clone(),
            enabled: *document
                .plugin_states
                .get(&plugin.plugin_id)
                .unwrap_or(&false),
            status: plugin.status.clone(),
        })
        .collect::<Vec<_>>();

    let mut recent_activity = document
        .commands
        .iter()
        .map(|command| ActivityEntry {
            id: command.command_id.clone(),
            channel: "command".to_string(),
            kind: command.kind.clone(),
            timestamp: command.timestamp.clone(),
            target_id: command.target_id.clone(),
        })
        .chain(document.events.iter().map(|event| ActivityEntry {
            id: event.event_id.clone(),
            channel: "event".to_string(),
            kind: event.kind.clone(),
            timestamp: event.timestamp.clone(),
            target_id: event.target_id.clone(),
        }))
        .collect::<Vec<_>>();
    recent_activity.sort_by(|left, right| {
        right
            .timestamp
            .cmp(&left.timestamp)
            .then_with(|| right.id.cmp(&left.id))
    });
    recent_activity.truncate(8);

    Ok(ProjectSnapshot {
        status,
        details,
        entities,
        endpoints,
        streams,
        plugins,
        recent_activity,
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
        stream_count: 0,
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

#[tauri::command]
fn load_project_snapshot(project_id: String) -> Result<ProjectSnapshot, String> {
    build_project_snapshot(&project_id)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            backend_status,
            available_fixture_projects,
            load_fixture_project,
            load_project_snapshot
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
        assert_eq!(status.stream_count, 1);
        assert_eq!(status.plugin_count, 1);
    }

    #[test]
    fn project_snapshot_exposes_workspace_entities_and_activity() {
        let snapshot = build_project_snapshot(DEFAULT_FIXTURE_ID).expect("snapshot should load");

        assert_eq!(snapshot.details.project_id, "prj_1001");
        assert_eq!(snapshot.entities.len(), 2);
        assert_eq!(snapshot.endpoints.len(), 1);
        assert_eq!(snapshot.streams.len(), 1);
        assert_eq!(snapshot.plugins.len(), 1);
        assert_eq!(snapshot.endpoints[0].transport_kind, "robot_controller");
        assert!(snapshot.plugins[0].enabled);
        assert!(!snapshot.recent_activity.is_empty());
    }
}
