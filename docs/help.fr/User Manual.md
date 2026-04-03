# Manuel d'utilisation de Constellation

**Version 0.3.4 | Mars 2026**

Constellation est une application de bureau de gestion des connaissances personnelles (PKM) pour gerer des bibliotheques de notes Markdown. Developpee avec Tauri v2, SvelteKit et Rust, elle fonctionne nativement sur Windows, macOS et Linux avec une prise en charge complete de l'arabe et du RTL.

---

## Table des matieres

1. [Premiers pas](#premiers-pas)
2. [Univers et bibliotheques](#univers-et-bibliotheques)
3. [Creer et modifier des notes](#creer-et-modifier-des-notes)
4. [Vue Etoiles (GraphMind)](#vue-etoiles-graphmind)
5. [Second ecran](#second-ecran)
6. [Proprietes et Frontmatter](#proprietes-et-frontmatter)
7. [Modeles](#modeles)
8. [Tableaux](#tableaux)
9. [Taches](#taches)
10. [Importateur](#importateur)
11. [Calendrier](#calendrier)
12. [Lens](#lens)
13. [Parametres](#parametres)
14. [Raccourcis clavier](#raccourcis-clavier)
15. [Prise en charge RTL et arabe](#prise-en-charge-rtl-et-arabe)
16. [Securite et confidentialite](#securite-et-confidentialite)
17. [Moteur Cognitif](#moteur-cognitif)

---

## 1. Premiers pas

### Installation

Telechargez le dernier installateur depuis la [page des versions de Constellation](https://github.com/eisaShamsi/Constellation/releases) :

- **Windows** : installateur `.exe` (NSIS) ou `.msi`
- **macOS** : image disque `.dmg`
- **Linux** : `.AppImage` ou paquet `.deb`

### Premier lancement

Lors de la premiere ouverture de Constellation, l'**Assistant de configuration de l'Univers** vous guide a travers :

1. **Choisir votre langue** — 15 langues prises en charge
2. **Creer ou importer une bibliotheque** — pointez vers un dossier existant de fichiers Markdown, ou commencez a zero
3. **Nommer votre univers** — l'univers est le conteneur de toutes vos bibliotheques

### Apercu de l'interface

| Element | Description |
|---------|-------------|
| **Barre laterale (Ribbon)** | Boutons de navigation : Arborescence, Recherche, Vue Etoiles, Calendrier, Modeles, Parametres |
| **Arborescence** | Parcourir les notes et dossiers de vos bibliotheques |
| **Editeur** | Lire et modifier vos notes Markdown |
| **Barre d'onglets** | Ouvrir plusieurs notes dans des onglets |
| **Barre d'etat** | Nombre de mots, nombre de caracteres, temps de lecture |

---

## 2. Univers et bibliotheques

### Qu'est-ce qu'un Univers ?

Un **Univers** est le conteneur principal qui regroupe toutes vos bibliotheques. Considerez-le comme votre espace de travail ou votre collection de bibliotheques.

### Qu'est-ce qu'une Bibliotheque ?

Une **Bibliotheque** est un dossier sur votre ordinateur contenant des fichiers Markdown (`.md`). Vous pouvez avoir plusieurs bibliotheques dans un seul univers — par exemple, une pour les notes de travail et une pour les notes personnelles.

### Gerer les bibliotheques

- **Ajouter une bibliotheque** : Parametres > Bibliotheques > Ajouter une bibliotheque, ou glissez un dossier dans l'application
- **Supprimer une bibliotheque** : Parametres > Bibliotheques > cliquez sur le bouton de suppression a cote du nom de la bibliotheque
- **Parametres de la bibliotheque** : Chaque bibliotheque peut avoir ses propres parametres d'apparence (polices, couleurs)

### Univers enfants

Vous pouvez imbriquer des univers dans des univers. Un **Univers enfant** est un autre dossier d'univers reference par votre univers parent. Les notes des univers enfants apparaissent dans la Vue Etoiles aux cotes de vos propres notes, avec les liens inter-bibliotheques affiches en lignes pointillees.

### Réouverture automatique

Constellation se souvient de votre dernier univers actif et le rouvre automatiquement au lancement. Si l'univers a été déplacé ou si son chemin a changé, Constellation le détecte et corrige automatiquement le chemin.

### Univers portables

Les univers de Constellation sont entierement portables. Vous pouvez deplacer le dossier d'un univers n'importe ou — un autre disque, une cle USB ou un autre ordinateur — et Constellation detectera et corrigera automatiquement tous les chemins internes lors de la reouverture.

Pour deplacer un univers :
1. Fermez Constellation
2. Deplacez ou copiez le dossier de l'univers vers le nouvel emplacement
3. Ouvrez Constellation → l'ecran d'accueil s'affiche (l'ancien chemin n'est plus valide)
4. Choisissez **Ouvrir un univers existant** et pointez vers le nouvel emplacement
5. Toutes les notes et bibliotheques apparaissent immediatement — les chemins sont corriges automatiquement

La structure du dossier univers suit le modele Obsidian : les notes vont directement dans le dossier racine, la configuration reside dans `.constellation/`.

---

## 3. Creer et modifier des notes

### Creer une note

| Methode | Action |
|---------|--------|
| **Clavier** | `Ctrl+N` |
| **Arborescence** | Clic droit sur un dossier > Nouvelle note |
| **Mission Control** | `Ctrl+P` > "Nouvelle note" |

### Modes d'edition

Constellation propose deux modes d'edition, selectionnables dans **Parametres > Editeur > Type d'editeur** :

#### Editeur Markdown (CodeMirror)

L'editeur par defaut pour les utilisateurs avances. Ecrivez directement en Markdown avec :

- **Apercu en direct** — affiche la mise en forme en ligne pendant la saisie
- **Mode source** — affiche la syntaxe Markdown brute
- **Barre d'outils de formatage** — apparait lors de la selection de texte
- **Commandes slash** — tapez `/` pour des insertions rapides
- **Autocompletion Wikilink** — tapez `[[` pour lier des notes
- **Curseurs multiples** — `Alt+Click` ou `Ctrl+D`

#### Editeur de document (TipTap)

Une experience de traitement de texte WYSIWYG avec une barre d'outils visuelle :

- Gras, Italique, Souligne, Barre, Surlignage
- Titres (H1–H3), Alignement du texte
- Listes a puces, Listes numerotees, Listes de taches
- Citations, Blocs de code, Lignes horizontales
- Tableaux (insertion, ajout/suppression de lignes et colonnes)
- Liens et Images

Les deux editeurs enregistrent au format Markdown standard. Vous pouvez passer de l'un a l'autre a tout moment sans perte de donnees.

### Callouts (Encadres)

Creez des blocs d'encadre stylises pour les notes, avertissements, astuces et autres indications :

```markdown
> [!note] Information importante
> Le contenu du callout se place ici.

> [!warning] Attention
> Cette action ne peut pas etre annulee.

> [!tip]- Cliquez pour developper
> Contenu de callout repliable.
```

Types pris en charge : `note`, `tip`, `warning`, `danger`, `success`, `question`, `failure`, `bug`, `example`, `quote`, `abstract`. Chaque type a une couleur et une icone distinctes. Ajoutez `-` apres le type pour le rendre repliable (demarre replie), ou `+` (demarre deploye).

### Syntaxe de surlignage

Entourez le texte de doubles signes egaux pour le surligner :

```markdown
Ceci est du ==texte surligne== dans votre note.
```

En apercu en direct, les marques `==` sont masquees et le texte apparait avec un fond jaune.

### Blocs de code

Les blocs de code delimites s'affichent avec une couleur de fond et une etiquette de langage :

````markdown
```javascript
const greeting = "Hello, world!";
```
````

Le nom du langage apparait sous forme de badge au-dessus du bloc de code.

### Integration d'images

Integrez des images directement dans vos notes :

```markdown
![Texte alternatif](https://example.com/image.png)   — URL externe
![[photo.jpg]]                                         — fichier local de la bibliotheque
```

En apercu en direct, les images sont rendues en ligne. Les images locales doivent se trouver dans le dossier de votre bibliotheque. Les images externes necessitent une connexion internet.

### Barre d'outils de tableau

Lorsque votre curseur se trouve dans un tableau markdown, une barre d'outils flottante apparait avec :

- **+ Ligne / + Colonne** — ajouter des lignes ou des colonnes
- **- Ligne / - Colonne** — supprimer des lignes ou des colonnes
- **Alignement** — alignement gauche, centre ou droite par colonne
- **Tri** — trier les lignes par ordre croissant ou decroissant selon la colonne actuelle
- **Tab / Shift+Tab** — naviguer entre les cellules du tableau

### Raccourcis de formatage du texte

| Raccourci | Action |
|-----------|--------|
| `Ctrl+B` | Gras |
| `Ctrl+I` | Italique |
| `Ctrl+Shift+S` | Barre |
| `Ctrl+Shift+H` | Surlignage |
| `Ctrl+K` | Inserer un wikilink |
| `Ctrl+Z` | Annuler |
| `Ctrl+Shift+Z` | Retablir |

### Lier des notes

Tapez `[[` pour ouvrir l'autocompletion des notes. Commencez a saisir un nom de note et selectionnez parmi les suggestions. Les liens apparaissent sous forme de wikilinks cliquables : `[[Nom de la note]]`.

Vous pouvez egalement lier vers des titres specifiques : `[[Nom de la note#Titre]]`.

---

## 4. Vue Etoiles (GraphMind)

La Vue Etoiles visualise vos notes sous forme de graphe 3D interactif propulse par le moteur **GraphMind** (Pixi.js WebGL).

### Ouvrir la Vue Etoiles

- Cliquez sur l'icone de graphe dans la barre laterale
- Appuyez sur `Ctrl+G`
- Mission Control (`Ctrl+P`) > "Vue Etoiles"

### Navigation

| Entree | Action |
|--------|--------|
| **Cliquer + glisser** | Deplacer le graphe |
| **Defiler** | Zoomer/dezoomer |
| **Cliquer sur un noeud** | Ouvrir la note |
| **Clic droit sur un noeud** | Menu contextuel (Ouvrir, Focaliser, Epingler, Masquer) |
| **Clic du milieu + glisser** | Rotation en 3D |
| **W/A/S/D** | Voler dans l'espace 3D |
| **0** | Reinitialiser la rotation en 2D |
| **Ctrl+F** | Rechercher et surligner |
| **Espace** | Basculer le mode focus |

### Modes de disposition

Appuyez sur `Ctrl+L` pour alterner entre :

- **Organique** — disposition basee sur les forces ou les groupes emergent naturellement
- **Hierarchique** — disposition en arbre de haut en bas
- **Temporel** — notes disposees par date de creation sur une chronologie

### Mode focus

Clic droit sur un noeud > **Focaliser** pour ne voir que son voisinage. Ajustez :

- **Profondeur** (1–5 sauts) — nombre de niveaux de connexions a afficher
- **Direction** (Tous/Entrants/Sortants) — tous les liens, entrants uniquement ou sortants uniquement

### Navigation 3D

Clic du milieu et glissez pour effectuer une rotation. Utilisez W/A/S/D/Q/E pour voler a travers le champ d'etoiles. Un gizmo d'axes XYZ dans le coin indique votre orientation. Appuyez sur `0` pour reinitialiser.

### Parametres

Cliquez sur l'icone d'engrenage pour :

- **Apparence** : Taille des noeuds, visibilite des etiquettes, taille de police, epaisseur des liens, afficher les orphelins
- **Physique** : Force de repulsion, force des liens, distance des liens
- **IA** : Seuil de liens semantiques (Phase 2)

### Legende

La legende en bas a droite affiche les couleurs des bibliotheques/dossiers avec des cases a cocher pour basculer la visibilite.

### Strates de Connaissance

La Vue Etoiles classe automatiquement vos notes en huit strates de connaissance selon le niveau d'abstraction :

| Strate | Description |
|--------|-------------|
| **Instantane** | Notes rapides et ephemeres |
| **Journal** | Evenements dates et entrees de journal |
| **Sujet** | Concepts atomiques autour d'une seule idee |
| **Carte** | Notes organisatrices reliant d'autres sujets |
| **Cadre** | Modeles et cadres de reflexion |
| **Principe** | Regles et axiomes verifies |
| **Conviction** | Valeurs et croyances fondamentales |
| **Artefact** | Oeuvres achevees et definitives |

La strate est determinee automatiquement a partir du frontmatter, de la structure et des liens de la note. Vous pouvez outrepasser la classification manuellement en ajoutant une propriete `stratum` dans le frontmatter.

### Cycle de Maturite

Chaque note traverse un cycle de maturite refletant son degre de developpement :

- **Graine** — Idee initiale ou brouillon brut
- **Pousse** — La note prend forme et possede quelques liens
- **Persistant** — Note mature, revisee et bien reliee
- **Canonique** — Reference definitive et faisant autorite

Le niveau de maturite est mis a jour automatiquement en fonction du nombre de liens, de la date de revision et de la frequence d'edition. Vous pouvez egalement le definir manuellement via la propriete `maturity` dans le frontmatter.

---

## 5. Second ecran

Ouvrez une fenetre separee pour une visualisation cote a cote des notes.

- **Ouvrir** : Cliquez sur l'icone du second ecran dans la barre laterale, ou `Ctrl+Shift+N`
- **Synchronisation** : Les notes s'ouvrent independamment dans le second ecran. Les parametres de police et de theme s'appliquent aux deux fenetres.
- **Largeur de la note** : Ajustable via le curseur de largeur dans la barre d'outils

---

## 6. Proprietes et Frontmatter

Les notes peuvent contenir du YAML Frontmatter en en-tete :

```yaml
---
tags: [project, active]
date: 2026-03-19
status: in-progress
---
```

Constellation detecte automatiquement les types de proprietes :

| Type | Exemple |
|------|---------|
| **Texte** | `author: John` |
| **Nombre** | `priority: 5` |
| **Date** | `date: 2026-03-19` |
| **Liste** | `tags: [a, b, c]` |
| **Case a cocher** | `done: true` |
| **Lien** | `related: [[Autre note]]` |

Basculez l'affichage des proprietes dans **Parametres > Editeur > Proprietes dans le document** (Visible / Masque / Source).

---

## 7. Modeles

Creez des modeles de notes reutilisables :

1. Creez un dossier pour les modeles dans votre bibliotheque
2. Definissez le chemin du dossier de modeles dans **Parametres > Modeles**
3. Lors de la creation d'une nouvelle note, choisissez un modele depuis le selecteur de modeles

Les modeles prennent en charge les variables :

| Variable | Remplacee par |
|----------|---------------|
| `{{date}}` | Date actuelle |
| `{{time}}` | Heure actuelle |
| `{{title}}` | Titre de la note |
| `{{clipboard}}` | Contenu du presse-papiers |

---

## 8. Tableaux

### Tableaux Markdown

Saisissez un tableau Markdown manuellement ou utilisez la commande slash `/table` :

```markdown
| En-tete 1 | En-tete 2 |
|-----------|-----------|
| Cellule 1 | Cellule 2 |
```

### Barre d'outils de tableau

Lorsque votre curseur se trouve dans un tableau, une barre d'outils flottante apparait avec :

- Ajouter/supprimer des lignes et colonnes
- Aligner les colonnes (gauche, centre, droite)
- Naviguer entre les cellules avec `Tab` / `Shift+Tab`

### Tableaux dans l'editeur de document

L'editeur de document (TipTap) offre une experience de tableau visuelle :

- Cliquez sur le bouton tableau pour inserer
- Utilisez le menu deroulant pour la gestion des lignes/colonnes
- Redimensionnez les colonnes en faisant glisser les bordures

---

## 9. Taches

Constellation prend en charge les cases a cocher de taches dans les notes :

```markdown
- [ ] Tache incomplete
- [x] Tache terminee
```

En mode Apercu en direct, les cases a cocher sont cliquables. Les taches peuvent etre recherchees et filtrees dans toutes vos bibliotheques.

---

## 10. Importateur

Importez des notes depuis d'autres outils PKM :

- **Obsidian** — importe les vaults avec une compatibilite wikilink complete
- **Dossiers Markdown** — importez n'importe quel dossier de fichiers `.md`
- **Autres formats** — HTML, fichiers texte

Allez dans **Parametres > Importateur** pour lancer un import.

---

## 11. Calendrier

La vue Calendrier affiche les notes organisees par date :

- Les notes avec une propriete `date` apparaissent a leur jour respectif
- Des notes quotidiennes peuvent etre creees pour n'importe quelle date
- Naviguez entre les mois avec les boutons flechees

Ouvrez le Calendrier depuis la barre laterale.

---

## 12. Lens

Lens fournit des vues filtrees de vos notes :

- Filtrer par tags, dossiers, proprietes
- Trier par nom, date ou proprietes personnalisees
- Enregistrer les configurations Lens pour un acces rapide

---

## 13. Parametres

Accedez aux Parametres depuis l'icone d'engrenage dans la barre laterale ou `Ctrl+,`.

### General

- Langue (15 langues)
- Theme (Clair / Sombre)
- Police d'interface, Police de texte, Police monospace, Taille de police
- Theme de police — combinaisons de polices predefinies (Machine a ecrire, Classique, Moderne, etc.) pour un changement rapide

### Editeur

- Type d'editeur (Markdown / Document)
- Vue par defaut (Lecture / Edition)
- Mode apercu en direct
- Numeros de ligne, Guides d'indentation, Correcteur orthographique
- Appariement automatique des crochets, Listes intelligentes

### Bibliotheques

- Ajouter/supprimer des bibliotheques
- Parametres d'apparence par bibliotheque
- Emplacement du dossier des pieces jointes

### Mises a jour

- Verifier les mises a jour
- Jeton GitHub pour les mises a jour de depots prives

---

## 14. Raccourcis clavier

### Globaux

| Raccourci | Action |
|-----------|--------|
| `Ctrl+N` | Nouvelle note |
| `Ctrl+O` | Saut stellaire (ouverture rapide) |
| `Ctrl+P` | Mission Control |
| `Ctrl+G` | Ouvrir la Vue Etoiles |
| `Ctrl+,` | Parametres |
| `Ctrl+Shift+F` | Rechercher dans la bibliotheque |
| `Ctrl+Shift+N` | Second ecran |

### Editeur

| Raccourci | Action |
|-----------|--------|
| `Ctrl+B` | Gras |
| `Ctrl+I` | Italique |
| `Ctrl+K` | Inserer un wikilink |
| `Ctrl+Z` | Annuler |
| `Ctrl+Shift+Z` | Retablir |
| `Ctrl+D` | Selectionner l'occurrence suivante |
| `Ctrl+/` | Basculer le commentaire |
| `Tab` | Indenter / cellule de tableau suivante |

### Vue Etoiles

| Raccourci | Action |
|-----------|--------|
| `Ctrl+F` | Rechercher et surligner |
| `Ctrl+L` | Changer le mode de disposition |
| `Espace` | Basculer le mode focus |
| `0` | Reinitialiser la rotation 3D |
| `W/A/S/D/Q/E` | Voler en 3D |
| `Escape` | Fermer la Vue Etoiles |

---

## 15. Prise en charge RTL et arabe

Constellation offre une prise en charge de premier ordre pour l'arabe, l'hebreu, le persan, l'ourdou et les autres ecritures RTL :

- **Detection automatique** : La direction de la note est detectee automatiquement a partir du contenu
- **Interface** : Interface RTL complete lorsque la langue arabe/hebraique est selectionnee
- **Editeur** : Edition de texte RTL avec mouvement de curseur et selection corrects
- **Vue Etoiles** : Les etiquettes arabes s'affichent de droite a gauche avec un repli de police adequat
- **Legende** : Les elements inversent l'ordre point/texte selon la langue du contenu
- **Polices de script** : Configurez les polices arabes, hebraiques et CJK independamment dans les Parametres

### Configuration pour l'arabe

1. Allez dans **Parametres > General > Langue** et selectionnez Arabe
2. Optionnellement, definissez une police arabe dediee dans **Parametres > General > Polices de script**
3. Les notes avec du contenu arabe s'afficheront automatiquement en RTL

---

## 16. Securite et confidentialite

- **Toutes les donnees restent locales** — pas de synchronisation cloud, pas de telemetrie, pas de suivi
- **Fichiers Markdown** — vos notes sont des fichiers texte brut qui vous appartiennent entierement
- **Aucun compte requis** — Constellation fonctionne entierement hors ligne
- **Mises a jour optionnelles** — verifiez les mises a jour manuellement via les Parametres
- **Open source** — consultez le code sur [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 17. Moteur Cognitif

Le Moteur Cognitif est le systeme d'intelligence integre de Constellation qui analyse vos notes et revele les motifs caches et les relations entre vos idees. Sa philosophie fondamentale :

> « La quantite de vos donnees n'a pas d'importance. Ce qui compte, ce n'est pas combien de sources vous stockez, mais comment vous formulez votre connaissance a partir d'elles et la reliez en une conscience unique et significative. »

Le Moteur Cognitif se compose de neuf outils integres : Liens types, Strates de connaissance, Cycle de maturite, Detecteur de tensions, Chaine de provenance, Moteur d'externalisation, Impulsion de revision, Sentiers et Vues multi-lentilles.

---

### 17.1 Liens types

#### De quoi s'agit-il ?

Les liens types sont des wikilinks portant un type de relation qui decrit la nature du lien entre deux notes. Au lieu d'ecrire simplement `[[note]]`, vous ecrivez `[[note|type-de-relation]]` pour exprimer la nature du lien : est-elle derivee ? La contredit-elle ? L'etend-elle ?

#### Pourquoi est-ce important ?

Un lien ordinaire dit « il y a une connexion » sans preciser laquelle. Les liens types transforment votre reseau de notes d'un amas de references en une veritable carte du savoir qui rend visibles les structures de pensee, les dependances et les raisonnements entre les idees.

#### Comment l'utiliser

1. Ouvrez une note dans l'editeur
2. Ecrivez un wikilink avec un type de relation : `[[Note cible|derives-from]]`
3. Types pris en charge : `derives-from` (derive de), `supports` (soutient), `contradicts` (contredit), `extends` (etend), `exemplifies` (illustre), `questions` (questionne)
4. Vous pouvez egalement ajouter des types via les proprietes de la note dans la barre laterale droite

#### Ou le voir ?

- **Vue Etoiles (GraphMind)** : Sous forme de lignes colorees et etiquetees entre les noeuds
- **Barre laterale droite** : Dans l'onglet « Retrolinks » avec indication du type de chaque lien
- **Onglet Provenance** : Utilise pour construire l'arbre genealogique du savoir

---

### 17.2 Strates de connaissance

#### De quoi s'agit-il ?

Le Moteur Cognitif classe automatiquement chaque note dans l'une des huit strates : Instantane, Journal, Sujet, Carte, Cadre, Principe, Conviction, Artefact. Le classement repose sur la structure, le contenu et le nombre de liens de la note.

#### Pourquoi est-ce important ?

Connaitre le type de chaque note revele l'equilibre des connaissances dans votre bibliotheque. Vos notes sont-elles principalement des instantanes ephemeres ou ont-elles evolue vers des principes et des cadres ? Cette prise de conscience de la nature du contenu est le premier pas vers la construction d'un savoir veritable plutot que la simple accumulation d'informations.

#### Comment l'utiliser

1. La classification se fait automatiquement — aucune action de votre part n'est necessaire
2. Pour outrepasser la classification automatique, ajoutez la propriete `stratum` dans le frontmatter :
   ```yaml
   ---
   stratum: framework
   ---
   ```
3. Valeurs disponibles : `snapshot`, `log`, `topic`, `map`, `framework`, `principle`, `conviction`, `artifact`

#### Ou le voir ?

- **Barre laterale droite** : Dans la section proprietes sous « Strate »
- **Vue Etoiles** : Sous forme de couleurs differentes des noeuds selon la strate
- **Parametres > Moteur Cognitif** : Pour activer ou desactiver la classification automatique

---

### 17.3 Cycle de maturite

#### De quoi s'agit-il ?

Le moteur suit le niveau de maturite de chaque note en quatre etapes : **Graine** → **Pousse** → **Persistant** → **Canonique**. Chaque note commence comme graine et murit progressivement avec l'ajout de contenu, de liens et de revisions.

#### Pourquoi est-ce important ?

La maturite distingue une idee brute d'un savoir abouti. La graine d'aujourd'hui peut devenir la reference de demain si vous lui accordez l'attention necessaire. Le suivi de maturite vous aide a identifier les notes qui meritent davantage de developpement et d'attention.

#### Comment l'utiliser

1. La maturite evolue automatiquement selon : le nombre de mots, le nombre de liens entrants et sortants, et la date de derniere modification
2. Pour definir la maturite manuellement, ajoutez la propriete `maturity` dans le frontmatter :
   ```yaml
   ---
   maturity: evergreen
   ---
   ```
3. Valeurs disponibles : `seed` (Graine), `sapling` (Pousse), `evergreen` (Persistant), `canonical` (Canonique)

#### Ou le voir ?

- **Barre laterale droite** : Une icone a cote du titre indique le stade de maturite actuel
- **Vue Etoiles** : Sous forme de taille du noeud — plus la note est mature, plus le noeud est grand
- **Parametres > Moteur Cognitif** : Pour activer ou desactiver le suivi de maturite

---

### 17.4 Detecteur de tensions

#### De quoi s'agit-il ?

Le Detecteur de tensions examine les notes liees et vous alerte lorsque des affirmations ou conclusions sont contradictoires entre deux notes ou plus. Il s'appuie sur l'analyse des liens types `contradicts` et la similarite thematique entre les notes.

#### Pourquoi est-ce important ?

Les tensions ne sont pas necessairement des erreurs — ce sont des invitations a une reflexion plus profonde. Lorsque deux idees dans votre bibliotheque se contredisent, cela signifie que votre comprehension a evolue ou qu'il existe une complexite qui merite d'etre exploree. Detecter les tensions vous empeche de construire inconsciemment un savoir sur des bases contradictoires.

#### Comment l'utiliser

1. Ajoutez un lien type `contradicts` entre les notes en conflit : `[[Autre note|contradicts]]`
2. Le moteur detecte egalement les tensions implicites par analyse du contenu
3. Consultez la liste des tensions detectees dans la barre laterale

#### Ou le voir ?

- **Barre laterale droite** : Dans l'onglet « Tensions » quand des contradictions sont detectees
- **Vue Etoiles** : Sous forme de lignes rouges pointillees entre les noeuds en conflit
- **Panneau de notifications** : Alertes lors de la detection d'une nouvelle tension

---

### 17.5 Chaine de provenance

#### De quoi s'agit-il ?

La Chaine de provenance retrace l'origine de chaque idee — d'ou elle vient et de quoi elle derive. Elle utilise les liens `[[note|derives-from]]` pour construire un arbre genealogique montrant le chemin d'evolution du savoir depuis la source originale jusqu'a la formulation actuelle.

#### Pourquoi est-ce important ?

Connaitre l'origine de vos idees distingue le savoir recu (de livres, articles, conferences) du savoir decouvert (vos propres conclusions et reflexions). Cette conscience de la source du savoir vous aide a evaluer la fiabilite de vos idees et a comprendre comment votre pensee s'est formee au fil du temps.

#### Comment l'utiliser

1. Lorsque vous creez une note derivee d'une source, ajoutez un lien : `[[Source originale|derives-from]]`
2. Des chaines a plusieurs niveaux sont possibles : note ← derivee de ← derivee de ← source originale
3. Classez les sources externes en ajoutant `source-type: received` dans le frontmatter

#### Ou le voir ?

- **Barre laterale droite** : L'onglet « Provenance » affiche l'arbre genealogique complet
- **Vue Etoiles** : Sous forme de direction des fleches sur les liens (de la source au derive)
- **Proprietes de la note** : Classification comme « recu » ou « decouvert » selon la chaine de provenance

### 17.6 Moteur d'externalisation

#### De quoi s'agit-il ?

Un pipeline de formalisation progressive qui suit la maturation de vos notes, des captures brutes aux idees cristallisees. Chaque note peut se voir attribuer l'une des quatre etapes :

| Etape | Icone | Signification |
|-------|-------|---------------|
| Ephemere | 🌱 | Capture rapide, pensee passagere |
| Litterature | 📖 | Reecrite a partir d'une source dans vos propres mots |
| Permanent | 🔗 | Idee atomique, un concept, connectee a votre graphe |
| Synthese | ✨ | Idee originale combinant plusieurs notes permanentes |

#### Pourquoi est-ce important ?

La plupart des apps traitent toutes les notes de la meme facon. Le Moteur d'externalisation rend la distinction visible — vous pouvez voir d'un coup d'oeil quelle part de votre bibliotheque est une capture brute et quelle part est une comprehension veritable.

#### Comment l'utiliser

1. Dans la barre de navigation (au-dessus de l'editeur), utilisez le menu deroulant des etapes pour selectionner une etape.
2. Ou developpez les Proprietes et utilisez le menu deroulant des etapes. Les deux se synchronisent instantanement avec l'arborescence des fichiers.
3. Pour promouvoir une note, changez le menu deroulant d'une etape a la suivante. En mode Focus, cliquez sur « Promouvoir en Permanent » en bas.
4. Pour supprimer une etape, selectionnez « — Etape — » dans le menu deroulant.

#### Ou le voir ?

- **Barre de navigation** : menu deroulant avec emoji + nom de l'etape
- **Panneau des proprietes** : menu deroulant quand la propriete `stage` existe
- **Arborescence des fichiers** : icone emoji a cote du nom de la note
- **Pied de page du mode Focus** : bouton « Promouvoir en Permanent »

### 17.7 Impulsion de revision

#### Qu'est-ce que c'est ?

L'Impulsion de revision est un systeme de resurgissement espace qui ramene les notes a votre attention a des intervalles croissants : 1 jour, puis 3, puis 7, puis 14, puis 30 jours apres la derniere revision. Il surveille egalement les notes etiquetees avec `#assumption` ou `#model` comme points de controle des modeles mentaux, et maintient une file « Jamais revisees » pour les notes capturees mais jamais revisitees.

#### Pourquoi c'est important ?

La connaissance se dissipe sans revisitation. Vous ecrivez une note aujourd'hui et dans trois semaines vous avez oublie qu'elle existe. La repetition espacee est la technique la mieux etablie en sciences cognitives pour combattre ce declin. L'Impulsion de revision applique ce principe a vos notes reelles.

#### Comment l'utiliser

1. Cliquez sur l'onglet **Impulsion de revision** dans la barre laterale gauche. Vous verrez trois sections : A reviser, Points de controle des modeles mentaux (`#assumption` / `#model`), et Jamais revisees.
2. Cliquez sur une note pour l'ouvrir et la lire.
3. Choisissez l'une des trois actions :
   - **Revisee** (coche) — planifie la prochaine revision au prochain intervalle (1 → 3 → 7 → 14 → 30 jours).
   - **Reporter 7j** (icone oeil) — reporte la note de 7 jours sans avancer l'intervalle.
   - **Rejeter** (icone archive) — retire la note de la file de revision definitivement.
4. Ouvrez la Palette de Commandes et tapez "Review due notes" pour acceder directement aux notes en attente.

#### Ou le voir ?

- **Barre laterale gauche** : L'onglet Impulsion de revision avec un compteur de notes en attente
- **Palette de Commandes** : Commande "Review due notes" pour un acces rapide

### 17.8 Sentiers

#### Qu'est-ce que c'est ?

Les Sentiers sont des sequences nommees et ordonnees de notes — comme les chapitres d'un livre ou les etapes d'une visite guidee de vos connaissances. Ils sont definis en ajoutant `trail: true` au frontmatter d'une note, puis en listant les wikiliens dans l'ordre dans le corps de la note.

#### Pourquoi c'est important ?

La connaissance n'est pas toujours un reseau. Parfois c'est un chemin — une sequence d'apprentissage, une progression d'arguments, un recit. Les Sentiers capturent cet ordre explicitement, ajoutant une dimension lineaire a votre bibliotheque non lineaire.

#### Comment l'utiliser

1. Creez une nouvelle note avec `trail: true` dans le frontmatter.
2. Dans le corps de la note, listez les wikiliens dans l'ordre souhaite.
3. Lorsque vous ouvrez une note appartenant a un sentier, la barre de navigation affiche un indicateur avec le nom du sentier et la position (ex. « Mon Sentier 2/5 »). Des fleches de navigation permettent d'aller a la note precedente et suivante.
4. Ouvrez la Palette de Commandes et tapez "Open Trail" pour voir tous les sentiers.

#### Ou le voir ?

- **Barre de navigation** : Indicateur du sentier avec nom, position et fleches de navigation
- **Palette de Commandes** : Commande "Open Trail" liste tous les sentiers

### 17.9 Vues multi-lentilles

#### Qu'est-ce que c'est ?

Les Vues multi-lentilles permettent de visualiser votre bibliotheque a travers differents schemas de classification — sans modifier la structure des dossiers ni dupliquer de notes. Une "lentille" est un regroupement virtuel qui reorganise vos notes selon une propriete ou un tag. Lentilles integrees : "Par etape" (Fugace/Litterature/Permanent/Synthese) et "Par sujet" (regroupement par tags). Des lentilles personnalisees peuvent etre creees dans les Parametres.

#### Pourquoi c'est important ?

Les structures de dossiers imposent une seule hierarchie, mais la connaissance ne tient pas dans un seul arbre. Les Vues multi-lentilles permettent de basculer entre differentes perspectives sans deplacer de fichiers. Les memes notes, vues a travers differentes lentilles organisationnelles.

#### Comment l'utiliser

1. Dans la barre laterale, trouvez le **selecteur de lentilles** en haut de l'arborescence (par defaut "Dossiers").
2. Selectionnez une lentille : "Par etape", "Par sujet" ou une lentille personnalisee. La barre laterale se reorganise instantanement.
3. Selectionnez "Dossiers" pour revenir a l'arborescence par defaut.
4. Pour creer une lentille personnalisee : ouvrez **Parametres > Gestion des connaissances**, cliquez sur **Creer une lentille**, nommez-la et choisissez la propriete frontmatter pour le regroupement.
5. Ou utilisez la Palette de Commandes : tapez "Create Lens".

#### Ou le voir ?

- **Selecteur dans la barre laterale** : Selecteur de lentilles en haut de l'arborescence
- **Parametres > Gestion des connaissances** : Creer, modifier et supprimer des lentilles personnalisees
- **Palette de Commandes** : Commande "Create Lens"

### Parametres du Moteur Cognitif

Tous les outils du Moteur Cognitif se configurent dans **Parametres > Moteur Cognitif** :

- **Classification des strates** — Activer ou desactiver la classification automatique
- **Suivi de maturite** — Activer ou desactiver le suivi du cycle de maturite
- **Liens types** — Ajuster le seuil de sensibilite pour la detection des liens (0.0 – 1.0)
- **Detecteur de tensions** — Activer ou desactiver la detection automatique des tensions
- **Substitution manuelle** — Ajoutez les proprietes `stratum` et `maturity` dans le frontmatter pour outrepasser la classification automatique

---

*Manuel d'utilisation de Constellation — Version 0.3.4 — Mars 2026*
*uconstellation.world*
