---
translation_status: AI-generated 2026-05-21 — native-speaker review recommended
language: fr
source: docs/help.uConstellation.World/The Cataloger/The Cataloger.md
aliases:
  - The Cataloger
  - Cataloger
  - Classify notes
  - Classification home
  - CECE home
  - Scan library
  - Classificateur
  - Classer les notes
  - Accueil de la classification
description: Le Classificateur est l'accueil à l'échelle de l'univers pour classer vos notes. C'est la vue pleine page du dock où vous exécutez le Constellation Epistemic Content Engine (CECE) sur votre Library, classez n'importe quelle note à la demande, générez les résumés de notes et parcourez la file d'attente de révision. Si Source Review est la carte sur laquelle vous agissez, le Classificateur est la pièce où vous le faites.
---

# Classificateur

> *« Classez chaque note selon son type de connaissance et sa source. »*

Le **Classificateur** est l'accueil à l'échelle de l'univers pour la classification. C'est une vue pleine page, ouverte depuis le dock à gauche, qui rassemble en un seul endroit tout ce dont vous avez besoin pour lire vos notes à travers la taxonomie de connaissances de Constellation : un contrôle pour scanner toute la Library, un moyen de classer n'importe quelle note à la demande, un bouton pour générer les résumés de notes, et la file d'attente de révision en direct où vous Acceptez, Modifiez, Rejetez ou désambiguïsez chaque suggestion.

Si vous avez utilisé le panneau **Source Review** dans le sidebar à droite, vous connaissez déjà les cartes. Le Classificateur est le même moteur et les mêmes cartes, promus hors d'un onglet étroit du sidebar et dotés de toute la fenêtre — plus deux choses que l'onglet du sidebar n'avait jamais : un sélecteur de notes et un bouton « Générer tous les résumés ».

---

## « le Classificateur » vs « les catalogueurs » — un mot rapide sur les noms

Ces deux noms se ressemblent à dessein, mais ils désignent des choses différentes :

- **le Classificateur** (la *pièce*, cette vue) est l'*endroit* — la salle pleine page où la classification a lieu.
- **les catalogueurs** (minuscule, pluriel) sont les *six lentilles* à l'intérieur du moteur — frontmatter, citations, racines de mots, notes liées, notes similaires et jugement IA — chacune lisant une note et votant. Cinq des six sont actives aujourd'hui ; la sixième (jugement IA) est construite mais pas encore activée.

Donc : vous ouvrez **le Classificateur**, et à l'intérieur **les catalogueurs** font la lecture. La mécanique des six lentilles est expliquée en détail dans la rubrique **Source Review** (Révision des Sources) — cette rubrique-ci parle de la pièce.

---

## Ce que c'est

Le Classificateur répond à une question : **« Comment chaque note de mon univers est-elle classée — et qu'est-ce qui nécessite encore ma décision ? »**

Il est construit autour de quatre choses empilées de haut en bas :

1. **Un en-tête avec trois actions** — *Classer une note…*, *Générer tous les résumés* et *Démarrer l'analyse*.
2. **Une barre de progression** — apparaît uniquement pendant qu'un scan de Library est en cours, montrant son avancement.
3. **La file d'attente de révision** — les mêmes cartes Accepter / Modifier / Rejeter / Désambiguïser que le panneau Source Review, désormais pleine largeur.
4. **Un résumé de note sous chaque carte** — un précis court et en langage clair de la note pour que vous puissiez décider sans l'ouvrir (voir *Résumés de notes* ci-dessous, et la rubrique dédiée **Note Summaries**).

Tout s'exécute **sur votre appareil**. Aucune note ne quitte jamais Constellation.

---

## Pourquoi c'est important

La classification est la manière dont Constellation transforme un tas de fichiers `.md` en un corpus de connaissances *façonné* — chaque note placée sur deux axes (d'où vient la connaissance, et quel type de connaissance c'est). Cette forme est ce qui alimente **Constellation Sight**, le panneau **Métadonnées Épistémiques** et la recherche consciente de la taxonomie.

Mais la classification est un travail à forte densité de décisions. Lorsque vous avez des centaines de notes non classées, le faire depuis un onglet étroit du sidebar — une note à la fois, sans moyen d'invoquer une note précise — est lent. Le Classificateur existe pour rendre le travail *posable* : ouvrez-le une fois, donnez-lui tout l'écran, et parcourez votre Library en une seule session concentrée. Le sélecteur de notes vous permet d'attirer n'importe quelle note par son nom ; les résumés vous permettent de juger une carte sans quitter la pièce ; le contrôle de scan ensemence la file d'attente en masse.

---

## Comment l'ouvrir

