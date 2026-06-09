---
aliases:
  - Sky View
  - Sky View
  - GraphMind
  - Sky View
  - Vue en étoiles des liens
  - Vue réseau
  - Connexions entre notes
  - Graphe 3D
description: Visualisez et explorez les connexions entre vos notes grâce à la Sky View interactive de Constellation, propulsée par le moteur GraphMind.
---

# Sky View

La Sky View affiche vos notes sous la forme d'un réseau interactif de nœuds et de liens, propulsé par le moteur **GraphMind** (Pixi.js WebGL). Chaque nœud est une note, et chaque ligne représente un `[[wikilink]]` entre deux notes. Plus une note possède de connexions, plus son nœud apparaît grand.

## Ouvrir la Sky View

| Méthode | Action |
|--------|--------|
| **Mission Control** | Appuyez sur `Ctrl+P`, tapez « star view » |
| **Clavier** | `Ctrl+G` |

Appuyez sur `Échap` pour fermer la Sky View.

> [!note]
> L'icône de ruban de la Sky View a été retirée du dock de gauche. La Sky View est désormais accessible via un raccourci clavier ou Mission Control. Le mode Sky View (Organigramme) est disponible sous forme d'onglet dans la barre latérale de Gestion des notes.

---

## Interagir avec le graphe

### Interactions de base

| Entrée | Comportement |
|-------|----------|
| **Déplacer (panoramique)** | Cliquez et faites glisser dans une zone vide |
| **Zoom** | Molette de défilement (2D) ou `Ctrl+Molette` (3D) |
| **Déplacer les nœuds** | Cliquez et faites glisser n'importe quel nœud pour le repositionner |
| **Survol** | Affiche le nom de la note dans la barre d'état et met en évidence les nœuds et liens connectés |
| **Clic sur un nœud** | Ouvre cette note dans l'éditeur |
| **Double-clic sur un nœud** | Zoome et centre sur ce nœud |
| **Clic droit sur un nœud** | Ouvre le menu contextuel |

### Menu contextuel

Faites un clic droit sur n'importe quel nœud pour accéder à :

| Action | Description |
|--------|-------------|
| **Ouvrir** | Ouvre la note dans l'éditeur |
| **Centrer** | Active le mode focus centré sur ce nœud |
| **Épingler** | Verrouille le nœud à sa position actuelle. Cliquez de nouveau pour désépingler. |
| **Masquer** | Masque le nœud du graphe. Utilisez « Tout afficher » dans la barre d'outils pour révéler les nœuds masqués. |

---

## Navigation 3D

La Sky View prend en charge une navigation 3D complète — survolez vos notes comme si vous naviguiez parmi les étoiles.

### Activer le mode 3D

**Clic central et glisser** (ou **Alt+clic et glisser**) pour faire pivoter le graphe dans l'espace 3D. Une fois la rotation effectuée, les commandes de navigation 3D deviennent actives.

### Commandes 3D

