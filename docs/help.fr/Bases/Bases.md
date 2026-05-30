---
translation_status: AI-generated 2026-05-30 — native-speaker review recommended
language: fr
source: docs/help.uConstellation.World/Bases/Bases.md
aliases:
  - Bases
  - Base Constellation
  - Tables de notes
  - Vues structurées
  - Fichiers de base
  - Fichiers .base
description: Apprenez à utiliser la Base Constellation — un tableau vivant de vos notes, une ligne par note et une colonne par propriété, que vous pouvez trier, modifier et remodeler sans jamais déplacer un fichier.
---

# Bases

Une **Base** transforme un ensemble de vos notes en un tableau vivant : **une ligne par note, une colonne par propriété**. Rien n'est copié ni déplacé — le tableau lit vos notes sur place et les reflète telles qu'elles sont en cet instant.

> [!tip] Robuste mais simple, par défaut
> Une Base s'ouvre avec une allure familière et épurée — juste les noms de vos notes et les champs qui vous importent. Les colonnes cognitives plus profondes de Constellation sont toujours **à un clic**, mais elles n'encombrent jamais le premier écran. C'est vous qui décidez de la quantité de structure à faire apparaître.

> [!info] Non destructif
> Une Base ne modifie jamais vos notes d'elle-même. C'est un petit fichier `.base` qui contient une requête — « montre ces notes, avec ces colonnes, dans cet ordre ». Vos fichiers Markdown restent exactement là où ils sont.

---

## Deux façons d'utiliser une Base

**1. En tant qu'onglet entier.** Ouvrez un fichier `.base` et il remplit l'onglet sous forme de tableau interactif.

**2. À l'intérieur d'une note.** Insérez un bloc de code délimité dans n'importe quelle note et il s'affiche en ligne :

````markdown
```base
view: table
```
````

Les deux sont propulsés par le même moteur, ils se comportent donc de façon identique.

---

## Créer une Base

