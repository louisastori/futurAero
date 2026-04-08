# OpenSpec - FutureAero

Statut: draft-initial

Ce dossier suit maintenant un flux `spec-driven`:

- `specs/`: specs stables et documents de reference.
- `changes/`: changements actifs, proposes puis promus.
- `archive/`: specs retirees du flux actif apres implementation et verification.
- `config.yaml`: configuration du moteur OpenSpec.
- `tool-manifest.yaml` et `tools/`: decomposition outillage et dependances.

## Principes directeurs

- IA locale par defaut, sans dependance cloud obligatoire.
- Approche white-box: chaque calcul, suggestion IA et resultat de simulation doit etre explicable.
- Continuite numerique: geometrie, cinematique, commande, simulation et validation vivent dans un meme projet.
- Reproductibilite: une scene simulee doit pouvoir etre rejouee avec les memes hypotheses.
- Extensibilite: chaque outil metier est un module clairement isole.

## Ordre de lecture

1. [specs/system/spec.md](./specs/system/spec.md)
2. [specs/reference/README.md](./specs/reference/README.md)
3. [archive/completed/2026-04/init-mvp-futureaero/proposal.md](./archive/completed/2026-04/init-mvp-futureaero/proposal.md)
4. [archive/completed/2026-04/init-mvp-futureaero/design.md](./archive/completed/2026-04/init-mvp-futureaero/design.md)
5. [archive/completed/2026-04/init-mvp-futureaero/tasks.md](./archive/completed/2026-04/init-mvp-futureaero/tasks.md)
6. [archive/README.md](./archive/README.md)

## Regles de rangement

- Les anciens fichiers `01..24` ne vivent plus a la racine.
- Les specs stables longues sont migrees dans [specs/reference/](./specs/reference/README.md).
- Les specs canoniques courtes promues depuis `changes/` vivent dans `specs/<domaine>/spec.md`.
- Chaque change OpenSpec doit etre range comme `proposal.md`, `design.md`, `tasks.md` et `specs/<domaine>/spec.md`.
- Une spec n'est archivee que lorsqu'elle sort du flux actif et qu'une implementation de reference existe.

## Notes de migration

- Le corpus numerote `01..24` a ete converti en documentation de reference sous `OpenSpec/specs/reference/`.
- Le premier point d'entree canonique est maintenant [specs/system/spec.md](./specs/system/spec.md).
- Les specs archivees restent sous `OpenSpec/archive/completed/<annee-mois>/`.
- Les changements completes sont archives sous forme de dossiers OpenSpec complets, pas comme gros fichiers uniques.
- Il n'y a actuellement aucun change actif dans `OpenSpec/changes/`.

## Agent Workflows

- `OpenSpec/changes/` reste la source de verite pour les changements actifs.
- `OpenSpec/archive/completed/<YYYY-MM>/` contient les changements archives une fois implementes et verifies.
- `.codex/skills/` contient les skills Codex adaptes a ce depot pour proposer, explorer, appliquer et archiver un change OpenSpec.
- `.gemini/commands/opsx/` et `.gemini/skills/` exposent le meme flux cote Gemini.
- Les prompts et skills doivent toujours referencer `OpenSpec/...` dans ce repo, pas `openspec/...`.

## Specs archivees

- [feat-ui-menus-shortcuts](./archive/completed/2026-04/feat-ui-menus-shortcuts/proposal.md)
- [init-mvp-futureaero](./archive/completed/2026-04/init-mvp-futureaero/proposal.md)
- [chore-github-pr-et-releases](./archive/completed/2026-04/chore-github-pr-et-releases/proposal.md)
