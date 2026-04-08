# Roadmap MVP

## Principe

Le perimetre doit etre volontairement plus petit qu'un logiciel CAD industriel etabli. Le MVP cherche la continuite entre conception, robotisation et simulation explicable, pas la couverture exhaustive.

## Phase 0 - Fondation spec et modele de donnees

- Stabiliser les objets metier.
- Definir le format projet.
- Definir le contrat des outils.
- Poser le pipeline d'historisation et de provenance.

## Phase 1 - Coeur CAD minimal

- Esquisses 2D avec contraintes essentielles.
- Generation de pieces 3D parametriques de base.
- Proprietes de materiaux et de masse.
- Navigation scene et inspection des objets.

## Phase 2 - Assemblages et robotisation

- Assemblages avec liaisons principales.
- Chaines cinematiques.
- Modelisation d'une cellule robotique.
- Zones de securite et verification de collision.

## Phase 3 - Simulation et IA locale

- Scenarios rejouables.
- Tableaux de resultats et rapports.
- Assistant IA local pour aide a la conception et a l'analyse.
- Explication des conflits et recommandations.
- Base perception: LiDAR virtuel, calibration, replay de donnees capteurs.

## Phase 4 - Validation et export

- Rapports d'ecarts.
- Export des donnees ciblees.
- Base de preparation a l'implementation reelle.
- Comparaison scene reelle / scene numerique par perception.

## Phase 5 - Integration et Safety

- ROS2, OPC UA, PLC et controleurs robots.
- Zones, interlocks, permissifs et LiDAR securite.
- Replay de traces industrielles.

## Phase 6 - Commissioning et Optimisation

- Workflow as-built vs as-designed.
- Sessions de commissioning et rapports terrain.
- Optimisation multi-objectifs.
- SDK plugins et ecosysteme d'extensions.

## Definition du MVP executable

Le MVP sera considere utile si un utilisateur peut:

- creer une petite cellule robotique,
- y placer des composants mecaniques,
- definir une sequence simple,
- lancer une simulation interpretable,
- obtenir une aide IA locale sur les conflits ou ameliorations.

## Risques majeurs

- Viser trop large et reproduire trop tot un CATIA complet.
- Sous-estimer la difficulte du noyau geometrique et des solveurs.
- Ajouter une IA avant d'avoir un modele de donnees propre.
- Melanger rendu visuel et verite physique sans contrat clair.
