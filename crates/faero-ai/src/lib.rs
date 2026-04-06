use std::{env, time::Duration};

use faero_types::{ExternalEndpoint, ProjectDocument};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434";
const DEFAULT_TIMEOUT_SECS: u64 = 45;
const DEFAULT_PREFERRED_MODELS: &[&str] = &["gemma3:4b", "phi3:mini"];
const MAX_HISTORY_MESSAGES: usize = 8;
const MAX_CONTEXT_REFERENCES: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiRuntimeStatus {
    pub available: bool,
    pub provider: String,
    pub endpoint: String,
    pub mode: String,
    pub local_only: bool,
    pub active_model: Option<String>,
    pub available_models: Vec<String>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiConversationMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiChatResponse {
    pub answer: String,
    pub runtime: AiRuntimeStatus,
    pub references: Vec<String>,
    pub warnings: Vec<String>,
    pub source: String,
}

#[derive(Debug, Error)]
pub enum AiError {
    #[error("empty chat message")]
    EmptyMessage,
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Clone)]
struct AiRuntimeConfig {
    endpoint: String,
    timeout_secs: u64,
    preferred_models: Vec<String>,
}

impl AiRuntimeConfig {
    fn from_env() -> Self {
        let endpoint = env::var("FUTUREAERO_OLLAMA_ENDPOINT")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_OLLAMA_ENDPOINT.to_string());
        let timeout_secs = env::var("FUTUREAERO_OLLAMA_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        let mut preferred_models = Vec::new();
        if let Some(model) = env::var("FUTUREAERO_OLLAMA_MODEL")
            .ok()
            .filter(|value| !value.trim().is_empty())
        {
            preferred_models.push(model);
        }
        preferred_models.extend(
            DEFAULT_PREFERRED_MODELS
                .iter()
                .map(|model| (*model).to_string()),
        );
        preferred_models = dedupe_preserving_order(preferred_models);

        Self {
            endpoint,
            timeout_secs,
            preferred_models,
        }
    }
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    #[serde(default)]
    models: Vec<OllamaModelTag>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelTag {
    name: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    stream: bool,
    messages: Vec<OllamaChatMessage>,
    options: OllamaChatOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatOptions {
    temperature: f32,
    top_p: f32,
    num_ctx: usize,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaChatMessage,
}

pub fn query_runtime_status() -> AiRuntimeStatus {
    let config = AiRuntimeConfig::from_env();
    let client = match build_http_client(config.timeout_secs) {
        Ok(client) => client,
        Err(error) => {
            return unavailable_runtime_status(
                &config,
                format!("client init failed: {error}"),
                Vec::new(),
            );
        }
    };

    match fetch_model_names(&client, &config) {
        Ok(models) => runtime_status_from_models(&config, models),
        Err(error) => unavailable_runtime_status(&config, error.to_string(), Vec::new()),
    }
}

pub fn chat_with_project(
    document: &ProjectDocument,
    locale: &str,
    history: &[AiConversationMessage],
    message: &str,
) -> Result<AiChatResponse, AiError> {
    let trimmed_message = message.trim();
    if trimmed_message.is_empty() {
        return Err(AiError::EmptyMessage);
    }

    let config = AiRuntimeConfig::from_env();
    let client = build_http_client(config.timeout_secs)?;
    let references = collect_context_references(document);

    let response = match fetch_model_names(&client, &config) {
        Ok(models) => {
            let runtime = runtime_status_from_models(&config, models.clone());
            if let Some(model) = runtime.active_model.clone() {
                match send_ollama_chat(
                    &client,
                    &config,
                    &model,
                    locale,
                    history,
                    trimmed_message,
                    document,
                ) {
                    Ok(answer) => AiChatResponse {
                        answer,
                        runtime,
                        references,
                        warnings: Vec::new(),
                        source: "ollama-local".to_string(),
                    },
                    Err(error) => fallback_response(
                        locale,
                        trimmed_message,
                        document,
                        references,
                        unavailable_runtime_status(&config, error.to_string(), models),
                    ),
                }
            } else {
                fallback_response(
                    locale,
                    trimmed_message,
                    document,
                    references,
                    unavailable_runtime_status(
                        &config,
                        "no preferred local model found".to_string(),
                        models,
                    ),
                )
            }
        }
        Err(error) => fallback_response(
            locale,
            trimmed_message,
            document,
            references,
            unavailable_runtime_status(&config, error.to_string(), Vec::new()),
        ),
    };

    Ok(response)
}

fn build_http_client(timeout_secs: u64) -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
}

fn fetch_model_names(client: &Client, config: &AiRuntimeConfig) -> Result<Vec<String>, AiError> {
    let response = client
        .get(format!("{}/api/tags", config.endpoint))
        .send()?
        .error_for_status()?
        .json::<OllamaTagsResponse>()?;

    Ok(response
        .models
        .into_iter()
        .map(|entry| entry.name)
        .collect())
}

fn runtime_status_from_models(config: &AiRuntimeConfig, models: Vec<String>) -> AiRuntimeStatus {
    let active_model = select_default_model(&models, &config.preferred_models);
    AiRuntimeStatus {
        available: !models.is_empty(),
        provider: "ollama".to_string(),
        endpoint: config.endpoint.clone(),
        mode: "grounded-chat".to_string(),
        local_only: true,
        active_model,
        available_models: models,
        warning: None,
    }
}

fn unavailable_runtime_status(
    config: &AiRuntimeConfig,
    warning: String,
    available_models: Vec<String>,
) -> AiRuntimeStatus {
    AiRuntimeStatus {
        available: false,
        provider: "ollama".to_string(),
        endpoint: config.endpoint.clone(),
        mode: "fallback-local".to_string(),
        local_only: true,
        active_model: select_default_model(&available_models, &config.preferred_models),
        available_models,
        warning: Some(warning),
    }
}

fn select_default_model(models: &[String], preferred_models: &[String]) -> Option<String> {
    preferred_models
        .iter()
        .find(|preferred| models.iter().any(|model| model == *preferred))
        .cloned()
        .or_else(|| models.first().cloned())
}

fn send_ollama_chat(
    client: &Client,
    config: &AiRuntimeConfig,
    model: &str,
    locale: &str,
    history: &[AiConversationMessage],
    message: &str,
    document: &ProjectDocument,
) -> Result<String, AiError> {
    let request = OllamaChatRequest {
        model: model.to_string(),
        stream: false,
        messages: build_ollama_messages(locale, history, message, document),
        options: OllamaChatOptions {
            temperature: 0.2,
            top_p: 0.9,
            num_ctx: 8_192,
        },
    };

    let response = client
        .post(format!("{}/api/chat", config.endpoint))
        .json(&request)
        .send()?
        .error_for_status()?
        .json::<OllamaChatResponse>()?;

    Ok(normalize_answer(response.message.content, locale, document))
}

fn build_ollama_messages(
    locale: &str,
    history: &[AiConversationMessage],
    message: &str,
    document: &ProjectDocument,
) -> Vec<OllamaChatMessage> {
    let mut messages = vec![OllamaChatMessage {
        role: "system".to_string(),
        content: build_system_prompt(locale, document),
    }];

    messages.extend(
        trim_history(history)
            .into_iter()
            .map(|entry| OllamaChatMessage {
                role: entry.role,
                content: entry.content,
            }),
    );
    messages.push(OllamaChatMessage {
        role: "user".to_string(),
        content: message.to_string(),
    });
    messages
}

fn build_system_prompt(locale: &str, document: &ProjectDocument) -> String {
    format!(
        "You are FutureAero Local AI, a local-only assistant for CAD, robotics, simulation, commissioning, integration and safety engineering.\n\
Use only the provided project context.\n\
Do not pretend to have internet, cloud or hidden tool access.\n\
If the context does not contain an answer, say so clearly.\n\
Do not suggest silent mutations.\n\
Keep the answer short, concrete and engineering-focused.\n\
When you mention an object, include its id when useful.\n\
Answer {}.\n\n\
Project context:\n{}",
        language_instruction(locale),
        build_project_summary(document)
    )
}

fn language_instruction(locale: &str) -> &'static str {
    match locale {
        "en" => "in English",
        "es" => "in Spanish",
        _ => "in French",
    }
}

