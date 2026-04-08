use std::{env, time::Duration};

use faero_types::{
    AiContextReference, AiProposedCommand, AiRiskLevel, AiStructuredExplain, ExternalEndpoint,
    ProjectDocument,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const DEFAULT_OLLAMA_ENDPOINT: &str = "http://127.0.0.1:11434";
const DEFAULT_TIMEOUT_SECS: u64 = 120;
const DEFAULT_PREFERRED_MODELS: &[&str] = &["gemma3:27b", "gemma3:12b", "gemma3:4b", "phi3:mini"];
const GEMMA3_MODEL_PREFIX: &str = "gemma3:";
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
    pub gemma3_models: Vec<String>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiConversationMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AiChatResponse {
    pub answer: String,
    pub runtime: AiRuntimeStatus,
    pub references: Vec<String>,
    pub structured: Option<AiStructuredExplain>,
    pub suggestion_id: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelSelection {
    active_model: Option<String>,
    warning: Option<String>,
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
    let client = build_http_client(config.timeout_secs);

    match fetch_model_names(&client, &config) {
        Ok(models) => runtime_status_from_models(&config, models, None),
        Err(error) => unavailable_runtime_status(&config, error.to_string(), Vec::new(), None),
    }
}

pub fn chat_with_project(
    document: &ProjectDocument,
    locale: &str,
    history: &[AiConversationMessage],
    message: &str,
    selected_model: Option<&str>,
) -> Result<AiChatResponse, AiError> {
    let trimmed_message = message.trim();
    if trimmed_message.is_empty() {
        return Err(AiError::EmptyMessage);
    }

    let config = AiRuntimeConfig::from_env();
    let client = build_http_client(config.timeout_secs);
    let references = collect_context_references(document);
    let structured = Some(build_structured_explain(document, trimmed_message));

    let response = match fetch_model_names(&client, &config) {
        Ok(models) => {
            let runtime = runtime_status_from_models(&config, models.clone(), selected_model);
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
                    Ok(answer) => {
                        let warnings = runtime.warning.clone().into_iter().collect();
                        AiChatResponse {
                            answer,
                            runtime,
                            references,
                            structured: structured.clone(),
                            suggestion_id: None,
                            warnings,
                            source: "ollama-local".to_string(),
                        }
                    }
                    Err(error) => fallback_response(
                        locale,
                        trimmed_message,
                        document,
                        references,
                        structured.clone(),
                        degraded_runtime_status(&config, error.to_string(), models, selected_model),
                    ),
                }
            } else {
                fallback_response(
                    locale,
                    trimmed_message,
                    document,
                    references,
                    structured.clone(),
                    unavailable_runtime_status(
                        &config,
                        "no preferred local model found".to_string(),
                        models,
                        selected_model,
                    ),
                )
            }
        }
        Err(error) => fallback_response(
            locale,
            trimmed_message,
            document,
            references,
            structured,
            unavailable_runtime_status(&config, error.to_string(), Vec::new(), selected_model),
        ),
    };

    Ok(response)
}

