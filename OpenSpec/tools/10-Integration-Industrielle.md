# Outil 10 - Integration Industrielle

## But

Relier le graphe projet a des systemes externes ROS2, OPC UA, PLC, controleurs robots et flux filaires/sans fil.

## Portee MVP etendu

- endpoints ROS2, OPC UA, PLC, robots, Bluetooth et Wi-Fi
- streams MQTT, WebSocket, TCP/UDP et serial
- mappings signaux et variables
- replay de traces
- replay de liaisons degradees
- mode live, replay et emulated

## Entrees

- scene projet
- endpoints externes
- mappings
- traces industrielles

## Sorties

- etat de connexion
- etat de liaison et qualite de lien
- signaux mappes
- traces rejouables
- rapport de binding

## Regles white-box

- chaque mapping doit etre inspectable
- chaque ecriture externe doit etre explicite
- chaque flux sans fil doit exposer son discovery, sa securite et sa qualite de liaison
- chaque trace doit rester rejouable

## Criteres d'acceptation

- un endpoint externe peut etre declare
- un signal du projet peut etre mappe a une source externe
- un flux Bluetooth, Wi-Fi ou telemetrique peut etre rejoue
- une trace peut etre importee et rejouee
