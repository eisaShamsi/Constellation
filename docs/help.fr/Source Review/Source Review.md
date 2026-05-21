# Source Review

> **Note de traduction :** Cette rubrique d'aide est une traduction
> générée par IA à partir de la version canonique en anglais à
> `help.uConstellation.World/Source Review/Source Review.md`. La
> relecture par un locuteur natif est en attente. Veuillez signaler
> les corrections via le dépôt du projet.

*(Constellation Epistemic Content Engine — CECE)*

Le panneau Source Review est l'endroit où Constellation vous demande de réviser les classifications produites par le **Constellation Epistemic Content Engine** (CECE). Chaque carte de la file d'attente affiche une note + la lecture du moteur sur la façon dont cette note s'inscrit dans votre taxonomie de connaissances. Vous Acceptez, Modifiez, Rejetez ou choisissez une puce Sibling Disambiguation — et au fil du temps, le moteur apprend la forme de votre Library.

Cette rubrique explique chaque partie d'une carte Source Review, ce que signifient les pastilles colorées, quand faire confiance au moteur et comment naviguer dans des centaines de cartes sans défiler indéfiniment.

> **Deux endroits, un seul panneau.** Les cartes décrites ici apparaissent à la fois dans l'onglet **Source Review** du sidebar à droite et dans la vue pleine page du **Classificateur** (l'icône en cartes empilées dans le dock à gauche). C'est le même panneau et le même moteur. Le Classificateur donne à la file d'attente toute la fenêtre, plus un sélecteur de notes et un bouton « Générer tous les résumés » — voir la rubrique **The Cataloger** (le Classificateur). Tout ce qui suit s'applique aux cartes dans l'un ou l'autre endroit.

---

## Ce que CECE fait réellement

Lorsque vous classez une note (clic droit → « Suggérer sources & type de contenu », ou via Paramètres → bouton Lancer le scan), CECE exécute **six catalogueurs indépendants** sur la note. Chaque catalogueur lit la note à travers sa propre lentille — frontmatter, citations, racines de mots, notes liées, notes similaires, jugement IA — et vote sur deux questions :

