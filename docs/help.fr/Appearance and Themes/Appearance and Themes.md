---
aliases:
  - Thèmes
  - Style Settings
  - Thème personnalisé
  - Importer un thème Obsidian
  - Supprimer un thème
  - Exporter les réglages de style
description: Personnalisez chaque partie visible de Constellation — thèmes, couleurs, typographie et style des composants via Apparence et l'onglet natif Style Settings.
---

# Apparence et thèmes

L'apparence de Constellation est contrôlée depuis deux endroits dans les **Paramètres** :

1. **Apparence** — choisissez ou créez un thème, importez des thèmes depuis le registre communautaire d'Obsidian, et ajustez les préférences globales de police et de mise en page.
2. **Style Settings** — un onglet dédié qui expose chaque pièce visible de l'interface de Constellation sous forme de contrôle ajustable en direct (curseurs, sélecteurs de couleurs, listes déroulantes). Les changements s'appliquent instantanément et sont enregistrés dans le thème actif.

Ensemble, ils vous permettent de remodeler l'application selon votre flux de travail, votre taille d'écran et votre goût personnel — sans modifier une seule ligne de CSS.

## Thèmes

Un **thème** est un ensemble nommé de couleurs, réglages et CSS qui définit l'apparence de Constellation. Constellation est livré avec six thèmes intégrés (Constellation Light/Dark, Nord Light/Dark, Solarized Light/Dark), tous associés entre modes clair et sombre.

### Choisir un thème

1. Ouvrez **Paramètres → Apparence**.
2. Cliquez sur n'importe quelle carte dans la grille **Thèmes**. Le thème s'applique immédiatement.
3. La carte active est mise en évidence par une bordure d'accent.

### Créer un thème personnalisé

1. Dans la grille des thèmes, cliquez sur la carte en pointillés **+ Nouveau thème**.
2. Donnez-lui un nom, choisissez clair ou sombre et sélectionnez cinq couleurs (fond, surface, texte, accent, bordure).
3. Cliquez sur **Enregistrer**. Votre thème apparaît désormais dans la grille.

Toutes les autres variables (états de survol, ombres, texte atténué) sont dérivées automatiquement de vos cinq couleurs via des calculs HSL, vous ne contrôlez donc que l'essentiel.

### Modifier ou supprimer un thème personnalisé

Survolez n'importe quelle carte de thème personnalisé :
- **✏️ (crayon)** — ouvre l'éditeur pour modifier le nom, le type ou les cinq couleurs principales.
- **✕ (croix rouge)** — supprime le thème après confirmation. Les thèmes intégrés ne peuvent pas être supprimés. Si vous supprimez le thème actif, Constellation revient au thème par défaut.

### Importer un thème de la communauté Obsidian

Cliquez sur **🟣 Thèmes Obsidian** pour parcourir plus de 200 thèmes communautaires :
1. Recherchez par nom ou auteur.
2. Cliquez sur **Aperçu** pour voir une maquette de la mise en page et la palette de cinq couleurs.
3. Cliquez sur **Importer** — le CSS du thème est téléchargé, adapté à Constellation (shim de sélecteurs + extraction de variables + couleurs de syntaxe CodeMirror) et ajouté à vos thèmes personnalisés.
4. Si le thème prend en charge les **Style Settings**, le nombre est affiché sur sa carte ; ces options apparaissent dans l'onglet Style Settings après l'importation.

## Style Settings

L'onglet **Style Settings** est le panneau de contrôle natif de Constellation, indépendant du thème. Il couvre chaque élément visible du chrome ainsi que l'éditeur, et fonctionne avec n'importe quel thème (intégré, personnalisé ou importé).

### Organisation

Les sections sont repliées par défaut. Cliquez sur le chevron pour les développer :

