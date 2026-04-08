# Spec Perception LiDAR Et Fusion Capteurs

## Objectif

Definir une couche perception avancee reliant le modele numerique a des observations capteurs reellement exploitables sur machine ou en atelier.

## Finalite produit

Cette couche doit permettre:

- de simuler un capteur avant installation,
- de rejouer une observation terrain,
- de fusionner plusieurs capteurs,
- de reconstruire une carte ou un nuage de points,
- de comparer l'observe au modele CAO ou a la cellule attendue,
- d'expliquer les ecarts de facon white-box.

## Capteurs cibles

### Capteurs prioritaires

- LiDAR 2D de securite
- LiDAR 3D rotatif
- LiDAR 3D solid-state
- camera RGB
- camera profondeur
- stereo
- IMU

### Capteurs complementaires

- radar courte portee
- capteur ultrason
- capteur force/couple
- encodeurs externes

## Cas d'usage prioritaires

- verifier qu'un LiDAR couvre la zone critique d'une cellule,
- detecter un obstacle non modele,
- comparer une implantation terrain au jumeau numerique,
- suivre une piece ou un AGV dans une cellule,
- reconstruire une zone de travail a partir de scans,
- mesurer une derive de position ou de hauteur,
- fusionner LiDAR + camera + IMU pour localiser un sous-systeme.

## Chaine de traitement

1. acquisition capteurs,
2. synchronisation temporelle,
3. calibration intra/extrinseque,
4. filtrage et nettoyage,
5. fusion capteurs,
6. reconstruction,
7. detection / segmentation / localisation,
8. comparaison au modele nominal,
9. emission de sorties versionnees.

## Sorties attendues

- scans LiDAR horodates,
- nuages de points,
- cartes d'occupation 2D/3D,
- cartes de distance,
- objets detectes,
- pistes d'objets dynamiques,
- estimation de pose,
- rapport d'ecarts scene observee / scene attendue.

## Regles white-box

- tout pipeline perception doit declarer ses etapes,
- tout resultat doit citer les capteurs sources,
- toute calibration doit exposer ses metriques qualite,
- toute carte ou detection doit etre rattachee a un repere et un horodatage,
- tout ecart detecte doit etre mesurable et rejouable.

## Calibration

Types minimaux:

- calibration intrinseque camera,
- calibration extrinseque capteur -> cellule,
- calibration temporelle entre capteurs,
- calibration multi-capteurs pour un rig.

Metriques minimales:

- erreur RMS,
- couverture des observations,
- stabilite temporelle,
- date de validite,
- dataset de calibration source.

## LiDAR

### Parametres essentiels

- type de LiDAR,
- nombre de canaux,
- frequence,
- champ de vue horizontal/vertical,
- portee mini/maxi,
- resolution angulaire,
- modele de bruit,
- comportement des multi-retours,
- sensibilite aux occultations.

### Sorties minimales

- balayage brut,
- points filtres,
- intensite ou equivalent si disponible,
- taux de couverture,
- points hors modele,
- alertes de derive.

## Fusion capteurs

Modes minimaux:

- LiDAR seul,
- LiDAR + IMU,
- LiDAR + camera,
- LiDAR + camera + IMU.

Le systeme doit rendre explicites:

- les capteurs actifs,
- l'ordre de fusion,
- les seuils utilises,
- les pertes de confiance,
- les facteurs de rejet.

## Reconstruction et cartes

Representations supportees:

- nuage de points brut,
- nuage de points aligne,
- carte d'occupation 2D,
- voxel grid 3D,
- carte de distance,
- carte semantique optionnelle.

## Comparaison scene observee / scene nominale

Le systeme doit pouvoir comparer:

- surfaces attendues vs surfaces observees,
- zones libres attendues vs occupees,
- presence d'obstacles,
- position d'equipements fixes,
- enveloppes de securite.

Sorties minimales:

- ecart maximum,
- ecart moyen,
- zones en derive,
- zones non observees,
- obstacles inconnus.

## Modes d'exploitation

### Mode simulation

- capteurs synthetiques branches sur une scene numerique

### Mode replay

- donnees terrain rejouees localement

### Mode hybride

- comparaison d'une observation terrain a un modele numerique cible

## Limites MVP

- pas de SLAM global complet au premier increment,
- pas de segmentation semantique avancee obligatoire,
- pas de traitement temps reel dur sur gros volumes hors cible machine adaptee.

## Extensions recommandees

- SLAM local ou semi-global,
- suivi multi-objets,
- calibration assistee par IA locale,
- comparaison as-built / as-designed,
- audit de zones de securite par perception.

## Criteres d'acceptation

- un LiDAR 2D ou 3D peut etre defini dans une cellule,
- un pipeline perception peut etre configure et rejoue,
- une calibration et ses metriques peuvent etre inspectees,
- un nuage de points ou une carte peuvent etre compares a la scene nominale,
- les ecarts remontes sont relies a un capteur, un repere et un instant.
