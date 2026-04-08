# Archive OpenSpec

Ce dossier contient les specs sorties du flux actif apres implementation et verification.

Regle:

- une spec n'est archivee qu'apres existence d'une implementation de reference,
- les tests ou checks associes doivent etre verts avant commit du jalon,
- l'archive conserve le change sous forme de dossier avec `proposal.md`, `design.md`, `tasks.md` et `specs/<domaine>/spec.md`.

Note:

- un increment livre peut mettre a jour le backlog et la matrice de tests sans archiver toute la spec mere si le domaine reste partiellement en cours.