| Entrée | Action |
|-------|--------|
| **Clic central et glisser** | Pivoter autour des axes X et Y |
| **Maj+clic central et glisser** | Pivoter autour de l'axe Z |
| **W / Flèche haut** | Avancer (vers l'écran) |
| **S / Flèche bas** | Reculer |
| **A / Flèche gauche** | Translation vers la gauche |
| **D / Flèche droite** | Translation vers la droite |
| **Q** | Descendre |
| **E** | Monter |
| **Ctrl+Molette** | Zoom (modifie le champ de vision) |
| **Molette normale** | Avancer/reculer dans la direction de la caméra |
| **0** | Réinitialiser la rotation à la vue plate 2D |
| **Bouton de réinitialisation** (icône ↺) | Identique à la touche `0` |

### Repère des axes XYZ

En mode 3D, un guide d'axes coloré apparaît dans le coin inférieur gauche :

| Axe | Couleur | Direction |
|------|-------|-----------|
| **X** | Rouge | Gauche–Droite |
| **Y** | Vert | Haut–Bas |
| **Z** | Bleu | Avant–Arrière (profondeur) |

Le repère pivote avec la caméra afin que vous connaissiez toujours votre orientation.

### Survol et clic en 3D

Vous pouvez survoler et cliquer sur les nœuds tout en naviguant en 3D. Le nom de la note apparaît dans la barre d'état, et un clic ouvre la note — exactement comme en mode 2D.

---

## Modes de disposition

La Sky View propose trois algorithmes de disposition. Passez de l'un à l'autre en appuyant sur `Ctrl+L` ou via le bouton de disposition dans la barre d'outils.

| Mode | Description | Idéal pour |
|------|-------------|----------|
| **Organique** | Disposition à forces dirigées. Les regroupements émergent naturellement de la densité des liens. | L'exploration générale — le mode par défaut. |
| **Hiérarchique** | Graphe orienté acyclique (DAG) descendant. | Les bibliothèques structurées avec des relations parent–enfant. |
| **Temporel** | Nœuds disposés le long d'un axe temporel horizontal selon la date de création. | Voir quand les notes ont été créées et comment la bibliothèque a grandi. |

Le changement de mode déclenche une transition animée fluide qui préserve votre orientation spatiale.

> [!tip]
> Le mode Hiérarchique est particulièrement utile pour les notes qui suivent une structure arborescente (par ex. des MOC reliant des sous-thèmes). Le mode Temporel révèle votre chronologie intellectuelle — quand des grappes de notes liées ont été créées.

---

## Mode focus

Le mode focus n'affiche qu'une note précise et son voisinage. C'est un graphe local dynamique et interactif.

### Activer le mode focus

- **Clic droit sur un nœud** → **Centrer**
- **Appuyez sur Espace** pour basculer le mode focus sur la note actuellement active

### Commandes du mode focus

En mode focus, une barre de contrôle apparaît en haut :

| Contrôle | Description |
|---------|-------------|
| **Curseur de profondeur** (1–5) | Combien de sauts de connexions afficher. 1 = liens directs uniquement, 5 = cinq niveaux de profondeur. |
| **Filtre de direction** (↔ / ← / →) | Afficher tous les liens, les entrants uniquement, ou les sortants uniquement. |
| **Bouton de fermeture** (×) | Revenir à la Sky View complète |

### Fil d'Ariane de navigation

À mesure que vous cliquez sur les nœuds en mode focus, un fil d'Ariane apparaît en haut et affiche votre parcours de navigation. Cliquez sur n'importe quel élément du fil d'Ariane pour revenir au graphe local de cette note.

> [!tip]
> Combinez le mode focus avec le curseur de profondeur pour explorer progressivement le voisinage d'une note. Commencez à la profondeur 1 pour voir les connexions directes, puis augmentez pour découvrir les relations de deuxième et troisième degré.

---

## Recherche-surbrillance

Appuyez sur `Ctrl+F` pour ouvrir la barre de recherche. Tapez une requête pour mettre en évidence les notes correspondantes.

Contrairement à un filtre, la recherche-surbrillance **atténue** les nœuds non concordants sans les supprimer. Vous conservez l'intégralité de la structure du graphe et le contexte spatial pendant que les nœuds correspondants sont mis en évidence.

> [!tip]
> La recherche fonctionne aussi bien dans le graphe complet qu'en mode focus. Vous pouvez également effectuer une recherche en mode 3D.

---

## Panneau des paramètres

Cliquez sur l'icône d'engrenage (⚙) dans la barre d'outils pour ouvrir le panneau des paramètres. Il comporte trois onglets :

### Apparence du graphe

| Contrôle | Description | Par défaut |
|---------|-------------|---------|
| **Taille des nœuds** | Agrandir ou réduire tous les nœuds | 1.5 |
| **Visibilité des étiquettes** | Quand les étiquettes apparaissent : Au survol, Toujours, ou Aucune | Au survol |
| **Taille de police des étiquettes** | Taille des étiquettes de nom de note | 12 |
| **Épaisseur des liens** | Largeur des lignes de liens | 1 |
| **Afficher les notes orphelines** | Inclure les notes sans liens | Activé |

> **Couleur d'arrière-plan du canevas.** La couleur derrière les bulles se règle dans **Paramètres → Style Setter → Sky View → Canevas → Arrière-plan** (pas dans ce panneau). Elle est indépendante de vos barres latérales/panneaux, ce qui vous permet de donner au graphe son propre fond — une couleur profonde pour faire ressortir les bulles, par exemple — sans modifier le reste de l'interface. Si elle n'est pas définie, le canevas correspond à la surface du panneau. Voir *Apparence et thèmes → Canevas de la Sky View*.

### Physique

| Contrôle | Description | Par défaut |
|---------|-------------|---------|
| **Répulsion** | Force avec laquelle les nœuds se repoussent | 50 |
| **Force de liaison** | Force avec laquelle les nœuds liés s'attirent | 0.05 |
| **Distance de liaison** | Distance cible entre les nœuds liés | 30 |
| **Relancer la simulation** | Redémarrer la disposition à forces depuis l'état actuel | — |

### IA

Paramètres des liens sémantiques par IA (Phase 2 — nécessite un modèle d'embedding local).

| Contrôle | Description |
|---------|-------------|
| **Afficher les liens sémantiques** | Activer/désactiver les liens en tirets détectés par l'IA |
| **Seuil de confiance** | Curseur permettant de filtrer les liens sémantiques selon leur score de similarité |

---

## Légende

La légende apparaît dans le coin inférieur droit et affiche les attributions de couleurs pour vos bibliothèques.

### Bascule du mode de couleur

Cliquez sur les boutons **Bibliothèque** ou **Dossier** en haut de la légende pour changer la façon dont les nœuds sont colorés :

| Mode | Coloration |
|------|----------|
| **Bibliothèque** | Chaque bibliothèque reçoit une couleur unique |
| **Dossier** | Chaque dossier de premier niveau reçoit une couleur unique |

### Cases de visibilité

Chaque entrée de la légende possède une case à cocher. Décochez une bibliothèque ou un dossier pour masquer ses nœuds du graphe. Cela vous permet de vous concentrer sur des sous-ensembles précis de votre base de connaissances.

> [!tip]
> En mode Dossier, le nombre de dossiers est indiqué entre parenthèses. Les longues listes de dossiers sont défilantes.

---

## Barre d'état

La barre d'état en bas à gauche affiche :

- **Nombre de nœuds** — total des nœuds visibles
- **Nombre de liens** — total des liens visibles
- **Nombre de MOC** — nombre de Cartes de contenu (notes-carrefours à forte connectivité)
- **Nom de la note survolée** — apparaît lorsque vous survolez un nœud

---

## Raccourcis clavier

| Raccourci | Action |
|----------|--------|
| `Ctrl+G` | Ouvrir la Sky View |
| `Échap` | Fermer la Sky View |
| `Ctrl+F` | Basculer la recherche-surbrillance |
| `Ctrl+L` | Faire défiler les modes de disposition (Organique → Hiérarchique → Temporel) |
| `Espace` | Basculer le mode focus sur la note active |
| `0` | Réinitialiser la rotation 3D à la vue plate 2D |
| `W/A/S/D` | Naviguer dans l'espace 3D (après rotation) |
| `Q/E` | Descendre/monter dans l'espace 3D |

---

## Prise en charge RTL

La Sky View offre une prise en charge de premier ordre pour l'arabe, l'hébreu et les autres écritures de droite à gauche (RTL) :

- **Les étiquettes des nœuds** détectent automatiquement la direction de l'écriture — les titres en arabe s'affichent de droite à gauche
- **Les éléments de la légende** inversent l'ordre point/texte selon la langue du contenu
- **Les infobulles et les panneaux** respectent la disposition RTL
- **Repli sur une police arabe** — les étiquettes utilisent les polices arabes du système (Noto Naskh Arabic, Segoe UI) lorsque la police principale ne couvre pas les glyphes arabes

---

## Incrustation Picture-in-Picture (PiP)

Lorsque la Sky View est ouverte et que vous cliquez sur un univers enfant, une bibliothèque ou un dossier dans la barre latérale de Gestion des notes, une fenêtre **Picture-in-Picture (PiP)** apparaît en incrustation redimensionnable par-dessus le graphe principal.

### Ce qu'affiche la PiP

La PiP affiche un sous-graphe filtré contenant uniquement les nœuds appartenant à la portée sélectionnée. Par exemple, cliquer sur une bibliothèque n'affiche que les notes de cette bibliothèque et leurs interconnexions.

### Fonctionnalités de la PiP

| Fonctionnalité | Description |
|---------|-------------|
| **Graphe filtré** | Seuls les nœuds de la portée sélectionnée apparaissent |
| **Légende filtrée** | La PiP possède sa propre légende n'affichant que les entrées pertinentes |
| **Redimensionnable** | Faites glisser les bords ou les coins pour redimensionner la fenêtre PiP |
| **Repositionnable** | Faites glisser la barre de titre pour déplacer la PiP n'importe où à l'écran |

### Synchronisation de la sélection entre modes

Cliquer sur un univers enfant, une bibliothèque, un dossier ou une note dans n'importe quel mode de la barre latérale (Arborescence, Liste ou Organigramme) met en évidence les nœuds correspondants dans le graphe de la Sky View. Cette synchronisation bidirectionnelle vous aide à conserver votre repérage spatial pendant que vous naviguez dans la barre latérale.

---

## Strates de connaissance

La Sky View dimensionne automatiquement les nœuds en fonction de leur niveau de connaissance (1 à 8) :

- Petits points : notes simples (Donnée, Information)
- Nœuds moyens : notes connectées (Proposition, Concept)
- Grands carrefours lumineux : notes de synthèse (Théorie, Paradigme, Vision du monde)

Les nœuds de niveau supérieur sont entourés d'un halo lumineux de couleur complémentaire pour offrir un contraste visuel. Cela s'active lorsqu'une bibliothèque compte 20 notes ou plus.

---

## Maturité des notes

Les nœuds affichent un anneau coloré indiquant la maturité :

- Aucun anneau : Graine (note récente)
- Anneau vert clair : Jeune pousse (en croissance)
- Anneau vert soutenu : Persistante (bien établie)
- Anneau doré : Canonique (référence faisant autorité)

La maturité est également indiquée dans l'arborescence des fichiers (bordure de gauche) et dans la barre d'onglets (point coloré).

---

## Lueur de provenance

Les nœuds de la Sky View affichent une légère lueur colorée indiquant l'origine de la connaissance :

- **Lueur bleue** : Connaissance reçue — la chaîne de sources de la note remonte à une référence externe (une note comportant un champ url, author ou doi dans son frontmatter)
- **Lueur ambre** : Connaissance découverte — la chaîne de sources de la note prend racine dans les notes propres de l'utilisateur

---

## Notes techniques

La Sky View est propulsée par le moteur **GraphMind**, un moteur de rendu Pixi.js WebGL avec une simulation d3-force s'exécutant dans un Web Worker dédié. Cette architecture garantit :

- **Un rendu à 60 ips** même avec des milliers de nœuds
- **Une disposition non bloquante** — la simulation à forces ne fige jamais l'interface
- **Le survol est purement visuel** — survoler ne déclenche jamais de recalcul de la physique
- **La simulation s'arrête une fois stabilisée** — dès que les nœuds ont trouvé leur position, le moteur physique s'arrête complètement. Seuls le déplacement d'un nœud ou la modification des paramètres le redémarrent.
