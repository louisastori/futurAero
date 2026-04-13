## Why

`ST-501` stabilise maintenant le graphe de controle minimal des cellules robotiques, mais le shell desktop execute encore `simulation.run.start` comme une commande synchrone qui synthese un artefact en memoire sans job explicite, progression publiee ni contrat de persistance clairement borne pour le run MVP. `ST-502` devient prioritaire maintenant pour faire de la simulation un increment observable, rejouable et exploitable par l UI, l IA locale et les artefacts projet.

## What Changes

- introduire un runner MVP `simulation.run.start` qui cree un job de simulation explicite avec phases et progression lisibles
- persister un artefact de run structure avec `summary`, `metrics`, `timeline`, `signalSamples`, `controllerStateSamples` et metadonnees moteur
- brancher le shell desktop et le backend Tauri sur le meme contrat de progression et de resultat au lieu d un simple effet instantane
- conserver la reproductibilite du run via `seed`, `engineVersion` et les entrees du scenario source

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `simulation`: la spec doit couvrir explicitement le job `simulation.run.start`, sa progression publiee et le contrat MVP des artefacts persistants de run

## Impact

- `crates/faero-sim`
- `crates/faero-core`
- `crates/faero-storage`
- `apps/desktop/src-tauri`
- `apps/desktop/src`
