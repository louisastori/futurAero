use std::path::PathBuf;

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}

pub fn fixture_path(name: &str) -> PathBuf {
    workspace_root().join("examples/projects").join(name)
}

pub fn schema_path(name: &str) -> PathBuf {
    workspace_root().join("schemas").join(name)
}

#[cfg(test)]
mod tests {
    use faero_storage::load_project;

    use super::*;

    #[test]
    fn root_schemas_are_valid_json_documents() {
        for schema in [
            "command.schema.json",
            "event.schema.json",
            "job.schema.json",
            "telemetry/bumper-status.schema.json",
        ] {
            let payload = std::fs::read_to_string(schema_path(schema))
                .expect("schema file should be readable");
            let json: serde_json::Value =
                serde_json::from_str(&payload).expect("schema should parse as json");
            assert_eq!(
                json["$schema"],
                "https://json-schema.org/draft/2020-12/schema"
            );
        }
    }

    #[test]
    fn official_fixtures_load_from_disk() {
        for fixture in [
            "empty-project.faero",
            "pick-and-place-demo.faero",
            "wireless-integration-demo.faero",
        ] {
            let project = load_project(fixture_path(fixture)).expect("fixture should load");
            assert!(!project.metadata.project_id.is_empty());
        }
    }

    #[test]
    fn wireless_fixture_contains_endpoint_and_stream() {
        let project = load_project(fixture_path("wireless-integration-demo.faero"))
            .expect("fixture should load");

        assert!(project.endpoints.contains_key("ext_wifi_001"));
        assert!(project.streams.contains_key("str_bumper_001"));
        assert_eq!(project.commands.len(), 2);
    }
}