Utilisez **Nouvelle Base** depuis la barre latérale (l'action « + » / Nouvelle Base). Constellation écrit pour vous un petit fichier **YAML** `.base` :

```yaml
schema: 1
lens: My Notes
scope:
  libraries: all
  federation: auto
columns:
  - dimension: note.name
view: table
```

| Champ | Signification |
|-------|---------------|
| `schema` | Version du format (actuellement `1`). |
| `lens` | Le nom affiché en haut du tableau. |
| `scope.libraries` | `all`, ou une liste de bibliothèques spécifiques à inclure. |
| `scope.federation` | `auto` — inclure aussi les notes de tout Univers lié (cUnivers). |
| `columns` | Les colonnes à afficher. Une nouvelle Base commence avec le seul **Nom** de la note. |
| `view` | `table` (le tableau est la vue de la Base). |

Vous avez rarement besoin de modifier ceci à la main — les commandes du tableau lui-même (ci-dessous) réécrivent chaque changement dans le fichier pour vous.

---

## Le tableau

- **Colonne Nom** — toujours en premier. Cliquez sur le nom d'une note pour l'ouvrir.
- **Chaque note correspondante devient une ligne.** Il n'y a **aucune limite de lignes**. Le tableau est *virtualisé* — il ne dessine que les lignes actuellement à l'écran — donc une Base portant sur des milliers de notes s'ouvre instantanément et défile sans à-coups.
- **Direction par cellule** — chaque valeur détecte son propre sens d'écriture, de gauche à droite ou de droite à gauche, afin que les tableaux multilingues se lisent correctement.
- Le pied de tableau indique la durée de la requête.

---

## Colonnes — ajouter, retirer, réordonner

### Ajouter une colonne

Cliquez sur **+ Ajouter une colonne**. Le sélecteur est regroupé en deux :

- **Vos champs** — les propriétés de frontmatter que Constellation a trouvées dans vos notes (par exemple `status`, `maturity`, `author`). Ce sont *vos* données.
- **Constellation** — les champs intégrés que l'app connaît toujours : **Nom**, **Chemin**, **Créé le** et **Résumé**.

Commencez à taper pour filtrer la liste. Les champs déjà présents dans le tableau sont marqués pour que vous ne les ajoutiez pas deux fois.

### Retirer une colonne

Survolez un en-tête de colonne et cliquez sur le **×**.

### Réordonner les colonnes

**Appuyez sur un en-tête de colonne et faites-le glisser latéralement.** Toute la colonne se soulève (elle s'estompe et l'en-tête affiche un contour de préhension), et une ligne verticale indique où elle va se déposer. Relâchez pour la déplacer. La colonne Nom reste fixe en première position.

Chaque ajout, retrait et réorganisation est réenregistré automatiquement dans le fichier `.base`.

---

## Tri

**Cliquez sur un en-tête de colonne pour trier selon celle-ci.** Chaque clic fait défiler **croissant → décroissant → désactivé** (une flèche indique le sens actuel).

Pour trier selon plusieurs colonnes, ouvrez le panneau **Tri** :

- Ajoutez plusieurs colonnes — la première est le tri principal, les suivantes départagent les égalités.
- Basculez n'importe quel niveau entre croissant et décroissant.
- Montez ou descendez les niveaux pour changer la priorité, ou retirez-les.

---

## Modifier une note depuis le tableau

Double-cliquez sur une cellule dans l'une de **vos** colonnes de frontmatter pour la modifier :

- **Champs en texte libre** — tapez la nouvelle valeur ; **Entrée** enregistre, **Échap** annule.
- **Champs de type liste** (comme `maturity`) — un **menu déroulant** apparaît avec les valeurs valides **dans leur ordre naturel** (pour `maturity` : *seed → sapling → evergreen → canonical*). Choisissez-en une, ou tapez la vôtre.

Le changement est écrit directement dans le frontmatter YAML de cette note sur le disque, et le tableau se met à jour sur place.

> [!note] Colonnes en lecture seule
> **Nom** et **Créé le** (ainsi que les autres colonnes Constellation intégrées) sont calculées pour vous, elles ne sont donc pas modifiables. Seuls vos propres champs de frontmatter peuvent être modifiés ici.

---

## Ouvrir une Base plus ancienne

Si vous passez d'Obsidian, ou d'une version antérieure de Constellation, vos fichiers `.base` existants utilisent un ancien format.

**Votre fichier n'est jamais touché.** Lorsque Constellation en ouvre un, il affiche un avis sobre expliquant que le format est plus ancien, et propose un bouton **Convertir en Base Constellation**. La conversion n'a lieu **que lorsque vous cliquez dessus** — elle met le fichier à niveau sur place vers le nouveau format YAML (en reportant ce qu'elle peut : le nom, les colonnes et les filtres de texte simples). Tant que vous ne choisissez pas de convertir, le fichier d'origine est laissé exactement tel qu'il était.

---

## Fédération

Une Base est consciente de l'Univers. Avec `federation: auto`, elle inclut les notes de tout Univers lié (cUnivers) aux côtés des vôtres. Les notes qui résident dans un Univers lié sont en lecture seule — vous pouvez les consulter et les trier dans la Base, mais la modification est réservée aux notes que vous possédez.

---

## Local-first & fichier-avant-app

Les Bases ne détiennent aucune donnée propre. Chaque valeur que vous voyez provient d'un véritable fichier `.md` sur votre disque, lu en direct. Supprimez le fichier `.base` et vos notes sont totalement épargnées — une Base n'est qu'une lentille que vous pointez vers des notes que vous possédez déjà.

---

## Clavier & souris

| Action | Ce qu'elle fait |
|--------|-----------------|
| **Cliquer** sur un en-tête de colonne | Trier selon celle-ci (croissant → décroissant → désactivé) |
| **Glisser** un en-tête de colonne | Réordonner cette colonne |
| **Cliquer** sur le × d'un en-tête | Retirer cette colonne |
| **Double-cliquer** sur une cellule de frontmatter | La modifier (menu déroulant pour les champs de type liste) |
| **Entrée** | Enregistrer la modification |
| **Échap** | Annuler la modification |
| **Cliquer** sur le nom d'une note | Ouvrir la note |
