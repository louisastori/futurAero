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
3. [changes/init-mvp-futureaero/proposal.md](./changes/init-mvp-futureaero/proposal.md)
4. [changes/init-mvp-futureaero/design.md](./changes/init-mvp-futureaero/design.md)
5. [changes/init-mvp-futureaero/tasks.md](./changes/init-mvp-futureaero/tasks.md)
6. [archive/README.md](./archive/README.md)

## Regles de rangement

- Les anciens fichiers `01..24` ne vivent plus a la racine.
- Les specs stables longues sont migrees dans [specs/reference/](./specs/reference/README.md).
- Les specs canoniques courtes promues depuis `changes/` vivent dans `specs/<domaine>/spec.md`.
- Une spec n'est archivee que lorsqu'elle sort du flux actif et qu'une implementation de reference existe.

## Notes de migration

- Le corpus numerote `01..24` a ete converti en documentation de reference sous `OpenSpec/specs/reference/`.
- Le premier point d'entree canonique est maintenant [specs/system/spec.md](./specs/system/spec.md).
- Les specs archivees restent sous `OpenSpec/archive/completed/<annee-mois>/`.

## Specs archivees

- [25-Spec-UI-Workspace-Et-Menus.md](./archive/completed/2026-04/25-Spec-UI-Workspace-Et-Menus.md)
- [26-Spec-GitHub-PR-Et-Releases.md](./archive/completed/2026-04/26-Spec-GitHub-PR-Et-Releases.md)