fn build_project_summary(document: &ProjectDocument) -> String {
    let entity_summary = summarize_named_items(
        document
            .nodes
            .values()
            .map(|entity| format!("{} [{}] ({})", entity.name, entity.entity_type, entity.id)),
    );
    let endpoint_summary = summarize_named_items(document.endpoints.values().map(format_endpoint));
    let stream_summary = summarize_named_items(document.streams.values().map(|stream| {
        format!(
            "{} [{}] on {}",
            stream.name,
            stream.direction.as_str(),
            stream.endpoint_id
        )
    }));
    let plugin_summary = summarize_named_items(document.plugin_manifests.values().map(|plugin| {
        let enabled = document
            .plugin_states
            .get(&plugin.plugin_id)
            .copied()
            .unwrap_or(false);
        format!(
            "{} v{} [{}]",
            plugin.plugin_id,
            plugin.version,
            if enabled { "enabled" } else { "disabled" }
        )
    }));

    format!(
        "projectName: {}\nprojectId: {}\nfixtureUnits: {} / {} / {}\nentities({}): {}\nendpoints({}): {}\nstreams({}): {}\nplugins({}): {}\ncommandsRecorded: {}\neventsRecorded: {}",
        document.metadata.name,
        document.metadata.project_id,
        document.metadata.display_units.length,
        document.metadata.display_units.angle,
        document.metadata.display_units.mass,
        document.nodes.len(),
        entity_summary,
        document.endpoints.len(),
        endpoint_summary,
        document.streams.len(),
        stream_summary,
        document.plugin_manifests.len(),
        plugin_summary,
        document.commands.len(),
        document.events.len(),
    )
}

