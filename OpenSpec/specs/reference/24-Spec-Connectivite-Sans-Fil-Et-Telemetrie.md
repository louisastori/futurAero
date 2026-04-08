# Spec Connectivite Sans Fil Et Telemetrie

## Objectif

Etendre l'integration locale pour prendre en compte les flux Bluetooth, Wi-Fi et autres transports de telemetrie utilises par des capteurs, boitiers edge, AGV/AMR, outillages mobiles et interfaces terrain.

## Transports cibles prioritaires

- Bluetooth Low Energy
- Bluetooth Classic / SPP
- Wi-Fi local
- MQTT sur reseau local
- WebSocket local
- TCP/UDP local
- serial / USB
- passerelles edge convertissant un protocole proprietaire vers un flux explicite

## Finalite produit

Cette couche doit permettre:

- de brancher des sources terrain non filaires au meme graphe projet,
- de journaliser discovery, pairing, securite et qualite de liaison,
- de rejouer une capture reseau ou radio hors ligne,
- de comparer comportement nominal et comportement observe sous conditions degradees,
- d'utiliser ces flux pour perception, commissioning, supervision et diagnostic.

## Objets centraux

- `ExternalEndpoint`
- `TelemetryStream`
- `NetworkCaptureDataset`
- profils de transport
- profils de timing
- profils de qualite de service
- etats de liaison

## Modes d'exploitation

### Live

- connexion directe au device, a la passerelle ou au broker local

### Replay

- relecture d'une capture reseau, Bluetooth ou serial

### Emulated

- endpoint simule produisant un flux conforme au schema

### Degraded

- injection ou replay de pertes, jitter, reconnexions et bande passante limitee

## Bluetooth et BLE

Le systeme doit pouvoir modeliser:

- l'adapter local utilise,
- le scan et la decouverte,
- le filtrage par identifiant, nom, service ou characteristic,
- le mode de pairing ou bonding,
- les notifications, lectures et ecritures explicites,
- les metriques de lien disponibles comme RSSI et taux de perte.

Cas d'usage:

- wearable de safety non certifiant mais observable,
- IMU mobile ou capteur embarque,
- outillage portable,
- boitier de diagnostic proche machine.

## Wi-Fi et telemetrie IP

Le systeme doit pouvoir modeliser:

- l'endpoint IP ou nom local,
- les modes `tcp`, `udp`, `mqtt`, `websocket`,
- la securisation locale,
- la resolution `static`, `mdns`, `broker` ou `gateway`,
- les budgets de latence et jitter,
- les pertes et reconnexions.

Cas d'usage:

- AGV/AMR,
- LiDAR ou camera sur boitier edge,
- module d'E/S Wi-Fi,
- tablette de commissioning ou supervision locale.

## Regles white-box

- aucun flux n'est accepte sans declaration du transport et du mode,
- aucun flux binaire n'est accepte sans codec ou schema explicite,
- discovery, pairing, securite et etat de liaison sont des donnees inspectables,
- les variations de latence, jitter, pertes et reconnexions sont journalisees quand observables,
- toute capture importee doit rester rejouable,
- un flux sans fil non certifie n'est pas la chaine unique de safety par defaut.

## Telemetrie et mapping

Chaque flux doit pouvoir etre relie a:

- un `Signal`,
- un `SensorModel`,
- un `ControllerModel`,
- une `CommissioningSession`,
- un `FieldCaptureDataset`,
- un `ValidationReport`.

Le mapping doit declarer:

- source,
- cible,
- unite ou codec,
- direction,
- frequence attendue,
- politique en cas de perte ou timeout.

## Diagnostics et replays

Le systeme doit permettre:

- d'importer une capture `pcap`, `ble_trace`, `mqtt_log`, `socket_dump` ou `serial_trace`,
- de rejouer le flux a vitesse reelle ou acceleree,
- d'injecter des pertes, du jitter et des deconnexions,
- de comparer l'effet sur la scene, la logique et les rapports,
- de conserver une trace diagnostique par session.

## Limites MVP

- pas de support exhaustif de tous les protocoles radio industriels proprietaires,
- pas de safety certifiee sur transport sans fil par defaut,
- pas de garantie temps reel dur sur Wi-Fi ou Bluetooth,
- pas de gestion de secrets distribues hors poste local dans le MVP.

## Criteres d'acceptation

- un endpoint Bluetooth ou Wi-Fi peut etre declare,
- un stream telemetrique peut etre mappe au graphe projet,
- une capture reseau ou radio peut etre importee et rejouee,
- un scenario de degradation de lien peut etre diagnostique,
- les metriques de liaison utiles sont visibles quand le transport les expose.
