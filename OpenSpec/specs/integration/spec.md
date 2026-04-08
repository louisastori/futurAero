# FutureAero Integration

Statut: stable

Source promue depuis: `OpenSpec/specs/reference/18-Spec-Integration-Industrielle.md`, `OpenSpec/specs/reference/24-Spec-Connectivite-Sans-Fil-Et-Telemetrie.md`, `OpenSpec/specs/reference/12-Backlog-Dev-Ready.md`

Cette spec canonique capture les exigences stables des endpoints externes, des bindings explicites et des replays diagnostiques.

## Requirements

### Requirement: External Endpoints Stay Typed

- **GIVEN** un endpoint externe declare dans le projet
- **WHEN** il represente ROS2, OPC UA, PLC, controleur robot, Wi-Fi ou Bluetooth
- **THEN** son type, son mode live/replay/emulated, son profil de transport et ses metriques de lien doivent etre inspectables localement.

### Requirement: Explicit Industrial Bindings

- **GIVEN** un mapping entre le graphe projet et un systeme externe
- **WHEN** un binding est cree
- **THEN** la source, la cible, la direction, les conversions et le statut du binding doivent rester des donnees auditable et non une logique implicite.

### Requirement: Trace Replay And Degraded Links

- **GIVEN** une trace industrielle ou un profil de degradation de liaison
- **WHEN** le shell lance un replay ou une simulation de lien degrade
- **THEN** le systeme doit conserver des rapports lisibles sur la latence, le jitter, les pertes, les reconnexions et l effet observe sur les flux.

## Supporting References

- [18-Spec-Integration-Industrielle.md](../reference/18-Spec-Integration-Industrielle.md)
- [24-Spec-Connectivite-Sans-Fil-Et-Telemetrie.md](../reference/24-Spec-Connectivite-Sans-Fil-Et-Telemetrie.md)
- [12-Backlog-Dev-Ready.md](../reference/12-Backlog-Dev-Ready.md)
