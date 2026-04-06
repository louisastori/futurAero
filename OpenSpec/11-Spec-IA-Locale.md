# Spec IA Locale

## Objectif

Definir un assistant IA local exploitable dans un contexte industriel sans perte de tracabilite et sans modification silencieuse du projet.

## Roles de l'IA dans le MVP

- expliquer,
- resumer,
- proposer,
- documenter.

## Increment shell desktop deja livre

Un premier increment executable du runtime IA locale est present dans le shell desktop:

- panneau `Assistant IA local` dans l interface,
- routage via commandes Tauri dediees,
- contexte injecte depuis le projet charge dans le shell,
- runtime Ollama local privilegie,
- fallback local explicite si le runtime modele est indisponible.

L'IA ne remplace pas:

- le solveur geometrique,
- le moteur de simulation,
- le systeme de validation des commandes.

## Modes d'usage

### Mode `explain`

Exemples:

- pourquoi une collision apparait,
- pourquoi un assemblage reste libre,
- pourquoi une sequence est bloquee.

### Mode `summarize`

Exemples:

- resume d'une cellule,
- resume d'un scenario,
- resume d'un rapport de validation.

### Mode `propose`

Exemples:

- proposer une correction de position,
- proposer une modification de sequence,
- proposer des parametres plus coherents.

### Mode `document`

Exemples:

- generer une note technique,
- generer une checklist de verification,
- generer un compte-rendu de simulation.

## Composants du runtime IA

- `ModelHost`: abstraction sur le modele de generation local.
- `Embedder`: vectorisation locale des objets et textes.
- `Retriever`: recuperation hybride par graph + texte + embeddings.
- `ContextBuilder`: assemble le contexte utile et cite les sources.
- `SuggestionPlanner`: convertit une intention en proposition structuree.
- `SafetyGate`: bloque les sorties non parsees ou non autorisees.
- `AiJournal`: persiste prompts, contexte, reponses et decisions.

## Modeles logiques

Types de modeles supportes:

- modele chat local,
- modele embedding local,
- reranker optionnel,
- modele vision/perception local,
- modele critic local.

Le systeme ne depend pas d'un fournisseur unique. Le contrat de sortie prime sur le nom du modele.

## Profils runtime

Profils minimaux:

- `eco`
- `standard`
- `max`
- `furnace`

`furnace` est le profil de qualite maximale locale. Il privilegie le raisonnement, la critique interne et l'usage agressif des ressources.

## Sources de contexte autorisees

- noeuds du graphe projet,
- relations du graphe,
- journaux commande/evenement,
- resultats de simulation,
- sorties perception, nuages de points et cartes d'occupation derives,
- rapports de validation,
- selection active utilisateur.

Sources interdites par defaut:

- internet,
- services cloud,
- fichiers hors projet non references,
- prompts caches non journalises.

## Pipeline de raisonnement

1. Recevoir l'intention utilisateur.
2. Identifier le mode d'usage.
3. Recuperer les objets cibles et voisins du graphe.
4. Recuperer extraits de journaux et mesures utiles.
5. Construire un contexte compact et cite.
6. Selectionner le profil runtime et le routage modele.
7. Demander une sortie strictement structuree.
8. Valider la sortie.
9. Persister la suggestion ou la reponse.

## Format de sortie attendu

```json
{
  "mode": "explain|summarize|propose|document",
  "summary": "string",
  "contextRefs": [
    { "entityId": "ent_01J...", "role": "source", "path": "metrics.collision_count" }
  ],
  "confidence": 0.82,
  "riskLevel": "low|medium|high",
  "limitations": [
    "Le modele n'a utilise que les runs de simulation disponibles."
  ],
  "proposedCommands": [],
  "explanation": [
    "La collision apparait quand la pince entre dans la zone du convoyeur a t=12.48s."
  ]
}
```

## Regles de securite

- Aucune sortie libre non parsee n'est executable.
- Toute proposition de mutation doit passer par `proposedCommands`.
- Toute suggestion cite des `contextRefs`.
- Toute reponse indique ses limites si le contexte est incomplet.
- Toute execution de suggestion reste une action explicite de l'utilisateur.

## Regles de qualite

- `confidence` ne peut jamais etre omise.
- `riskLevel` est obligatoire si `proposedCommands` n'est pas vide.
- Une suggestion a risque `high` ne peut pas etre appliquee en lot sans revue.

## Regles de contexte

- Le contexte doit rester minimal et cible.
- Les extraits de timeline doivent etre fenetres autour des evenements critiques.
- Les gros assemblages sont resumes par structure avant d'injecter du detail.
- Les donnees perception denses sont resumees par statistiques, zones et ecarts avant injection brute.
- En mode `furnace`, le systeme peut construire un contexte brut beaucoup plus large puis le compresser avant generation finale.

## Cas de sortie refuses

- reponse non JSON parseable,
- reference a un objet inexistant,
- commande non supportee,
- absence de justification pour une mutation,
- confiance trop basse sous seuil produit configurable.

## Journalisation IA

Chaque session IA enregistre:

- `sessionId`
- `userIntent`
- `mode`
- `modelInfo`
- `contextRefs`
- `promptHash`
- `responseHash`
- `createdSuggestionIds`
- `acceptedSuggestionIds`

## Application d'une suggestion

1. l'utilisateur demande une suggestion,
2. le runtime cree une `AiSuggestion`,
3. l'UI affiche resume, explication, refs et commandes,
4. l'utilisateur applique ou rejette,
5. si appliquee, les commandes sont rejouees par le coeur normal.

## Exemples de demandes MVP

- "Explique pourquoi le robot entre en collision avec le convoyeur."
- "Propose une modification de sequence pour reduire le temps de cycle."
- "Explique pourquoi le LiDAR detecte un obstacle qui n'existe pas dans le modele."
- "Compare la carte observee a la cellule nominale et liste les derives."
- "Genere une note technique de la cellule courante."

## Criteres d'acceptation

- Une suggestion peut etre inspectee sans toucher au projet.
- Une suggestion appliquee laisse une trace commande/evenement classique.
- Une reponse peut citer les mesures et objets qui ont servi au raisonnement.
- Le profil `furnace` peut activer plusieurs passes de raisonnement local sans casser les garde-fous du projet.
