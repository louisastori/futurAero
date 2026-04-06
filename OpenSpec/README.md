# OpenSpec - FutureAero

Statut: draft-initial

Ce dossier pose la base de specification d'un logiciel desktop d'ingenierie assistee par IA locale.
Le produit vise un usage proche d'un environnement CAD/CAE industriel, avec un focus ajoute sur la robotisation, le jumeau numerique et la preparation a l'implementation reelle.

## Principes directeurs

- IA locale par defaut, sans dependance cloud obligatoire.
- Approche white-box: chaque calcul, suggestion IA et resultat de simulation doit etre explicable.
- Continuite numerique: geometrie, cinematique, commande, simulation et validation vivent dans un meme projet.
- Reproductibilite: une scene simulee doit pouvoir etre rejouee avec les memes hypotheses.
- Extensibilite: chaque outil metier est un module clairement isole.

## Ordre de lecture

1. [01-Vision-Produit.md](./01-Vision-Produit.md)
2. [02-Exigences-Systeme.md](./02-Exigences-Systeme.md)
3. [03-Architecture-Desktop-IA-Locale.md](./03-Architecture-Desktop-IA-Locale.md)
4. [04-Moteur-Simulation-Reel.md](./04-Moteur-Simulation-Reel.md)
5. [05-Roadmap-MVP.md](./05-Roadmap-MVP.md)
6. [06-Modele-De-Donnees.md](./06-Modele-De-Donnees.md)
7. [07-Format-De-Projet.md](./07-Format-De-Projet.md)
8. [08-Architecture-Technique.md](./08-Architecture-Technique.md)
9. [09-Contrats-Internes.md](./09-Contrats-Internes.md)
10. [10-Spec-Simulation-Detaillee.md](./10-Spec-Simulation-Detaillee.md)
11. [11-Spec-IA-Locale.md](./11-Spec-IA-Locale.md)
12. [12-Backlog-Dev-Ready.md](./12-Backlog-Dev-Ready.md)
13. [13-Plan-Repo-Et-Scaffold.md](./13-Plan-Repo-Et-Scaffold.md)
14. [14-Schemas-Commandes-Evenements.md](./14-Schemas-Commandes-Evenements.md)
15. [15-Matrice-De-Tests-Et-Definition-Of-Done.md](./15-Matrice-De-Tests-Et-Definition-Of-Done.md)
16. [16-Spec-Perception-Lidar-Et-Fusion-Capteurs.md](./16-Spec-Perception-Lidar-Et-Fusion-Capteurs.md)
17. [17-Spec-IA-Ultra-Locale.md](./17-Spec-IA-Ultra-Locale.md)
18. [18-Spec-Integration-Industrielle.md](./18-Spec-Integration-Industrielle.md)
19. [19-Spec-Safety-Zones-Interlocks-Lidar-Securite.md](./19-Spec-Safety-Zones-Interlocks-Lidar-Securite.md)
20. [20-Spec-As-Built-Vs-As-Designed-Et-Commissioning.md](./20-Spec-As-Built-Vs-As-Designed-Et-Commissioning.md)
21. [21-Spec-Optimization-Engine.md](./21-Spec-Optimization-Engine.md)
22. [22-Spec-Plugin-SDK.md](./22-Spec-Plugin-SDK.md)
23. [23-Politique-De-Tests-Et-Couverture-100.md](./23-Politique-De-Tests-Et-Couverture-100.md)
24. [24-Spec-Connectivite-Sans-Fil-Et-Telemetrie.md](./24-Spec-Connectivite-Sans-Fil-Et-Telemetrie.md)
25. [tool-manifest.yaml](./tool-manifest.yaml)
26. Dossier [tools](./tools)

## Ce que couvre cette premiere base

- Le cadrage produit et les limites du MVP.
- Les exigences fonctionnelles et non fonctionnelles.
- Une architecture cible pour une application desktop a forte composante native.
- Un modele de donnees et un format projet diffables.
- Des contrats internes entre outils, jobs et moteur.
- Un modele de simulation oriente "vie reelle" mais traceable.
- Une spec IA locale orientee actions inspectables.
- Un backlog et un scaffold de developpement directement exploitables.
- Une matrice de tests et une definition of done technique.
- Une couche perception avancee avec LiDAR, calibration et fusion capteurs.
- Une strategie IA locale ultra-haute performance orientee workstation et multi-modeles.
- Une integration industrielle native avec ROS2, OPC UA, PLC et robots.
- Une connectivite terrain filaire et sans fil avec Bluetooth, Wi-Fi, MQTT, WebSocket, TCP/UDP et serial.
- Une couche safety avec zones, interlocks et LiDAR securite.
- Un workflow commissioning et as-built vs as-designed.
- Un moteur d'optimisation multi-objectifs.
- Un SDK plugins extensible et isole.
- Une politique de tests avec cible de couverture a 100%.
- Un workspace desktop avec menus inspires de Visual Studio et commandes reliees au pipeline interne.
- Un shell desktop interactif avec session backend, snapshots de projet et execution de commandes depuis l espace de travail.
- Un cadre GitHub explicite pour branch protections, PR, checks requis et releases.
- Un premier decoupage des outils a programmer.
- Un workflow GitHub avec remote canonique, PR, checks obligatoires et pipeline GitHub Actions.

## Specs archivees

Les specs sorties du flux actif apres implementation et verification sont deplacees dans `OpenSpec/archive/completed/<annee-mois>/`.

- [25-Spec-UI-Workspace-Et-Menus.md](./archive/completed/2026-04/25-Spec-UI-Workspace-Et-Menus.md)
- [26-Spec-GitHub-PR-Et-Releases.md](./archive/completed/2026-04/26-Spec-GitHub-PR-Et-Releases.md)

## Ce que cette base ne tranche pas encore

- Le choix final du noyau geometrique.
- Le detail des formats d'import/export industriels.
- Le niveau de precision physique requis par secteur.
- Le choix final des premiers vendors robots, PLC et bus terrain.
- Le choix final des profils de securite, pairing et discovery par transport sans fil.
- Les regles finales de protection de branche GitHub et de publication des releases.

## Definition de white-box dans ce projet

- Les contraintes de conception sont visibles et editees.
- Les hypotheses de simulation sont versionnees.
- Les suggestions IA citent leurs sources internes au projet.
- Les actions proposees par l'IA peuvent etre rejouees ou refusees et ne modifient rien de facon silencieuse.
- Les resultats critiques peuvent etre relies a une chaine de causes lisible par un ingenieur.
