# Conformance - integration

Statut: verified

## Requirements Coverage

### Requirement: External Endpoints Stay Typed
- Evidence: `crates/faero-integration/src/lib.rs` typage explicite des endpoints (ROS2, OPC UA, PLC, robot, Wi-Fi, Bluetooth).
- Evidence: `ExternalEndpoint` conserve mode, profil transport, adressage et metriques lien.
- Verification: `cargo test -p faero-integration stub_factories_produce_expected_transport_kinds`

### Requirement: Explicit Industrial Bindings
- Evidence: `IntegrationStubRegistry::register_binding` conserve les bindings en donnees auditables.
- Evidence: `binding_report` expose nombre de bindings et invalides.
- Verification: `cargo test -p faero-integration binding_report_counts_registered_bindings`

### Requirement: Trace Replay And Degraded Links
- Evidence: `replay_trace` et `simulate_link` calculent latence, jitter, pertes et bande passante effectifs.
- Evidence: `degraded_wireless_profile` modelise un lien degrade reproductible.
- Verification: `cargo test -p faero-integration replay_trace_returns_basic_report`