fn summarize_named_items(items: impl Iterator<Item = String>) -> String {
    let values = items.take(MAX_CONTEXT_REFERENCES).collect::<Vec<_>>();
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join("; ")
    }
}

fn format_endpoint(endpoint: &ExternalEndpoint) -> String {
    let address = endpoint
        .addressing
        .host
        .as_ref()
        .map(
            |host| match (endpoint.addressing.port, endpoint.addressing.path.as_ref()) {
                (Some(port), Some(path)) => format!("{host}:{port}{path}"),
                (Some(port), None) => format!("{host}:{port}"),
                (None, Some(path)) => format!("{host}{path}"),
                (None, None) => host.clone(),
            },
        )
        .or_else(|| endpoint.addressing.device_id.clone())
        .unwrap_or_else(|| "n/a".to_string());

    format!(
        "{} [{}] ({}) @ {}",
        endpoint.name,
        endpoint.endpoint_type.as_str(),
        endpoint.id,
        address
    )
}

fn trim_history(history: &[AiConversationMessage]) -> Vec<AiConversationMessage> {
    history
        .iter()
        .filter(|entry| matches!(entry.role.as_str(), "user" | "assistant"))
        .filter_map(|entry| {
            let content = entry.content.trim();
            if content.is_empty() {
                None
            } else {
                Some(AiConversationMessage {
                    role: entry.role.clone(),
                    content: content.to_string(),
                })
            }
        })
        .rev()
        .take(MAX_HISTORY_MESSAGES)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn collect_context_references(document: &ProjectDocument) -> Vec<String> {
    let mut references = vec![format!("project:{}", document.metadata.project_id)];
    references.extend(
        document
            .nodes
            .keys()
            .take(3)
            .map(|id| format!("entity:{id}")),
    );
    references.extend(
        document
            .endpoints
            .keys()
            .take(2)
            .map(|id| format!("endpoint:{id}")),
    );
    references.extend(
        document
            .streams
            .keys()
            .take(2)
            .map(|id| format!("stream:{id}")),
    );
    references.extend(
        document
            .plugin_manifests
            .keys()
            .take(1)
            .map(|id| format!("plugin:{id}")),
    );
    dedupe_preserving_order(references)
        .into_iter()
        .take(MAX_CONTEXT_REFERENCES)
        .collect()
}

fn fallback_response(
    locale: &str,
    message: &str,
    document: &ProjectDocument,
    references: Vec<String>,
    runtime: AiRuntimeStatus,
) -> AiChatResponse {
    let warnings = runtime.warning.clone().into_iter().collect::<Vec<_>>();
    AiChatResponse {
        answer: build_fallback_answer(locale, message, document, &runtime),
        runtime,
        references,
        warnings,
        source: "fallback-local".to_string(),
    }
}

fn build_fallback_answer(
    locale: &str,
    message: &str,
    document: &ProjectDocument,
    runtime: &AiRuntimeStatus,
) -> String {
    let project_digest = format!(
        "{} | {} entites | {} endpoints | {} flux | {} plugins",
        document.metadata.name,
        document.nodes.len(),
        document.endpoints.len(),
        document.streams.len(),
        document.plugin_manifests.len()
    );

    match locale {
        "en" => format!(
            "The local AI runtime did not answer, so I am staying in grounded local-summary mode.\nProject: {project_digest}.\nYour request was: \"{message}\".\nYou can continue asking about the current project, but for live model-backed answers check Ollama on {} and ensure a local model is loaded.",
            runtime.endpoint
        ),
        "es" => format!(
            "El runtime de IA local no respondio, asi que sigo en modo resumen local guiado por el proyecto.\nProyecto: {project_digest}.\nTu solicitud fue: \"{message}\".\nPuedes seguir preguntando sobre el proyecto actual, pero para respuestas con modelo en vivo revisa Ollama en {} y confirma que haya un modelo local cargado.",
            runtime.endpoint
        ),
        _ => format!(
            "Le runtime IA local n a pas repondu, donc je reste en mode resume local guide par le projet.\nProjet: {project_digest}.\nTa demande etait: \"{message}\".\nTu peux continuer a poser des questions sur le projet courant, mais pour une vraie reponse modele en direct il faut verifier Ollama sur {} et la presence d un modele local charge.",
            runtime.endpoint
        ),
    }
}

fn normalize_answer(answer: String, locale: &str, document: &ProjectDocument) -> String {
    let normalized = answer.trim().to_string();
    if normalized.is_empty() {
        build_fallback_answer(
            locale,
            "assistant returned an empty answer",
            document,
            &AiRuntimeStatus {
                available: false,
                provider: "ollama".to_string(),
                endpoint: DEFAULT_OLLAMA_ENDPOINT.to_string(),
                mode: "fallback-local".to_string(),
                local_only: true,
                active_model: None,
                available_models: Vec::new(),
                warning: Some("empty assistant answer".to_string()),
            },
        )
    } else {
        normalized
    }
}

fn dedupe_preserving_order(values: Vec<String>) -> Vec<String> {
    let mut seen = Vec::new();
    let mut deduped = Vec::new();
    for value in values {
        if seen.iter().any(|entry: &String| entry == &value) {
            continue;
        }
        seen.push(value.clone());
        deduped.push(value);
    }
    deduped
}

trait AsLabel {
    fn as_str(&self) -> &'static str;
}

impl AsLabel for faero_types::StreamDirection {
    fn as_str(&self) -> &'static str {
        match self {
            faero_types::StreamDirection::Inbound => "inbound",
            faero_types::StreamDirection::Outbound => "outbound",
            faero_types::StreamDirection::Bidirectional => "bidirectional",
        }
    }
}

