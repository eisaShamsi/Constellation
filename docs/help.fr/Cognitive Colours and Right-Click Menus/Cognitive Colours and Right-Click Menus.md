---
aliases:
  - Couleurs cognitives
  - Couleurs de cognition
  - Style des étiquettes de propriétés
  - Style des badges de taxonomie
  - Couleurs de maturité
  - Couleurs de confiance
  - Couleurs d'origine
  - Couleurs d'étape
  - Couleurs de catégorie de correspondance
  - Menu clic droit
  - Menu contextuel
  - Clic droit sur le corps de la note
  - Clic droit sur une propriété
  - Clic droit sur un résultat de recherche
  - Unifier à la demande
description: Restylez les étiquettes de propriétés du frontmatter et les badges de taxonomie, définissez une couleur partagée unique pour chaque état cognitif (Maturité, Confiance, Origine, Étape, Catégorie de correspondance) afin que toutes les surfaces s'unifient à la demande, et utilisez les menus clic droit à l'échelle de l'application sur le corps de la note, le panneau Propriétés et les résultats de recherche.
---

# Couleurs cognitives et menus clic droit

Ce sujet couvre deux choses arrivées ensemble : **deux nouvelles catégories du Concepteur de style** — **Propriétés** (restyler les petites étiquettes de votre frontmatter) et **Couleurs cognitives** (un contrôle de couleur par état cognitif, partagé dans toute l'application) — et les **menus clic droit à l'échelle de l'application** qui placent les bonnes actions à un clic, sur le corps de la note, sur une propriété du frontmatter et sur un résultat de recherche.

> Le Concepteur de style est le studio de conception en pleine page que vous ouvrez depuis **Paramètres → Apparence → « ✦ Ouvrir le Concepteur de style »**, ou depuis son propre onglet **Concepteur de style** dans la barre latérale des Paramètres. Les deux catégories ci-dessous figurent dans la liste de gauche des *Surfaces*, aux côtés d'Interface, Éditeur, Liens et le reste. Pour le comportement général du Concepteur — Inspecter, Conserver / Abandonner / Réinitialiser, Styles enregistrés — voir [[Appearance and Themes]].

---

## Concepteur de style → Propriétés

La catégorie **Propriétés** restyle les petites étiquettes qui apparaissent dans le **frontmatter** d'une note (son bloc de propriétés YAML) — les pastilles que vous voyez pour `tags`, `aliases` et consorts dans le panneau Propriétés et en haut de la note. Jusqu'ici elles étaient figées ; désormais c'est à vous de les façonner.

Ouvrez le **Concepteur de style** et cliquez sur **Propriétés** dans la liste de gauche. Le centre affiche un aperçu en direct des pastilles de propriétés ; cliquez sur un contrôle à droite et l'aperçu se restyle au fil de votre édition. Deux éléments :

### Étiquettes de propriétés

Les pastilles ordinaires d'étiquettes du frontmatter (par exemple, chaque valeur d'une liste `tags`). Quatre contrôles :