- **Constellation — Couleurs** — fond et surfaces, texte, accent
- **Constellation — Typographie** — tailles de police interface/note/code, tailles H1–H6, graisse des titres, hauteurs de ligne, espacement des paragraphes
- **Constellation — Mise en page et forme** — coins (petits/moyens/grands rayons), largeurs de bordure, ombres, longueur de ligne de l'éditeur, marges latérales
- **Constellation — Composants** — dock ruban, barre d'actions latérale, barre de mise en page (bascules de panneaux), barre supérieure/bande d'onglets, barre d'état, explorateur de fichiers, barre latérale droite, boutons, étiquettes, callouts
- **Constellation — Éditeur** — liens, code et blocs, citation en bloc, curseur et sélection

### Modifier une valeur

- **Sélecteurs de couleur** — cliquez sur l'échantillon, choisissez une couleur. L'hex s'affiche à côté.
- **Curseurs** — glissez pour ajuster. La valeur numérique apparaît dans l'unité (px, %, etc.).
- **Interrupteurs** — cliquez pour activer/désactiver des classes (principalement pour les thèmes importés).
- **Listes déroulantes** — choisissez une option (style de décoration de lien, etc.).
- **Flèche de réinitialisation (↺)** — apparaît au survol au bout de chaque ligne. Un clic efface votre surcharge et restaure la valeur par défaut du thème.

### Comment fonctionne l'enregistrement

- Les modifications sont enregistrées automatiquement dans les **styleSettingsValues** du thème actif.
- Si vous modifiez un Style Setting alors qu'un thème intégré est actif, Constellation **clone automatiquement** le thème intégré dans vos thèmes personnalisés (sous la forme `{Nom} (custom)`), puis y enregistre vos modifications. Le thème intégré reste intact.
- L'étiquette **Enregistré dans :** en bas de l'onglet indique quel thème contient actuellement vos surcharges.
- Cliquez sur **Tout réinitialiser aux valeurs par défaut** pour effacer toutes les surcharges du thème actif.

### Importer / Exporter les Style Settings

Barre d'outils en haut de l'onglet Style Settings :

- **📋 Coller depuis le presse-papiers** — un clic : lit le presse-papiers et fusionne le JSON valide dans le thème actif.
- **⬆️ Importer / Coller** — ouvre une zone de texte ; collez du JSON manuellement. Choisissez **Fusionner** (ajoute/remplace) ou **Tout remplacer** (efface, utilise uniquement le collé).
- **📄 Depuis un fichier** — ouvrez un fichier `.json` exporté depuis le plugin Style Settings d'Obsidian ou une autre installation Constellation.
- **📋 Copier** — copie les valeurs actuelles dans le presse-papiers au format JSON formaté.
- **⬇️ Exporter** — enregistre les valeurs sous `{theme-name}-style-settings.json`.

Le format JSON correspond exactement à celui du plugin Style Settings d'Obsidian — un objet plat qui mappe les IDs de réglages à des valeurs de chaîne :

```json
{
  "h1-size": "36",
  "interactive-accent": "#7c3aed",
  "my-themed-color@@light": "#ffffff",
  "my-themed-color@@dark": "#1e1e2e"
}
```

Cela signifie que vous pouvez copier vos Style Settings depuis Obsidian et les coller directement dans Constellation, ou inversement.

## Ce que vous pouvez contrôler

Chaque réglage se trouve dans l'un des cinq blocs ci-dessus. Points forts :

### Typographie

- **Taille de police d'interface** — barre latérale, barres d'outils, menus
- **Taille de police de note** — texte du corps dans l'éditeur
- **Taille de police de code** — code en ligne et blocs de code
- **Tailles H1 – H6** — chaque niveau de titre individuellement
- **Graisse des titres** — légèreté ou épaisseur de tous les titres
- **Hauteurs de ligne** — normale (corps) et serrée (titres et UI dense)
- **Espacement des paragraphes** — écart entre paragraphes

### Composants de la coque