impl AsLabel for faero_types::EndpointType {
    fn as_str(&self) -> &'static str {
        match self {
            faero_types::EndpointType::Ros2 => "ros2",
            faero_types::EndpointType::Opcua => "opcua",
            faero_types::EndpointType::Plc => "plc",
            faero_types::EndpointType::RobotController => "robot_controller",
            faero_types::EndpointType::BluetoothLe => "bluetooth_le",
            faero_types::EndpointType::BluetoothClassic => "bluetooth_classic",
            faero_types::EndpointType::WifiDevice => "wifi_device",
            faero_types::EndpointType::MqttBroker => "mqtt_broker",
            faero_types::EndpointType::WebsocketPeer => "websocket_peer",
            faero_types::EndpointType::TcpStream => "tcp_stream",
            faero_types::EndpointType::UdpStream => "udp_stream",
            faero_types::EndpointType::SerialDevice => "serial_device",
            faero_types::EndpointType::FieldbusTrace => "fieldbus_trace",
            faero_types::EndpointType::CustomStream => "custom_stream",
        }
    }
}

#[cfg(test)]
mod tests {
    use faero_types::{
        Addressing, ConnectionMode, EndpointType, EntityRecord, ExternalEndpoint, LinkMetrics,
        PluginManifest, ProjectDocument, QosProfile, StreamDirection, TelemetryStream,
        TimingProfile, TransportProfile,
    };

    use super::*;