1. Dans le **dock à gauche** (la bande verticale d'icônes au bord extrême de la fenêtre), cliquez sur l'**icône en cartes empilées** — trois petites cartes superposées les unes sur les autres. Elle se trouve parmi les autres icônes d'espace de travail comme l'œil de Sight et le neurone du Nervous System.
2. Le Classificateur s'ouvre en **vue pleine page**, prenant le contrôle de la zone de contenu.
3. Pour le fermer : cliquez sur le **(×)** en haut à droite de l'en-tête, ou appuyez sur **Esc**. Vous revenez là où vous étiez.

> **Note sur Esc :** si la fenêtre contextuelle de recherche *Classer une note…* est ouverte, appuyer sur **Esc** ferme uniquement cette fenêtre et laisse le Classificateur ouvert. Appuyez à nouveau sur **Esc** (avec la fenêtre fermée) pour fermer le Classificateur lui-même.

---

## Ce que vous voyez

### L'en-tête — trois actions

En haut du Classificateur, trois contrôles sont côte à côte :

| Contrôle | Ce qu'il fait |
|---|---|
| **Classer une note…** | Ouvre une petite boîte de recherche. Tapez quelques lettres du titre de n'importe quelle note, choisissez-la dans les résultats, et le moteur la classe sur-le-champ — sans avoir besoin d'ouvrir la note d'abord. La nouvelle suggestion apparaît dans la file d'attente ci-dessous. |
| **Générer tous les résumés** | Pré-calcule un résumé court pour chaque note qui n'en a pas déjà un. S'exécute discrètement en arrière-plan ; la progression s'affiche dans la barre d'état en bas de la fenêtre ; vous pouvez annuler à tout moment. (Détaillé dans la rubrique **Note Summaries**.) |
| **Démarrer l'analyse** | Exécute le moteur sur l'ensemble de votre **Library active** d'un coup, mettant en file d'attente une suggestion pour chaque note qui n'est pas encore classée. Pendant son exécution, le bouton affiche *En cours…* et une barre de progression apparaît sous l'en-tête. |

### La barre de progression

Directement sous l'en-tête, une fine barre apparaît **uniquement pendant qu'un scan de Library est en cours**. Elle indique combien de notes ont été traitées et vous laisse voir le scan se terminer. Lorsqu'aucun scan n'est en cours, la barre est masquée et la file d'attente se trouve juste sous l'en-tête.

### La file d'attente de révision

L'essentiel du Classificateur est la **file d'attente de révision** — les mêmes cartes que vous voyez dans le panneau Source Review, simplement pleine largeur. Chaque carte affiche une note, la lecture du moteur sur la façon dont elle s'inscrit dans votre taxonomie (Source × Content Type), les six petites pastilles de catalogueur, et les actions que vous pouvez entreprendre :

- **Accept** — écrit la suggestion du moteur dans la note et efface la carte.
- **Edit** — vous choisissez vous-même les valeurs depuis une arborescence.
- **Reject** — efface la carte sans rien écrire.
- **Disambiguate** — sur une carte « split », vous choisissez la bonne valeur parmi les puces candidates.

La mécanique complète des cartes — les pastilles colorées, les régimes de confiance, la Sibling Disambiguation, les puces de filtre de la file d'attente, « Approve all » et la calibration par Library — est documentée dans la rubrique **Source Review**. Le Classificateur utilise exactement ce panneau ; rien dans les cartes ne change entre le sidebar et la vue pleine page.

### Le résumé de note sous chaque carte

Sous le titre de chaque carte se trouve une courte ligne **Résumé** — quelques phrases qui vous disent de quoi parle la note, pour que vous puissiez juger la carte sans ouvrir la note. Cela est produit par le **Note Summary Creator (NSC)** ; voir la section suivante et la rubrique **Note Summaries**.

---

## Classer une seule note — le sélecteur de notes

Le bouton *Classer une note…* résout un problème simple : dans l'onglet du sidebar, vous ne pouviez classer que la note actuellement ouverte. Le Classificateur n'a pas de « note ouverte », il vous donne donc un moyen d'invoquer n'importe quelle note par son nom.

**Pour classer une seule note :**

1. Cliquez sur **Classer une note…**. Une boîte de recherche se déroule avec l'espace réservé *Rechercher des notes…*.
2. Commencez à taper le titre de la note. Après une brève pause, les notes correspondantes apparaissent dans une liste (jusqu'à dix).
3. Cliquez sur la note souhaitée. Le moteur la classe, la fenêtre se ferme, et une carte fraîche pour cette note apparaît dans la file d'attente ci-dessous.
4. Si quelque chose tourne mal (une rare erreur du moteur), le message apparaît à l'intérieur de la fenêtre afin que vous sachiez que la classification ne s'est pas exécutée.

Vous n'avez pas à ouvrir la note, et vous ne perdez pas votre place dans la file d'attente. C'est la manière la plus rapide de classer une note précise que vous avez en tête.

---

## Résumés de notes (NSC) à l'intérieur du Classificateur

Chaque carte de la file d'attente porte un court **Résumé** de sa note, affiché sous le titre. Le résumé est produit par le **Note Summary Creator (NSC)** et suit une règle ferme : **si vous avez écrit un résumé, le moteur utilise le vôtre ; il n'en génère un que lorsque vous ne l'avez pas fait.**

L'ordre de priorité est :

1. **Votre résumé de frontmatter** — un champ `summary:`, `description:`, `abstract:` ou `excerpt:` dans les propriétés de la note. Utilisé exactement tel que vous l'avez écrit.
2. **Votre callout de résumé** — un bloc `> [!summary]`, `> [!abstract]` ou `> [!tldr]` dans le corps de la note. Utilisé exactement tel que vous l'avez écrit, diacritiques compris.
3. **Un résumé généré** — uniquement si vous n'avez écrit ni l'un ni l'autre des éléments ci-dessus. Constellation lit la note, trouve ses phrases les plus centrales, et affiche les trois premières dans leur ordre d'origine.

Le moteur **n'écrit jamais un résumé généré dans votre note** — vos fichiers `.md` sont la source de vérité et le Classificateur ne fait jamais que les *lire*.

Le bouton **Générer tous les résumés** pré-calcule les résumés de toute la Library en arrière-plan, de sorte que les cartes affichent leur résumé instantanément au lieu de se remplir au fil de votre défilement. Le détail complet — y compris comment les résumés générés sont produits et que faire si un résumé semble erroné — se trouve dans la rubrique **Note Summaries**.

---

## Ce que le Classificateur ne fait *pas*

- **Il ne classe pas automatiquement en arrière-plan par défaut.** Les scans sont quelque chose que vous *démarrez*. (Il existe un mode arrière-plan optionnel dans Paramètres → Intelligence → CECE, désactivé par défaut — voir **Source Review**.)
- **Il n'appelle aucun service cloud.** Les cinq catalogueurs actifs sont heuristiques et locaux. La sixième lentille (jugement IA, un modèle de langage local) est intégrée à la conception mais pas encore activée, elle reste donc silencieuse sur chaque carte aujourd'hui.
- **Il ne change pas la formulation de vos notes.** Accepter une carte écrit des *propriétés* de classification (les champs frontmatter `sources:` et `content_type:`). Il ne modifie jamais votre prose, et il n'écrit jamais un résumé généré dans le fichier.

---

## Flux de travail courants

**« Je viens d'ouvrir le Classificateur pour la première fois — par où commencer ? »**
Cliquez sur **Démarrer l'analyse** pour mettre en file d'attente une suggestion pour chaque note non classée de la Library. Regardez la barre de progression se remplir. Puis parcourez la file d'attente, en acceptant celles que le moteur a bien classées et en désambiguïsant les cartes split. Les résumés sous chaque carte vous permettent de décider rapidement.

**« Je veux classer une seule note précise, pas toute la Library. »**
Cliquez sur **Classer une note…**, tapez son titre, cliquez dessus. Une carte apparaît dans la file d'attente. Acceptez-la ou modifiez-la.

**« Mes cartes mettent un instant à afficher leurs résumés. »**
Cliquez sur **Générer tous les résumés** une fois. Cela pré-calcule le résumé de chaque note en arrière-plan (progression dans la barre d'état). Une fois terminé, les résumés apparaissent instantanément.

**« La file d'attente compte des centaines de cartes — comment me concentrer ? »**
Utilisez les puces de filtre au-dessus de la file d'attente (documentées dans **Source Review**) : commencez par *Catalogers agreed* et *Approve all* pour effacer les faciles, puis attaquez les cartes split.

---

## Rubriques connexes

- **Source Review** (Révision des Sources) — les cartes elles-mêmes : les six catalogueurs, les pastilles colorées, les régimes de confiance, la Sibling Disambiguation, les filtres de file d'attente, « Approve all » et la calibration par Library. Le Classificateur intègre ce panneau.
- **Note Summaries** — comment la ligne Résumé sous chaque carte est produite, la priorité donnée d'abord à l'auteur, et le pré-calcul *Générer tous les résumés*.
- **Cognitive Engine** — la philosophie plus large de formulation des connaissances dans laquelle s'inscrit la classification.
- **Métadonnées Épistémiques** — les propriétés `sources:` et `content_type:` que la classification écrit, et comment les lire.
- **Constellation Sight** — la vue spatiale que la classification Source × Content Type alimente.
