## Context

`ST-501` vient de normaliser le graphe de controle minimal des cellules robotiques entre `faero-robotics`, `faero-sim`, le backend Tauri et le fallback web. Le prochain manque visible se situe sur `simulation.run.start`: le shell sait deja creer un artefact `SimulationRun`, mais ce flux reste un effet instantane du backend desktop sans contrat explicite de job, sans progression publiee au shell, et sans separation claire entre les entrees du run, les artefacts persistants et l etat de completion.

`ST-502` touche plusieurs couches a la fois: `faero-sim` doit exposer un contrat de runner plus explicite, `faero-core` et `faero-storage` doivent persister un run lisible, et le shell desktop doit suivre une progression observable avant la future extension collision/report de `ST-503`.

## Goals / Non-Goals

**Goals:**

- exposer `simulation.run.start` comme un job MVP lisible avec `status`, `phase`, `progress` et `progressSamples`
- persister dans le projet les entrees du run (`seed`, `engineVersion`, scenario source) et les artefacts de sortie MVP
- garder le runner deterministe et rejouable a entrees identiques
- aligner backend Tauri, fallback web et UI sur le meme contrat de run persiste

**Non-Goals:**

- ajouter la detection de collision et le rapport complet de `ST-503`
- construire un ordonnanceur multi-jobs ou un worker separe du processus desktop
- persister les artefacts perception lourds (`pointclouds`, `occupancy_maps`) de la roadmap detaillee

## Decisions

### Representer le runner MVP comme une entite `SimulationRun` qui embarque son envelope de job

Le MVP doit rester lisible dans le graphe projet et simple a rejouer. Le choix retenu est donc de persister un `SimulationRun` unique contenant:

- les entrees de scenario utiles au replay (`seed`, `engineVersion`, `stepCount`, ids sources)
- un bloc `job` avec `jobId`, `status`, `phase`, `progress` et `progressSamples`
- les artefacts MVP (`summary`, `metrics`, `timelineSamples`, `signalSamples`, `controllerStateSamples`, `contacts`)

Alternative rejetee:

- creer un `JobEnvelope` top-level separe du `SimulationRun`
  Rejete pour le MVP car cela complique inutilement la lecture du run dans le shell alors qu un seul type d analyse est concerne.

### Garder une execution in-process mais publier une progression explicite

Le runner MVP n a pas encore besoin d un worker externe. L execution reste donc dans le backend desktop, mais elle doit publier des etapes explicites (`queued`, `running`, `trace_persisted`, `completed`) et conserver cet historique dans les artefacts du run.

Alternative rejetee:

- attendre une vraie infrastructure asynchrone avant d exposer la progression
  Rejete car `ST-502` demande deja un contrat observable pour `simulation.run.start` avant le raffinement du moteur.

### Centraliser les metadonnees deterministes de run dans `faero-sim`

La source de verite pour `seed`, `engineVersion`, `stepCount`, `cycleTimeMs` et les `progressSamples` doit vivre dans `faero-sim`, pas dans le shell. Le backend desktop assemble le scenario a partir de la cellule robotique et persiste la sortie, mais il ne doit pas redefinir la semantique du run.

Alternative rejetee:

- continuer a calculer les metadonnees du run directement dans `apps/desktop/src-tauri`
  Rejete pour eviter une derive entre tests Rust et representation desktop.

### Aligner fallback et UI sur le meme shape de run persiste

Le web fallback et les tests UI doivent lire la meme structure `SimulationRun` que le backend desktop: `scenario`, `summary`, `metrics`, `job`, `timelineSamples`, `signalSamples`, `controllerStateSamples`, `contacts`.

Alternative rejetee:

- garder un run fallback plus pauvre et seulement illustratif
  Rejete parce que les tests desktop doivent verifier la meme lecture de progression et d artefacts que le backend reel.

## Risks / Trade-offs

- [Le runner reste synchrone en implementation interne] -> Mitigation: rendre le contrat de progression stable des maintenant pour pouvoir deplacer l execution vers un worker plus tard sans casser l UI.
- [Les artefacts de run peuvent grossir vite] -> Mitigation: limiter `ST-502` aux artefacts MVP deja attendus par la spec canonique et laisser les datasets lourds aux increments suivants.
- [Le fallback peut diverger du backend] -> Mitigation: partager un shape de run unique et couvrir les champs critiques (`job`, `summary`, `metrics`, timeline) par tests desktop.

## Migration Plan

1. Etendre `faero-sim` avec un resultat de runner MVP qui porte explicitement les metadonnees de job et les artefacts persistants.
2. Brancher `simulation.run.start` dans le backend Tauri pour creer un `SimulationRun` persiste avec progression publiee.
3. Aligner le fallback web et les tests desktop sur la meme structure.
4. Valider par tests Rust et UI avant d etendre le moteur collision/report dans `ST-503`.

## Open Questions

- Est-ce que la progression desktop doit aussi etre emise en temps reel via event Tauri, ou seulement persistee dans `progressSamples` pour ce premier increment?
- Faut-il persister un bloc `metrics` distinct du `summary` des `ST-502`, ou garder des metriques MVP minimales tant que `ST-503` n est pas livre?
