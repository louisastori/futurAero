# Design: Assembly Command Pipeline

## Context

Le depot dispose deja d un solveur simple dans `crates/faero-assembly` et d une integration desktop qui peut creer un `Assembly` avec des occurrences et mates pre-remplis. En revanche, ce flux ne reflète pas encore les contrats OpenSpec de type `assembly.occurrence.add`, `assembly.occurrence.transform`, `assembly.mate.add` et `assembly.mate.remove`. Le coeur partage ne porte pas de types assembly dedies, la mutation passe surtout par un gros payload d entite et le recent activity ne distingue pas encore clairement les operations d assemblage.

Le prochain increment doit rester MVP et white-box:
- tout doit rester lisible dans le projet,
- toute mutation doit rester traçable par commande/evenement,
- l integration doit rester locale et sans dependance additionnelle,
- le scope doit couvrir occurrences + mates sans ouvrir tout de suite les joints cinematiques complets.

## Goals / Non-Goals

**Goals:**
- introduire un contrat OpenSpec `assembly` testable et archiveable,
- definir des types partages pour les occurrences, mates et solve reports d assemblage,
- brancher le backend desktop sur un pipeline de commandes d assemblage explicites,
- recalculer et persister un solve report apres chaque mutation d assemblage,
- exposer a l UI un detail lisible sur le nombre d occurrences, de mates, le statut et les warnings.

**Non-Goals:**
- implementer des joints `fixed|revolute|prismatic`,
- introduire des references geometriques riches de type faces/edges CAD,
- ajouter un editeur graphique 3D de mates dans cette iteration,
- remodeler tout le stockage du projet hors des besoins assembly MVP.

## Decisions

### 1. Porter l assemblage comme types partages explicites
Le change ajoute des structures assembly partagees dans `faero-types` pour eviter que le backend desktop et les tests manipulent des blobs JSON differents. Cela garde un modele unique entre backend, stockage, UI et IA.

Alternative consideree:
- Continuer a stocker des objets assembly uniquement comme JSON libre dans `EntityRecord.data`.
- Rejetee car cela fragilise les validations, les tests de serialisation et la lisibilite du contrat.

### 2. Conserver l entite `Assembly` comme point d ancrage persiste
Les occurrences, mates et le solve report restent portes par une entite `Assembly` dans le graphe partage, au lieu d introduire une nouvelle famille top-level de stockage. Cela limite le cout d integration avec `faero-storage`, les snapshots desktop et les fixtures existantes.

Alternative consideree:
- Creer des collections top-level dediees pour les occurrences et mates.
- Rejetee pour ce jalon car la valeur immediate est dans la pipeline de commandes, pas dans un remaniement complet du format `.faero`.

### 3. Ajouter des commandes coeur dediees a l assemblage
`faero-core` doit exposer des commandes explicites pour creer un assemblage, ajouter ou deplacer une occurrence, ajouter un mate et supprimer un mate. Ces commandes doivent produire des kinds reconnaissables dans l activite (`assembly.occurrence.add`, `assembly.occurrence.transform`, `assembly.mate.add`, `assembly.mate.remove`) puis recalculer le solve report et publier `assembly.solved` ou `assembly.unsolved`.

Alternative consideree:
- Continuer a tout faire avec `CreateEntity` / `ReplaceEntity`.
- Rejetee car on perd le contrat metier et les evenements assembly explicites demandes par OpenSpec.

### 4. Garder un modele de mate MVP oriente occurrences
Le solveur actuel raisonne sur des occurrences et un type de mate simple (`coincident`, `offset`). Le change reste sur ce niveau de detail pour permettre une mise en oeuvre immediate et testable. Les references geometriques fines de type face/edge restent un prolongement futur lorsque la geometrie assembly sera plus riche.

Alternative consideree:
- Imposer des `sourceRef` / `targetRef` geometriques complets des maintenant.
- Rejetee car le depot ne porte pas encore les primitives CAD necessaires pour tenir proprement cette promesse.

## Risks / Trade-offs

- [Compatibilite des donnees] → Mitiger en acceptant les `Assembly` deja presents puis en normalisant leur payload lors des prochaines mutations.
- [Double representation resume + solve report] → Mitiger en deriveant le resume UI du solve report persiste plutot qu en maintenant deux sources de verite.
- [Scope trop large sur l UI] → Mitiger en limitant l integration desktop a creation, lecture et mutations de base, sans editeur 3D complet.
- [Divergence avec les schemas de reference long-form] → Mitiger en documentant explicitement que ce change couvre le contrat MVP occurrence-level, pas encore les references geometriques riches.

## Migration Plan

- Aucun changement de deploiement externe n est necessaire.
- Les nouveaux types assembly sont introduits de maniere additive.
- Les fixtures ou entites assembly existantes restent lisibles; une normalisation opportuniste peut etre appliquee lors de leur mise a jour par le backend desktop.

## Open Questions

- Faut-il ajouter un exemple de fixture `.faero` centree assembly dans ce change ou garder cela pour l iteration suivante consacree aux sous-ensembles et joints ?
- Quelle granularite de messages UI est la plus utile apres une mutation assembly: notification courte unique ou detail complet visible seulement dans le panneau de proprietes ?
