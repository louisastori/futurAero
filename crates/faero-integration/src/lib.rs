use std::collections::BTreeMap;

use faero_types::{
    Addressing, ConnectionMode, EndpointType, ExternalEndpoint, LinkMetrics, NetworkCaptureDataset,
    TransportProfile,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayReport {
    pub capture_id: String,
    pub replayable: bool,
    pub sample_count: u32,
}

#[derive(Debug, Default)]
pub struct IntegrationStubRegistry {
    endpoints: BTreeMap<String, ExternalEndpoint>,
    traces: BTreeMap<String, NetworkCaptureDataset>,
}

impl IntegrationStubRegistry {
    pub fn seeded() -> Self {
        let mut registry = Self::default();
        registry.register_endpoint(stub_ros2_endpoint());
        registry.register_endpoint(stub_opcua_endpoint());
        registry.register_endpoint(stub_plc_endpoint());
        registry.register_endpoint(stub_robot_controller_endpoint());
        registry.register_endpoint(stub_wifi_endpoint());
        registry
    }

    pub fn register_endpoint(&mut self, endpoint: ExternalEndpoint) {
        self.endpoints.insert(endpoint.id.clone(), endpoint);
    }

    pub fn register_trace(&mut self, trace: NetworkCaptureDataset) {
        self.traces.insert(trace.id.clone(), trace);
    }

    pub fn endpoint(&self, id: &str) -> Option<&ExternalEndpoint> {
        self.endpoints.get(id)
    }

    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    pub fn simulate_link(&self, endpoint_id: &str, degraded: bool) -> Option<LinkMetrics> {
        let endpoint = self.endpoints.get(endpoint_id)?;
        if !degraded {
            return Some(endpoint.link_metrics.clone());
        }

        Some(LinkMetrics {
            latency_ms: endpoint
                .link_metrics
                .latency_ms
                .map(|latency| latency + 120),
            jitter_ms: endpoint.link_metrics.jitter_ms.map(|jitter| jitter + 25),
            drop_rate: Some(0.05),
            rssi_dbm: endpoint.link_metrics.rssi_dbm.map(|rssi| rssi - 15),
            bandwidth_kbps: endpoint
                .link_metrics
                .bandwidth_kbps
                .map(|bandwidth| bandwidth / 3),
        })
    }

    pub fn replay_trace(&self, trace_id: &str) -> Option<ReplayReport> {
        let trace = self.traces.get(trace_id)?;
        Some(ReplayReport {
            capture_id: trace.id.clone(),
            replayable: true,
            sample_count: trace.asset_refs.len() as u32,
        })
    }
}

pub fn stub_ros2_endpoint() -> ExternalEndpoint {
    endpoint("ext_ros2_001", "ROS2 Bridge", EndpointType::Ros2, "ros2")
}

pub fn stub_opcua_endpoint() -> ExternalEndpoint {
    endpoint(
        "ext_opcua_001",
        "OPCUA Cellule",
        EndpointType::Opcua,
        "opcua",
    )
}

pub fn stub_plc_endpoint() -> ExternalEndpoint {
    endpoint("ext_plc_001", "PLC Mock", EndpointType::Plc, "plc")
}

pub fn stub_robot_controller_endpoint() -> ExternalEndpoint {
    endpoint(
        "ext_robot_001",
        "Robot Controller Mock",
        EndpointType::RobotController,
        "robot_controller",
    )
}

pub fn stub_wifi_endpoint() -> ExternalEndpoint {
    ExternalEndpoint {
        id: "ext_wifi_001".to_string(),
        name: "Wireless Edge".to_string(),
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
            host: Some("wireless-edge.local".to_string()),
            port: Some(9001),
            path: Some("/telemetry".to_string()),
            device_id: None,
        },
        signal_map_ids: vec!["sig_wireless_001".to_string()],
        mode: ConnectionMode::Live,
        link_metrics: LinkMetrics {
            latency_ms: Some(18),
            jitter_ms: Some(4),
            drop_rate: Some(0.0),
            rssi_dbm: Some(-55),
            bandwidth_kbps: Some(10000),
        },
        status: "connected".to_string(),
    }
}

fn endpoint(
    id: &str,
    name: &str,
    endpoint_type: EndpointType,
    transport_kind: &str,
) -> ExternalEndpoint {
    ExternalEndpoint {
        id: id.to_string(),
        name: name.to_string(),
        endpoint_type,
        transport_profile: TransportProfile {
            transport_kind: transport_kind.to_string(),
            adapter_id: None,
            discovery_mode: Some("static".to_string()),
            credential_policy: None,
            security_mode: Some("local".to_string()),
        },
        connection_profile: serde_json::json!({ "stub": true }),
        addressing: Addressing {
            host: Some("127.0.0.1".to_string()),
            port: Some(4840),
            path: None,
            device_id: None,
        },
        signal_map_ids: vec![],
        mode: ConnectionMode::Replay,
        link_metrics: LinkMetrics {
            latency_ms: Some(5),
            jitter_ms: Some(1),
            drop_rate: Some(0.0),
            rssi_dbm: None,
            bandwidth_kbps: Some(1000),
        },
        status: "ready".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_registry_contains_requested_stubs() {
        let registry = IntegrationStubRegistry::seeded();

        assert_eq!(registry.endpoint_count(), 5);
        assert_eq!(
            registry
                .endpoint("ext_ros2_001")
                .map(|endpoint| &endpoint.name),
            Some(&"ROS2 Bridge".to_string())
        );
        assert_eq!(
            registry
                .endpoint("ext_robot_001")
                .map(|endpoint| endpoint.transport_profile.transport_kind.as_str()),
            Some("robot_controller")
        );
    }

    #[test]
    fn link_degradation_increases_latency_and_drop_rate() {
        let registry = IntegrationStubRegistry::seeded();

        let metrics = registry
            .simulate_link("ext_wifi_001", true)
            .expect("wifi endpoint should exist");

        assert_eq!(metrics.drop_rate, Some(0.05));
        assert_eq!(metrics.latency_ms, Some(138));
    }

    #[test]
    fn replay_trace_returns_basic_report() {
        let mut registry = IntegrationStubRegistry::seeded();
        registry.register_trace(NetworkCaptureDataset {
            id: "trace_wifi_001".to_string(),
            endpoint_id: "ext_wifi_001".to_string(),
            capture_type: "pcap".to_string(),
            timestamp_range: "2026-04-06T00:00:00Z/2026-04-06T00:00:10Z".to_string(),
            asset_refs: vec!["assets/blobs/trace_wifi_001.pcap".to_string()],
            link_metrics: LinkMetrics {
                latency_ms: Some(20),
                ..LinkMetrics::default()
            },
            status: "ready".to_string(),
        });

        let report = registry
            .replay_trace("trace_wifi_001")
            .expect("trace should replay");

        assert_eq!(report.sample_count, 1);
        assert!(report.replayable);
    }
}
