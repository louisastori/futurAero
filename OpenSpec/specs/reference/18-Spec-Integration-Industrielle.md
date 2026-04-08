# Spec Integration Industrielle

## Objectif

Definir une couche d'integration industrielle locale reliant le graphe projet a des systemes externes reels ou emules.

## Protocoles et cibles prioritaires

- ROS2
- OPC UA
- PLC generiques
- controleurs robots
- Bluetooth Low Energy
- Bluetooth Classic / SPP
- Wi-Fi local
- MQTT, WebSocket, TCP/UDP
- serial / USB et gateways edge

## Finalite produit

Cette couche doit permettre:

- de lier scene numerique et signaux terrain,
- de relier aussi des capteurs, modules mobiles et boitiers terrain en filaire ou sans fil,
- de rejouer une integration hors ligne,
- de preparer une mise en service reelle,
- d'observer et de journaliser les echanges sans boite noire.

## Objets centraux

- endpoints externes,
- mappings de signaux,
- bindings robot <-> controleur,
- streams telemetriques,
- traces industrielles,
- captures reseau et sans fil,
- profils de connexion,
- profils de transport et de securite,
- etats de liaison.

## Modes d'exploitation

### Live

- connexion directe a un systeme externe

### Replay

- relecture locale d'une trace capturee

### Emulated

- endpoint simule sans materiel reel

### Gateway

- endpoint acces via passerelle locale ou edge box

## ROS2

Le systeme doit pouvoir modeliser:

- nodes,
- topics,
- services,
- actions,
- frames et conventions de repere,
- QoS utiles au projet.

Cas d'usage:

- publier et consommer etats de cellule,
- rejouer des topics,
- mapper perception et robotique au graphe projet.

## OPC UA

Le systeme doit pouvoir modeliser:

- endpoint,
- namespaces,
- node ids,
- variables et methodes,
- profils de securite.

Cas d'usage:

- exposition d'etats projet,
- lecture/ecriture de variables process,
- replay de valeurs historisees.

## PLC

Le systeme doit pouvoir modeliser:

- tags,
- blocs ou zones memoire,
- cyclage logique,
- etats d'E/S,
- profils vendor abstraits.

Cas d'usage:

- synchronisation de permissifs et interlocks,
- capture de traces de commissioning,
- comparaison logique attendue / logique observee.

## Robots

Le systeme doit pouvoir modeliser:

- controleur cible,
- programmes ou sequences lies,
- positions, etats et alarmes,
- mappings d'axes et de frames.

Cas d'usage:

- preparer un binding robot/cellule,
- comparer etat robot attendu et observe,
- rejouer une trace de mouvement.

## Bluetooth et BLE

Le systeme doit pouvoir modeliser:

- adapters locaux,
- discovery, scan et filtrage de devices,
- services, characteristics et notifications,
- pairing, bonding et niveau de securite,
- RSSI, pertes et reconnections.

Cas d'usage:

- capteur mobile ou wearable,
- module E/S compact,
- telemetrie d'outillage portable,
- diagnostic terrain depuis un boitier proche.

## Wi-Fi et telemetrie IP

Le systeme doit pouvoir modeliser:

- endpoints Wi-Fi locaux,
- modes `tcp`, `udp`, `mqtt`, `websocket`,
- profil de securite reseau,
- qualite de lien: debit, latence, jitter, pertes,
- decouverte `static`, `mdns`, `broker` ou `gateway`.

Cas d'usage:

- AGV/AMR ou robot mobile,
- boitier LiDAR ou vision en edge,
- passerelle de telemetrie locale,
- tablette de commissioning sur reseau usine.

## Regles de connectivite white-box

- chaque flux doit declarer transport, mode, direction et codec ou schema,
- chaque endpoint sans fil doit declarer discovery, pairing ou securite quand applicable,
- la qualite de liaison observable doit etre journalisee: latence, jitter, pertes, RSSI ou equivalent,
- un flux critique doit pouvoir etre rejoue a partir d'une trace capturee,
- une liaison sans fil non certifiee n'est pas acceptee par defaut comme unique chaine safety.

## Regles white-box

- chaque mapping doit citer source, cible et conversion,
- chaque endpoint doit declarer son mode live/replay/emulated,
- chaque endpoint sans fil doit declarer son profil de transport,
- chaque trace importee doit etre rejouable,
- aucune ecriture externe ne doit etre implicite.

## Limites MVP

- pas d'objectif de certification protocolaire,
- pas de support exhaustif de tous vendors des le premier increment,
- pas de promesse de safety certifiee sur Bluetooth ou Wi-Fi sans profil valide,
- pas de logique temps reel dur garantie par le shell desktop seul.

## Criteres d'acceptation

- un endpoint ROS2, OPC UA, PLC, Bluetooth ou Wi-Fi peut etre declare dans le projet,
- un signal du graphe peut etre mappe a un endpoint externe,
- un stream telemetrique peut etre mappe a un signal ou a une vue projet,
- un etat de liaison degrade peut etre rejoue et diagnostique,
- une trace industrielle peut etre importee et rejouee,
- un robot du modele peut etre relie a un controleur cible via binding explicite.