fn build_http_client(timeout_secs: u64) -> Client {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .expect("ollama client should initialize")
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

fn runtime_status_from_models(
    config: &AiRuntimeConfig,
    models: Vec<String>,
    requested_model: Option<&str>,
) -> AiRuntimeStatus {
    let selection = resolve_model_selection(&models, &config.preferred_models, requested_model);
    AiRuntimeStatus {
        available: !models.is_empty(),
        provider: "ollama".to_string(),
        endpoint: config.endpoint.clone(),
        mode: "grounded-chat".to_string(),
        local_only: true,
        active_model: selection.active_model,
        gemma3_models: collect_gemma3_models(&models),
        available_models: models,
        warning: selection.warning,
    }
}

fn unavailable_runtime_status(
    config: &AiRuntimeConfig,
    warning: String,
    available_models: Vec<String>,
    requested_model: Option<&str>,
) -> AiRuntimeStatus {
    let selection =
        resolve_model_selection(&available_models, &config.preferred_models, requested_model);
    AiRuntimeStatus {
        available: false,
        provider: "ollama".to_string(),
        endpoint: config.endpoint.clone(),
        mode: "fallback-local".to_string(),
        local_only: true,
        active_model: selection.active_model,
        gemma3_models: collect_gemma3_models(&available_models),
        available_models,
        warning: combine_warnings(Some(warning), selection.warning),
    }
}

fn degraded_runtime_status(
    config: &AiRuntimeConfig,
    warning: String,
    available_models: Vec<String>,
    requested_model: Option<&str>,
) -> AiRuntimeStatus {
    let selection =
        resolve_model_selection(&available_models, &config.preferred_models, requested_model);
    AiRuntimeStatus {
        available: !available_models.is_empty(),
        provider: "ollama".to_string(),
        endpoint: config.endpoint.clone(),
        mode: if available_models.is_empty() {
            "fallback-local".to_string()
        } else {
            "degraded-chat".to_string()
        },
        local_only: true,
        active_model: selection.active_model,
        gemma3_models: collect_gemma3_models(&available_models),
        available_models,
        warning: combine_warnings(Some(warning), selection.warning),
    }
}

fn resolve_model_selection(
    models: &[String],
    preferred_models: &[String],
    requested_model: Option<&str>,
) -> ModelSelection {
    if let Some(requested_model) = requested_model
        .map(str::trim)
        .filter(|model| !model.is_empty())
    {
        if models.iter().any(|model| model == requested_model) {
            return ModelSelection {
                active_model: Some(requested_model.to_string()),
                warning: None,
            };
        }

        return ModelSelection {
            active_model: select_default_model(models, preferred_models),
            warning: Some(format!(
                "requested local model `{requested_model}` not available"
            )),
        };
    }

    ModelSelection {
        active_model: select_default_model(models, preferred_models),
        warning: None,
    }
}

fn select_default_model(models: &[String], preferred_models: &[String]) -> Option<String> {
    preferred_models
        .iter()
        .find(|preferred| models.iter().any(|model| model == *preferred))
        .cloned()
        .or_else(|| models.first().cloned())
}

fn collect_gemma3_models(models: &[String]) -> Vec<String> {
    models
        .iter()
        .filter(|model| model.starts_with(GEMMA3_MODEL_PREFIX))
        .cloned()
        .collect()
}

fn combine_warnings(primary: Option<String>, secondary: Option<String>) -> Option<String> {
    match (primary, secondary) {
        (Some(primary), Some(secondary)) => Some(format!("{primary}; {secondary}")),
        (Some(primary), None) => Some(primary),
        (None, Some(secondary)) => Some(secondary),
        (None, None) => None,
    }
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

fn build_structured_explain(document: &ProjectDocument, message: &str) -> AiStructuredExplain {
    let latest_run = latest_entity_by_type(document, "SimulationRun");
    let latest_safety = latest_entity_by_type(document, "SafetyReport");
    let latest_robot_cell = latest_entity_by_type(document, "RobotCell");
    let asks_about_safety = contains_any_keyword(message, &["safety", "bloc", "interlock"]);
    let asks_about_collision = contains_any_keyword(message, &["collision", "contact"]);

    let mut context_refs = vec![AiContextReference {
        entity_id: None,
        role: "source".to_string(),
        path: "metadata.projectId".to_string(),
    }];
    let mut limitations = Vec::new();
    let mut explanation = Vec::new();
    let mut summary = format!(
        "Le projet {} contient {} entites exploitables.",
        document.metadata.name,
        document.nodes.len()
    );
    let mut confidence = 0.58;
    let mut risk_level = AiRiskLevel::Low;
    let mut proposed_commands = Vec::new();

    if let Some(report) = latest_safety.filter(|_| asks_about_safety || !asks_about_collision) {
        let blocked = report
            .data
            .get("summary")
            .and_then(|summary| summary.get("inhibited"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let status = report
            .data
            .get("summary")
            .and_then(|summary| summary.get("status"))
            .and_then(|value| value.as_str())
            .unwrap_or("unknown");
        let blocking_interlocks = report
            .data
            .get("summary")
            .and_then(|summary| summary.get("blockingInterlockCount"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let active_zones = report
            .data
            .get("summary")
            .and_then(|summary| summary.get("activeZoneCount"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0);

        context_refs.push(AiContextReference {
            entity_id: Some(report.id.clone()),
            role: "source".to_string(),
            path: "summary.status".to_string(),
        });
        context_refs.push(AiContextReference {
            entity_id: Some(report.id.clone()),
            role: "source".to_string(),
            path: "summary.blockingInterlockCount".to_string(),
        });
        summary = if blocked {
            format!(
                "Le dernier rapport safety bloque l action avec {} interlock(s) sur {}.",
                blocking_interlocks, report.name
            )
        } else {
            format!(
                "Le dernier rapport safety reste {} avec {} zone(s) active(s) sur {}.",
                status, active_zones, report.name
            )
        };
        explanation.push(format!(
            "Le rapport {} expose {} zone(s) active(s) et {} interlock(s) bloquant(s).",
            report.id, active_zones, blocking_interlocks
        ));
        if let Some(causes) = report
            .data
            .get("summary")
            .and_then(|summary| summary.get("causeZoneIds"))
            .and_then(|value| value.as_array())
        {
            let cause_ids = causes
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>();
            if !cause_ids.is_empty() {
                explanation.push(format!(
                    "Les causes explicites du blocage sont: {}.",
                    cause_ids.join(", ")
                ));
            }
        }
        confidence = if blocked { 0.9 } else { 0.78 };
        risk_level = if blocked {
            AiRiskLevel::High
        } else {
            AiRiskLevel::Medium
        };
        proposed_commands.extend(build_safety_suggestion_commands(document, report, blocked));
    } else if let Some(run) = latest_run {
        let collision_count = run
            .data
            .get("summary")
            .and_then(|summary| summary.get("collisionCount"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        let blocked = run
            .data
            .get("summary")
            .and_then(|summary| summary.get("blockedSequenceDetected"))
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let blocked_state = run
            .data
            .get("summary")
            .and_then(|summary| summary.get("blockedStateId"))
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let cycle_time_ms = run
            .data
            .get("summary")
            .and_then(|summary| summary.get("cycleTimeMs"))
            .and_then(|value| value.as_u64())
            .unwrap_or(0);

        context_refs.push(AiContextReference {
            entity_id: Some(run.id.clone()),
            role: "source".to_string(),
            path: "summary.collisionCount".to_string(),
        });
        context_refs.push(AiContextReference {
            entity_id: Some(run.id.clone()),
            role: "source".to_string(),
            path: "summary.blockedSequenceDetected".to_string(),
        });
        if run
            .data
            .get("contacts")
            .and_then(|value| value.as_array())
            .is_some_and(|contacts| !contacts.is_empty())
        {
            context_refs.push(AiContextReference {
                entity_id: Some(run.id.clone()),
                role: "source".to_string(),
                path: "contacts[0]".to_string(),
            });
        }

        summary = if collision_count > 0 {
            format!(
                "Le dernier run detecte {} collision(s) sur {}.",
                collision_count, run.name
            )
        } else if blocked {
            format!(
                "Le dernier run se termine sans collision mais avec une sequence bloquee sur {}.",
                run.name
            )
        } else {
            format!(
                "Le dernier run {} termine sans collision en {} ms.",
                run.name, cycle_time_ms
            )
        };
        explanation.push(format!(
            "Le run {} garde un temps de cycle de {} ms.",
            run.id, cycle_time_ms
        ));
        if let Some(contact) = run
            .data
            .get("contacts")
            .and_then(|value| value.as_array())
            .and_then(|contacts| contacts.first())
        {
            let left = contact
                .get("leftEntityId")
                .and_then(|value| value.as_str())
                .unwrap_or("left");
            let right = contact
                .get("rightEntityId")
                .and_then(|value| value.as_str())
                .unwrap_or("right");
            let timestamp_ms = contact
                .get("timestampMs")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            explanation.push(format!(
                "Le premier contact critique apparait entre {} et {} a t={} ms.",
                left, right, timestamp_ms
            ));
        }
        if let Some(blocked_state) = blocked_state {
            explanation.push(format!(
                "La machine a etats reste bloquee dans l etat `{}`.",
                blocked_state
            ));
        }
        confidence = if collision_count > 0 || blocked {
            0.86
        } else {
            0.74
        };
        risk_level = if collision_count > 0 {
            AiRiskLevel::High
        } else if blocked {
            AiRiskLevel::Medium
        } else {
            AiRiskLevel::Low
        };
        proposed_commands.extend(build_run_suggestion_commands(document, run, collision_count, blocked));
    } else {
        limitations.push(
            "Le modele n a trouve ni run de simulation ni rapport safety persiste.".to_string(),
        );
        if latest_robot_cell.is_some() {
            proposed_commands.push(AiProposedCommand {
                kind: "simulation.run.start".to_string(),
                target_id: latest_robot_cell.as_ref().map(|entity| entity.id.clone()),
                payload: serde_json::json!({}),
            });
        }
    }

    if latest_run.is_none() {
        limitations.push("Aucun run de simulation disponible dans le graphe courant.".to_string());
    }
    if latest_safety.is_none() {
        limitations.push("Aucun rapport safety disponible dans le graphe courant.".to_string());
    }
    if let Some(robot_cell) = latest_robot_cell {
        context_refs.push(AiContextReference {
            entity_id: Some(robot_cell.id.clone()),
            role: "context".to_string(),
            path: "sequenceValidation.estimatedCycleTimeMs".to_string(),
        });
    }
    if explanation.is_empty() {
        explanation.push(format!(
            "La demande \"{}\" a ete analysee a partir du graphe projet courant.",
            message.trim()
        ));
    }
    if limitations.is_empty() {
        limitations.push("Le raisonnement est borne aux artefacts locaux disponibles.".to_string());
    }

    AiStructuredExplain {
        summary,
        context_refs: context_refs
            .into_iter()
            .take(MAX_CONTEXT_REFERENCES)
            .collect(),
        confidence,
        risk_level,
        limitations,
        proposed_commands: proposed_commands
            .into_iter()
            .take(4)
            .collect(),
        explanation,
    }
}

fn build_safety_suggestion_commands(
    document: &ProjectDocument,
    report: &faero_types::EntityRecord,
    blocked: bool,
) -> Vec<AiProposedCommand> {
    let mut commands = Vec::new();

    if blocked
        && let Some(signal) = find_signal_entity(document, "sig_safety_clear")
    {
        commands.push(AiProposedCommand {
            kind: "entity.properties.update".to_string(),
            target_id: Some(signal.id.clone()),
            payload: serde_json::json!({
                "changes": {
                    "currentValue": true
                }
            }),
        });
    }

    commands.push(AiProposedCommand {
        kind: "analyze.safety".to_string(),
        target_id: report
            .data
            .get("robotCellId")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        payload: serde_json::json!({}),
    });

    dedupe_commands(commands)
}

fn build_run_suggestion_commands(
    document: &ProjectDocument,
    run: &faero_types::EntityRecord,
    collision_count: u64,
    blocked: bool,
) -> Vec<AiProposedCommand> {
    let mut commands = Vec::new();

    if blocked
        && let Some(signal) = find_signal_entity(document, "sig_progress_gate")
    {
        commands.push(AiProposedCommand {
            kind: "entity.properties.update".to_string(),
            target_id: Some(signal.id.clone()),
            payload: serde_json::json!({
                "changes": {
                    "currentValue": 1.0
                }
            }),
        });
    }

    if collision_count > 0
        && let Some(signal) = find_signal_entity(document, "sig_cycle_start")
    {
        commands.push(AiProposedCommand {
            kind: "entity.properties.update".to_string(),
            target_id: Some(signal.id.clone()),
            payload: serde_json::json!({
                "changes": {
                    "currentValue": false
                }
            }),
        });
    }

    commands.push(AiProposedCommand {
        kind: "simulation.run.start".to_string(),
        target_id: run
            .data
            .get("robotCellId")
            .and_then(|value| value.as_str())
            .map(str::to_string),
        payload: serde_json::json!({}),
    });

    dedupe_commands(commands)
}

fn find_signal_entity<'a>(
    document: &'a ProjectDocument,
    signal_id: &str,
) -> Option<&'a faero_types::EntityRecord> {
    document.nodes.values().find(|entity| {
        entity.entity_type == "Signal"
            && entity
                .data
                .get("signalId")
                .and_then(|value| value.as_str())
                == Some(signal_id)
    })
}

fn dedupe_commands(commands: Vec<AiProposedCommand>) -> Vec<AiProposedCommand> {
    let mut seen = Vec::<String>::new();
    let mut deduped = Vec::new();
    for command in commands {
        let key = format!(
            "{}|{}|{}",
            command.kind,
            command.target_id.clone().unwrap_or_default(),
            command.payload
        );
        if seen.iter().any(|entry| entry == &key) {
            continue;
        }
        seen.push(key);
        deduped.push(command);
    }
    deduped
}

fn latest_entity_by_type<'a>(
    document: &'a ProjectDocument,
    entity_type: &str,
) -> Option<&'a faero_types::EntityRecord> {
    document
        .nodes
        .values()
        .filter(|entity| entity.entity_type == entity_type)
        .max_by(|left, right| left.id.cmp(&right.id))
}

fn contains_any_keyword(message: &str, keywords: &[&str]) -> bool {
    let lower = message.to_ascii_lowercase();
    keywords.iter().any(|keyword| lower.contains(keyword))
}

fn fallback_response(
    locale: &str,
    message: &str,
    document: &ProjectDocument,
    references: Vec<String>,
    structured: Option<AiStructuredExplain>,
    runtime: AiRuntimeStatus,
) -> AiChatResponse {
    let source = if runtime.available {
        "degraded-local".to_string()
    } else {
        "fallback-local".to_string()
    };
    let warnings = runtime.warning.clone().into_iter().collect::<Vec<_>>();
    AiChatResponse {
        answer: build_fallback_answer(locale, message, document, &runtime),
        runtime,
        references,
        structured,
        suggestion_id: None,
        warnings,
        source,
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
            "{}\nProject: {project_digest}.\nYour request was: \"{message}\".\nYou can continue asking about the current project, but for live model-backed answers check Ollama on {} and ensure a local model is loaded.",
            if runtime.available {
                format!(
                    "The local AI runtime is reachable, but model {} did not finish in time or returned an error, so I am temporarily staying in grounded local-summary mode.",
                    runtime.active_model.as_deref().unwrap_or("unknown")
                )
            } else {
                "The local AI runtime did not answer, so I am staying in grounded local-summary mode.".to_string()
            },
            runtime.endpoint
        ),
        "es" => format!(
            "{}\nProyecto: {project_digest}.\nTu solicitud fue: \"{message}\".\nPuedes seguir preguntando sobre el proyecto actual, pero para respuestas con modelo en vivo revisa Ollama en {} y confirma que haya un modelo local cargado.",
            if runtime.available {
                format!(
                    "El runtime de IA local es accesible, pero el modelo {} no termino a tiempo o devolvio un error, asi que sigo temporalmente en modo resumen local guiado por el proyecto.",
                    runtime.active_model.as_deref().unwrap_or("desconocido")
                )
            } else {
                "El runtime de IA local no respondio, asi que sigo en modo resumen local guiado por el proyecto.".to_string()
            },
            runtime.endpoint
        ),
        _ => format!(
            "{}\nProjet: {project_digest}.\nTa demande etait: \"{message}\".\nTu peux continuer a poser des questions sur le projet courant, mais pour une vraie reponse modele en direct il faut verifier Ollama sur {} et la presence d un modele local charge.",
            if runtime.available {
                format!(
                    "Le runtime IA local est joignable, mais le modele {} n a pas termine a temps ou a renvoye une erreur, donc je passe temporairement en mode resume local guide par le projet.",
                    runtime.active_model.as_deref().unwrap_or("inconnu")
                )
            } else {
                "Le runtime IA local n a pas repondu, donc je reste en mode resume local guide par le projet.".to_string()
            },
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
                gemma3_models: Vec::new(),
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
    use std::{
        env,
        io::{BufRead, BufReader, Read, Write},
        net::{TcpListener, TcpStream},
        panic::{AssertUnwindSafe, catch_unwind},
        sync::{Mutex, MutexGuard, OnceLock},
        thread,
    };

    use faero_types::{
        Addressing, ConnectionMode, EndpointType, EntityRecord, ExternalEndpoint, LinkMetrics,
        PluginManifest, ProjectDocument, QosProfile, StreamDirection, TelemetryStream,
        TimingProfile, TransportProfile,
    };

    use super::*;

    #[derive(Clone)]
    struct MockHttpResponse {
        status: u16,
        body: String,
    }

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct TestEnvScope {
        previous: Vec<(String, Option<String>)>,
        _guard: MutexGuard<'static, ()>,
    }

    impl TestEnvScope {
        fn new(entries: &[(&str, Option<&str>)]) -> Self {
            let guard = match env_lock().lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let previous = entries
                .iter()
                .map(|(key, _)| ((*key).to_string(), env::var(key).ok()))
                .collect::<Vec<_>>();

            for (key, value) in entries {
                match value {
                    Some(value) => unsafe { env::set_var(key, value) },
                    None => unsafe { env::remove_var(key) },
                }
            }

            Self {
                previous,
                _guard: guard,
            }
        }
    }

    impl Drop for TestEnvScope {
        fn drop(&mut self) {
            for (key, value) in self.previous.drain(..) {
                match value {
                    Some(value) => unsafe { env::set_var(&key, value) },
                    None => unsafe { env::remove_var(&key) },
                }
            }
        }
    }

    fn recover_env_lock_after_panic() -> MutexGuard<'static, ()> {
        match env_lock().lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn read_request_path(stream: &mut TcpStream) -> String {
        let mut reader = BufReader::new(stream.try_clone().expect("stream should clone"));
        let mut request_line = String::new();
        reader
            .read_line(&mut request_line)
            .expect("request line should be readable");

        let mut content_length = 0usize;
        loop {
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .expect("header line should be readable");
            if line == "\r\n" || line.is_empty() {
                break;
            }
            let lower = line.to_ascii_lowercase();
            if let Some(value) = lower.strip_prefix("content-length:") {
                content_length = value.trim().parse::<usize>().unwrap_or(0);
            }
        }

        if content_length > 0 {
            let mut body = vec![0u8; content_length];
            reader
                .read_exact(&mut body)
                .expect("request body should be readable");
        }

        request_line
            .split_whitespace()
            .nth(1)
            .expect("request path should be present")
            .to_string()
    }

    fn write_response(stream: &mut TcpStream, response: &MockHttpResponse) {
        let reason = match response.status {
            200 => "OK",
            500 => "Internal Server Error",
            404 => "Not Found",
            _ => "Mock",
        };
        let payload = response.body.as_bytes();
        write!(
            stream,
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            response.status,
            reason,
            payload.len()
        )
        .expect("headers should be written");
        stream.write_all(payload).expect("body should be written");
        stream.flush().expect("response should flush");
    }

    fn spawn_mock_ollama_server(
        expected: Vec<(&'static str, MockHttpResponse)>,
    ) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let endpoint = format!(
            "http://127.0.0.1:{}",
            listener
                .local_addr()
                .expect("listener should expose local address")
                .port()
        );

        let handle = thread::spawn(move || {
            for (expected_path, response) in expected {
                let (mut stream, _) = listener.accept().expect("connection should arrive");
                let path = read_request_path(&mut stream);
                assert_eq!(path, expected_path);
                write_response(&mut stream, &response);
            }
        });

        (endpoint, handle)
    }

    fn spawn_drop_on_chat_server(
        tags_response: MockHttpResponse,
    ) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let endpoint = format!(
            "http://127.0.0.1:{}",
            listener
                .local_addr()
                .expect("listener should expose local address")
                .port()
        );

        let handle = thread::spawn(move || {
            let (mut tags_stream, _) = listener.accept().expect("tag connection should arrive");
            let tags_path = read_request_path(&mut tags_stream);
            assert_eq!(tags_path, "/api/tags");
            write_response(&mut tags_stream, &tags_response);
            drop(tags_stream);

            let (mut chat_stream, _) = listener.accept().expect("chat connection should arrive");
            let chat_path = read_request_path(&mut chat_stream);
            assert_eq!(chat_path, "/api/chat");
            drop(chat_stream);
        });

        (endpoint, handle)
    }

    fn sample_document() -> ProjectDocument {
        let mut document = ProjectDocument::empty("AI Demo".to_string());
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
            "gemma3:27b".to_string(),
            "mistral:7b".to_string(),
        ];
        let preferred = vec![
            "gemma3:27b".to_string(),
            "gemma3:12b".to_string(),
            "gemma3:4b".to_string(),
            "phi3:mini".to_string(),
        ];

        assert_eq!(
            select_default_model(&models, &preferred),
            Some("gemma3:27b".to_string())
        );
    }

    #[test]
    fn select_default_model_falls_back_to_first_or_none() {
        let models = vec!["mistral:7b".to_string()];
        let preferred = vec!["gemma3:27b".to_string()];

        assert_eq!(
            select_default_model(&models, &preferred),
            Some("mistral:7b".to_string())
        );
        assert_eq!(select_default_model(&[], &preferred), None);
    }

    #[test]
    fn resolve_model_selection_prefers_requested_model_and_warns_when_missing() {
        let models = vec![
            "gemma3:27b".to_string(),
            "gemma3:12b".to_string(),
            "phi3:mini".to_string(),
        ];
        let preferred = vec![
            "gemma3:27b".to_string(),
            "gemma3:12b".to_string(),
            "gemma3:4b".to_string(),
            "phi3:mini".to_string(),
        ];

        let requested = resolve_model_selection(&models, &preferred, Some("gemma3:12b"));
        assert_eq!(requested.active_model, Some("gemma3:12b".to_string()));
        assert_eq!(requested.warning, None);

        let missing = resolve_model_selection(&models, &preferred, Some("gemma3:4b"));
        assert_eq!(missing.active_model, Some("gemma3:27b".to_string()));
        assert_eq!(
            missing.warning,
            Some("requested local model `gemma3:4b` not available".to_string())
        );
    }

    #[test]
    fn collect_gemma3_models_keeps_only_gemma3_variants() {
        let models = vec![
            "phi3:mini".to_string(),
            "gemma3:12b".to_string(),
            "gemma3:27b".to_string(),
            "mistral:7b".to_string(),
        ];

        assert_eq!(
            collect_gemma3_models(&models),
            vec!["gemma3:12b".to_string(), "gemma3:27b".to_string()]
        );
    }

    #[test]
    fn combine_warnings_covers_all_match_arms() {
        assert_eq!(
            combine_warnings(
                Some("primary warning".to_string()),
                Some("secondary warning".to_string())
            ),
            Some("primary warning; secondary warning".to_string())
        );
        assert_eq!(
            combine_warnings(Some("primary warning".to_string()), None),
            Some("primary warning".to_string())
        );
        assert_eq!(
            combine_warnings(None, Some("secondary warning".to_string())),
            Some("secondary warning".to_string())
        );
        assert_eq!(combine_warnings(None, None), None);
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
            vec!["gemma3:27b".to_string()],
            None,
        );

        let answer = build_fallback_answer("fr", "Resume le projet", &document, &runtime);
        assert!(answer.contains("AI Demo"));
        assert!(answer.contains(runtime.endpoint.as_str()));
    }

    #[test]
    fn runtime_config_defaults_and_overrides_are_deterministic() {
        {
            let _env = TestEnvScope::new(&[
                ("FUTUREAERO_OLLAMA_ENDPOINT", None),
                ("FUTUREAERO_OLLAMA_TIMEOUT_SECS", None),
                ("FUTUREAERO_OLLAMA_MODEL", None),
            ]);
            let config = AiRuntimeConfig::from_env();
            assert_eq!(config.endpoint, DEFAULT_OLLAMA_ENDPOINT);
            assert_eq!(config.timeout_secs, DEFAULT_TIMEOUT_SECS);
            assert_eq!(
                config.preferred_models,
                vec![
                    "gemma3:27b".to_string(),
                    "gemma3:12b".to_string(),
                    "gemma3:4b".to_string(),
                    "phi3:mini".to_string()
                ]
            );
        }

        {
            let _env = TestEnvScope::new(&[
                ("FUTUREAERO_OLLAMA_ENDPOINT", Some("http://127.0.0.1:18080")),
                ("FUTUREAERO_OLLAMA_TIMEOUT_SECS", Some("9")),
                ("FUTUREAERO_OLLAMA_MODEL", Some("phi3:mini")),
            ]);
            let config = AiRuntimeConfig::from_env();
            assert_eq!(config.endpoint, "http://127.0.0.1:18080");
            assert_eq!(config.timeout_secs, 9);
            assert_eq!(
                config.preferred_models,
                vec![
                    "phi3:mini".to_string(),
                    "gemma3:27b".to_string(),
                    "gemma3:12b".to_string(),
                    "gemma3:4b".to_string()
                ]
            );
        }

        {
            let _env = TestEnvScope::new(&[
                ("FUTUREAERO_OLLAMA_ENDPOINT", Some("  ")),
                ("FUTUREAERO_OLLAMA_TIMEOUT_SECS", Some("invalid")),
                ("FUTUREAERO_OLLAMA_MODEL", Some("   ")),
            ]);
            let config = AiRuntimeConfig::from_env();
            assert_eq!(config.endpoint, DEFAULT_OLLAMA_ENDPOINT);
            assert_eq!(config.timeout_secs, DEFAULT_TIMEOUT_SECS);
            assert_eq!(
                config.preferred_models,
                vec![
                    "gemma3:27b".to_string(),
                    "gemma3:12b".to_string(),
                    "gemma3:4b".to_string(),
                    "phi3:mini".to_string()
                ]
            );
        }
    }

    #[test]
    fn query_runtime_status_reads_local_models_and_handles_failures() {
        let (endpoint, handle) = spawn_mock_ollama_server(vec![(
            "/api/tags",
            MockHttpResponse {
                status: 200,
                body: r#"{"models":[{"name":"phi3:mini"},{"name":"gemma3:4b"},{"name":"gemma3:27b"}]}"#.to_string(),
            },
        )]);

        {
            let _env = TestEnvScope::new(&[
                ("FUTUREAERO_OLLAMA_ENDPOINT", Some(endpoint.as_str())),
                ("FUTUREAERO_OLLAMA_MODEL", None),
            ]);
            let status = query_runtime_status();
            assert!(status.available);
            assert_eq!(status.provider, "ollama");
            assert_eq!(status.active_model, Some("gemma3:27b".to_string()));
            assert_eq!(
                status.gemma3_models,
                vec!["gemma3:4b".to_string(), "gemma3:27b".to_string()]
            );
            assert_eq!(
                status.available_models,
                vec![
                    "phi3:mini".to_string(),
                    "gemma3:4b".to_string(),
                    "gemma3:27b".to_string()
                ]
            );
        }
        handle.join().expect("server thread should finish");

        let unavailable = {
            let _env = TestEnvScope::new(&[
                ("FUTUREAERO_OLLAMA_ENDPOINT", Some("http://127.0.0.1:1")),
                ("FUTUREAERO_OLLAMA_MODEL", None),
            ]);
            query_runtime_status()
        };
        assert!(!unavailable.available);
        assert_eq!(unavailable.mode, "fallback-local");
        assert!(unavailable.warning.is_some());
    }

    #[test]
    fn chat_with_project_rejects_empty_message() {
        let _error = chat_with_project(&sample_document(), "fr", &[], "   ", None)
            .expect_err("empty message should fail");
    }

    #[test]
    fn query_runtime_status_handles_invalid_tag_payloads() {
        let (endpoint, handle) = spawn_mock_ollama_server(vec![(
            "/api/tags",
            MockHttpResponse {
                status: 200,
                body: r#"{"models":"broken"}"#.to_string(),
            },
        )]);

        let status = {
            let _env = TestEnvScope::new(&[
                ("FUTUREAERO_OLLAMA_ENDPOINT", Some(endpoint.as_str())),
                ("FUTUREAERO_OLLAMA_MODEL", None),
            ]);
            query_runtime_status()
        };
        handle.join().expect("server thread should finish");

        assert!(!status.available);
        assert_eq!(status.mode, "fallback-local");
    }

    #[test]
    fn chat_with_project_falls_back_when_chat_connection_or_payload_is_invalid() {
        let document = sample_document();

        let (disconnecting_endpoint, disconnecting_handle) =
            spawn_drop_on_chat_server(MockHttpResponse {
                status: 200,
                body: r#"{"models":[{"name":"gemma3:27b"}]}"#.to_string(),
            });
        let disconnecting_response = {
            let _env = TestEnvScope::new(&[
                (
                    "FUTUREAERO_OLLAMA_ENDPOINT",
                    Some(disconnecting_endpoint.as_str()),
                ),
                ("FUTUREAERO_OLLAMA_MODEL", None),
            ]);
            chat_with_project(&document, "fr", &[], "Resume le projet", Some("gemma3:27b"))
        }
        .expect("disconnecting chat should fallback");
        disconnecting_handle
            .join()
            .expect("disconnecting server thread should finish");
        assert_eq!(disconnecting_response.source, "degraded-local");
        assert!(disconnecting_response.runtime.available);
        assert_eq!(disconnecting_response.runtime.mode, "degraded-chat");

        let (invalid_payload_endpoint, invalid_payload_handle) = spawn_mock_ollama_server(vec![
            (
                "/api/tags",
                MockHttpResponse {
                    status: 200,
                    body: r#"{"models":[{"name":"gemma3:27b"}]}"#.to_string(),
                },
            ),
            (
                "/api/chat",
                MockHttpResponse {
                    status: 200,
                    body: r#"{"message":"broken"}"#.to_string(),
                },
            ),
        ]);
        let invalid_payload_response = {
            let _env = TestEnvScope::new(&[
                (
                    "FUTUREAERO_OLLAMA_ENDPOINT",
                    Some(invalid_payload_endpoint.as_str()),
                ),
                ("FUTUREAERO_OLLAMA_MODEL", None),
            ]);
            chat_with_project(&document, "fr", &[], "Resume le projet", Some("gemma3:27b"))
        }
        .expect("invalid chat payload should fallback");
        invalid_payload_handle
            .join()
            .expect("invalid payload server thread should finish");
        assert_eq!(invalid_payload_response.source, "degraded-local");
        assert!(invalid_payload_response.runtime.available);
        assert_eq!(invalid_payload_response.runtime.mode, "degraded-chat");
    }

    #[test]
    fn chat_with_project_returns_live_answer_when_ollama_is_available() {
        let document = sample_document();
        let history = vec![
            AiConversationMessage {
                role: "assistant".to_string(),
                content: "Etat precedent".to_string(),
            },
            AiConversationMessage {
                role: "user".to_string(),
                content: "Quelle est la cellule ?".to_string(),
            },
        ];
        let (endpoint, handle) = spawn_mock_ollama_server(vec![
            (
                "/api/tags",
                MockHttpResponse {
                    status: 200,
                    body: r#"{"models":[{"name":"gemma3:4b"},{"name":"gemma3:27b"}]}"#.to_string(),
                },
            ),
            (
                "/api/chat",
                MockHttpResponse {
                    status: 200,
                    body: r#"{"message":{"role":"assistant","content":"  Reponse locale validee.  "}}"#
                        .to_string(),
                },
            ),
        ]);

        let response = {
            let _env = TestEnvScope::new(&[
                ("FUTUREAERO_OLLAMA_ENDPOINT", Some(endpoint.as_str())),
                ("FUTUREAERO_OLLAMA_MODEL", None),
            ]);
            chat_with_project(
                &document,
                "fr",
                &history,
                "Resume le projet",
                Some("gemma3:27b"),
            )
        }
        .expect("chat should succeed");
        handle.join().expect("server thread should finish");

        assert_eq!(response.source, "ollama-local");
        assert_eq!(response.answer, "Reponse locale validee.");
        assert!(response.runtime.available);
        assert_eq!(
            response.runtime.active_model,
            Some("gemma3:27b".to_string())
        );
        assert!(response.warnings.is_empty());
        assert!(
            response
                .references
                .contains(&"project:prj_ai_001".to_string())
        );
        assert!(
            response
                .references
                .iter()
                .any(|reference| reference.starts_with("entity:"))
        );
    }

    #[test]
    fn chat_with_project_falls_back_when_runtime_has_no_models_or_chat_errors() {
        let document = sample_document();

        let (empty_endpoint, empty_handle) = spawn_mock_ollama_server(vec![(
            "/api/tags",
            MockHttpResponse {
                status: 200,
                body: r#"{"models":[]}"#.to_string(),
            },
        )]);
        let empty_response = {
            let _env =
                TestEnvScope::new(&[("FUTUREAERO_OLLAMA_ENDPOINT", Some(empty_endpoint.as_str()))]);
            chat_with_project(&document, "en", &[], "Explain the project", None)
        }
        .expect("fallback response should still succeed");
        empty_handle.join().expect("server thread should finish");

        assert_eq!(empty_response.source, "fallback-local");
        assert_eq!(empty_response.runtime.mode, "fallback-local");
        assert_eq!(empty_response.runtime.active_model, None);
        assert!(
            empty_response
                .warnings
                .iter()
                .any(|warning| warning.contains("no preferred local model found"))
        );
        assert!(empty_response.answer.contains("Project: AI Demo"));

        let (failing_endpoint, failing_handle) = spawn_mock_ollama_server(vec![
            (
                "/api/tags",
                MockHttpResponse {
                    status: 200,
                    body: r#"{"models":[{"name":"phi3:mini"},{"name":"gemma3:27b"}]}"#.to_string(),
                },
            ),
            (
                "/api/chat",
                MockHttpResponse {
                    status: 500,
                    body: r#"{"error":"unavailable"}"#.to_string(),
                },
            ),
        ]);
        let failing_response = {
            let _env = TestEnvScope::new(&[
                (
                    "FUTUREAERO_OLLAMA_ENDPOINT",
                    Some(failing_endpoint.as_str()),
                ),
                ("FUTUREAERO_OLLAMA_MODEL", None),
            ]);
            chat_with_project(
                &document,
                "es",
                &[],
                "Resume el proyecto",
                Some("gemma3:27b"),
            )
        }
        .expect("chat failure should fallback");
        failing_handle.join().expect("server thread should finish");

        assert_eq!(failing_response.source, "degraded-local");
        assert!(failing_response.runtime.available);
        assert_eq!(failing_response.runtime.mode, "degraded-chat");
        assert!(
            failing_response
                .warnings
                .iter()
                .any(|warning| warning.contains("500"))
        );
        assert!(failing_response.answer.contains("Proyecto: AI Demo"));
    }

    #[test]
    fn chat_with_project_falls_back_when_tag_lookup_fails() {
        let document = sample_document();
        let (endpoint, handle) = spawn_mock_ollama_server(vec![(
            "/api/tags",
            MockHttpResponse {
                status: 500,
                body: r#"{"error":"tags failed"}"#.to_string(),
            },
        )]);

        let response = {
            let _env =
                TestEnvScope::new(&[("FUTUREAERO_OLLAMA_ENDPOINT", Some(endpoint.as_str()))]);
            chat_with_project(&document, "fr", &[], "Resume le projet", None)
        }
        .expect("tag failure should fallback");
        handle.join().expect("server thread should finish");

        assert_eq!(response.source, "fallback-local");
        assert!(
            response
                .warnings
                .iter()
                .any(|warning| warning.contains("500"))
        );
        assert!(response.answer.contains("Projet: AI Demo"));
    }

    #[test]
    fn helpers_cover_locales_labels_references_and_normalization() {
        let document = sample_document();
        let prompt = build_system_prompt("es", &document);
        assert!(prompt.contains("Answer in Spanish."));
        assert_eq!(language_instruction("en"), "in English");
        assert_eq!(language_instruction("es"), "in Spanish");
        assert_eq!(language_instruction("fr"), "in French");

        let messages = build_ollama_messages(
            "fr",
            &[
                AiConversationMessage {
                    role: "system".to_string(),
                    content: "ignore".to_string(),
                },
                AiConversationMessage {
                    role: "user".to_string(),
                    content: " besoin ".to_string(),
                },
            ],
            "Etat courant",
            &document,
        );
        assert_eq!(messages[0].role, "system");
        assert_eq!(messages[1].role, "user");
        assert_eq!(
            messages.last().map(|message| message.role.as_str()),
            Some("user")
        );

        let references = collect_context_references(&document);
        assert_eq!(references[0], "project:prj_ai_001");
        assert!(
            references
                .iter()
                .any(|reference| reference == "endpoint:ext_wifi_001")
        );

        assert_eq!(StreamDirection::Inbound.as_str(), "inbound");
        assert_eq!(StreamDirection::Outbound.as_str(), "outbound");
        assert_eq!(StreamDirection::Bidirectional.as_str(), "bidirectional");
        assert_eq!(EndpointType::Ros2.as_str(), "ros2");
        assert_eq!(EndpointType::Opcua.as_str(), "opcua");
        assert_eq!(EndpointType::Plc.as_str(), "plc");
        assert_eq!(EndpointType::RobotController.as_str(), "robot_controller");
        assert_eq!(EndpointType::BluetoothLe.as_str(), "bluetooth_le");
        assert_eq!(EndpointType::BluetoothClassic.as_str(), "bluetooth_classic");
        assert_eq!(EndpointType::MqttBroker.as_str(), "mqtt_broker");
        assert_eq!(EndpointType::WebsocketPeer.as_str(), "websocket_peer");
        assert_eq!(EndpointType::TcpStream.as_str(), "tcp_stream");
        assert_eq!(EndpointType::UdpStream.as_str(), "udp_stream");
        assert_eq!(EndpointType::SerialDevice.as_str(), "serial_device");
        assert_eq!(EndpointType::FieldbusTrace.as_str(), "fieldbus_trace");
        assert_eq!(EndpointType::CustomStream.as_str(), "custom_stream");

        assert_eq!(
            normalize_answer("  ".to_string(), "fr", &document),
            build_fallback_answer(
                "fr",
                "assistant returned an empty answer",
                &document,
                &AiRuntimeStatus {
                    available: false,
                    provider: "ollama".to_string(),
                    endpoint: DEFAULT_OLLAMA_ENDPOINT.to_string(),
                    mode: "fallback-local".to_string(),
                    local_only: true,
                    active_model: None,
                    available_models: Vec::new(),
                    gemma3_models: Vec::new(),
                    warning: Some("empty assistant answer".to_string()),
                },
            )
        );
        assert_eq!(
            normalize_answer("  already clean  ".to_string(), "fr", &document),
            "already clean"
        );
    }

    #[test]
    fn format_endpoint_covers_address_variants_and_summary_none_case() {
        let mut document = sample_document();
        document
            .plugin_states
            .insert("plg.integration.viewer".to_string(), false);
        let host_port_path = document
            .endpoints
            .remove("ext_wifi_001")
            .expect("sample endpoint should exist");
        assert!(format_endpoint(&host_port_path).contains("edge-box.local:9001/telemetry"));

        let host_port = ExternalEndpoint {
            addressing: Addressing {
                host: Some("edge-box.local".to_string()),
                port: Some(9001),
                path: None,
                device_id: None,
            },
            ..host_port_path.clone()
        };
        assert!(format_endpoint(&host_port).ends_with("@ edge-box.local:9001"));

        let host_path = ExternalEndpoint {
            addressing: Addressing {
                host: Some("edge-box.local".to_string()),
                port: None,
                path: Some("/telemetry".to_string()),
                device_id: None,
            },
            ..host_port_path.clone()
        };
        assert!(format_endpoint(&host_path).ends_with("@ edge-box.local/telemetry"));

        let host_only = ExternalEndpoint {
            addressing: Addressing {
                host: Some("edge-box.local".to_string()),
                port: None,
                path: None,
                device_id: None,
            },
            ..host_port_path.clone()
        };
        assert!(format_endpoint(&host_only).ends_with("@ edge-box.local"));

        let device_only = ExternalEndpoint {
            addressing: Addressing {
                host: None,
                port: None,
                path: None,
                device_id: Some("dev-01".to_string()),
            },
            ..host_port_path.clone()
        };
        assert!(format_endpoint(&device_only).ends_with("@ dev-01"));

        let none_address = ExternalEndpoint {
            addressing: Addressing::default(),
            ..host_port_path
        };
        assert!(format_endpoint(&none_address).ends_with("@ n/a"));

        let none_summary = summarize_named_items(Vec::<String>::new().into_iter());
        assert_eq!(none_summary, "none");

        let summary = build_project_summary(&document);
        assert!(summary.contains("[disabled]"));
    }

    #[test]
    fn recover_env_lock_after_panic_returns_clean_guard_when_not_poisoned() {
        env_lock().clear_poison();
        let _guard = recover_env_lock_after_panic();
    }

    #[test]
    fn with_env_restores_previous_values_even_after_panic() {
        unsafe { env::set_var("FUTUREAERO_TEST_SENTINEL", "sentinel") };
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _env = TestEnvScope::new(&[("FUTUREAERO_TEST_SENTINEL", Some("temporary"))]);
            assert_eq!(
                env::var("FUTUREAERO_TEST_SENTINEL").ok().as_deref(),
                Some("temporary")
            );
            panic!("forced test panic")
        }));
        assert!(panic_result.is_err());
        assert_eq!(
            env::var("FUTUREAERO_TEST_SENTINEL").ok().as_deref(),
            Some("sentinel")
        );

        {
            let _guard = recover_env_lock_after_panic();
        }

        {
            let _env = TestEnvScope::new(&[("FUTUREAERO_TEST_SENTINEL", Some("restored"))]);
            assert_eq!(
                env::var("FUTUREAERO_TEST_SENTINEL").ok().as_deref(),
                Some("restored")
            );
        }
        assert_eq!(
            env::var("FUTUREAERO_TEST_SENTINEL").ok().as_deref(),
            Some("sentinel")
        );
        unsafe { env::remove_var("FUTUREAERO_TEST_SENTINEL") };
    }

    #[test]
    fn write_response_supports_not_found_and_custom_status_reason() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let address = listener
            .local_addr()
            .expect("listener should expose local address");

        let client = thread::spawn(move || {
            let mut stream = TcpStream::connect(address).expect("client should connect");
            stream
                .write_all(b"GET /missing HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .expect("request should write");
            let mut payload = String::new();
            stream
                .read_to_string(&mut payload)
                .expect("response should read");
            payload
        });

        let (mut stream, _) = listener.accept().expect("server should accept");
        let _ = read_request_path(&mut stream);
        write_response(
            &mut stream,
            &MockHttpResponse {
                status: 404,
                body: "{}".to_string(),
            },
        );
        drop(stream);
        let response = client.join().expect("client should join");
        assert!(response.contains("404 Not Found"));

        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let address = listener
            .local_addr()
            .expect("listener should expose local address");
        let client = thread::spawn(move || {
            let mut stream = TcpStream::connect(address).expect("client should connect");
            stream
                .write_all(b"GET /custom HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .expect("request should write");
            let mut payload = String::new();
            stream
                .read_to_string(&mut payload)
                .expect("response should read");
            payload
        });

        let (mut stream, _) = listener.accept().expect("server should accept");
        let _ = read_request_path(&mut stream);
        write_response(
            &mut stream,
            &MockHttpResponse {
                status: 418,
                body: "{}".to_string(),
            },
        );
        drop(stream);
        let response = client.join().expect("client should join");
        assert!(response.contains("418 Mock"));
    }

    #[test]
    fn structured_explain_proposes_commands_for_blocked_runs() {
        let mut document = sample_document();
        document.nodes.insert(
            "ent_sig_001".to_string(),
            EntityRecord {
                id: "ent_sig_001".to_string(),
                entity_type: "Signal".to_string(),
                name: "Progress Gate".to_string(),
                revision: "rev_seed".to_string(),
                status: "active".to_string(),
                data: serde_json::json!({
                    "signalId": "sig_progress_gate",
                    "kind": "scalar",
                    "currentValue": 0.62,
                    "parameterSet": {
                        "unit": "ratio"
                    }
                }),
            },
        );
        document.nodes.insert(
            "ent_run_001".to_string(),
            EntityRecord {
                id: "ent_run_001".to_string(),
                entity_type: "SimulationRun".to_string(),
                name: "Run Demo".to_string(),
                revision: "rev_seed".to_string(),
                status: "active".to_string(),
                data: serde_json::json!({
                    "robotCellId": "ent_cell_001",
                    "summary": {
                        "collisionCount": 0,
                        "blockedSequenceDetected": true,
                        "blockedStateId": "transfer",
                        "cycleTimeMs": 3497
                    },
                    "contacts": []
                }),
            },
        );

        let explain = build_structured_explain(&document, "explique le blocage");

        assert!(
            explain
                .proposed_commands
                .iter()
                .any(|command| command.kind == "entity.properties.update")
        );
        assert!(
            explain
                .proposed_commands
                .iter()
                .any(|command| command.kind == "simulation.run.start")
        );
    }
}