- **Source (axe horizontal)** : d'où *vient* cette connaissance ? Exemples : témoignage (quelqu'un me l'a dit), perception (je l'ai vu), inférence (je l'ai déduit), révélation (texte sacré), et huit autres.
- **Content Type (axe vertical)** : quel *type* de connaissance est-ce ? Exemples : état épistémique (doute / certitude / croyance), contenu sémantique (concept / proposition / fait / théorie), entrée sensorielle, entité symbolique, construction d'ordre supérieur (vision du monde / doctrine).

Les deux axes sont **indépendants**. Une note sur « Je doute de l'alunissage » est témoignage (quelqu'un l'a rapporté) sur l'axe source + états-épistémiques/doute (la position de l'utilisateur à son égard) sur l'axe content-type.

Après le vote des catalogueurs, une **couche de synthèse** combine leurs votes en une seule classification par axe, avec l'un des trois régimes de confiance :

- **Unanimous** — chaque catalogueur s'étant exprimé était d'accord
- **Strong majority** — la plupart étaient d'accord, un dissident (la carte affiche le nom du dissident)
- **Split** — pas de majorité claire ; le moteur a « refusé d'attribuer » et vous demande à *vous* de choisir

Tout s'exécute **sur votre appareil**. Aucune note ne quitte jamais Constellation.

---

## Les deux axes en langage clair

### Source — *d'où vient cette connaissance ?*

Onze valeurs possibles plus *non classifiable* :

- **Perception** — observation sensorielle de première main
- **Inférence** — raisonnement à partir de prémisses (déduction, induction, analogie)
- **Témoignage** — rapport d'autrui (une citation, une référence, une source citée)
- **Transmission massive** — rapports convergents par de nombreux témoins indépendants (sunnite *al-tawatur*)
- **Comparaison** — connaissance par analogie à un cas connu (juridique *qiyās*, analogies scientifiques)
- **Postulation** — inférence à la meilleure explication (*arthapatti*)
- **Non-appréhension** — connaissance de l'absence
- **Mémoire** — rappel d'expérience passée
- **Disposition innée** — connaissance pré-expérientielle (*fitrah*)
- **Inspiration** — appréhension mystique ou créative (*kashf*)
- **Révélation** — transmission par texte sacré ou prophétique (*al-wahy*)
- **Non classifiable** — choisir de ne pas inclure cette classification

### Content Type — *quel type de connaissance est-ce ?*

Cinq branches de premier niveau avec sous-branches :

- **Entrées sensorielles** — signaux bruts (visuels, acoustiques, chimiques, …)
- **Entités symboliques** — signes, symboles, codes
- **Contenus sémantiques** — concepts, propositions, faits, idées, informations
- **États épistémiques** — doute, croyance, opinion, certitude, connaissance, illusion
- **Constructions d'ordre supérieur** — théories, doctrines, visions du monde, paradigmes

Les deux axes ont plusieurs couches de raffinement sous chaque valeur de premier niveau (par ex. *epistemic-states/knowledge/by-content/propositional* est une feuille).

---

## Les six catalogueurs

Chaque catalogueur est une *lentille* à travers laquelle CECE lit une note. La carte Source Review les affiche sous forme de **six petites pastilles colorées** dans le coin supérieur droit. Survolez n'importe quelle pastille pour voir son nom + statut.

| Pastille | Catalogueur | Ce qu'il lit |
|---|---|---|
| 🔵 bleu | **Votre frontmatter** (Autorité Utilisateur) | Les champs `sources:` et `content_type:` que vous avez déjà définis. Si vous avez classé la note vous-même, cette lentille a une *autorité absolue* — la synthèse adopte votre choix et ignore les autres. |
| 🌹 rose | **Citations & structure** (Structurel) | Citations, blockquotes, blocs de code, marqueurs de théorèmes, phrases de définition (« le concept de X est défini comme… »), références à des figures. Lit la forme structurelle de la note. |
| 🟡 ambre | **Racines & lexique** (Linguistique) | Analyse des racines arabes (CAE), correspondance de mots-clés en surface, équivalence de termes interlinguistique (Bridge). Détecte la classification consciente de l'arabe que les embeddings purs ratent. |
| 🟢 sarcelle | **Notes liées** (Graphe) | Living Links typés (`[[Note\|supports]]`, `[[Note\|contradicts]]`, etc.) vers d'autres notes classées. Hérite de la classification des voisins lorsqu'ils se regroupent. |
| 🟣 violet | **Notes similaires** (Sémantique) | Similarité d'embeddings avec vos notes déjà classées (k-Nearest-Neighbor). Apporte le consensus lorsque le vecteur de contenu de cette note se regroupe avec des notes classées. |
| 🟢 vert | **Jugement IA** (Raisonnement) | Un LLM local (Qwen3-4B Q5_K_M) exécutant une inférence contrainte par grammaire. *Pas encore actif* — câblage du modèle reporté à une version ultérieure. La pastille reste silencieuse sur chaque carte aujourd'hui. |

### Statut de la pastille

- **Pleine** — s'est exprimée + d'accord avec la synthèse
- **En anneau** — s'est exprimée + en désaccord avec la synthèse (cette lentille a choisi quelque chose de différent)
- **Contour pointillé** — silencieuse (aucun signal dans cette lentille pour cette note)

Le groupe de pastilles est l'indicateur de santé de l'ensemble en un coup d'œil. Une carte avec les six pastilles pleines est la classification la plus forte possible du moteur (rare). Une carte avec une ou deux pastilles en anneau montre son raisonnement honnêtement — les lentilles étaient en désaccord.

---

## Les trois régimes de confiance

Après le vote des catalogueurs, CECE étiquette chaque axe avec l'un des trois régimes :

- **Unanimous** — chaque catalogueur s'étant exprimé a choisi la même valeur principale. La carte n'a pas de pastille spéciale.
- **Strong majority (un dissident)** — la plupart étaient d'accord ; un dissident est affiché par son nom. La carte a une pastille violette « Strong majority » dans l'en-tête.
- **Split** — pas de majorité claire. La carte a une pastille dorée « Catalogers split — needs your call », **une bordure gauche dorée** et un formulaire Sibling Disambiguation avec des puces parmi lesquelles choisir.

Chaque axe obtient son propre régime de manière indépendante. Une carte peut être Unanimous en horizontal + Split en vertical (ou vice versa). La pastille d'en-tête résume le pire régime entre les deux axes.

---

## Sibling Disambiguation

Lorsqu'un axe est Split, CECE refuse de deviner et fait apparaître à la place les valeurs candidates sous forme de **puces radio** sous une invite :

> *« Les catalogueurs se sont divisés entre ces candidats. Choisissez celui qui correspond le mieux à la note : »*

Vous cliquez sur une puce → le moteur écrit ce choix dans le frontmatter de la note, supprime la carte de la file d'attente et met à jour les données de fiabilité par Library.

Si l'AUTRE axe était réglé (Unanimous ou Strong majority), CECE écrit *aussi* la valeur de cet axe en même temps — de sorte qu'un seul clic sur une puce termine les deux axes, pas seulement celui que vous avez choisi. La même carte ne vous demandera jamais deux fois.

Si les deux axes sont Split, vous choisissez une puce par axe (deux clics).

---

## La piste de raisonnement

Chaque carte a un bouton bascule **« ▸ Pourquoi cette classification ? »** (ou « ▾ Masquer le raisonnement » si ouvert). Le déployer affiche une ligne par catalogueur s'étant exprimé :

- **Pastille de couleur de la lentille** correspondant au groupe de pastilles
- **Étiquette du catalogueur** (par ex. « Racines & lexique »)
- **Confiance auto-déclarée** entre crochets : `[high]` `[medium]` `[low]`
- **Raisonnement d'une ligne** expliquant ce qui s'est déclenché (par ex. *« Linguistic match: vertical → semantic-contents/concept (weight 0.80) »*)
- **Puces de règles conviviales** sous le raisonnement, comme `Surface keyword match`, `Side-channel preference rule`, `Arabic root match (CAE)` — ce sont les règles spécifiques que chaque catalogueur a déclenchées

Pendant vos **50 premières révisions** la piste se déploie automatiquement sur chaque carte afin que vous puissiez développer une intuition sur le moment où faire confiance au moteur. Après cela, la piste se replie sur demande sur les cartes Unanimous et reste auto-déployée sur les cartes Strong majority + Split (où le désaccord est informatif).

Vous pouvez remplacer cette valeur par défaut à tout moment dans Paramètres → Intelligence → CECE → Visibilité de la piste de raisonnement :

- **Toujours afficher** — ouvert sur chaque carte
- **Uniquement en cas de désaccord (par défaut)** — ouvert sur les cartes Split + Strong majority, plus les 50 premières révisions
- **Toujours masquer** — clic manuel requis pour déployer

---

## Le filtre de composition de la file d'attente

Au-dessus de la barre de comptage se trouvent **cinq puces** qui découpent votre file d'attente selon le type de décision dont chaque carte a besoin de votre part :

| Puce | Affiche |
|---|---|
| **All** *(par défaut)* | la file d'attente complète |
| **Both axes need your call** | cartes où À LA FOIS horizontal ET vertical sont Split |
| **Source needs your call** | cartes où horizontal est Split + vertical est réglé |
| **Content type needs your call** | cartes où vertical est Split + horizontal est réglé |
| **Catalogers agreed** | cartes où aucun axe n'est Split — candidats rapides à approuver |

Chaque puce affiche le compte de son seau (par ex. *« Source needs your call (43) »*). Les seaux vides sont grisés et désactivés. Cliquer sur une puce re-rend les cartes visibles ; la barre de comptage et les calculs Approve All opèrent toujours sur la file d'attente **complète** indépendamment du filtre actif, de sorte que vous pouvez toujours voir les totaux réels.

Le filtre résout le problème de l'aiguille dans la botte de foin lorsque votre file d'attente compte des centaines de cartes. Vous voulez d'abord effacer tous les candidats à approuver ? Cliquez sur **Catalogers agreed** puis sur **Approve all**. Vous voulez vous concentrer sur les cas les plus difficiles ? Cliquez sur **Both axes need your call**.

---

## Le résumé de note sous chaque carte

Sous le titre de chaque carte se trouve une courte ligne **Résumé** — quelques phrases qui vous disent de quoi parle la note, pour que vous puissiez décider comment la classer sans l'ouvrir. Constellation affiche toujours un résumé que *vous* avez écrit (un champ de frontmatter `summary:` / `description:` / `abstract:` / `excerpt:`, ou un callout `> [!summary]` / `[!abstract]` / `[!tldr]` dans le corps) et n'en génère un que lorsque vous ne l'avez pas fait. Les résumés générés sont extractifs — les phrases les plus centrales de la note elle-même — et ne sont jamais réécrits dans votre fichier. Le détail complet se trouve dans la rubrique **Note Summaries**.

---

## Actions par carte

Chaque carte a quatre actions en bas (ou trois sur les cartes Split où Disambig remplace Accept/Edit) :

- **Accept** — écrit la valeur principale de la synthèse du moteur sur les deux axes dans le frontmatter de la note, supprime la carte de la file d'attente. Met à jour la fiabilité par catalogueur.
- **Edit** — ouvre un sélecteur arborescent pour les deux axes ; vous choisissez les valeurs manuellement. Même mise à jour de fiabilité.
- **Reject** — efface la carte sans rien écrire. Le moteur re-suggérera si vous reclassez plus tard. (Le rejet ne met PAS à jour la fiabilité — l'utilisateur « ne veut aucune de celles-ci » est ambigu en tant que signal de retour.)
- **Puce Sibling Disambiguation** — sur les cartes Split, cliquez sur l'une des puces candidates. Écrit la valeur choisie (et écrit automatiquement l'autre axe s'il était réglé).

---

## La période de calibration de la confiance

Vos **50 premières révisions** de cartes classées par CECE sont une *période de calibration de la confiance*. Pendant cette période, la piste de raisonnement se déploie automatiquement sur chaque carte (indépendamment du régime), et une bannière discrète en haut du panneau vous rappelle : *« Showing reasoning trails until you review N more cards — helps you learn when to trust the catalogers. »*

Après 50 révisions, la bannière disparaît et les pistes se replient au comportement par défaut sur demande. Vous pouvez le remplacer via Paramètres si vous souhaitez les garder toujours ouvertes ou toujours fermées.

L'objectif de la période de calibration : CECE est un système probabiliste qui s'améliore à mesure que vous le corrigez (fiabilité par Library). Voir *pourquoi* chaque catalogueur a voté comme il l'a fait pendant les 50 premières révisions vous permet de développer votre propre intuition sur le moment où ses conclusions sont fiables sur le contenu spécifique de cette Library.

---

## Calibration par Library

Paramètres → Intelligence → CECE → **Per-Library calibration** ouvre un tableau en lecture seule montrant la précision par axe de chaque catalogueur sur la Library active :

```
Cataloger          Horizontal      Vertical
─────────          ──────────      ────────
Your frontmatter   12/12 (100%)    4/4 (100%)
Citations          18/22 (82%)     6/8 (75%)
Wordstems          24/28 (86%)     20/26 (77%)
Linked notes       3/4 (uniform)   2/3 (uniform)
Similar notes      14/19 (74%)     12/19 (63%)
AI judgment        — (not running) — (not running)
```

Les chiffres sont des comptes corrects/total. Le pourcentage est affiché après qu'un catalogueur a 20+ corrections sur cette Library × axe (le seuil pour des données de précision stables). En dessous du seuil, l'étiquette affiche **(uniform)** — le catalogueur contribue avec des votes pondérés uniformément jusqu'à ce que suffisamment de données soient accumulées.

Différentes Libraries peuvent avoir des précisions par catalogueur très différentes. Le catalogueur Linguistique excelle sur les Libraries riches en arabe ; le catalogueur Graphe excelle sur les Libraries densément liées. La couche de synthèse utilise les données de calibration par Library pour pondérer les votes — de sorte qu'un catalogueur qui s'est trompé 70 % du temps dans *cette* Library voit ses votes sous-pondérés au prochain tour de synthèse.

---

## Classification en arrière-plan

La file d'attente Source Review peut croître de deux manières :

1. **Manuel** (par défaut) — vous faites un clic droit sur une note → « Suggérer sources & type de contenu », ou vous déclenchez Paramètres → Lancer le scan de classification.
2. **Arrière-plan** — Paramètres → Intelligence → CECE → Classification en arrière-plan. Deux modes :
   - **On note save** — auto-classifie chaque note ~1,5 seconde après que vous arrêtez de taper (s'appuie sur la sauvegarde debounced existante ; ne se déclenche jamais par frappe de touche).
   - **On app start** — scanne les notes non classées une fois par lancement.

La classification en arrière-plan est **désactivée par défaut**. Les deux modes en arrière-plan s'exécutent sur un thread d'arrière-plan + émettent des événements de progression ; la frappe reste instantanée ; vous pouvez annuler depuis l'en-tête du panneau Source Review.

---

## Flux de travail courants

**« Je viens d'installer CECE — par où commencer ? »**
Ouvrez le panneau Source Review. Faites un clic droit sur 5-10 notes de votre arborescence de fichiers → « Suggérer sources & type de contenu » pour ensemencer la file d'attente. Cliquez à travers les cartes une à la fois. La piste de raisonnement se déploie automatiquement pendant vos 50 premières révisions — lisez-la. Après 5-10 cartes, vous commencerez à voir quels catalogueurs sont fiables sur votre contenu.

**« Ma file d'attente compte 1 200 cartes — où me concentrer ? »**
Utilisez les puces de filtre. Commencez par **Catalogers agreed** (candidats à approuver) → cliquez sur Approve all pour les effacer. Puis **Source needs your call** + **Content type needs your call** pour les cas Split qui nécessitent une décision chacun. **Both axes need your call** est l'ensemble le plus difficile ; gardez-le pour la fin.

**« Comment savoir quand choisir Accept vs Reject vs Edit vs Disambig ? »**
- **Accept** lorsque la valeur principale de la synthèse correspond à votre lecture de la note.
- **Reject** lorsque aucune des suggestions ne convient (par ex. le moteur a manqué quelque chose que vous savez sur la note).
- **Edit** lorsque vous voulez une valeur qui n'est dans aucune des suggestions.
- **Puce Sibling Disambiguation** lorsque la carte est Split et que l'un des candidats est correct.

**« Comment voir à quels catalogueurs je fais le plus confiance ? »**
Ouvrez Paramètres → Intelligence → CECE → Per-Library calibration. Le tableau montre la précision par catalogueur sur les corrections que vous avez effectuées sur cette Library.

---

## Rubriques connexes

- **The Cataloger** (le Classificateur) — l'accueil pleine page de ces cartes, avec un sélecteur de notes (« Classer une note… ») et un bouton « Générer tous les résumés ».
- **Note Summaries** — comment la ligne Résumé sous chaque carte est produite, et la priorité donnée d'abord à l'auteur qui préfère toujours vos propres mots.
- **Cognitive Engine** — la philosophie plus large de formulation des connaissances dans laquelle s'inscrit CECE.
- **Properties** — les champs de frontmatter `sources:` et `content_type:` dans lesquels CECE écrit.
- **Knowledge Hierarchy** — comment Source × Content Type s'inscrit dans la structure Universe / Library / Folder / Note.
