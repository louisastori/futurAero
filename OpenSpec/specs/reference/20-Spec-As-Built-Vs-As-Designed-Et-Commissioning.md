# Spec As-Built Vs As-Designed Et Commissioning

## Objectif

Definir un workflow de mise en service reliant le nominal numerique au reel observe.

## Finalite produit

Cette couche doit permettre:

- de lancer une session de commissioning,
- de connecter scene numerique et systemes terrain,
- d'importer des captures terrain,
- de comparer le reel observe au modele attendu,
- de journaliser ajustements, ecarts et validation finale.

## Phases d'une session

1. selection du nominal cible,
2. connexion ou replay des sources terrain,
3. calibration et verifications prealables,
4. captures terrain,
5. comparaison as-built vs as-designed,
6. ajustements et corrections,
7. revalidation,
8. cloture et sign-off.

## Donnees d'entree

- scene nominale,
- profils de calibration,
- traces PLC/robot,
- captures perception,
- etats safety,
- objectifs de commissioning.

## Sorties

- session de commissioning,
- captures versionnees,
- rapport d'ecarts,
- journal d'ajustements,
- statut de validation.

## Comparaisons minimales

- position d'equipements,
- enveloppes de securite,
- trajectoires ou cibles,
- etats d'E/S,
- presence d'obstacles ou d'ecarts geometriques.

## Tolerances

Le systeme doit supporter:

- tolerances geometriques,
- tolerances temporelles,
- tolerances de sequence,
- tolerances safety.

## Sign-off

Une session cloturee doit conserver:

- hypotheses,
- ecarts restants,
- corrections appliquees,
- etat final de validation.

## Regles white-box

- chaque ecart doit citer sa source nominale et sa source observee,
- chaque ajustement doit etre journalise,
- chaque session doit etre rejouable au moins en mode analyse.

## Criteres d'acceptation

- une session de commissioning peut etre creee et suivie,
- une capture terrain peut etre rattachee a une session,
- un rapport as-built vs as-designed peut quantifier les ecarts,
- une session peut etre reouverte, ajustee et cloturee avec statut.
