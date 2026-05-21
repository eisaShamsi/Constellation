---
translation_status: AI-generated 2026-05-21 — native-speaker review recommended
language: fr
source: docs/help.uConstellation.World/Note Summaries/Note Summaries.md
aliases:
  - Note Summaries
  - Note Summary
  - Summary
  - NSC
  - Note Summary Creator
  - Build all summaries
  - Résumés de notes
  - Résumé de note
  - Résumé
description: Les Résumés de notes vous donnent un précis court et en langage clair d'une note pour que vous puissiez la juger sans l'ouvrir. Constellation honore toujours un résumé que vous avez écrit vous-même — dans le frontmatter ou dans un callout de résumé — et n'en génère un que lorsque vous ne l'avez pas fait. Les résumés générés sont extractifs (les phrases les plus centrales de la note elle-même), en lecture seule (jamais réécrits dans votre fichier) et entièrement calculés sur votre appareil.
---

# Résumés de notes

> *Si vous avez écrit un résumé, Constellation utilise le vôtre. Il n'en écrit un que lorsque vous ne l'avez pas fait — et même alors, jamais dans votre fichier.*

Un **Résumé de note** est un précis court d'une note — quelques phrases qui vous disent de quoi parle la note d'un coup d'œil. Les résumés sont produits par le **Note Summary Creator (NSC)**. Aujourd'hui ils apparaissent sous le titre de chaque note dans la file d'attente du **Classificateur** / **Source Review**, pour que vous puissiez décider comment classer une carte sans ouvrir la note qui se trouve derrière.

Cette rubrique explique d'où viennent les résumés, l'ordre de priorité strict qui préfère toujours *vos* mots à ceux de la machine, comment les résumés générés sont construits, et comment les pré-calculer pour toute une Library d'un coup.

---

## Pourquoi les résumés existent

Lorsque vous parcourez une file d'attente de révision de centaines de cartes, le titre seul ne suffit souvent pas à se rappeler ce que dit réellement une note. Ouvrir chaque note pour rafraîchir sa mémoire casse le flux. Un résumé d'un coup d'œil sous le titre corrige cela : vous lisez trois phrases, vous vous rappelez la note, vous prenez la décision, vous passez à la suite.

Mais un résumé est aussi un petit acte d'écriture. Si vous avez déjà distillé une note dans vos propres mots — dans un champ `summary:` ou un callout `> [!summary]` — alors c'est *celui-là* qui devrait s'afficher, pas une supposition de la machine. La première règle de Constellation pour les résumés est donc une règle de respect pour votre écriture : **le vôtre l'emporte.**

---

## D'où vient un résumé — l'ordre de priorité

Pour toute note, Constellation choisit le résumé en parcourant cette liste et en s'arrêtant au premier qui existe :

1. **Votre résumé de frontmatter.** Si les propriétés de la note contiennent un champ `summary:`, `description:`, `abstract:` ou `excerpt:` (vérifiés dans cet ordre), son texte est utilisé **exactement tel que vous l'avez écrit**.
2. **Votre callout de résumé.** Si le corps de la note contient un callout `> [!summary]`, `> [!abstract]` ou `> [!tldr]`, son texte est utilisé **exactement tel que vous l'avez écrit** — y compris les diacritiques et la ponctuation, préservés à l'identique.
3. **Un résumé généré.** Uniquement si vous n'avez écrit ni l'un ni l'autre des éléments ci-dessus, Constellation en génère un — en lisant la note et en extrayant ses phrases les plus centrales (voir ci-dessous).
4. **Un repli sur le texte d'ouverture.** Pour une note que le moteur ne peut pas découper en phrases (par exemple un texte dans une écriture sans ponctuation de phrase claire), il affiche les premières lignes de la note plutôt qu'un résumé classé.

