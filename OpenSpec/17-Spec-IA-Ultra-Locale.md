# Spec IA Ultra Locale

## Objectif

Definir un mode IA locale "meilleur du meilleur" privilegiant la qualite de raisonnement, la profondeur d'analyse et l'usage maximal des ressources de la machine.

## Positionnement

Ce mode n'est pas un mode economique. Il assume:

- gros modeles locaux,
- orchestration multi-modeles,
- gros contextes,
- saturation GPU/CPU/RAM/NVMe quand necessaire,
- latence acceptee si elle augmente nettement la qualite.

## Nom de mode recommande

- `max`
- `furnace`

`furnace` est le mode le plus agressif et doit chercher la meilleure sortie locale acceptable avant de rendre la main.

## Objectifs du mode `furnace`

- maximiser la qualite des explications,
- maximiser la precision des suggestions structurees,
- croiser CAD, assemblage, simulation, perception et historique,
- produire un raisonnement cite, exploitable et riche,
- utiliser plusieurs modeles specialises plutot qu'un seul modele moyen.

## Architecture logique

### Modeles ou roles recommandes

- `frontier_reasoner_local`: raisonnement principal long contexte
- `code_structurer_local`: generation de commandes, schemas, patchs et transformations
- `vision_geometry_local`: lecture de viewport, captures, nuages de points, cartes
- `embedder_local`: embeddings projet/perception/docs
- `reranker_local`: tri du contexte
- `compressor_local`: synthese de contexte long
- `critic_local`: relecture, contradiction check, validation interne

## Strategie d'orchestration

1. construire un contexte brut tres large,
2. reranker et compresser sans perdre les refs critiques,
3. lancer le reasoner principal,
4. faire relire par un critic local sur les points a risque,
5. convertir en sortie structuree,
6. verifier schemas, refs et commandes,
7. journaliser la chaine complete.

## Politique ressources

### Principe

Par defaut, en mode `furnace`, le systeme doit viser la meilleure qualite locale permise par la station et non la plus faible consommation.

### Regles

- privilegier le modele local le plus fort disponible,
- preferer precision elevee ou quantization minimale compatible,
- utiliser le multi-GPU si present,
- reserver de la RAM pour gros contextes et index en memoire,
- utiliser NVMe local pour caches, KV cache, index et artefacts temporaires,
- autoriser des batchs plus lourds si cela augmente la qualite du retrieval ou de la critique.

## Classes materiel ciblees

### Classe A

- 1 GPU haut de gamme
- 24 a 48 Go VRAM
- 64 a 128 Go RAM
- NVMe rapide

### Classe B

- 2 GPU ou plus
- 48 a 192 Go VRAM cumulee
- 128 a 512 Go RAM
- plusieurs NVMe

### Classe C

- workstation ou serveur local multi-GPU
- plus de 192 Go VRAM cumulee
- plus de 512 Go RAM
- stockage NVMe massif

Le mode `furnace` doit monter en qualite avec la classe machine sans changer le contrat produit.

## Contexte maximal exploitable

Le mode `furnace` doit pouvoir croiser:

- graphe projet complet,
- historique de commandes,
- journaux d'evenements,
- plusieurs runs de simulation,
- plusieurs runs perception,
- rapports de validation,
- selection active,
- contraintes, derives et incidents.

## Strategies de contexte

- index mixte graph + texte + embeddings,
- fenetrage par criticite temporelle,
- summaries hierarchiques pour gros assemblages,
- compression sans perte des refs critiques,
- conservation des identifiants source a chaque etape.

## Types de raisonnement avances

- analyse causale multi-source,
- recherche d'alternatives de configuration,
- comparaison multi-runs,
- explication de derives perception vs nominal,
- critique de sequence robotique,
- generation de plan de correction,
- redaction technique longue.

## Modes de sortie

### `deep_explain`

- explication longue et multi-source

### `design_critic`

- critique de conception et de risques

### `scenario_optimizer`

- suggestions de sequence, implantation, parametrage

### `industrial_report`

- note technique dense, structuree et citee

## Regles white-box renforcees

- la sortie doit citer ses refs de maniere systematique,
- les contradictions detectees par le critic doivent etre journalisees,
- toute suggestion doit inclure hypotheses, limites et risque,
- les reponses longues doivent rester decomposables en observations puis interpretation,
- aucun "black-box apply" n'est autorise, meme en mode `furnace`.

## Validation interne a deux passes

Passage 1:

- generation principale

Passage 2:

- critique locale
- verification schema
- verification refs
- verification commandes

Une sortie a fort impact peut exiger une troisieme passe de coherence.

## Priorites de qualite

Ordre de priorite en mode `furnace`:

1. exactitude structurelle
2. exactitude factuelle sur le projet
3. profondeur de raisonnement
4. couverture des alternatives
5. latence

## Cas d'usage prioritaires

- expliquer une collision complexe impliquant mecanique, logique et perception,
- proposer une reconfiguration de cellule avec justification complete,
- comparer plusieurs iterations de simulation et de perception,
- generer un rapport d'industrialisation argumente,
- analyser un ecart LiDAR/nominal et proposer les causes probables.

## Journalisation supplementaire

Le mode `furnace` doit journaliser en plus:

- profil machine detecte,
- mode ressources actif,
- modeles sollicites,
- ordre de passage des modeles,
- temps par etape,
- taille du contexte brut et compresse,
- score de confiance final et score critique.

## Seuils d'activation

Le mode `furnace` ne s'active que si:

- l'utilisateur le demande explicitement, ou
- une preference projet l'impose, ou
- une tache marquee critique le requiert.

## Garde-fous

- laisser une reserve systeme minimale pour ne pas tuer la machine,
- pouvoir annuler une inference longue,
- reduire proprement le profil si la VRAM manque,
- ne jamais desynchroniser l'UI du coeur metier.

## Criteres d'acceptation

- le systeme peut router une meme demande vers plusieurs modeles locaux,
- le mode `furnace` privilegie la qualite plutot que la latence,
- la chaine de critique interne est journalisee,
- les sorties restent conformes aux schemas et garde-fous du projet.