- **Dock ruban (icônes de gauche)** — largeur, taille de bouton, taille d'icône, rayon, couleurs
- **Barre d'actions latérale** — icônes nouvelle note/tableau/dossier — taille, couleur, hauteur, fond
- **Barre de mise en page (bascules de panneaux)** — bascules barre latérale gauche/séparation/droite — taille de bouton, taille d'icône, couleurs, couleur d'état actif
- **Barre supérieure / bande d'onglets** — visible uniquement quand des notes sont ouvertes en onglets ; contrôle hauteur de bande, fond, hauteur/police/rayon d'onglet, couleurs d'onglet actif et inactif
- **Barre d'état** — hauteur, taille de police, fond, couleur de texte
- **Barre latérale droite (inspecteur)** — fond, hauteur de ligne d'onglets, taille d'icône d'onglet, couleurs
- **Explorateur de fichiers (barre latérale gauche)** — ligne des notes d'Univers, lignes des univers enfants (cUniverse), noms de bibliothèques, dossiers, notes — chacun avec taille, graisse et couleur indépendantes ; plus espacement vertical des lignes

### Éditeur

- **Tailles de titres** (H1–H6) et graisse
- **Hauteur de ligne** dans le corps de note
- **Code en ligne** fond, couleur de texte, rayon, taille de police
- **Couleur de lien** (par défaut + survol) et style de décoration (aucun/souligné/pointillé)
- **Largeur de barre de callout** et **rayon de callout**
- **Couleur de curseur** et **fond de sélection**

### Couleurs (chaque couleur de l'application)

- Fond (principal/alt), surfaces, fond de survol, bordures, fond d'entrée
- Texte (normal/atténué/faible/sur accent), états erreur/avertissement/succès
- Accent (accent interactif + survol), texte sur accent

## Questions fréquentes

### Puis-je styliser la barre de titre Windows (« Constellation v0.3.4 — … ») ?

Non — cette barre est dessinée par le système d'exploitation (Windows/macOS/Linux). Constellation n'a pas d'accès CSS à celle-ci. Tout ce qui est en dessous est entièrement stylisable.

### Pourquoi le curseur de largeur de barre latérale ne fonctionne-t-il pas ?

La largeur de la barre latérale est contrôlée par la poignée sur le bord (glissez pour redimensionner). Nous ne dupliquons délibérément pas ce contrôle dans Style Settings pour éviter des sources de vérité en conflit.

### Où vivent mes Style Settings ?

Dans `Universe/settings.json` sous `customThemes[i].styleSettingsValues`, par thème. Ils voyagent avec votre Univers — si vous synchronisez votre répertoire Univers entre appareils, votre style vient avec.

### Puis-je partager un thème avec quelqu'un ?

Oui :
- **Thème complet** — dans l'éditeur de thème, cliquez sur **Exporter**. Partagez le fichier `.json`. Le destinataire clique sur **↓ Importer** dans la grille des thèmes et le sélectionne.
- **Seules les valeurs Style Settings** — dans l'onglet Style Settings, cliquez sur **Exporter** pour exporter uniquement les valeurs de curseurs/couleurs (pas la structure du thème). Utile pour appliquer vos réglages personnels par-dessus le thème de quelqu'un d'autre.

### Un thème Obsidian importé paraît cassé. Que faire ?

Les thèmes Obsidian peuvent être complexes. Cas connus :
- Les thèmes utilisant des **couleurs HSL séparées** (comme Minimal) — pris en charge dans Constellation à partir de cette version.
- Les thèmes dépendant de la structure DOM spécifique d'Obsidian peuvent s'afficher partiellement. Constellation inclut un shim de classes qui mappe les sélecteurs les plus courants, mais les thèmes très structurels peuvent nécessiter d'ajuster les cinq couleurs principales ou de corriger les valeurs Style Settings à la main pour compenser.

## Liens

- [[Universe]] — où les thèmes et les valeurs Style Settings sont stockés
- [[Libraries]] — accents de couleur par bibliothèque (définis dans les paramètres de bibliothèque, indépendants des thèmes)
- [[Importer]] — pour importer des notes, pas des thèmes (l'import de thème est dans Apparence)