> **La seule règle qui compte le plus :** les étapes 1 et 2 signifient qu'un résumé que vous avez écrit n'est *jamais* écrasé. Si vous voyez un résumé généré sur une note que vous pensiez avoir résumée, cela signifie que le moteur n'a pas trouvé votre résumé là où il regarde — vérifiez que votre champ de frontmatter est l'un des quatre noms ci-dessus, ou que votre callout est l'un des trois types ci-dessus.

---

## Comment un résumé généré est construit

Lorsque Constellation doit générer un résumé (parce que vous n'en avez pas écrit), il fait de la synthèse **extractive** — il sélectionne des phrases qui sont déjà dans votre note, plutôt que d'inventer une nouvelle prose. La méthode est bien établie (TextRank, Mihalcea & Tarau 2004) :

1. **Découpage en phrases.** Le corps de la note est segmenté en phrases à l'aide de la norme Unicode pour les limites de phrases, de sorte que cela fonctionne à travers les langues et les écritures.
2. **Lecture du sens de chaque phrase.** Chaque phrase est transformée en une petite « empreinte de sens » numérique (un embedding) à l'aide d'un modèle compact embarqué sur l'appareil.
3. **Classement par centralité.** Les phrases les plus similaires en sens au plus grand nombre d'*autres* phrases obtiennent le score le plus élevé — ce sont les phrases qui représentent le mieux la note dans son ensemble.
4. **Prendre les trois premières, dans l'ordre.** Les trois phrases les mieux classées sont affichées **dans l'ordre où elles apparaissent dans la note**, pour que le résumé se lise naturellement plutôt que dans le désordre.

Les notes très longues sont gérées en douceur — le moteur plafonne la quantité de corps qu'il scanne et le nombre de phrases qu'il classe, de sorte que résumer une note énorme ne ralentit jamais l'app ni ne risque un plantage.

Parce qu'il est extractif, un résumé généré est toujours fait de phrases que vous avez réellement écrites. Il ne vous fera jamais dire ce que vous n'avez pas dit.

---

## Les résumés sont en lecture seule — File-Over-App

Constellation **n'écrit jamais un résumé généré dans votre note.** Vos fichiers `.md` sont la source de vérité ; le résumé que vous voyez sur une carte est calculé à la volée et mis en cache séparément, non sauvegardé dans le texte ou le frontmatter du fichier.

C'est délibéré, et cela suit le principe *File-Over-App* de Constellation : l'app est une fenêtre sur vos fichiers, non un éditeur qui les change discrètement. Si vous voulez qu'un résumé vive *dans* la note, écrivez-en un vous-même (un champ `summary:` ou un callout `[!summary]`) — et alors, par la règle de priorité ci-dessus, Constellation affichera le vôtre et cessera de générer.

Tout est calculé **sur votre appareil.** Aucun texte de note n'est jamais envoyé où que ce soit pour être résumé.

---

## Quand les résumés apparaissent, et comment ils se remplissent

Les résumés s'affichent sous le titre de la note sur chaque carte dans la file d'attente du **Classificateur** / **Source Review**.

Par défaut ils se remplissent **paresseusement et en douceur** : à mesure que les cartes défilent dans la vue, Constellation calcule leurs résumés quelques-uns à la fois, en faisant une pause chaque fois qu'un scan de classification de Library est en cours pour que les deux ne se disputent jamais les ressources. Cela garde l'app réactive — vous pouvez brièvement voir une carte avant que son résumé n'apparaisse, puis le résumé surgit un instant plus tard.

Si vous préférez avoir chaque résumé prêt à l'avance, utilisez **Générer tous les résumés**.

---

## Générer tous les résumés — pré-calculer toute la Library

Le bouton **Générer tous les résumés** (dans l'en-tête du **Classificateur**) pré-calcule un résumé pour **chaque note qui n'en a pas déjà un à jour**, de sorte que les cartes affichent leur résumé instantanément au lieu de se remplir au fil de votre défilement.

**Pour l'utiliser :**

1. Ouvrez le **Classificateur** (l'icône en cartes empilées dans le dock à gauche).
2. Cliquez sur **Générer tous les résumés** dans l'en-tête. Le bouton se change en *Création des résumés de notes…*.
3. La progression apparaît dans la **barre d'état** en bas de la fenêtre — vous pouvez continuer à travailler pendant son exécution.
4. Pour arrêter plus tôt, utilisez le contrôle **Annuler** sur la barre de progression de la barre d'état. Une exécution partielle n'est pas un problème ; elle reprend là où elle s'est arrêtée la fois suivante.

Quelques points à connaître :

- Elle ne s'exécute **que lorsque vous le demandez** — elle ne démarre jamais d'elle-même, elle ne peut donc jamais ralentir le démarrage de l'app.
- Elle s'exécute **en arrière-plan** sur un thread séparé ; la frappe et la navigation restent instantanées.
- Elle est **reprenable** — si vous l'annulez, ou fermez l'app en cours d'exécution, l'exécution suivante continue là où elle s'est arrêtée plutôt que de tout recommencer.
- Elle ne calcule que les résumés **manquants ou périmés** — les notes dont le résumé est déjà à jour sont ignorées, de sorte qu'une seconde exécution est rapide.

---

## S'assurer que votre propre résumé est utilisé

Sur une carte, le résumé apparaît sous une seule étiquette **Résumé** — la carte n'indique pas par une pastille si le texte vient de vous ou du moteur. Ce qui décide cela, c'est la priorité ci-dessus : si une note a l'un des champs de frontmatter ou l'un des callouts de résumé, Constellation affiche *celui-là* et n'en génère jamais.

Donc si une note affiche un résumé qui se lit comme si la machine l'avait choisi, c'est que cette note n'a ni résumé de frontmatter ni callout de résumé — et la solution est d'en ajouter un :

- Ajoutez un champ `summary:` (ou `description:` / `abstract:` / `excerpt:`) au frontmatter de la note, **ou**
- Ajoutez un callout `> [!summary]` (ou `[!abstract]` / `[!tldr]`) au corps.

La prochaine fois que le résumé de cette note sera calculé — au prochain chargement de sa carte, ou après que vous aurez lancé **Générer tous les résumés** — vos mots prennent le relais.

---

## Flux de travail courants

**« Une note affiche un résumé de la machine, alors que j'en ai écrit un. »**
Constellation n'a pas trouvé votre résumé là où il regarde. Assurez-vous que votre champ de frontmatter est nommé `summary`, `description`, `abstract` ou `excerpt`, **ou** que votre callout est `[!summary]`, `[!abstract]` ou `[!tldr]`. Puis rouvrez le Classificateur (ou cliquez sur *Générer tous les résumés*) pour rafraîchir.

**« Je veux que chaque carte affiche son résumé à l'instant où j'ouvre le Classificateur. »**
Cliquez sur **Générer tous les résumés** une fois et laissez-le finir. Après cela, les résumés sont pré-calculés et apparaissent immédiatement.

**« Je veux que le résumé fasse partie de la note elle-même, sur le disque. »**
Écrivez-le vous-même — ajoutez un champ de frontmatter `summary:` ou un callout `> [!summary]`. Constellation affichera alors votre version (et cessera d'en générer une), et vos mots vivent dans le fichier où n'importe quelle autre app peut les lire aussi.

---

## Rubriques connexes

- **The Cataloger** (le Classificateur) — l'accueil pleine page où les résumés apparaissent sous chaque carte, et où vit *Générer tous les résumés*.
- **Source Review** (Révision des Sources) — les cartes de classification sur lesquelles reposent les résumés.
- **Properties** — les champs de frontmatter `summary:` / `description:` / `abstract:` / `excerpt:`, et comment les ajouter.
- **Editing and Formatting** — comment écrire un callout `> [!summary]` dans une note.
