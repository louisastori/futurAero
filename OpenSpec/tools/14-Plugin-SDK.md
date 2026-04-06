# Outil 14 - Plugin SDK

## But

Permettre l'extension controlee du produit sans fragiliser le coeur.

## Portee MVP etendu

- manifests plugins
- permissions explicites
- contributions UI et outils
- activation et desactivation
- compatibilite SDK

## Entrees

- manifest plugin
- capabilities
- permissions
- points d'entree

## Sorties

- plugin installe
- etat d'activation
- contribution chargee
- audit de permissions

## Regles white-box

- chaque capability doit etre declaree
- chaque permission doit etre visible
- aucun plugin n'ecrit directement dans le coeur

## Criteres d'acceptation

- un plugin peut etre installe et active
- ses permissions sont auditables
- un plugin incompatible est bloque proprement
