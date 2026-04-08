# FutureAero Plugin SDK

Statut: stable

Source promue depuis: `OpenSpec/specs/reference/22-Spec-Plugin-SDK.md`, `OpenSpec/archive/completed/2026-04/chore-github-pr-et-releases/specs/system/spec.md`, `OpenSpec/specs/reference/12-Backlog-Dev-Ready.md`

Cette spec canonique capture les exigences stables du Plugin SDK, de l audit des manifests et du workflow de promotion/versionnement associe.

## Requirements

### Requirement: Plugin Manifests Stay Declarative And Auditable

- **GIVEN** un plugin installe dans FutureAero
- **WHEN** son manifest est charge
- **THEN** `permissions`, `contributions`, `releaseChannel`, `compatibility` et `signature` doivent etre lisibles, auditables et validables avant activation.

### Requirement: Enable Disable Flow Stays Reversible

- **GIVEN** un plugin installe
- **WHEN** il est active, desactive ou rejete
- **THEN** l operation doit rester reversible sans corrompre le projet et laisser un statut d audit explicite.

### Requirement: Stable Specs Are Promoted From Changes

- **GIVEN** une exigence devenue stable
- **WHEN** elle n est plus propre a un changement actif
- **THEN** elle doit etre promue depuis `OpenSpec/changes/` vers `OpenSpec/specs/` tandis que le detail historique reste dans `OpenSpec/specs/reference/`.

## Supporting References

- [22-Spec-Plugin-SDK.md](../reference/22-Spec-Plugin-SDK.md)
- [chore-github-pr-et-releases/spec.md](../../archive/completed/2026-04/chore-github-pr-et-releases/specs/system/spec.md)
- [12-Backlog-Dev-Ready.md](../reference/12-Backlog-Dev-Ready.md)