- **Fond de l'étiquette** — la couleur de remplissage de la pastille.
- **Texte de l'étiquette** — la couleur du texte à l'intérieur de la pastille.
- **Rayon de l'étiquette** — l'arrondi des coins de la pastille (0 px = carré, jusqu'à 20 px = entièrement arrondi).
- **Hauteur** — la hauteur de la pastille en pixels (14–32 px).

### Badges de taxonomie

Les badges utilisés pour les valeurs de type taxonomie. Trois contrôles :

- **Arrière-plan** — la couleur de remplissage du badge.
- **Texte** — la couleur du texte à l'intérieur du badge.
- **Rayon** — l'arrondi des coins (0–20 px).

> **Rien ne change tant que vous ne touchez pas à un contrôle.** Chaque valeur démarre exactement sur l'apparence que vous avez aujourd'hui, de sorte que la catégorie Propriétés laisse vos notes identiques jusqu'à ce que vous choisissiez délibérément une couleur ou déplaciez un curseur. Cliquez sur **Conserver** pour enregistrer l'apparence pour cet Univers.

---

## Concepteur de style → Couleurs cognitives

Constellation peint votre **vocabulaire cognitif** en couleur — la *maturité* d'une note, la *confiance* d'un lien, l'endroit d'où une idée *vient*, l'*étape* de vie dans laquelle elle se trouve, et *pourquoi* un résultat de recherche a correspondu. Le problème était que chacune de ces couleurs était décidée séparément sur chaque surface : une note « flétrissante » pouvait être d'un certain vert dans l'arborescence et d'un vert différent dans la Vue Étoiles. La catégorie **Couleurs cognitives** vous donne **un contrôle de couleur par état**, et tout ce qui affiche cet état le suit.

Ouvrez le **Concepteur de style** et cliquez sur **Couleurs cognitives** dans la liste de gauche. Le centre affiche une légende de couleurs pour l'ensemble que vous éditez ; choisissez un contrôle à droite et la légende se met à jour en direct. Il y a cinq ensembles.

### Maturité — à quel point une idée est établie

Cinq états, du plus jeune au plus établi : **Graine**, **Jeune pousse**, **Persistante**, **Canonique**, **Flétrissante**. Chacun reçoit une couleur, utilisée par les points de note dans l'arborescence, le marqueur de maturité de l'onglet et l'inspecteur de note.

### Confiance — à quel point un lien est certain

Quatre états : **Hypothèse**, **Preuve**, **Établi**, **Contesté**. Une couleur chacun.

### Origine — d'où vient une idée

Quatre états : **Reçu** (issu d'une source), **Découvert** (le vôtre), **Mixte** et **Aucune**. Une couleur chacun.

### Étape — où une note se situe dans sa vie

Six états, dans l'ordre : **Étincelle**, **Naissance**, **Croissance**, **Maturité**, **Dormance**, **Archivage**. Une couleur chacun.

### Catégorie de correspondance — pourquoi un résultat de recherche a correspondu

Sept sortes de correspondance : **Titre**, **Contenu**, **Étiquette**, **Wikilien**, **Propriété**, **Sémantique** (une correspondance fondée sur le sens, et non sur un mot exact) et **Structuré** (une correspondance par requête sur une propriété). La couleur que vous définissez ici est partagée par la surbrillance de recherche dans l'éditeur, le badge de correspondance et la surbrillance de la ligne de résultat dans le panneau de recherche.

### « Unifier à la demande » — la règle qui rend cela sûr

Les couleurs cognitives suivent une règle délibérée : **rien ne change tant que vous ne choisissez pas une couleur.** Chaque surface conserve la couleur qu'elle a aujourd'hui comme repli propre. Dès l'instant où vous définissez la couleur d'un état ici, **toutes** les surfaces qui affichent cet état adoptent votre couleur d'un coup — arborescence, onglets, inspecteur, surbrillances de recherche, etc. Définissez « Persistante » une fois, et chaque marqueur Persistante dans toute l'application s'accorde. Laissez un état intact et il a exactement l'apparence d'avant.

C'est pourquoi la catégorie peut être livrée sans altérer une seule apparence existante : elle unifie *à la demande*, jamais par défaut. Cliquez sur **Conserver** pour enregistrer vos couleurs pour cet Univers.

---

## Menus clic droit à l'échelle de l'application

Constellation vous offre désormais un menu clic droit (contextuel) complet aux trois endroits où vous en voulez un le plus souvent : le **corps de la note**, une **propriété du frontmatter** et un **résultat de recherche**. Chaque menu ne propose que les actions qui ont du sens là où vous avez cliqué.

### Clic droit sur le corps de la note

Cliquez avec le bouton droit n'importe où dans le texte d'une note pour obtenir le menu d'édition :

- **Lien** / **Lien externe** — enveloppe la sélection (ou insère au curseur) sous forme de `[[wikilien]]` ou de lien `[texte](url)`.
- **Format ▸** — un sous-menu déroulant : Gras, Italique, Souligner, Barré, Surligné, Code en ligne, Math en ligne, Basculer commentaire, Exposant, Indice, Effacer le formatage.
- **Paragraphe ▸** — un sous-menu : Liste à puces, Liste numérotée, Liste de tâches, les niveaux de titre **H1–H6** et **Corps**, et Citation.
- **Insérer ▸** — un sous-menu : Note de bas de page, Tableau, Encadré, Ligne horizontale, Bloc de code, Bloc mathématique, Image.
- **Presse-papiers** — Couper, Copier, Coller, Coller en texte brut, Tout sélectionner.
- **Style…** — saute directement dans le **Concepteur de style** centré sur la catégorie **Éditeur**, afin que vous puissiez restyler la chose même sur laquelle vous avez fait un clic droit.

### Clic droit sur une propriété du frontmatter

Cliquez avec le bouton droit sur une **ligne** de propriété dans le panneau Propriétés (ou dans le bloc de propriétés en haut de la note) et vous obtenez des actions de propriété en plus du menu d'édition complet :

- **Copier la valeur** — copie la valeur de la propriété dans le presse-papiers.
- **Copier le nom** — copie la clé de la propriété.
- **Supprimer la propriété** — supprime cette ligne de propriété.
- **Ajouter une propriété** — ajoute une nouvelle ligne de propriété vide.
- …suivis des mêmes éléments **Format / Paragraphe / Insérer / presse-papiers** que pour le corps de la note, et d'un élément **Style…** qui ouvre le Concepteur de style centré sur la catégorie **Propriétés** — ainsi « Style… » sur une étiquette de propriété stylise les étiquettes de propriétés, et non le corps de la note.

### Clic droit sur un résultat de recherche

Cliquez avec le bouton droit sur un résultat dans le panneau de recherche pour un ensemble **sûr** d'actions de note — celles qui ne mettent jamais vos fichiers en danger :

- **Ouvrir** — ouvrir la note.
- **Ouvrir dans un nouvel onglet** — l'ouvrir à côté de ce que vous avez.
- **Révéler dans l'arborescence** — mettre la note en évidence dans l'arborescence pour que vous voyiez où elle réside.
- **Copier le lien** / **Copier le chemin** — copier un wikilien vers la note, ou son chemin de fichier.
- **Ajouter un marque-page** — ajouter la note à vos marque-pages.
- **Afficher dans l'explorateur** — révéler le fichier dans le gestionnaire de fichiers de votre système d'exploitation.
- **Ouvrir dans l'app par défaut** — ouvrir le fichier dans l'application que votre système utilise pour le Markdown.
- **Style…** — ouvrir le Concepteur de style centré sur la catégorie **Couleurs cognitives** (où vivent les couleurs de correspondance de recherche).

> **Par conception, le menu des résultats de recherche n'a ni Renommer, ni Déplacer, ni Supprimer.** Un panneau de recherche affiche des résultats provenant de tout votre Univers et ne conserve pas sa propre copie à la seconde près de l'arborescence ; une action destructrice à cet endroit pourrait donc agir sur une vue périmée. Constellation garde ces opérations dans l'arborescence (et le Navigateur de notes), où la vue est toujours à jour. Le menu de recherche sert à *atteindre* une note en toute sécurité, non à restructurer votre bibliothèque.

---

## Bon à savoir

- **Local et privé.** Tout cela est calculé à partir de vos propres notes et réglages sur votre appareil. Rien n'est envoyé où que ce soit.
- **Il parle votre langue.** Chaque élément de menu, chaque nom de catégorie, chaque libellé d'état apparaît dans la langue d'interface que vous avez choisie et se reflète correctement pour les langues qui se lisent de droite à gauche. Les couleurs des états cognitifs elles-mêmes sont universelles — une couleur signifie le même état dans toutes les langues.
- **« Style… » atterrit toujours sur la bonne surface.** Chaque entrée « Style… » ouvre le Concepteur de style centré sur la catégorie de la chose sur laquelle vous avez fait un clic droit : le corps de la note → **Éditeur**, une propriété → **Propriétés**, un résultat de recherche → **Couleurs cognitives**. Vous n'avez jamais à chercher les bons contrôles.

---

## Voir aussi

- [[Appearance and Themes]] — le comportement général du Concepteur de style, les thèmes, les polices et les Styles enregistrés
- [[Properties]] — visualiser et éditer les propriétés du frontmatter dont vous restylez les étiquettes ici
- [[Search]] — le panneau de recherche dont les résultats portent le menu clic droit
- [[Cognitive Engine]] — ce que signifient Maturité, Confiance, Origine et Étape en tant que mesures de la connaissance
- [[Knowledge Formulation]] — les niveaux de confiance des liens vivants que représentent les couleurs de Confiance
