## Context

`ST-502` a stabilise un contrat MVP de runner avec `scenario`, `job`, `summary`, `metrics` et artefacts persistants. Le prochain manque visible est qualitatif: lorsqu un run detecte un contact ou une collision, le shell ne sait pas encore localiser clairement l incident ni produire un rapport de run lisible qui resume la gravite, l instant critique et les prochaines actions.

Le besoin est transversal. `faero-sim` doit enrichir les artefacts, `faero-storage` doit persister ce nouveau shape, le backend Tauri doit le propager dans le snapshot, et le shell desktop doit l afficher sans recalcul heuristique parallele.

## Goals / Non-Goals

**Goals:**

- enrichir les contacts persistes avec un contexte localise lisible
- produire un `report` de run persiste qui resume collisions, blocages, instant critique et recommandations
- reutiliser ce rapport dans le backend desktop, le fallback web et l IA locale
- garder le contrat deterministe pour les memes entrees de simulation

**Non-Goals:**

- augmenter la fidelite physique du moteur de collision
- introduire un pipeline PDF ou export documentaire complet
- ajouter un ordonnanceur asynchrone multi-runs ou du streaming temps reel hors contrat MVP

## Decisions

### Enrichir les contacts existants plutot que creer un artefact collision parallele

Les collisions sont deja representees par `contacts`. Le choix retenu est d enrichir ces entrees avec un contexte localise minimal comme `locationLabel`, `phase` et `stateId` afin qu un run collided reste lisible sans devoir joindre un second dataset.

Alternative rejetee:

- creer un artefact `CollisionDataset` separe
  Rejete pour l MVP car cela duplique la source de verite et complique la lecture du run dans le shell.

### Persister un bloc `report` directement dans `SimulationRun`

Le rapport de run doit vivre au meme endroit que `summary`, `metrics` et `job`. Le bloc `report` portera un `headline`, une liste courte de `findings`, des `criticalEventIds` et des `recommendedActions` reutilisables par l UI et l IA.

Alternative rejetee:

- creer une entite `ValidationReport` separee des `SimulationRun`
  Rejete pour `ST-503` car le besoin porte encore sur un seul run MVP et doit rester lisible en une seule lecture.

### Deriver le rapport a partir des artefacts persistants, pas depuis l UI

La semantique du rapport doit etre centralisee cote backend et moteur. `faero-sim` produit la base du rapport a partir de `metrics`, `contacts`, `timelineSamples` et `controllerStateSamples`, puis Tauri persiste ce resultat tel quel.

Alternative rejetee:

- reconstruire le rapport dans React a partir des artefacts bruts
  Rejete pour eviter des derives entre backend reel, fallback et autoprompts IA.

## Risks / Trade-offs

- [Le rapport peut devenir verbeux] -> Mitigation: limiter l MVP a un headline, quelques findings et actions recommandees deterministes.
- [La localisation peut rester approximative si les paires de contact sont pauvres] -> Mitigation: reutiliser les ids persistants et exposer un `locationLabel` lisible sans promettre une geometrie plus riche que le moteur actuel.
- [Le fallback peut diverger du backend sur les conclusions du rapport] -> Mitigation: partager le meme shape `report` et couvrir les champs critiques par tests de parite.

## Migration Plan

1. Etendre `faero-types` et `faero-sim` pour produire des contacts localises et un `report` de run.
2. Persister `report` dans `SimulationRun` cote Tauri et `faero-storage`.
3. Aligner fallback, UI desktop et IA locale sur ce nouveau contrat.
4. Valider par tests Rust, round-trip `.faero` et tests UI avant increment suivant.

## Open Questions

- Le rapport MVP doit-il distinguer explicitement `warning_report` et `collision_report`, ou un seul shape avec `status` suffit-il ?
- Faut-il deja preparer des ancres pour un futur export PDF/markdown du rapport, ou attendre un increment posterieur ?