    fn sample_document() -> ProjectDocument {
        let mut document = ProjectDocument::empty("AI Demo");
        document.metadata.project_id = "prj_ai_001".to_string();
        document.nodes.insert(
            "ent_cell_001".to_string(),
            EntityRecord {
                id: "ent_cell_001".to_string(),
                entity_type: "RobotCell".to_string(),
                name: "Cellule Demo".to_string(),
                revision: "rev_seed".to_string(),
                status: "active".to_string(),
                data: serde_json::json!({ "robotIds": ["ent_robot_001"] }),
            },
        );
        document.endpoints.insert(
            "ext_wifi_001".to_string(),
            ExternalEndpoint {
                id: "ext_wifi_001".to_string(),
                name: "WiFi Edge".to_string(),
                endpoint_type: EndpointType::WifiDevice,
                transport_profile: TransportProfile {
                    transport_kind: "wifi".to_string(),
                    adapter_id: Some("wlan0".to_string()),
                    discovery_mode: Some("mdns".to_string()),
                    credential_policy: Some("runtime_prompt".to_string()),
                    security_mode: Some("wpa3".to_string()),
                },
                connection_profile: serde_json::json!({ "retryBackoffMs": 250 }),
                addressing: Addressing {
                    host: Some("edge-box.local".to_string()),
                    port: Some(9001),
                    path: Some("/telemetry".to_string()),
                    device_id: None,
                },
                signal_map_ids: vec!["sig_001".to_string()],
                mode: ConnectionMode::Live,
                link_metrics: LinkMetrics {
                    latency_ms: Some(12),
                    jitter_ms: Some(2),
                    drop_rate: Some(0.0),
                    rssi_dbm: Some(-44),
                    bandwidth_kbps: Some(8_000),
                },
                status: "ready".to_string(),
            },
        );
        document.streams.insert(
            "str_001".to_string(),
            TelemetryStream {
                id: "str_001".to_string(),
                name: "BumperStatus".to_string(),
                endpoint_id: "ext_wifi_001".to_string(),
                stream_type: "mqtt_topic".to_string(),
                direction: StreamDirection::Inbound,
                codec_profile: serde_json::json!({ "encoding": "json" }),
                schema_ref: "schemas/telemetry/bumper-status.schema.json".to_string(),
                timing_profile: TimingProfile {
                    expected_rate_hz: 20,
                    max_latency_ms: 80,
                },
                qos_profile: QosProfile {
                    delivery: "at_least_once".to_string(),
                    ordering: "best_effort".to_string(),
                },
                status: "ready".to_string(),
            },
        );
        document.plugin_manifests.insert(
            "plg.integration.viewer".to_string(),
            PluginManifest {
                id: "ent_plugin_001".to_string(),
                plugin_id: "plg.integration.viewer".to_string(),
                version: "0.1.0".to_string(),
                capabilities: vec!["panel".to_string()],
                permissions: vec!["project.read".to_string()],
                entrypoints: vec!["plugins/integration-viewer/index.js".to_string()],
                compatibility: vec!["faero-core@0.1".to_string()],
                status: "installed".to_string(),
            },
        );
        document
            .plugin_states
            .insert("plg.integration.viewer".to_string(), true);
        document
    }

    #[test]
    fn selects_first_available_preferred_model() {
        let models = vec![
            "phi3:mini".to_string(),
            "gemma3:4b".to_string(),
            "mistral:7b".to_string(),
        ];
        let preferred = vec!["gemma3:4b".to_string(), "phi3:mini".to_string()];

        assert_eq!(
            select_default_model(&models, &preferred),
            Some("gemma3:4b".to_string())
        );
    }

    #[test]
    fn project_summary_contains_core_workspace_counts() {
        let summary = build_project_summary(&sample_document());

        assert!(summary.contains("projectName: AI Demo"));
        assert!(summary.contains("entities(1):"));
        assert!(summary.contains("endpoints(1):"));
        assert!(summary.contains("streams(1):"));
        assert!(summary.contains("plugins(1):"));
    }

    #[test]
    fn trim_history_keeps_recent_user_and_assistant_messages_only() {
        let history = vec![
            AiConversationMessage {
                role: "system".to_string(),
                content: "ignore".to_string(),
            },
            AiConversationMessage {
                role: "user".to_string(),
                content: "first".to_string(),
            },
            AiConversationMessage {
                role: "assistant".to_string(),
                content: "second".to_string(),
            },
            AiConversationMessage {
                role: "user".to_string(),
                content: "  ".to_string(),
            },
        ];

        let trimmed = trim_history(&history);
        assert_eq!(trimmed.len(), 2);
        assert_eq!(trimmed[0].content, "first");
        assert_eq!(trimmed[1].content, "second");
    }

    #[test]
    fn fallback_answer_mentions_project_and_endpoint() {
        let document = sample_document();
        let runtime = unavailable_runtime_status(
            &AiRuntimeConfig::from_env(),
            "offline".to_string(),
            vec!["gemma3:4b".to_string()],
        );

        let answer = build_fallback_answer("fr", "Resume le projet", &document, &runtime);
        assert!(answer.contains("AI Demo"));
        assert!(answer.contains(runtime.endpoint.as_str()));
    }
}
