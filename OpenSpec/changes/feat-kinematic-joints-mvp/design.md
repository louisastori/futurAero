# Design: MVP Joint Model

## Context

Le depot couvre maintenant l assemblage explicite au niveau occurrences et mates, mais aucun type partage ne represente encore un joint mecanique. Le backlog `ST-303` demande pourtant des joints `fixed|revolute|prismatic` avec position pilotable, limites et degres de liberte exposes. Ce besoin se situe a l intersection de `faero-assembly` et `faero-robotics`: l assemblage doit porter la structure, la robotique doit pouvoir consommer un etat lisible, et le coeur doit garder une mutation white-box.

Le change doit rester strictement MVP:
- pas de solveur multibody complet,
- pas de cinematique inverse,
- pas de UI 3D riche d edition,
- mais un contrat clair et persistant pour les joints et leur etat.

## Goals / Non-Goals

**Goals:**
- definir un modele partage de joint compatible avec l assemblage persiste,
- supporter les types `fixed`, `revolute` et `prismatic`,
- persister limites, etat courant et degres de liberte dans un format lisible,
- fournir des commandes coeur explicites pour creer un joint et mettre a jour son etat,
- exposer un resume lisible dans les snapshots desktop.

**Non-Goals:**
- resoudre une chaine cinematique complexe ou des boucles fermees,
- synchroniser un joint avec la simulation physique complete,
- livrer une surface UI d edition graphique des axes et repères,
- couvrir d autres types de joints au dela du MVP.

## Decisions

### 1. Porter les joints dans le payload `Assembly`
Comme pour les occurrences et mates MVP, les joints restent persistes dans l entite `Assembly`. Cela garde un point d entree unique pour la lecture desktop, la persistance `.faero` et les futures integrations.

Alternative consideree:
- stocker les joints comme entites top-level separees.
- Rejetee pour cette iteration car cela complexifie inutilement le graphe avant d avoir le besoin de references transverses plus riches.

### 2. Modeliser explicitement le type, les limites et l etat courant
Le contrat MVP doit inclure:
- `jointType`,
- `sourceOccurrenceId`,
- `targetOccurrenceId`,
- `axis`,
- `limits`,
- `currentPosition`,
- `degreesOfFreedom`.

Alternative consideree:
- deriver les degres de liberte uniquement depuis `jointType` au moment de l affichage.
- Rejetee car le change veut une lecture white-box et testable directement dans le projet.

### 3. Ajouter un pipeline coeur minimal `joint.create` / `joint.state.set`
Le coeur doit porter les commandes de creation et de mise a jour de l etat de joint. Cela permet d alimenter l historique commande/evenement et de preparer le branchement futur de `faero-robotics`.

Alternative consideree:
- modifier le payload d assemblage via `ReplaceEntity`.
- Rejetee pour les memes raisons que les occurrences et mates: on perd le contrat metier et les evenements auditable.

### 4. Garder une evaluation des degres de liberte simple et deterministe
Pour ce jalon, `fixed=0`, `revolute=1`, `prismatic=1`. Le change se concentre sur la lisibilite et la validite du contrat, pas sur une analyse cinematique globale plus complexe.

Alternative consideree:
- calculer des degres de liberte aggregates au niveau assemblage avec interaction entre mates et joints.
- Rejetee a ce stade car cela ferait glisser le change vers un solveur mecanique plus ambitieux que le MVP cible.

## Risks / Trade-offs

- [Modele trop simplifie pour certains cas reels] → Mitiger en documentant clairement le scope MVP et en gardant le schema extensible.
- [Chevauchement avec les mates assembly] → Mitiger en traitant le joint comme une liaison mecanique explicite distincte des mates geometriques.
- [Attentes trop fortes cote robotique] → Mitiger en ne promettant dans ce change que des etats et limites lisibles, pas une chaine robot complete.

## Migration Plan

- Changement additif dans `faero-types` et `faero-core`.
- Les assemblies existants restent valides avec une liste de joints vide par defaut.
- Aucun rollback special hors revert du commit n est necessaire pour ce jalon.

## Open Questions

- Faut-il exposer des presets d axes locaux au desktop des ce change ou garder un axe explicite unique saisi par le backend ?
- Le prochain increment doit-il brancher d abord `joint.state.set` au shell desktop ou a `faero-robotics` pour les sequences ?
