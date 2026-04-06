use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackendStatus {
    runtime: String,
    project_name: String,
    entity_count: usize,
    endpoint_count: usize,
    plugin_count: usize,
}

#[tauri::command]
fn backend_status() -> BackendStatus {
    let graph = faero_core::ProjectGraph::new("Desktop Shell");
    let document = graph.document();

    BackendStatus {
        runtime: "tauri-rust".to_string(),
        project_name: graph.project_name().to_string(),
        entity_count: graph.entity_count(),
        endpoint_count: graph.endpoint_count(),
        plugin_count: document.plugin_manifests.len(),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![backend_status])
        .run(tauri::generate_context!())
        .expect("error while running FutureAero desktop shell");
}

