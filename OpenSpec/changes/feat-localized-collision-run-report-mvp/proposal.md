## Why

`ST-502` rend maintenant `SimulationRun` lisible et persiste, mais un run collided ne remonte encore qu un compteur global et une liste brute de contacts. `ST-503` est le prochain increment logique pour rendre les collisions localisables, expliquer un instant critique et produire un rapport de run exploitable par le shell desktop et l IA locale.

## What Changes

- enrichir les collisions persistees avec un contexte localise lisible directement depuis `SimulationRun`
- ajouter un bloc `report` dans `SimulationRun` pour resumer le resultat, les evenements critiques et les actions recommandees
- brancher le backend desktop, le fallback web et l UI sur ce rapport de run au lieu de reconstituer l explication uniquement depuis des compteurs
- conserver un contrat deterministe et rejouable pour les runs collision et nominals

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `simulation`: la spec doit couvrir la localisation des collisions et le rapport de run persiste dans `SimulationRun`

## Impact

- `crates/faero-types`
- `crates/faero-sim`
- `crates/faero-storage`
- `crates/faero-ai`
- `apps/desktop/src-tauri`
- `apps/desktop/src`
