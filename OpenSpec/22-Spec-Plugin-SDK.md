# Spec Plugin SDK

## Objectif

Definir un SDK plugins stable, isole et versionnable pour etendre FutureAero sans fragiliser le coeur.

## Finalite produit

Le SDK doit permettre d'ajouter:

- outils metier,
- connecteurs industriels,
- solveurs ou adaptateurs,
- import/export,
- panneaux UI,
- analyses et rapports.

## Types de plugins

- domaine metier,
- connecteur integration,
- analyseur,
- UI panel,
- import/export,
- adaptateur IA,
- provider de perception ou optimisation.

## Manifest

Chaque plugin doit declarer:

- identite,
- version,
- compatibilite,
- capabilities,
- permissions,
- entrypoints,
- migrations eventuelles.

## Permissions minimales

- lecture graphe projet,
- mutation graphe projet,
- acces assets,
- acces UI,
- acces integration externe,
- acces IA,
- acces perception,
- acces optimisation.

## Regles d'isolation

- aucun acces implicite au systeme de fichiers projet,
- aucune mutation hors commandes coeur,
- permissions explicites et revocables,
- plugins desactivables sans corrompre le projet.

## Contributions possibles

- commandes,
- evenements,
- panneaux UI,
- formats de fichiers,
- adaptateurs protocole,
- calculateurs et rapports,
- widgets de visualisation.

## Compatibilite

- version semver du SDK,
- compatibilite declaree plugin <-> application,
- degradations propres si capability absente.

## Tests du SDK

Le SDK doit fournir:

- fixtures de plugins,
- tests de permissions,
- tests de compatibilite manifest,
- tests d'isolation,
- tests de hot enable/disable.

## Regles white-box

- chaque capability doit etre declarative,
- chaque permission doit etre visible,
- chaque plugin actif doit etre listable,
- chaque erreur plugin doit etre attribuable.

## Criteres d'acceptation

- un plugin peut etre installe, active et desactive,
- un plugin peut contribuer une capability sans ecriture directe dans le coeur,
- les permissions d'un plugin sont auditables,
- un plugin incompatible est bloque proprement.
