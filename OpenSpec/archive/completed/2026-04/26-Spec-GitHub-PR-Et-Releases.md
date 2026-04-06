# Spec GitHub PR Et Releases

Statut: implemente-et-teste puis archive le 2026-04-06

Implementation de reference:

- `.github/workflows/ci.yml`
- `README.md`
- `OpenSpec/13-Plan-Repo-Et-Scaffold.md`
- `OpenSpec/15-Matrice-De-Tests-Et-Definition-Of-Done.md`
- `OpenSpec/23-Politique-De-Tests-Et-Couverture-100.md`

## Objectif

Definir le role exact de GitHub pour la gouvernance du depot, la protection de branche, le pipeline de PR et la publication des releases.

## Remote canonique

- le remote GitHub de reference est `origin`
- le depot canonique est `https://github.com/louisastori/futurAero.git`
- la branche de reference est `main`

## Branch protections attendues

La branche `main` doit etre protegee avec les regles suivantes:

- push direct interdit sauf cas exceptionnel reserve aux administrateurs si explicitement voulu
- merge uniquement via pull request
- checks obligatoires tous au vert avant merge
- branche a jour avec `main` avant merge si un check requis l'exige
- historique lineaire recommande
- suppression de branche apres merge recommandee

## Checks requis de PR

Checks GitHub attendus:

- `rust / Format`
- `rust / Lint`
- `rust / Tests`
- `rust / Coverage gate`
- `frontend-scaffold / Frontend lint`
- `frontend-scaffold / Frontend tests`
- `desktop-shell / Desktop shell format`
- `desktop-shell / Desktop shell lint`
- `desktop-shell / Desktop shell backend tests`

Le nom exact peut suivre le workflow GitHub Actions, mais la couverture des domaines ci-dessus est obligatoire.

## Regles de revue PR

- au moins une revue humaine avant merge
- PR liee a au moins une story ou un scope OpenSpec
- description claire du changement, du risque et des tests executes
- aucune PR rouge ou partiellement verifiee vers `main`

## Politique de merge

Mode recommande:

- `Squash and merge` par defaut pour garder un historique lisible

Regles:

- titre de squash explicite
- message de merge contenant le scope principal
- pas de merge d'une PR qui contourne les checks requis

## Releases

Le pipeline de release doit a terme permettre:

- creation d'un tag versionne
- build des artefacts desktop
- publication des notes de release
- rattachement des checks de pipeline a la release candidate

## Criteres d'acceptation

- la protection de `main` est definie dans les specs
- les checks requis sont listes explicitement
- les PR et releases ont un workflow GitHub lisible
- la politique de merge est coherente avec la CI du depot
