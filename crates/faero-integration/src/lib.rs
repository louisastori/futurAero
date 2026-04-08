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
    pub degraded: bool,
    pub effective_metrics: LinkMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingReport {
    pub endpoint_id: String,
    pub binding_count: usize,
    pub invalid_binding_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinkDegradationProfile {
    pub latency_penalty_ms: u32,
    pub jitter_penalty_ms: u32,
    pub drop_rate: f64,
    pub rssi_penalty_dbm: i32,
    pub bandwidth_divider: u32,
}

#[derive(Debug, Default)]
pub struct IntegrationStubRegistry {
    endpoints: BTreeMap<String, ExternalEndpoint>,
    traces: BTreeMap<String, NetworkCaptureDataset>,
    signal_bindings: BTreeMap<String, Vec<String>>,
}

impl IntegrationStubRegistry {
    pub fn seeded() -> Self {
        let mut registry = Self::default();
        registry.register_endpoint(stub_ros2_endpoint());
        registry.register_endpoint(stub_opcua_endpoint());
        registry.register_endpoint(stub_plc_endpoint());
        registry.register_endpoint(stub_robot_controller_endpoint());
        registry.register_endpoint(stub_wifi_endpoint());
        registry.register_endpoint(stub_bluetooth_endpoint());
        registry.register_binding("ext_ros2_001", "topic:/cell/state");
        registry.register_binding("ext_opcua_001", "node:/Objects/Cell/Speed");
        registry.register_binding("ext_plc_001", "tag:plc.cycle_start");
        registry.register_binding("ext_robot_001", "robot:program/status");
        registry.register_binding("ext_wifi_001", "mqtt:/telemetry/status");
        registry.register_binding("ext_ble_001", "gatt:/battery/state");
        registry
    }

    pub fn register_endpoint(&mut self, endpoint: ExternalEndpoint) {
        self.endpoints.insert(endpoint.id.clone(), endpoint);
    }

    pub fn register_trace(&mut self, trace: NetworkCaptureDataset) {
        self.traces.insert(trace.id.clone(), trace);
    }

    pub fn register_binding(&mut self, endpoint_id: &str, binding: &str) {
        self.signal_bindings
            .entry(endpoint_id.to_string())
            .or_default()
            .push(binding.to_string());
    }

    pub fn endpoint(&self, id: &str) -> Option<&ExternalEndpoint> {
        self.endpoints.get(id)
    }

    pub fn endpoint_count(&self) -> usize {
        self.endpoints.len()
    }

    pub fn binding_report(&self, endpoint_id: &str) -> Option<BindingReport> {
        let endpoint = self.endpoints.get(endpoint_id)?;
        let bindings = self
            .signal_bindings
            .get(endpoint_id)
            .cloned()
            .unwrap_or_default();
        let invalid_binding_count = bindings
            .iter()
            .filter(|binding| {
                matches!(
                    endpoint.endpoint_type,
                    EndpointType::Plc | EndpointType::RobotController
                ) && !binding.contains(':')
            })
            .count();

        Some(BindingReport {
            endpoint_id: endpoint_id.to_string(),
            binding_count: bindings.len(),
            invalid_binding_count,
        })
    }

    pub fn simulate_link(
        &self,
        endpoint_id: &str,
        degradation: Option<&LinkDegradationProfile>,
    ) -> Option<LinkMetrics> {
        let endpoint = self.endpoints.get(endpoint_id)?;
        let Some(profile) = degradation else {
            return Some(endpoint.link_metrics.clone());
        };

        Some(LinkMetrics {
            latency_ms: endpoint
                .link_metrics
                .latency_ms
                .map(|latency| latency + profile.latency_penalty_ms),
            jitter_ms: endpoint
                .link_metrics
                .jitter_ms
                .map(|jitter| jitter + profile.jitter_penalty_ms),
            drop_rate: Some(profile.drop_rate),
            rssi_dbm: endpoint
                .link_metrics
                .rssi_dbm
                .map(|rssi| rssi - profile.rssi_penalty_dbm),
            bandwidth_kbps: endpoint
                .link_metrics
                .bandwidth_kbps
                .map(|bandwidth| bandwidth / profile.bandwidth_divider.max(1)),
        })
    }

    pub fn replay_trace(
        &self,
        trace_id: &str,
        degradation: Option<&LinkDegradationProfile>,
    ) -> Option<ReplayReport> {
        let trace = self.traces.get(trace_id)?;
        let effective_metrics = degradation
            .map(|profile| LinkMetrics {
                latency_ms: trace
                    .link_metrics
                    .latency_ms
                    .map(|latency| latency + profile.latency_penalty_ms),
                jitter_ms: trace
                    .link_metrics
                    .jitter_ms
                    .map(|jitter| jitter + profile.jitter_penalty_ms),
                drop_rate: Some(profile.drop_rate),
                rssi_dbm: trace
                    .link_metrics
                    .rssi_dbm
                    .map(|rssi| rssi - profile.rssi_penalty_dbm),
                bandwidth_kbps: trace
                    .link_metrics
                    .bandwidth_kbps
                    .map(|bandwidth| bandwidth / profile.bandwidth_divider.max(1)),
            })
            .unwrap_or_else(|| trace.link_metrics.clone());
        Some(ReplayReport {
            capture_id: trace.id.clone(),
            replayable: true,
            sample_count: trace.asset_refs.len() as u32,
            degraded: degradation.is_some(),
            effective_metrics,
        })
    }
}

pub fn degraded_wireless_profile() -> LinkDegradationProfile {
    LinkDegradationProfile {
        latency_penalty_ms: 120,
        jitter_penalty_ms: 25,
        drop_rate: 0.05,
        rssi_penalty_dbm: 15,
        bandwidth_divider: 3,
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

pub fn stub_bluetooth_endpoint() -> ExternalEndpoint {
    ExternalEndpoint {
        id: "ext_ble_001".to_string(),
        name: "BLE Tool".to_string(),
        endpoint_type: EndpointType::BluetoothLe,
        transport_profile: TransportProfile {
            transport_kind: "bluetooth_le".to_string(),
            adapter_id: Some("hci0".to_string()),
            discovery_mode: Some("scan".to_string()),
            credential_policy: Some("pairing".to_string()),
            security_mode: Some("le_secure_connections".to_string()),
        },
        connection_profile: serde_json::json!({ "mtu": 247 }),
        addressing: Addressing {
            host: None,
            port: None,
            path: None,
            device_id: Some("ble-tool-001".to_string()),
        },
        signal_map_ids: vec!["sig_ble_001".to_string()],
        mode: ConnectionMode::Live,
        link_metrics: LinkMetrics {
            latency_ms: Some(35),
            jitter_ms: Some(10),
            drop_rate: Some(0.01),
            rssi_dbm: Some(-62),
            bandwidth_kbps: Some(256),
        },
        status: "paired".to_string(),
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

        assert_eq!(registry.endpoint_count(), 6);
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
        assert_eq!(
            registry
                .endpoint("ext_ble_001")
                .map(|endpoint| endpoint.transport_profile.transport_kind.as_str()),
            Some("bluetooth_le")
        );
    }

    #[test]
    fn link_degradation_increases_latency_and_drop_rate() {
        let registry = IntegrationStubRegistry::seeded();
        let metrics = registry
            .simulate_link("ext_wifi_001", Some(&degraded_wireless_profile()))
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
            asset_refs: vec![
                "assets/blobs/trace_wifi_001.pcap".to_string(),
                "assets/blobs/trace_wifi_001.sidecar.json".to_string(),
            ],
            link_metrics: LinkMetrics {
                latency_ms: Some(20),
                ..LinkMetrics::default()
            },
            status: "ready".to_string(),
        });

        let report = registry
            .replay_trace("trace_wifi_001", Some(&degraded_wireless_profile()))
            .expect("trace should replay");

        assert_eq!(report.sample_count, 2);
        assert!(report.replayable);
        assert!(report.degraded);
        assert_eq!(report.effective_metrics.latency_ms, Some(140));
    }

    #[test]
    fn simulate_link_returns_live_metrics_and_none_for_unknown_endpoint() {
        let registry = IntegrationStubRegistry::seeded();

        let live_metrics = registry
            .simulate_link("ext_wifi_001", None)
            .expect("wifi endpoint should exist");
        assert_eq!(live_metrics.latency_ms, Some(18));
        assert_eq!(live_metrics.drop_rate, Some(0.0));

        assert!(registry.simulate_link("missing.endpoint", None).is_none());
    }

    #[test]
    fn replay_trace_returns_none_when_trace_is_missing() {
        let registry = IntegrationStubRegistry::seeded();
        assert!(registry.replay_trace("missing.trace", None).is_none());
    }

    #[test]
    fn binding_report_counts_registered_bindings() {
        let registry = IntegrationStubRegistry::seeded();
        let report = registry
            .binding_report("ext_plc_001")
            .expect("plc endpoint should exist");

        assert_eq!(report.binding_count, 1);
        assert_eq!(report.invalid_binding_count, 0);
    }

    #[test]
    fn stub_factories_produce_expected_transport_kinds() {
        assert_eq!(
            stub_ros2_endpoint().transport_profile.transport_kind,
            "ros2"
        );
        assert_eq!(
            stub_opcua_endpoint().transport_profile.transport_kind,
            "opcua"
        );
        assert_eq!(stub_plc_endpoint().transport_profile.transport_kind, "plc");
        assert_eq!(
            stub_robot_controller_endpoint()
                .transport_profile
                .transport_kind,
            "robot_controller"
        );
        assert_eq!(
            stub_wifi_endpoint().transport_profile.transport_kind,
            "wifi"
        );
        assert_eq!(
            stub_bluetooth_endpoint().transport_profile.transport_kind,
            "bluetooth_le"
        );
    }
}
