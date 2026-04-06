# Politique De Tests Et Couverture 100

## Objectif

Definir une politique de qualite avec une cible a 100 pourcent sur le scope first-party du MVP, tout en imposant une gate GitHub versionnee et reellement executable.

## Politique cible

La cible long terme du projet est:

- 100 pourcent line coverage
- 100 pourcent region coverage
- 100 pourcent function coverage
- 100 pourcent command handler coverage
- 100 pourcent event handler coverage
- 100 pourcent schema validation coverage

## Gate effectivement imposee

La gate executee par GitHub Actions et par `scripts/test.ps1` est definie dans `config/coverage-gate.json`.

Sur l'etat actuel du socle MVP, la gate Rust imposee est:

- 99.5 pourcent line coverage minimum
- 97.5 pourcent region coverage minimum
- 100 pourcent function coverage minimum

Cette gate est ratchetee vers le haut a chaque reduction mesuree des lignes ou regions non couvertes.

## Scope couvert

Cette exigence s'applique a tout le code first-party MVP:

- crates Rust internes,
- packages UI internes,
- schemas de commandes/evenements/jobs,
- validateurs,
- orchestrateurs,
- adapters first-party,
- host plugins.

## Exclusions et ecarts controles

Les exclusions ne sont autorisees que si elles sont:

- externes au projet,
- generees automatiquement,
- vendorisees,
- declarees explicitement dans un fichier versionne tel que `config/coverage-gate.json`.

Une exclusion implicite est interdite.

## Coverage par domaine critique

- coeur transactionnel: 100 pourcent
- stockage: 100 pourcent
- contrats et schemas: 100 pourcent
- safety: 100 pourcent
- commissioning/as-built: 100 pourcent
- optimisation: 100 pourcent
- plugin host et permissions: 100 pourcent
- integration industrielle first-party: 100 pourcent via mocks et replays, y compris filaire, sans fil et telemetrie

## Coverage scenario

En plus de la couverture de code, le projet exige:

- coverage des fixtures officielles,
- coverage des commandes critiques,
- coverage des modes live/replay/degrade quand applicable,
- coverage des modes IA `standard` et `furnace`,
- coverage des transitions de liaison Bluetooth, Wi-Fi ou equivalentes quand presentes,
- coverage des transitions safety critiques.

## Regles CI

La CI doit:

- calculer couverture a chaque PR,
- refuser tout score < a la gate versionnee du depot,
- publier un rapport de couverture lisible,
- lister les exclusions actives,
- verifier la couverture des schemas et des handlers.

## Role de GitHub pour la CI

GitHub est la plateforme de reference pour appliquer cette politique:

- GitHub Actions execute le pipeline sur `push` et `pull_request`,
- les checks de tests, lint et couverture sont remontes comme statuts de PR,
- le backend du shell desktop Tauri est verifie par un job dedie dans la pipeline,
- les scripts locaux `scripts/lint.ps1` et `scripts/test.ps1` doivent couvrir le meme scope que les checks critiques GitHub,
- la gate Rust lue par GitHub et par les scripts locaux provient du meme fichier `config/coverage-gate.json`,
- le merge vers `main` est bloque si un check requis est rouge,
- les artefacts et rapports de couverture sont attaches aux runs du pipeline.

## Regles de PR

- aucune PR sans tests associes,
- aucune PR qui baisse la gate versionnee ou la couverture mesuree sans justification explicite,
- aucune PR qui ajoute une exclusion sans justification ecrite,
- toute commande nouvelle doit venir avec tests succes et echec.
- aucune PR ne doit etre mergee sur GitHub sans pipeline vert et checks requis valides.

## Methodes recommandees

- unit tests fins,
- integration tests par flux commande -> evenement,
- replay tests,
- property tests sur validateurs et mappings,
- fixture tests pour scenes,
- snapshot tests structures pour manifests et rapports.

## Position produit

Le 100 pourcent de couverture reste la cible produit. La gate versionnee sert a garantir une discipline executable et a eviter les faux verts ou les seuils fictifs. Elle complete mais ne remplace pas les tests scenario, les replays, les validations safety et les revues techniques.

## Criteres d'acceptation

- la cible 100 pourcent est ecrite noir sur blanc,
- la gate GitHub effectivement imposee est versionnee dans le depot,
- les exclusions sont strictement controlees,
- les domaines critiques du MVP sont explicitement couverts,
- les scripts locaux et GitHub lisent la meme politique.
