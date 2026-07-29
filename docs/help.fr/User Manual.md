# Manuel d'utilisation de Constellation

**Version 0.1.0 | Mars 2026**

Constellation est une application de bureau de gestion des connaissances personnelles (PKM) pour gerer des bibliotheques de notes Markdown. Developpee avec Tauri v2, SvelteKit et Rust, elle fonctionne nativement sur Windows, macOS et Linux avec une prise en charge complete de l'arabe et du RTL.

---

## Table des matieres

1. [Premiers pas](#premiers-pas)
2. [Univers et bibliotheques](#univers-et-bibliotheques)
3. [Creer et modifier des notes](#creer-et-modifier-des-notes)
4. [Recherche](#recherche)
5. [Vue Etoiles (GraphMind)](#vue-etoiles-graphmind)
6. [Vue fractionnee](#vue-fractionnee)
7. [Index](#index)
8. [Constellation Sight](#constellation-sight)
9. [Second ecran](#second-ecran)
10. [Proprietes et Frontmatter](#proprietes-et-frontmatter)
10b. [Révision des Sources (CECE)](#10b-révision-des-sources-constellation-epistemic-content-engine--cece)
11. [Modeles](#modeles)
12. [Tableaux](#tableaux)
13. [Taches](#taches)
14. [Importateur](#importateur)
15. [Calendrier](#calendrier)
16. [Lens](#lens)
17. [Parametres](#parametres)
18. [Raccourcis clavier](#raccourcis-clavier)
19. [Prise en charge RTL et arabe](#prise-en-charge-rtl-et-arabe)
20. [Securite et confidentialite](#securite-et-confidentialite)
21. [Carte des connaissances](#carte-des-connaissances)
22. [Moteur Cognitif](#moteur-cognitif)

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

### L'Arborescence (Explorateur de fichiers)

L'**Arborescence** est l'Explorateur de fichiers de Constellation : elle sert a parcourir **et a organiser** vos notes et dossiers. En plus de l'arbre classique, elle porte desormais les outils de gestion de fichiers dont vous avez besoin pour une grande bibliotheque.

**Filtrer par nom.** Un champ de filtre se trouve en haut de l'arborescence. Tapez n'importe quel fragment d'un nom de note ou de dossier (dans n'importe quelle langue) et l'arbre se reduit aux correspondances, en ouvrant les dossiers qui les contiennent pour que rien ne reste cache. Le filtre parcourt **toutes** les bibliotheques — celles qui sont repliees sont chargees et deployees automatiquement, puis restaurees exactement comme vous les aviez laissees quand vous effacez le filtre. Il ne cherche que dans les **noms**, jamais dans le contenu des notes (chercher *a l'interieur* des notes est le role du Search Hub).

**Trier de huit facons.** Le bouton de tri fait defiler **Nom** (A → Z / Z → A), **Modifie** (plus recent / plus ancien), **Cree** (plus recent / plus ancien) et **Taille** (plus grand / plus petit) ; les dossiers restent toujours en haut. Survolez le bouton pour voir le tri actuel.

**Selection multiple.** **Ctrl-clic** (⌘-clic sur Mac) pour ajouter ou retirer une note ou un dossier de la selection ; **Maj-clic** pour selectionner une plage entiere. Un simple clic sur une note l'ouvre toujours — la selection reste en place jusqu'a ce que vous appuyiez sur **Echap** ou que vous l'effaciez. Les lignes selectionnees sont surlignees par une barre d'accentuation.

**Operations par lot.** Lorsque des elements sont selectionnes, une barre apparait en bas de la barre laterale et affiche leur nombre, avec **Deplacer**, **Ajouter une etiquette** et **Supprimer**. Chaque action s'applique a toute la selection via les memes operations sures et controlees qu'une note isolee — l'etiquetage par lot ne corrompt donc jamais une note, et la suppression passe par la corbeille. Les notes provenant d'univers enfants lies (en lecture seule) sont ignorees automatiquement.

**Les bases restent :** deplier/replier les dossiers au clic ou avec les fleches, clic droit pour le menu contextuel (Ouvrir, Renommer, Deplacer, Ajouter une etiquette, Supprimer…), et glisser-deposer pour deplacer des notes entre dossiers.

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

### Synchronisation et modifications externes

Constellation applique le principe **File Over App** — vos notes sont de simples fichiers `.md` sur le disque, et l'application surveille leurs modifications. Si une note arrive ou change *depuis l'extérieur* de Constellation pendant que l'application est ouverte — une synchronisation Obsidian depuis un autre appareil, un `git pull`, un outil de synchronisation cloud (iCloud / Syncthing / OneDrive), ou un fichier que vous déposez dans le dossier d'une bibliothèque — Constellation la prend en compte **automatiquement**, en une seconde environ, **sans redémarrage** :

- La note apparaît dans l'**arborescence des fichiers**.
- Elle devient repérable dans le **Saut stellaire** (`Ctrl+O`), la **Recherche**, l'**Index**, les **rétroliens** et le **nombre de notes** de la bibliothèque — tout se met à jour tout seul.
- Si vous renommez un dossier depuis l'extérieur de l'application, ses notes restent repérables à leur nouvel emplacement et les anciennes entrées sont nettoyées.
- Un lot volumineux (un `git pull` de nombreuses notes, ou une première synchronisation) est indexé en arrière-plan — la frappe reste instantanée pendant que la recherche se met à jour.

Vous n'avez rien à faire : Constellation maintient son index de recherche synchronisé avec vos fichiers à mesure qu'ils changent sur le disque. *(Un détail : renommer un dossier depuis l'**extérieur** de l'application réinitialise l'historique du calendrier de révision et du poids des liens de ces notes — le texte de la note lui-même reste intact. Renommer des dossiers **à l'intérieur** de Constellation préserve tout.)*

**Si la note modifiée est actuellement OUVERTE dans un onglet**, Constellation la met à jour en toute sécurité — votre travail n'est jamais écrasé en silence :

- Si vous n'avez **aucune modification non enregistrée** dans cette note, la note ouverte se rafraîchit discrètement pour afficher la modification externe, de sorte que votre prochaine frappe s'appuie sur la nouvelle version. *(Auparavant, une note ouverte continuait d'afficher l'ancien texte et votre prochaine frappe pouvait enregistrer par-dessus la modification externe en silence — cela ne peut plus se produire.)*
- Si vous **avez bien des modifications non enregistrées** dans cette note au moment précis où une modification externe arrive — un véritable conflit — Constellation ne touche jamais à votre travail non enregistré. Il conserve **votre** version dans l'éditeur, écrit la version externe entrante dans une **copie annexe** à côté de la note (nommée `<note>.conflict-<timestamp>.md.txt`, de sorte que rien n'est jamais perdu), et affiche une bannière : *« Une modification externe de {note} a été conservée comme copie séparée — votre version reste inchangée. »* Cliquez sur **Afficher la copie** pour ouvrir le dossier sur cette copie annexe et fusionner à la main si vous le souhaitez. La copie annexe est un fichier `.txt` inerte — elle n'apparaît jamais dans votre barre latérale ni dans la recherche, et ne déclenche jamais une nouvelle synchronisation.

**Fusionner les deux versions.** La bannière de conflit comporte aussi un bouton **Fusionner…**. Il ouvre une vue plein écran à deux colonnes — **Votre version** à gauche (modifiable) à côté de la **Copie externe** à droite (en lecture seule) — avec les différences mises en évidence et les parties identiques repliées. À côté de chaque différence, un bouton **Copier vers la vôtre** reporte cette modification externe dans votre version ; vous pouvez aussi modifier librement la colonne de gauche pour combiner les deux à la main. Une fois terminé, **Enregistrer la fusion** écrit votre note réconciliée et déplace la copie annexe vers la corbeille de la bibliothèque (récupérable, jamais supprimée) ; **Annuler** ne change rien — les deux versions restent exactement telles qu'elles étaient. Constellation ne fusionne jamais automatiquement — la réconciliation est toujours votre choix.

**Si la note modifiée était FERMÉE à ce moment-là**, sa réouverture vous montre le fichier le plus récent. Constellation conserve une copie de sauvegarde de chaque note que vous ouvrez — c'est elle qui protège le travail non enregistré en cas d'échec d'une sauvegarde — et jusqu'à la version 0.1 cette copie pouvait l'emporter sur une note modifiée ailleurs pendant qu'elle était fermée : vous la rouvriez, vous voyiez l'*ancien* texte, et au changement d'onglet suivant Constellation réécrivait l'ancienne version par-dessus le fichier plus récent. Silencieusement, en signalant une sauvegarde réussie.

> [!important] Corrigé en 0.1
> La copie de sauvegarde indique désormais si elle contient du travail qui n'a jamais été écrit sur le disque, ou seulement une copie de ce qui était déjà enregistré. Seule la première peut l'emporter. Une note modifiée sur un autre appareil, par `git pull` ou par un outil de synchronisation pendant qu'elle était fermée s'ouvre donc sur **le fichier le plus récent** — et le travail non enregistré reste protégé exactement comme avant.
>
> Un détail à connaître : les copies faites par la version précédente ne portent pas cette marque, donc l'ancien comportement subsiste pour une note jusqu'à la première fois où vous la rouvrez et la refermez.

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

### Vos onglets reviennent au redémarrage

Jusqu'ici, fermer Constellation faisait oublier quelles notes étaient ouvertes — chaque lancement repartait d'une fenêtre vide. Désormais, l'application mémorise vos onglets ouverts, celui qui était actif et si la fenêtre était fractionnée, puis remet tout en place automatiquement au lancement suivant. Vous retrouvez votre bureau tel que vous l'aviez laissé.

- La mémorisation est **par Univers** et se met à jour discrètement environ une seconde après chaque ouverture, fermeture ou réorganisation d'onglets. Un plantage ou un arrêt forcé ne fait perdre, au pire, que la dernière seconde de *disposition* — jamais le contenu des notes (la protection du contenu repose sur un mécanisme distinct, plus ancien).
- Une note déplacée ou supprimée pendant que l'application était fermée est simplement ignorée ; le reste de vos onglets revient quand même.
- Pour désactiver la fonction : **Paramètres > Éditeur > Restaurer les onglets au redémarrage**. La désactivation supprime aussi la session enregistrée — désactiver signifie *cesser de mémoriser*.
- Les **Espaces de travail** nommés ne sont pas concernés : ils restent vos instantanés délibérés, enregistrés à la main. Cette fonction n'est que le « dernier état » mis à jour en continu.
- Limite connue : avec une vue fractionnée, la fraction elle-même revient, mais la répartition des onglets entre les volets n'est pas encore mémorisée.

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

#### Personnaliser les callouts — couleurs, icones et vos propres types

Les couleurs et les icones des callouts vous appartiennent : vous pouvez les modifier, et meme inventer vos propres types de callout. Ouvrez le Style Setter (le bouton 🎨 dans la barre laterale), choisissez la categorie Editeur, puis cliquez sur Callouts. Le centre affiche un gestionnaire de Callouts unique ou chaque callout occupe une ligne montrant sa couleur, son icone et son nom.

- Recolorer un callout integre. Cliquez sur la pastille de couleur de sa ligne. Une petite palette s'ouvre avec vos couleurs enregistrees (cliquez sur l'une d'elles pour l'appliquer) ainsi qu'un selecteur « Personnalise… » pour toute nouvelle couleur — une couleur que vous choisissez est aussi ajoutee a votre palette pour la prochaine fois. Les changements de couleur des types integres sont enregistres lorsque vous appuyez sur Conserver/Appliquer dans le Style Setter.
- Changer l'icone d'un callout integre. Cliquez sur l'icone de sa ligne. La Bibliotheque d'emojis et d'icones s'ouvre — choisissez n'importe quel emoji ou icone vectorielle. Le changement s'applique partout immediatement, dans la couleur de ce callout. Un petit ↺ apparait pour vous permettre de revenir uniquement sur cette icone.
- Reinitialiser les types integres. Le bouton « ↺ Reinitialiser cet element » en haut du gestionnaire retablit toutes les couleurs et icones des callouts integres a leurs valeurs par defaut. (Vos callouts personnalises ne sont pas touches — supprimez-les individuellement.)
- Creer votre propre type de callout. Sous le separateur se trouve la ligne d'ajout. Saisissez un Nom (par exemple Decision, ou فكرة), un mot Declencheur (le [!mot] que vous taperez — toute langue fonctionne, y compris l'arabe), choisissez une couleur et une icone, puis cliquez sur Ajouter. Desormais, taper > [!decision] (ou > [!فكرة]) dans n'importe quelle note affiche votre callout. Si vous ne tapez pas de titre apres le declencheur, l'en-tete du callout affiche le nom de votre callout en gras.
- Modifier ou supprimer un callout personnalise. Utilisez ✎ (modifier le nom/declencheur) et ✕ (supprimer) sur sa ligne. Supprimer un type laisse le texte [!…] de vos notes intact — il revient simplement a l'apparence d'une note ordinaire jusqu'a ce que vous recreiez le type.

Vos callouts personnalises, couleurs et icones sont enregistres avec cet Univers, ils accompagnent donc votre bibliotheque.

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

### Enregistrement et récupération

Constellation **enregistre automatiquement** au fil de votre saisie — il n'y a pas de bouton d'enregistrement. Vos modifications sont écrites dans le fichier `.md` un instant après une pause (et chaque fois que vous changez de note ou fermez un onglet). Une note n'est marquée « enregistrée » qu'une fois qu'elle est réellement écrite sur le disque.

**Fermer l'application est aussi un point d'enregistrement.** Quand vous fermez Constellation, chaque note contenant une saisie non enregistrée est écrite dans son fichier **avant** que la fenêtre ne se ferme — y compris les mots tapés à la toute dernière seconde avant votre clic sur ✕. Une fermeture normale (rien à enregistrer) est instantanée, exactement comme avant ; s'il y a quelque chose à écrire, la fenêtre peut rester ouverte un bref instant (cinq secondes au maximum) le temps que vos notes soient mises en sécurité sur le disque.

Si un enregistrement **échoue** un jour — par exemple parce qu'un outil de synchronisation (iCloud / OneDrive / Syncthing) ou un antivirus verrouille brièvement le fichier — Constellation ne **perd pas** votre travail :

- Une bannière apparaît en haut : *« Impossible d'enregistrer {note} — votre modification est en sécurité et sera réessayée. »* Votre saisie reste à l'écran et est conservée en toute sécurité en mémoire (et dans un tampon de récupération qui survit à un redémarrage).
- Constellation **réessaie automatiquement toutes les quelques secondes** : dès que le fichier se libère, votre modification est écrite d'elle-même — même si vous vous êtes éloigné.
- Vous pouvez aussi cliquer sur **Réessayer maintenant** dans la bannière pour enregistrer immédiatement. La bannière disparaît une fois la note enregistrée.

Vous n'avez jamais à craindre qu'un fichier verrouillé ou momentanément indisponible vous coûte une modification.

---

## 4. Recherche

Constellation dispose d'un moteur de recherche hybride multilingue base sur SQLite FTS5 avec classement BM25, filtres de requete structures et normalisation optimisee pour l'arabe. La recherche est accessible depuis la barre laterale.

### Comment rechercher

Cliquez sur l'icone de recherche dans la barre laterale ou utilisez `Ctrl+Shift+F` pour activer le mode recherche. Tapez votre requete et les resultats apparaissent apres un bref delai (300ms). Appuyez sur `Escape` ou cliquez sur `x` pour effacer la recherche et revenir a l'arborescence.

### Syntaxe de recherche

| Syntaxe | Exemple | Ce qui est trouve |
|---------|---------|-------------------|
| Texte libre | `gestion de projet` | Notes contenant ces mots dans le titre ou le corps |
| Filtre par tag | `#recherche` | Notes etiquetees avec `#recherche` |
| Filtre par propriete | `status=actif` | Notes avec propriete frontmatter `status` egale a `actif` |
| Filtre par wikilink | `links to [[Climat]]` | Notes contenant un lien vers `[[Climat]]` |
| Portee bibliotheque | `in:MaBibliotheque` | Restreint les resultats a une bibliotheque specifique |
| Combine | `#recherche status=actif economie` | Tous les filtres appliques ensemble |

### Badges de type de correspondance

Chaque resultat affiche un badge colore indiquant comment la correspondance a ete trouvee. Le badge affiche une lettre localisee pour l'accessibilite (adapte aux daltoniens) :

| Badge | Couleur | Signification |
|-------|---------|---------------|
| **T** | Bleu | Correspondance de titre — le terme apparait dans le nom de la note |
| **C** | Vert | Correspondance de contenu — le terme apparait dans le corps de la note |
| **S** | Violet | Correspondance semantique — conceptuellement lie (necessite un modele d'embedding) |
| **P** | Ambre | Correspondance de propriete — trouve via filtre de propriete frontmatter |
| **#** | Rose | Correspondance de tag — trouve via filtre de tag |
| **W** | Bleu clair | Correspondance de wikilink — trouve via filtre de wikilink |

Les lettres des badges sont localisees pour les 15 langues prises en charge.

### Resultats epingles (Naviguer entre les resultats)

Les resultats restent visibles apres avoir clique sur l'un d'eux. La note ouverte est mise en surbrillance dans la liste pour que vous sachiez quel resultat vous visualisez. Cliquez sur un autre resultat pour y naviguer sans relancer la recherche.

Pour effacer la recherche, appuyez sur `Escape` ou cliquez sur `x`.

### Navigation au clavier

| Touche | Action |
|--------|--------|
| `Fleche bas` | Selectionner le resultat suivant |
| `Fleche haut` | Selectionner le resultat precedent |
| `Enter` | Ouvrir le resultat selectionne |
| `Escape` | Effacer la recherche et revenir a l'arborescence |

### Mise en surbrillance du terme recherche

Lorsque vous ouvrez une note depuis les resultats, toutes les occurrences du terme sont mises en surbrillance dans l'editeur. Cela fonctionne avec la detection des diacritiques arabes — chercher "ادارة" mettra en surbrillance "إدارة" et toutes les variantes diacritiques.

### Historique de recherche

Cliquez sur le champ de recherche lorsqu'il est vide pour voir vos recherches recentes (20 dernieres requetes). Chaque entree affiche le texte et le temps ecoule depuis son execution. Cliquez sur une entree pour relancer cette recherche instantanement. Utilisez le lien "Effacer l'historique" en bas pour supprimer tout l'historique.

L'historique de recherche est stocke localement sur votre appareil et persiste entre les redemarrages.

### Search Hub

Le Search Hub est une experience de recherche en plein ecran. Cliquez sur l'icone de loupe dans la barre du dock pour l'ouvrir. Les deux barres laterales se replient pour offrir un espace maximum. Tapez n'importe quel terme et Constellation recherche partout simultanement, regroupant les resultats en 5 categories : Titres, Contenus, Tags, Proprietes et Wikilinks. Chaque categorie dispose d'une section depliable avec un badge de comptage. Cliquez sur un resultat pour l'ouvrir dans l'editeur avec toutes les occurrences surlignees. Un bouton "Retour au Search Hub" apparait pour revenir sans relancer la recherche.

### Operateurs de liens

Constellation prend en charge 6 operateurs de recherche de topologie de liens :

| Syntaxe | Ce qu'il trouve |
|---------|-----------------|
| `links to [[X]]` | Notes qui pointent vers X (backlinks) |
| `links from [[X]]` | Notes vers lesquelles X pointe (liens sortants) |
| `mutual [[X]]` | Notes liees a X ET X lie en retour (bidirectionnel) |
| `mentions [[X]]` | Notes contenant le nom de X sans [[wikilink]] |
| `orphans` | Notes sans liens entrants ni sortants |
| `links between [[X]] and [[Y]]` | Notes qui pointent vers X et Y |

Lors de la saisie d'un operateur de lien, l'autocompletion `[[` affiche toutes les notes de l'univers. Apres avoir selectionne une note, tapez `#` pour la completion des titres ou `|type:` pour la completion du type de lien.

---

## 5. Vue Etoiles (GraphMind)

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

## 6. Vue fractionnee

La vue fractionnee vous permet de modifier plusieurs notes cote a cote dans la fenetre principale.

### Ouvrir la vue fractionnee

- **Palette de commandes** : `Ctrl+P` puis tapez "Split View"
- **Raccourci clavier** : Utilisez le raccourci assigne pour alterner entre les modes
- **Cycle** : Desactive → Vertical (cote a cote) → Horizontal (haut et bas) → Desactive

### Modifier en vue fractionnee

Chaque volet est un editeur entierement independant avec :
- Barre d'outils complete (gras, italique, titres, alignement, etc.)
- Navigation par fil d'Ariane (bibliotheque / nom de la note)
- Panneau de proprietes et menu deroulant de stade
- Support de sauvegarde (`Ctrl+S` sauvegarde le volet actif)
- Modification du titre et renommage du fichier

### Redimensionner les volets

Faites glisser le separateur entre les volets pour les redimensionner. Chaque separateur est independant — avec 3 notes ou plus ouvertes, vous pouvez redimensionner n'importe quelle paire adjacente sans affecter les autres. Fonctionne en mode vertical et horizontal.

### Focus

Cliquez sur n'importe quel volet pour le mettre au premier plan. Le volet actif recoit les raccourcis clavier et est suivi par les panneaux de la barre laterale droite (Proprietes, Retroliens, etc.).

---

## 7. Index

L'Index est un glossaire complet de termes de toutes vos bibliotheques — chaque mot significatif, classe par ordre alphabetique avec le nombre d'occurrences.

### Ouvrir l'Index

- **Bouton du dock** : Cliquez sur l'icone de l'Index (livre) dans le dock gauche
- **Palette de commandes** : `Ctrl+P` puis tapez "Index"

### Pipeline NLP Multilingue

L'Index traite le texte via un pipeline sensible a la langue avant l'indexation :

- **Arabe** : Algorithme Lucene Light10 — supprime le tashkeel, unifie le hamza, supprime l'article defini (الـ), supprime les suffixes grammaticaux
- **Hebreu** : Suppression des prefixes (ב/ל/מ/ה/ו/כ/ש)
- **Anglais** : Racinisation de type Porter (pluriels, formes verbales, suffixes)
- **Francais/Espagnol/Portugais/Allemand** : Suppression de suffixes specifiques a la langue
- **Russe/Turc/Hindi/Persan** : Suppression de suffixes morphologiques
- **Les 15 langues** : Filtrage des mots vides (articles, prepositions, conjonctions)

### Navigation

- **Onglets de langue** : Basculez entre Tous, Arabe, Hebreu, Anglais ou # (caracteres speciaux)
- **Barre alphabetique** : Cliquez sur une lettre pour filtrer les termes commencant par cette lettre — le compteur de termes se met a jour pour afficher le nombre de correspondances
- **Cliquez a nouveau sur la meme lettre** pour effacer le filtre et afficher tous les termes
- **Modes de tri** : Alphabetique (par defaut) ou par frequence (les plus courants en premier)

### Modifier depuis l'Index

Cliquez sur une note dans les references d'un terme pour l'ouvrir dans un volet d'apercu fractionne a cote de l'Index. Le volet d'apercu est un editeur complet — vous pouvez modifier, sauvegarder, changer les proprietes et promouvoir le stade. Le terme recherche est surligne dans la note et deplace automatiquement a sa position.

Appuyez sur `Ctrl+Clic` pour ouvrir la note comme un onglet normal. Un bouton « Retour a l'Index » apparait dans la barre d'onglets — cliquez dessus pour revenir exactement ou vous etiez dans l'Index.

### Integration avec le Second Ecran

Lorsque le Second Ecran est ouvert :
- **Cliquez sur un terme** → Le Second Ecran affiche toutes les notes contenant ce terme dans une vue fractionnee (liste de notes + editeur)
- **Ctrl+Clic sur plusieurs termes** → Le Second Ecran affiche le mode comparaison avec chaque terme dans sa propre colonne

---

## 8. Constellation Sight

Constellation Sight visualise l'ensemble de votre systeme de connaissances sous forme de graphe en puits gravitationnel. Il repond a la question : **"A quoi ressemble mon savoir et quelle est sa sante ?"**

### Ouvrir Sight

Cliquez sur le **bouton Sight** (icone d'oeil) dans le ruban gauche. Le graphe en puits gravitationnel apparait. Cliquez sur x pour fermer.

### Le Graphe en Puits Gravitationnel

Les notes sont disposees en anneaux concentriques par importance (centralite). Les notes les plus connectees se trouvent au centre ; les notes peripheriques aux bords. Au sein de chaque anneau, les notes sont groupees par bibliotheque (votre organisation). Couleur du noeud = bibliotheque.

| Element | Signification |
|---------|---------------|
| **Grand noeud** | Haute centralite — relie differents domaines de connaissance |
| **Petit noeud** | Peripherique — au sein d'un domaine |
| **Couleur du noeud** | Appartenance a une bibliotheque |
| **Ligne pleine** | Lien entre deux notes |
| **Fleches de direction** | Petites fleches indiquant la direction du lien |
| **Epaisseur de ligne** | Niveau de confiance (epais = etabli, fin = hypothese) |

### Interaction

- **Simple clic** sur un noeud : met en surbrillance son voisinage (toutes les notes connectees). Tout le reste s'estompe.
- **Double-clic** : ouvre la note dans l'editeur.
- **Clic sur espace vide** : efface la surbrillance.
- **Defiler** : zoom. **Glisser** : panoramique. **Ajuster a l'ecran** : bouton de la barre d'outils.

### Recherche dans Sight

Cliquez sur la loupe. Prend en charge tous les operateurs : `links to [[X]]`, `links from [[X]]`, `mutual [[X]]`, `orphans`, `supports [[X]]`, `contradicts [[X]]`, `#tag`, texte libre et recherche semantique. Les resultats affichent des couleurs directionnelles : vert (entrant), rouge (sortant).

### Panneau d'Analyse (SightPanel)

Cliquez sur l'icone de grille pour ouvrir la barre laterale. Affiche : score de Sante de l'Univers (0-100), compteurs de notes/liens/orphelins, barres de type de lien et de confiance, top 10 des ponts et Aperçus des Connaissances (preuves les plus solides, fondations faibles, tensions, stagnation, les plus connectes, lacunes de connaissance).

### Parametres

Icone d'engrenage : ajustez l'epaisseur du trait de lien, l'opacite et la taille des fleches. Les parametres persistent entre les sessions.

### 8a. Champs de tradition par note (MIG-029)

La puce de tradition en haut a gauche de Sight vous permet de recadrer la coupole a travers 24 traditions savantes reparties en 10 familles epistemiques. Pour neuf de ces traditions (celles aux formes sectorielle / concentrique / en echelle), chaque note peut etre **classifiee explicitement** via un champ dans le frontmatter. Les notes sans le champ tombent dans un compartiment par defaut raisonnable propre a la tradition ; les notes AVEC le champ tombent dans le compartiment que vous avez nomme.

Ajoutez le champ au frontmatter YAML d'une note :

```yaml
---
masadir_source: sunnah
---
```

Passez a la puce de cette tradition → votre note tombera dans son secteur nomme au lieu du defaut.

**Champs autorises et valeurs :**

| Tradition | Champ frontmatter | Valeurs autorisees | Defaut si absent |
|---|---|---|---|
| **masādir** (uṣūl al-fiqh sunnite) | `masadir_source` | `quran` / `sunnah` / `ijma` / `qiyas` | `quran` |
| **pramāṇa** (Nyāya indien) | `pramana_kind` | `pratyaksha` / `anumana` / `upamana` / `shabda` | `pratyaksha` |
| **Burhān d'Ibn Rushd** | `burhan_kind` | `burhan` / `jadal` / `khataba` / `shir` | `shir` (anneau le plus exterieur) |
| **PaRDeS** (hermeneutique juive) | `pardes_level` | `peshat` / `remez` / `derash` / `sod` | `peshat` |
| **Peirce** (3 categories phaneroscopiques) | `peirce_category` | `firstness` / `secondness` / `thirdness` | `firstness` |
| **Habermas** (3 interets de connaissance) | `habermas_interest` | `technical` / `practical` / `emancipatory` | `technical` |
| **Germes menciens** (4 germes moraux) | `mencian_sprout` | `ceyin` / `xiuwu` / `cirang` / `shifei` | `ceyin` |
| **Sān biǎo mohiste** (3 standards) | `mohist_zone` | `ben` / `yuan` / `yong` | distribue par hash sur 3 zones |
| **Sŏngnihak coreen** (debat Quatre-Sept) | `songnihak_cell` | `li-sa` / `li-chil` / `qi-chil` / `qi-sa` | `li-sa` |

**Comportement :**
- Si vous ecrivez une valeur que la tradition ne reconnait pas (faute de frappe ou invention), la note tombe dans le compartiment par defaut. Pas de plantage, pas de pepin d'affichage.
- Les changements de frontmatter se propagent automatiquement — sauvegardez la note → le prochain rendu de la coupole refletera le changement.
- Le meme champ n'est lu que par sa tradition nommee. Definir `masadir_source: sunnah` sur une note n'a aucun effet quand vous passez a PaRDeS ou Peirce — chaque tradition lit son propre champ de facon independante.
- C'est la facon la plus explicite de controler la grammaire spatiale de la coupole. Sans ces champs, la geometrie est correcte mais chaque note tombe par defaut dans le meme compartiment ; avec eux, la puce devient analytiquement significative.

**Traditions sans champs par note** (regroupent actuellement toutes les etoiles par d'autres moyens — dossier / bibliotheque / hash) :

- Aristotélicienne (par defaut, pas de remappage)
- Polanyi (brouillard degrade ; pas de sectorisation)
- Husserl, Longino, Maqāṣid d'al-Shāṭibī, Prophétie maïmonidienne, 13 middot talmudiques, Wang Yangming, Pluriversel de Mignolo, Transmodernité de Dussel, Maldonado-Torres, Akan de Wiredu, ʿUmrān d'Ibn Khaldūn, Ibuanyidanda

(De futures migrations pourront ajouter des champs frontmatter par note pour celles-ci a mesure que la demande des utilisateurs emerge.)

---

## 9. Second ecran

Le second ecran est une fenetre complementaire basee sur les modes qui s'adapte au mode actuel de votre barre laterale.

- **Ouvrir** : Cliquez sur l'icone du second ecran dans la barre laterale, ou `Ctrl+Shift+2`
- **Fermeture automatique** : Lorsque vous fermez la fenetre principale, le second ecran se ferme automatiquement

### Complementaire base sur les modes

Le second ecran modifie son contenu en fonction du mode actif de la barre laterale dans la fenetre principale :

| Mode de la barre laterale | Le second ecran affiche |
|---|---|
| **Explorateur de fichiers** | Tableau de bord de l'univers — statistiques, repartition des bibliotheques, univers enfants, etiquettes, notes recemment modifiees/ouvertes |
| **Vue du ciel** | Arbre de la Vue du ciel avec structure des repertoires |
| **Vue Etoiles** | Complementaire Vue Etoiles avec retroliens, liens avant, etiquettes et graphe local |

### Tableau de bord de l'univers (Mode Explorateur de fichiers)

Lorsque la fenetre principale est en mode Explorateur de fichiers, le second ecran affiche un tableau de bord avec :

- **Cartes de statistiques** — Nom de l'univers, nombre d'univers enfants, total des bibliotheques, dossiers et notes
- **Univers enfants** — Chaque univers enfant avec ses bibliotheques liees et le nombre de dossiers/notes
- **Bibliotheques** — Chaque bibliotheque avec le nombre de dossiers/notes dans des boites statistiques colorees
- **Recemment modifiees** — Notes que vous avez modifiees dans la session actuelle (suivies lors de la sauvegarde)
- **Recemment ouvertes** — Notes que vous avez ouvertes mais pas modifiees dans la session actuelle
- **Etiquettes** — Toutes les etiquettes de toutes les bibliotheques triees par nombre ; cliquez sur une etiquette pour voir toutes les notes associees

### Interaction du tableau de bord

Lorsque le tableau de bord est actif dans la fenetre principale, cliquer sur les elements les envoie au second ecran :

- **Recemment modifiees/ouvertes** : Cliquez sur une note pour l'ouvrir en tant qu'editeur complet sur le second ecran
- **Etiquettes** : Cliquez sur une etiquette pour afficher toutes les notes l'utilisant en vue fractionnee — liste de notes a gauche, editeur complet a droite

Toutes les modifications sur le second ecran sont synchronisees automatiquement avec la fenetre principale.

### Modification de notes dans le second ecran

Le second ecran prend en charge la modification complete des notes — tapez, sauvegardez, renommez et modifiez les proprietes comme dans la fenetre principale. Les modifications se synchronisent automatiquement avec la fenetre principale.

### Synchronisation des parametres

Tous les parametres visuels se propagent instantanement au second ecran — aucun redemarrage necessaire :

- **Langue** : Les changements de langue de l'interface s'appliquent immediatement
- **Theme** : Le mode clair/sombre/systeme bascule instantanement
- **Polices** : Police d'interface, police de texte, police monospace et polices specifiques aux ecritures
- **Taille de police** : Tailles de police d'interface et d'editeur
- **Editeur** : Largeur de ligne lisible, numeros de ligne, barre d'outils flottante
- **Couleur d'accent** : Changements de couleur d'accent du theme

---

## 10. Proprietes et Frontmatter

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

### Styles de liste

Une liste comme `tags:` ou `aliases:` peut s'écrire de quatre façons, qui signifient toutes la même chose. Constellation lit les quatre et affiche les mêmes puces :

- **Indentée** — `tags:` puis `  - a` en dessous. C'est ce qu'écrit Constellation.
- **Non indentée** — `tags:` puis `- a` en partant de la marge gauche. YAML parfaitement valide, et ce que produisent beaucoup d'autres outils : c'est donc courant dans les bibliothèques importées.
- **Sur une seule ligne** — `tags: [a, b]`.
- **Sur la ligne suivante** — `tags:` puis `  [a, b]` en dessous.

> [!important] Corrigé en 0.1
> La forme non indentée était lue comme une liste *vide*. Les éléments étaient toujours dans le fichier, mais le panneau n'affichait rien — et ajouter une seule nouvelle étiquette remplaçait alors la liste entière, faisant disparaître les éléments précédents. Tous les styles sont désormais lus correctement. Le même défaut touchait `aliases:`, qui sert à résoudre les liens vers les autres noms d'une note, ainsi que les liens typés comme `supports:` et `contains:`, qui alimentent le graphe de connexions.

### Blocs de texte long

`description: |` suivi de lignes indentées contient plusieurs lignes de prose en une seule valeur. Constellation affiche cette ligne **en lecture seule** avec un aperçu de la première ligne — la même règle que pour les champs imbriqués : ce qu'il ne peut pas réécrire sans risque, il ne le réécrit pas. Modifiez-la directement dans le fichier `.md`.

---

## 10b. Révision des Sources (Constellation Epistemic Content Engine — CECE)

> *(Note de traduction : traduction générée par IA du chapitre V3-§10.F ; relecture par locuteur natif en attente.)*

Deux des propriétés frontmatter les plus importantes — `sources:` et `content_type:` — décrivent *comment vous en êtes venu à savoir* quelque chose et *quel type de connaissance* il s'agit. Le **Epistemic Content Engine** (CECE) de Constellation classifie chaque note selon ces deux axes automatiquement à l'aide d'un ensemble de 6 catalogueurs. Le panneau **Révision des Sources** est l'endroit où vous examinez et corrigez ces classifications.

### Ce que fait le moteur

Lorsque vous classifiez une note (clic droit → « Suggérer sources et type de contenu », ou via Paramètres > Lancer le scan, ou automatiquement via le commutateur de scan en arrière-plan), CECE exécute six catalogueurs indépendants contre la note. Chacun lit la note à travers une lentille différente et vote sur deux questions :

- **Source** (axe horizontal) — d'où *provient* cette connaissance ? Onze valeurs possibles : perception, inférence, témoignage, transmission-massive, comparaison, postulation, non-appréhension, mémoire, disposition-innée, inspiration, révélation. Plus *non classifiable*.
- **Type de contenu** (axe vertical) — quelle *sorte* de connaissance est-ce ? Cinq branches principales : entrées sensorielles, entités symboliques, contenus sémantiques, états épistémiques, constructions d'ordre supérieur.

Les deux axes sont indépendants. Une note « Je doute de l'alunissage » est témoignage (quelqu'un l'a rapporté) sur la source + états-épistémiques/doute (votre posture) sur le type de contenu.

Le moteur s'exécute **sur votre appareil** — aucune note ne quitte jamais Constellation.

### Les six catalogueurs

Chaque catalogueur est une lentille. La carte de Révision des Sources les affiche comme six petits points colorés en haut à droite de chaque carte :

- **Votre frontmatter** (bleu) — adopte ce que vous avez déjà défini, avec autorité absolue
- **Citations et structure** (rose) — citations, blocs de citation, marqueurs de théorème, formules de définition
- **Racines lexicales et lexique** (ambre) — analyse de racines arabes + équivalence terminologique interlingue
- **Notes liées** (sarcelle) — Living Links typés vers d'autres notes classifiées
- **Notes similaires** (violet) — similarité par embeddings avec vos notes déjà classifiées
- **Jugement de l'IA** (vert) — un LLM local (Qwen3-4B ; *pas encore actif*, reporté à une version future)

Un point plein signifie que ce catalogueur s'est exprimé et approuve la synthèse. Un point cerclé signifie qu'il s'est exprimé mais a contesté. Un point au contour pointillé signifie qu'il est resté silencieux (aucun signal dans cette lentille).

### Trois régimes de confiance

Après le vote des catalogueurs, chaque axe atterrit dans l'un des trois régimes :

- **Unanime** — chaque catalogueur s'exprimant était d'accord
- **Forte majorité (une dissidence)** — la plupart étaient d'accord ; un dissident nommé
- **Divisé** — pas de majorité claire ; le moteur refuse de deviner et vous demande de choisir

Chaque axe obtient son propre régime indépendamment — une carte peut être Unanime sur l'horizontal + Divisée sur le vertical, etc.

### Sibling Disambiguation

Lorsqu'un axe est Divisé, le moteur fait apparaître les valeurs candidates sous forme de **chips** sous une invite : *« Choisissez celui qui convient le mieux à la note. »* Cliquez sur un chip → le moteur écrit ce choix dans le frontmatter de la note et retire la carte de la file d'attente. Si l'AUTRE axe était réglé (Unanime ou Forte majorité), le moteur écrit *aussi* la valeur de cet axe en même temps — un clic termine les deux axes lorsqu'un seul était Divisé.

### La piste de raisonnement

Chaque carte a un commutateur *« ▸ Pourquoi cette classification ? »*. En l'étendant, on voit une ligne par catalogueur s'exprimant avec le raisonnement, la confiance auto-déclarée et des chips de règles conviviaux (« Correspondance de mot-clé en surface », « Correspondance de racine arabe (CAE) », « Marqueur de définition », etc.) — ce sont les règles spécifiques que chaque catalogueur a déclenchées.

Pendant vos **50 premières révisions**, la piste se déplie automatiquement sur chaque carte (une *période de calibration de confiance*) afin que vous puissiez développer votre intuition pour savoir quand faire confiance au moteur. Après cela, les pistes se replient à la demande sur les cartes Unanimes. Substituez à tout moment dans **Paramètres > Intelligence > CECE > Visibilité de la piste de raisonnement**.

### Le filtre de composition de la file d'attente

Au-dessus de la barre de comptage, cinq chips découpent la file d'attente selon le type de décision dont chaque carte a besoin :

- **Tout** — la file complète
- **Les deux axes requièrent votre décision** — les deux axes Divisés
- **La source requiert votre décision** — horizontal Divisé + vertical réglé
- **Le type de contenu requiert votre décision** — vertical Divisé + horizontal réglé
- **Catalogueurs en accord** — aucun axe Divisé (candidats à tampon automatique)

Chaque chip affiche son décompte. Le filtre est un découpeur de couche de rendu — la mathématique de Tout Accepter opère toujours sur la file complète, quel que soit le filtre actif.

### Actions par carte

- **Accepter** — écrit la synthèse du moteur en primaire sur les deux axes ; retire la carte. Met à jour la fiabilité par catalogueur.
- **Modifier** — ouvre un sélecteur arborescent pour les deux axes ; choisissez manuellement. Même mise à jour de fiabilité.
- **Rejeter** — efface la carte sans écrire.
- **Chip Sibling Disambiguation** — uniquement sur les cartes Divisées.

### Calibration par Bibliothèque

**Paramètres > Intelligence > CECE > Calibration par Bibliothèque** ouvre un tableau en lecture seule indiquant la précision de chaque catalogueur par axe sur la Bibliothèque active. Différentes Bibliothèques ont des précisions par catalogueur différentes — Linguistique excelle sur les Bibliothèques majoritairement arabes, Graphe excelle sur celles densément liées. La couche de synthèse utilise ces données de calibration pour pondérer les votes.

Un catalogueur a besoin de **20 corrections** avant que son ratio de précision soit affiché. En dessous de ce seuil, l'étiquette indique *« (uniforme) »* — le catalogueur contribue avec des votes pondérés uniformément jusqu'à accumulation suffisante de données.

### Classification en arrière-plan

Par défaut, CECE classifie les notes uniquement lorsque vous le lui demandez (clic droit ou bouton de scan dans Paramètres). Vous pouvez activer la classification automatique dans **Paramètres > Intelligence > CECE > Classification en arrière-plan** :

- **À l'enregistrement de la note** — classifie chaque note ~1,5 seconde après que vous arrêtez de taper (chevauche l'enregistrement debounced existant ; ne se déclenche jamais à chaque frappe ; la frappe reste instantanée)
- **Au démarrage de l'application** — scanne les notes non classifiées une fois par lancement

### Le Classificateur — l'accueil pleine page
Les mêmes cartes vivent aussi dans une vue pleine page appelée **le Classificateur**, ouverte depuis l'**icône en cartes empilées dans le dock à gauche**. C'est le même moteur et la même file d'attente, dotés de toute la fenêtre au lieu d'un onglet étroit du sidebar — et il ajoute deux contrôles que l'onglet du sidebar n'a jamais eus :
- **Classer une note…** — une boîte de recherche qui vous permet de classer *n'importe quelle* note par son nom, sans l'ouvrir d'abord. Tapez quelques lettres, choisissez la note, et une carte fraîche apparaît dans la file d'attente.
- **Générer tous les résumés** — pré-calcule le résumé de note (voir ci-dessous) pour chaque note qui n'en a pas, en arrière-plan, avec la progression dans la barre d'état.

Un bouton **Démarrer l'analyse** (le même scan à l'échelle de l'univers que dans Paramètres) et une barre de progression en direct complètent l'en-tête. Fermez le Classificateur avec le **(×)** ou **Esc**. (Lorsque la boîte de recherche *Classer une note…* est ouverte, le premier **Esc** ferme uniquement cette boîte.)
Une note sur les noms : **le Classificateur** est la *pièce* (la vue pleine page) ; **les catalogueurs** sont les *six lentilles* à l'intérieur du moteur qui votent sur chaque carte. Ne confondez pas les deux.

### Résumés de notes
Sous le titre de chaque carte se trouve un court **Résumé** — quelques phrases qui vous disent de quoi parle la note, pour que vous puissiez la classer sans l'ouvrir. Constellation préfère toujours un résumé que *vous* avez écrit et n'en génère un que lorsque vous ne l'avez pas fait :
1. Un **champ de frontmatter** `summary:` / `description:` / `abstract:` / `excerpt:`, utilisé tel quel.
2. Un **callout** `> [!summary]` / `[!abstract]` / `[!tldr]` dans le corps, utilisé tel quel.
3. Sinon, un résumé **généré** — les trois phrases les plus centrales de la note, extraites (jamais inventées) et affichées dans leur ordre d'origine.

Les résumés générés sont en **lecture seule** — Constellation n'en réécrit jamais un dans votre note (File-Over-App), et tout est calculé **sur votre appareil**. Si vous voulez qu'un résumé vive dans le fichier, écrivez-en un vous-même et Constellation affichera le vôtre à la place.

Pour plus de détails (chaque statut de point, chaque chip de règle, parcours clic par clic des scénarios courants), consultez les sujets **Révision des Sources**, **The Cataloger** (le Classificateur) et **Note Summaries** dans le système d'aide.

---

## 10c. Métadonnées Épistémiques

Un petit ensemble de champs frontmatter optionnels pour enregistrer des informations plus riches sur la manière dont la connaissance d'une note a été acquise, qui détient la position, à quelle discipline elle appartient, et quand vous avez révisé pour la dernière fois votre vision. Ajouté dans MIG-022 §A en réponse à l'analyse des lacunes (`docs/epistemic-content-gap-analysis.md`).

Ces champs sont **tous optionnels**. Les notes sans eux fonctionnent sans changement.

### Référence rapide

| Field | Type | Purpose |
|---|---|---|
| `held_by` | text | De qui est cette position ? (par défaut `user` ; peut être `"al-Shāfiʿī"`, `"Ḥanafī"`, etc.) |
| `domain` | list | Étiquettes disciplinaires pour la récupération (`[fiqh, ʿibādāt]`) |
| `function` | text | À quoi sert cette note (`reference` / `seed` / `actionable` / `shipped`) |
| `provenance_civilization` | text | Vocabulaire traditionnel (`sunni-usuli` / `analytic-western` / `nyaya` / etc.) |
| `updated_at` | date | Quand vous avez délibérément révisé votre vision pour la dernière fois (distinct du mtime du système de fichiers) |
| `ikhtilāf` | list of objects | Désaccord savant structuré (`[{school, position}, ...]`) |
| `warrant` | text | Étiquette de degré (parsée mais inerte jusqu'à ce que le Warrant Research workstream soit livré) |
| `warrant_notes` | text | Texte libre étayant le degré de garantie (également inerte) |

### Comment ils apparaissent dans le panneau Propriétés

Chaque champ est rendu avec l'éditeur approprié au type :
- Champs texte → entrée texte
- `domain` → liste d'étiquettes (Entrée pour ajouter, × pour supprimer)
- `updated_at` → sélecteur de date
- **`ikhtilāf` → widget personnalisé** avec deux entrées côte à côte par ligne (school + position) plus un bouton supprimer par ligne, et un bouton « Ajouter une école » en bas. Le widget lit depuis et écrit vers le YAML structuré, de sorte que les allers-retours préservent chaque champ.

### Et `supersedes` ?

`supersedes` est une *relation entre notes* (cette note remplace une antérieure), non une propriété d'une note unique. Constellation le gère comme un **lien typé**, non comme un scalaire YAML :

```markdown
Ceci remplace mon analyse antérieure : [[old-note-id|supersedes]]
```

Le suffixe `|supersedes` sur le wikilink en fait un lien typé de la sorte `supersedes` — pastille bleu-gris ardoise distincte, apparaît dans les panneaux Backlinks + Outgoing Links, participe à la Living Link Architecture.

### Ce que ce n'est PAS

Les nouveaux champs sont du **schéma** — un vocabulaire reconnu que vous pouvez remplir. CECE ne les consomme pas actuellement pour la classification. De futurs MIGs (Warrant Research workstream, MIG-023 axe temporel) livreront des fonctionnalités qui lisent `warrant`, `updated_at` et compagnie.

Pour plus de détails + un exemple détaillé, consultez le sujet **Métadonnées Épistémiques** dans le système d'aide.

---

## 11. Modeles

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

## 12. Tableaux

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

## 13. Taches

Constellation prend en charge les cases a cocher de taches dans les notes :

```markdown
- [ ] Tache incomplete
- [x] Tache terminee
```

En mode Apercu en direct, les cases a cocher sont cliquables. Les taches peuvent etre recherchees et filtrees dans toutes vos bibliotheques.

---

## 14. Importateur

Importez des notes depuis d'autres outils PKM :

- **Obsidian** — importe les vaults avec une compatibilite wikilink complete
- **Dossiers Markdown** — importez n'importe quel dossier de fichiers `.md`
- **Autres formats** — HTML, fichiers texte

Allez dans **Parametres > Importateur** pour lancer un import.

---

## 15. Calendrier

Le **Calendrier** est une vue mensuelle en plein écran, ouverte depuis le **dock de gauche** (l'icône du calendrier). Les jours comportant des notes ou des tâches à échéance sont signalés par des **points** colorés. L'en-tête affiche le mois dans le calendrier que vous avez choisi ; si vous avez défini un **calendrier secondaire**, un sous-titre en dessous indique la plage équivalente dans ce calendrier (par exemple, un mois grégorien affiche son intervalle hégirien, « Dhū al-Ḥijja 1447 – Muḥarram 1448 AH »).

**Cliquer sur un jour.** Chaque cellule de jour est interactive :

- **Cliquez sur l'espace vide (ou sur le numéro du jour)** → ouvre (ou crée) la **note quotidienne** de ce jour. Cliquer sur une date qui possède déjà une note quotidienne se contente de l'**ouvrir** — jamais de créer un doublon.
- **Cliquez sur un point** → ouvre l'élément correspondant. Un point **doré** est la note quotidienne ; un point **violet** est une autre note modifiée ce jour-là ; un point **rouge** est une tâche à échéance ce jour-là. (Les couleurs sont personnalisables dans le Style Setter → Calendrier.) Si un jour comporte plusieurs notes ou tâches, cliquer sur le point affiche une petite **liste** dans laquelle choisir.
- **Cliquez sur un point de tâche** → ouvre la note **positionnée sur la ligne de cette tâche**, prête à être modifiée. Dans la liste des tâches, vous pouvez aussi **cocher la case d'une tâche pour la terminer** directement depuis le calendrier — les tâches terminées disparaissent immédiatement. Seules les tâches qui portent leur propre `📅 YYYY-MM-DD` apparaissent dans le calendrier (c'est la date qui les place sur un jour).

**Calendriers culturels (huit).** Dans **Paramètres → Calendrier**, vous pouvez définir le **système de calendrier** — **grégorien, hégirien (islamique), hégirien solaire (persan), hébraïque, indien (Saka), bouddhiste, chinois ou coréen** — et toute la grille du mois bascule vers celui-ci, affichant à la fois la date du calendrier choisi (en grand) et la date grégorienne (en petit) dans chaque cellule, ainsi que la phase de la lune. Chaque en-tête de mois affiche le **nom** du mois, son **numéro entre parenthèses** et l'**année** — le numéro aide pour les calendriers dont l'ordre des mois est peu familier. Les calendriers **chinois et coréen** sont *luni-solaires* : ils insèrent parfois un **mois intercalaire** (闰六月 / 윤6월), que le calendrier affiche comme sa propre page afin que la navigation ne saute ni ne double jamais ce mois. Le calendrier hégirien utilise un moteur astronomique précis ; les mois sacrés sont mis en évidence et les événements islamiques sont marqués. Vous pouvez également choisir le **début de semaine** (dimanche/lundi) et activer ou désactiver la **colonne du numéro de semaine**.

**Options du calendrier hégirien.** Sous **Paramètres → Calendrier → « Calendrier hégirien (islamique) »**, deux réglages supplémentaires sont disponibles :

- **Méthode de calcul** — **Astronomique (conjonction lunaire)**, qui suit la véritable nouvelle lune (la plus précise, par défaut), ou **Tabulaire (al-Tawfīqāt al-Ilhāmiyyah** — les concordances inspirées**)**, le cycle arithmétique classique.
- **Correction de mois** — décalez le début d'un mois hégirien de ±1 ou ±2 jours pour le faire coïncider avec une **observation locale de la lune**. Choisissez l'année et le mois hégiriens, sélectionnez un décalage, puis cliquez sur **Définir** ; la correction s'applique à ce mois et à tous les mois suivants. Vos corrections sont répertoriées (chacune pouvant être retirée), avec un bouton **Tout effacer**.

Les deux réglages (ainsi que vos corrections) sont enregistrés **avec votre univers**, et voyagent donc d'un appareil à l'autre.

**Options d'affichage chinois et coréen.** La Corée utilise le calendrier lunaire chinois, de sorte que les deux partagent des dates identiques — ce qui les distingue, c'est l'écriture et l'année. Lorsque l'un ou l'autre est votre calendrier principal ou secondaire, **Paramètres → Calendrier** affiche deux réglages supplémentaires : un **affichage de l'année** (chinois : le cycle sexagésimal 丙午年, l'année simple, ou les deux ; coréen : l'ère **Dangi** 단기 4359, l'année, ou le sexagésimal 병오년) et les **noms des mois** — *écriture native* (五月 / 5월) ou *phonétique*, la prononciation du mois écrite dans votre propre langue (anglais « Wǔyuè / Owol » ; arabe « وُو-يوي / أوه-وُل »).

**Personnaliser le calendrier.** Ouvrez le **Style Setter** (dock de gauche, ou **Paramètres → Style Setter**) et choisissez la surface **Calendrier** pour restyler chaque élément — chacun possède sa propre **couleur et taille de texte** (numéros des jours, la date de renvoi, la pastille du mois, les en-têtes des jours de la semaine, les numéros de semaine, le glyphe lunaire, la mise en évidence d'Aujourd'hui, les lignes de la grille, et les points de note/tâche/événement), ainsi que la **police** du calendrier. Un aperçu en direct et en pleine taille se met à jour à mesure que vous modifiez ; cliquez sur **Conserver** pour appliquer.

> **Les noms de fichiers des notes quotidiennes restent toujours grégoriens** (`YYYY-MM-DD`) quel que soit le calendrier affiché — ainsi vos fichiers restent portables et se trient correctement. La date culturelle est affichée dans le calendrier (et peut être enregistrée dans le frontmatter de la note).

Le Calendrier sert pleinement les notes quotidiennes : cliquez sur n'importe quel jour pour l'ouvrir, ou exécutez la commande **« Note quotidienne »** (palette de commandes) pour sauter à aujourd'hui.

**Enregistrer une date culturelle dans une note.** Deux outils optionnels écrivent la date culturelle dans les **propriétés** d'une note (le nom de fichier reste toujours grégorien `YYYY-MM-DD`) :

- **Tampon hégirien des notes quotidiennes** — *Paramètres → Calendrier → « Tamponner la date hégirienne dans les notes quotidiennes. »* Lorsqu'il est activé (disponible uniquement tant que le calendrier hégirien est votre calendrier **principal ou secondaire**), chaque **nouvelle** note quotidienne reçoit une ligne `hijri:`, par exemple `hijri: 1448-01-06`. Les notes que vous possédez déjà ne sont jamais touchées.
- **« + Hijri » dans les Propriétés d'une note** — ouvrez les **Propriétés** de n'importe quelle note, survolez la date, et un petit bouton **« + Hijri »** apparaît (en plus de « + Jalali », « + Hébraïque », etc. — **un bouton par calendrier non grégorien que vous avez sélectionné**). Cliquez dessus et Constellation lit la date grégorienne de la note et ajoute l'équivalent, par exemple `jalali: 1405-03-30`. Le bouton coréen écrit l'année **Dangi** ; un **mois intercalaire** chinois/coréen est signalé par un `L` (par exemple `chinese: 2025-06L-17`). Si la note n'a pas de propriété de date, c'est la date de création du fichier qui est utilisée.

---

## 16. Lens et Constellation Base

Une **Lens** est une requete enregistree qui affiche une liste filtree et triee de notes avec les proprietes qui vous interessent. Constellation offre deux modes :

### La Base en onglet entier

Ouvrez un fichier `.base` et il remplit l'onglet sous forme de tableau interactif : une ligne par note, une colonne par propriété, sans limite de lignes (le tableau est virtualisé, donc des milliers de notes défilent sans à-coups). Trois façons de retrouver une note dans un grand tableau :

- **Rechercher dans cette base** — le champ de recherche de l'en-tête filtre les lignes au fur et à mesure de votre saisie, en correspondant au nom d'une note *et* au texte de chaque colonne visible. Le badge de comptage affiche `correspondances / total` pendant que vous filtrez (par exemple `4/7684`). Il cherche dans toutes les écritures — tapez en arabe pour trouver des titres arabes. Le filtrage est instantané, même sur des milliers de lignes.
- **Barre alphabétique** — sur une base de 50 lignes ou plus, une fine bande de lettres apparaît le long du bord du tableau, construite à partir des premières lettres des titres réels de vos notes (elle affiche donc A–Z pour l'anglais, أ ب ت… pour l'arabe, et les lettres appropriées pour toute autre écriture). Cliquez sur une lettre pour sauter directement à la première note qui commence par elle — si le tableau n'est pas déjà trié par Nom, il se trie d'abord par Nom, puis saute.
- **Clic droit sur une ligne** — ouvre le menu de note standard : Ouvrir, Ouvrir dans un nouvel onglet, Marque-page, Copier le chemin / le nom, Révéler dans l'arborescence des fichiers, Ouvrir dans l'application par défaut, Afficher dans l'explorateur système, Style… (Renommer, déplacer et supprimer ne sont délibérément pas proposés ici — faites-le depuis l'arborescence des fichiers.)

(Voir le sujet d'aide **Bases** dans l'application pour la présentation complète.)

### Constellation Base — blocs Lens integres

Vous pouvez integrer une Lens directement dans le corps de toute note Markdown via un bloc de code ` ```base ` :

````markdown
```base
schema: 1
view: list
dimensions: [note.name, note.created_at]
sort: [note.created_at, desc]
limit: 20
```
````

En affichage de la note, le bloc de code est remplace par un tableau interactif des notes correspondantes. En apercu en direct, cliquez sur la pastille **Lens** pour modifier le bloc.

**Dimensions disponibles en v1 :** `note.name`, `note.path`, `note.created_at`, `note.headline`.

**Federation :** par defaut, les blocs Lens lisent dans l'univers actif ET chaque cUnivers lie. Definissez `federation: active` dans le YAML pour limiter a l'univers actif.

### Five Acts — Lenses integrees

La section **Five Acts** de la barre laterale (au-dessus de Workspace Bases) liste les notes hotes preconfigurees par Constellation dans `{universe}/Five Acts/*.md`. v1 inclut une : **Observation — Recent Captures** (liste federee des 20 notes les plus recentes). Vous pouvez editer librement ces notes — Constellation ne reecrira pas vos modifications.

### Panneau Lens classique

L'ancien panneau Lens (filtrer par tags, dossiers, proprietes) reste disponible dans **Parametres → Panneaux → Lens**.

---

### Structure (liens structurels)

Le panneau **Structure** montre où la note ouverte se situe à l'intérieur d'une *œuvre* plus vaste — un livre, un scénario, un cours, une Carte de Contenu. Il répond à une question différente de celle des panneaux Rétroliens (Backlinks) et Liens sortants. Ceux-ci répondent à *« comment cette idée se rattache-t-elle à une autre idée ? »* (les liens de pensée — appuie, contredit, cause…). Structure répond à *« où cette note se situe-t-elle dans l'ensemble de l'œuvre que je compose ? »* — Livre → Partie → Chapitre → Scène.

C'est la **colonne vertébrale compositionnelle** d'une œuvre : la table des matières, le plan ordonné. Elle est délibérément tenue **à l'écart** de toute mesure de pensée, de maturité et de connexion — placer une note « sous un Livre » ne change jamais la maturité de cette note, ses décomptes de connexions ni sa présence dans la Vue Étoiles. Une table des matières relève de l'écriture d'auteur, non d'une affirmation à juger.

**Les deux sortes de lien structurel** (vous ne saisissez jamais qu'un seul côté — Constellation déduit l'inverse pour vous) :

- **`parent`** — la place de *cette note* sous un parent unique (par exemple, un chapitre déclare la partie à laquelle il appartient).
- **`contains`** — la liste ordonnée des enfants de *cette note* (par exemple, un livre énumère ses parties dans l'ordre de lecture).

**Créer un lien structurel** — ouvrez les **Propriétés** de la note (l'onglet Propriétés dans la barre latérale de droite, ou le bloc de propriétés en haut de la note) :

1. Cliquez sur **+ Ajouter une propriété** et saisissez la clé `parent` ou `contains`.
2. Dans la valeur, saisissez le **nom de la note cible** — juste le nom, par exemple `Part I - The Cartographer`. Constellation l'enveloppe dans un `[[link]]` pour vous ; vous ne saisissez **pas** les crochets. (Si vous collez un nom qui comporte déjà des crochets, il est tout de même stocké proprement sous la forme d'un unique `[[name]]` — jamais un double encadrement.)
3. Pour `contains`, ajoutez chaque enfant comme sa propre puce, dans l'ordre où vous voulez qu'ils se lisent — cet ordre devient l'ordre du plan.

Les liens structurels **se renomment en toute sécurité** : renommez un chapitre et sa place dans la structure suit automatiquement, car le lien pointe vers la note, et non vers un morceau de texte figé.

**Lire le panneau Structure** — ouvrez l'onglet **Structure** dans la barre latérale de droite (juste après Rétroliens) :

- Le panneau affiche l'**œuvre entière** sous forme de plan indenté (puces sarcelle), coiffé du titre **OUTLINE** (Plan) avec un décompte des descendants — pas seulement les propres enfants de la note ouverte.
- La note que vous consultez actuellement est **mise en évidence** (« vous êtes ici ») au sein de ce plan.
- Un **fil d'Ariane** en haut affiche le chemin remontant la colonne vertébrale (par exemple *The Atlas of Lost Places › Part I › Chapter 1*). Cliquez sur n'importe quel maillon — ou n'importe quelle ligne du plan — pour sauter à cette note.
- Une bascule **Whole work ⇄ This note** (Œuvre entière ⇄ Cette note), en haut à droite du panneau, alterne entre l'œuvre entière et la seule sous-arborescence de la note ouverte. Elle n'apparaît que lorsque la note possède réellement un parent, de sorte que les deux vues diffèrent.
- Si la structure boucle accidentellement sur elle-même (le parent de la note A est B, et le parent de B est A), le plan dessine la chaîne puis s'arrête proprement, marquant le point de coupure par un petit **↻**. Il ne se bloque jamais.

**Résoudre un conflit (Contested).** Si deux notes revendiquent le même enfant — l'une via le propre `parent` de l'enfant, l'autre via une liste `contains` — le panneau signale cette ligne comme **Contested** (En litige) (un badge ambre ⚠ nommant l'autre revendicateur) plutôt que de l'abandonner silencieusement. Deux boutons à un clic le résolvent :

- **Keep** (Conserver) — conserver le parent déclaré par l'enfant lui-même (cette note renonce à sa revendication sur l'enfant).
- **Move here** (Déplacer ici) — accepter cette note comme parent (le `parent` de l'enfant bascule vers cette note).

L'un ou l'autre bouton met à jour les fichiers de notes directement et rafraîchit le plan. Rien n'est jamais modifié sans votre clic.

---

## 17. Parametres

Accedez aux Parametres depuis l'icone d'engrenage dans la barre laterale ou `Ctrl+,`.

### General

- Langue (15 langues)
- Theme (Clair / Sombre)
- Police d'interface, Police de texte, Police monospace, Taille de police
- Theme de police — combinaisons de polices predefinies (Machine a ecrire, Classique, Moderne, etc.) pour un changement rapide
- **Themes** — choisissez parmi six themes integres, creez des themes personnalises (editeur de cinq couleurs), importez des themes depuis le registre communautaire d'Obsidian (200+ themes), ou importez un fichier `.json`. Supprimez n'importe quel theme personnalise avec le bouton ✕ au survol.

### Style Settings

Un onglet dedie pour la personnalisation fine de chaque element visible de l'interface, applique en direct au theme actif.

- **Couleurs** — fond, surfaces, texte (normal/attenue/faible), accent, bordures, couleurs d'etat
- **Typographie** — tailles de police interface/note/code, tailles H1–H6, graisse des titres, hauteurs de ligne, espacement des paragraphes
- **Mise en page et forme** — rayons de coin petit/moyen/grand, largeurs de bordure, ombres, longueur de ligne lisible de l'editeur, marges laterales
- **Composants** — dock ruban, barre d'actions laterale, barre de mise en page (bascules de panneaux), barre superieure/bande d'onglets, barre d'etat, barre laterale droite (inspecteur), explorateur de fichiers (notes d'Univers, univers enfants, bibliotheques, dossiers, notes), boutons, etiquettes, callouts — chacun avec taille, rayon, couleur independants, et style d'etat actif le cas echeant
- **Editeur** — couleur/survol/decoration du lien, couleur/fond/rayon du code en ligne, largeur/couleur de la barre de citation, couleur du curseur, fond de selection

**Importer / Exporter** — barre d'outils en haut de l'onglet :
- Coller depuis le presse-papiers (un clic)
- Importer / Coller (zone de texte avec Fusionner ou Remplacer)
- Depuis un fichier (.json)
- Copier (valeurs actuelles dans le presse-papiers)
- Exporter (.json)

Le format correspond exactement au plugin Style Settings d'Obsidian, vous pouvez donc partager des reglages entre Obsidian et Constellation.

Les modifications sont enregistrees automatiquement dans le theme actif ; si vous modifiez un theme integre, il est automatiquement clone dans vos themes personnalises pour que les changements persistent sans modifier l'original.

### Le Configurateur de style

Le **Configurateur de style** (Style Setter) est un studio de conception plein ecran — ouvrez-le depuis **Parametres → Apparence → "✦ Open Style Setter."** Il affiche votre interface reelle au centre ; cliquez sur n'importe quelle partie (barre laterale, titre de la note, titre, lien, la page de la note) et les commandes de cet element apparaissent a droite, l'apercu se mettant a jour instantanement. Les cartes de theme (Midnight / Daylight / Chocolate / Nord) amorcent tout un rendu — le studio lui-meme le porte pendant que vous concevez — et la liste des *Surfaces* previsualise le rendu sur toute l'application, pas seulement l'editeur. **"Apply to app"** applique votre accent, vos fonds, la couleur du texte et les polices au veritable Constellation ; **Esc** ou **✕** ne ferme que le Configurateur, pas les Parametres. Pour l'instant, appliquer est un apercu en direct de la session — enregistrer un rendu comme un Style permanent et nomme (avec des echantillons de couleur reutilisables et renommables, ainsi que l'export / import) arrive prochainement.

### Surcharges du moteur arabe

Un panneau par Univers ou vous fixez la facon dont le moteur arabe analyse certaines formes de surface — vos propres neologismes, des noms locaux, des emprunts specifiques a un domaine, ou les cas ou vous etes en desaccord avec la lecture automatique du moteur. Chaque surcharge l'emporte sur le FST generatif, la cascade et le repli heuristique. Ajouter ou retirer une surcharge declenche une reindexation ciblee sur les seules notes qui contiennent la forme de surface concernee — pas de reconstruction complete. Voir le chapitre 19 (« Prise en charge RTL et arabe ») pour la procedure pas a pas.

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

## 18. Raccourcis clavier

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

## 19. Prise en charge RTL et arabe

Constellation offre une prise en charge de premier ordre pour l'arabe, l'hebreu, le persan, l'ourdou et les autres ecritures RTL :

- **Detection automatique** : La direction de la note est detectee automatiquement a partir du contenu
- **Interface** : Interface RTL complete lorsque la langue arabe/hebraique est selectionnee
- **Editeur** : Edition de texte RTL avec **comportement du curseur à la Word** (voir ci-dessous)
- **Vue Etoiles** : Les etiquettes arabes s'affichent de droite a gauche avec un repli de police adequat
- **Legende** : Les elements inversent l'ordre point/texte selon la langue du contenu
- **Polices de script** : Configurez les polices arabes, hebraiques et CJK independamment dans les Parametres

### Comportement du curseur et des flèches en texte arabe et bilingue

L'éditeur de Constellation suit la même logique que Microsoft Word sous Windows, si bien que vos automatismes restent valables :

- **Les flèches déplacent le curseur d'un caractère du *texte*, dans l'ordre de lecture** — pas d'une position à l'écran. En arabe pur ou en anglais pur, le résultat correspond exactement à la flèche pressée. À une couture entre l'arabe et l'anglais (par ex. une phrase arabe contenant un mot anglais), le curseur parcourt proprement chaque caractère dans l'ordre d'écriture et « saute » la couture — ce saut est correct, et c'est lui qui empêche le curseur de sembler coincé à la frontière.
- **Home** va au *début* de lecture de la ligne — le bord **droit** sur une ligne arabe ; **End** va à la *fin* de lecture — le bord **gauche**. Appuyer sur **Enter** sur une ligne arabe place le curseur de la nouvelle ligne à **droite**.
- **Le triple-clic** sélectionne le **texte** du paragraphe (pas le vide à côté). **Le double-clic** sélectionne un mot.
- Un **mot latin en fin de ligne arabe** conserve une position de curseur nette et stable au lieu de perdre sa direction.

### Sélectionner et naviguer par unité

Chaque unité de texte a son sélecteur rapide, identique dans les notes en arabe, en anglais ou mixtes :

- **Mot** — double-clic. **Phrase** — **Ctrl+clic** n'importe où dedans, ou **Ctrl+Shift+S** avec le curseur à l'intérieur. La détection de phrase comprend la ponctuation arabe : **؟ ۔ !** et le point terminent une phrase, tandis que le point-virgule arabe **؛** est une pause *à l'intérieur* d'une phrase — et les décimaux comme 3.14 ne coupent jamais. (Ctrl+clic remplace l'ancien geste d'ajout de curseur.)
- **Ligne** — **Ctrl+L**. **Paragraphe** (le bloc entre lignes vides) — **Ctrl+Shift+L**, ou triple-clic. Le surlignage épouse le texte — sur une ligne arabe, la sélection s'arrête aux mots au lieu de s'étirer sur le vide à gauche.
- **Écran entier** — **Shift+Page Down/Up**. **Tout** — **Ctrl+A**.
- **Se déplacer par paragraphe** — **Ctrl+↓** saute au début du paragraphe suivant, **Ctrl+↑** au début du paragraphe courant (de nouveau pour le précédent). Ajoutez **Shift** pour sélectionner paragraphe par paragraphe.

### Forcer la direction d'un paragraphe

Parfois la détection automatique n'est pas ce que vous voulez — un paragraphe arabe qui s'ouvre sur un nom de marque anglais, ou un paragraphe anglais que vous voulez lire de droite à gauche :

- **Appuyez puis relâchez Ctrl+Shift du côté droit du clavier** → le paragraphe où se trouve le curseur devient **100 % droite-à-gauche**. **Ctrl+Shift du côté gauche** → **100 % gauche-à-droite**. (La convention de Microsoft Word.)
- Le basculement se déclenche **au relâchement**, sans autre touche entre-temps — Ctrl+Shift+S et les autres raccourcis continuent donc de fonctionner sans changement.
- Le forçage est **absolu** (il l'emporte sur la détection automatique), s'applique au paragraphe entier ou à chaque paragraphe couvert par une sélection, et est enregistré **dans le texte lui-même** sous forme de caractère de direction invisible — il survit aux redémarrages et à la synchronisation, et voyage avec le texte dans Word ou Obsidian.
- Un seul **Ctrl+Z** l'annule. Le Markdown reste intact : les listes, titres et citations gardent leurs marqueurs ; les blocs de code, les tableaux et les lignes qui *commencent* par un #tag sont volontairement laissés tels quels.

### Configuration pour l'arabe

1. Allez dans **Parametres > General > Langue** et selectionnez Arabe
2. Optionnellement, definissez une police arabe dediee dans **Parametres > General > Polices de script**
3. Les notes avec du contenu arabe s'afficheront automatiquement en RTL

### Surcharges du moteur arabe

Le moteur arabe de Constellation est un analyseur morphologique a cinq couches qui tourne sous chaque recherche, chaque lien et chaque entree d'index. Il comprend racines, schemes, noms propres, emprunts et reparations phonologiques — de sorte qu'une requete pour كاتب trouve aussi كتبنا et كتاب, mais que وائل reste intact comme nom propre au lieu d'etre mutile en ائل.

Le panneau **Surcharges arabes** dans les Parametres est l'endroit ou vous enseignez votre propre terminologie au moteur. Chaque surcharge est la reponse souveraine — elle l'emporte sur le FST generatif, la cascade et le repli heuristique.

**Quand utiliser les surcharges :**
- Noms de personnes, toponymes locaux ou termes specifiques a votre domaine que le moteur ne connait pas
- Neologismes ou acronymes propres a votre Univers
- Emprunts dont vous voulez preserver une orthographe particuliere
- Tout cas ou l'analyse automatique du moteur contredit votre facon de lire le mot

**Pas a pas :**

1. Ouvrez les **Parametres** (icone d'engrenage ou `Ctrl + ,` / `Cmd + ,`) et selectionnez **Surcharges arabes** dans la barre laterale.
2. Cliquez sur **Ajouter une surcharge**.
3. Remplissez :
   - **Forme de surface** — le mot arabe tel que vous le tapez
   - **Lemme** — la forme canonique que le moteur doit renvoyer
   - **Racine** (optionnelle) — 3 ou 4 consonnes si le mot a une racine classique
   - **Scheme** (optionnel) — par ex. `فاعل`
   - **Categorie** — Nom propre / Nom / Adjectif / Adverbe / Verbe / Particule / Etranger / Inconnu
   - **Note** (optionnelle) — une ligne de contexte pour vous-meme
4. Cliquez sur **Enregistrer**. Le panneau affiche **Reindexation…** pendant que chaque note contenant la forme de surface est retokenisee, puis **N note(s) reindexee(s)** une fois termine.
5. Pour retirer une surcharge, cliquez sur le **x** de sa ligne — le meme balayage de reindexation s'execute en sens inverse.

Les surcharges sont stockees par Univers dans `<univers>/.constellation/arabic-overrides.json` — texte brut, trie alphabetiquement, ecrit de maniere atomique. Vous pouvez mettre le fichier sous controle de version ou le partager entre appareils.

---

## 20. Securite et confidentialite

- **Toutes les donnees restent locales** — pas de synchronisation cloud, pas de telemetrie, pas de suivi
- **Fichiers Markdown** — vos notes sont des fichiers texte brut qui vous appartiennent entierement
- **Aucun compte requis** — Constellation fonctionne entierement hors ligne
- **Mises a jour optionnelles** — verifiez les mises a jour manuellement via les Parametres
- **Open source** — consultez le code sur [github.com/eisaShamsi/Constellation](https://github.com/eisaShamsi/Constellation)

---

## 21. Carte des connaissances

La Carte des connaissances est une visualisation radiale (sunburst) qui montre la structure, la densite et la maturite de votre univers de connaissances.

### Ouvrir la Carte

- **Bouton du dock**: Cliquez sur l'icone Carte des connaissances dans la barre laterale gauche
- **Palette de commandes**: `Ctrl+P` puis tapez "Constellation Map"

### Ce que vous voyez

- **Centre**: Le nom de votre Univers avec le nombre total de notes et de mots
- **Premier anneau**: Les bibliotheques (chacune avec sa couleur). Si votre univers a des univers enfants, ils apparaissent ici aussi.
- **Anneaux plus profonds**: Dossiers et sous-dossiers dans chaque bibliotheque
- **Segments exterieurs**: Notes individuelles

### Modes de couleur

Basculez entre trois modes via le menu deroulant:
- **Maturite**: graine (gris) → jeune pousse (vert clair) → persistant (vert) → canonique (or) → fane
- **Strate**: L1 (bleu) → L8 (rouge) — montre la complexite des connaissances
- **Bibliotheque**: tous les segments heritent de la couleur de leur bibliotheque

### Navigation par exploration

Cliquez sur un segment de dossier pour zoomer. Un fil d'Ariane montre votre chemin. Cliquez sur un element du fil pour revenir, ou appuyez sur Echap. Cliquez sur un segment de note pour l'ouvrir dans l'editeur.

### Retour a la Carte

Apres avoir ouvert une note depuis la Carte, un bouton "Retour a la Carte" apparait dans la barre d'onglets. Cliquez pour revenir exactement ou vous etiez — meme niveau d'exploration preserve.

---

## 22. Moteur Cognitif

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

*Manuel d'utilisation de Constellation — Version 0.1.0 — Mars 2026*
*uconstellation.world*

---

## 23. Connexions suggérées

Constellation sert à *formuler* la connaissance, et la connaissance est connexion. Les **Connexions suggérées** trouvent, parmi les notes déjà présentes dans votre Bibliothèque, celles qui sont les plus apparentées à celle que vous regardez — les proches auxquels elle devrait être reliée mais ne l'est pas encore — et transforment chacune d'elles en un **lien typé** d'un seul clic. C'est un « plus de notes comme celle-ci », mais pour la pensée.

**Chaque suggestion est typée.** Lorsque vous en acceptez une, Constellation demande *comment* les deux notes sont liées — soutient, contredit, illustre, dérivé-de, et ainsi de suite, ou simplement **associatif**. Un lien typé est un fragment de raisonnement que vous pourrez plus tard relire, rechercher et remettre en question ; la fonction n'ajoute jamais de liens en masse et n'ajoute jamais un lien sans type en silence. (Voir **Formulation des connaissances** et **Propriétés**.)

**Comment elles les trouvent.** Les candidats proviennent **uniquement de votre propre Bibliothèque**, classés au moyen de l'index de recherche en temps réel de Constellation selon le vocabulaire partagé le plus *distinctif* — les mots rares et révélateurs, pas les mots courants. Chaque suggestion affiche les **termes en commun** qui expliquent pourquoi elle est apparue, de sorte que vous n'acceptez jamais une supposition opaque.

**Cinq endroits, une seule liste.** La même liste de suggestions apparaît dans le **Réviseur** (🕐, pour les notes qu'il signale comme *orphelines* ou *fragiles*), l'**onglet Rétroliens** (barre latérale droite), l'**Inspecteur 360°**, l'**onglet Santé** et la **Vue Étoiles** (🌌 — clic droit sur une étoile → **Suggérer des connexions…**).

**Entrants ou sortants — et pourquoi vous n'avez pas à choisir.** Les surfaces de diagnostic (l'**Inspecteur 360°** et l'**onglet Santé**) suggèrent des connexions **entrantes** — *quelles notes devraient pointer **ici***. Les surfaces générales (l'**onglet Rétroliens** et la **Vue Étoiles**) suggèrent des connexions **sortantes** — *vers quoi cette note devrait pointer*. La surface choisit la direction qui convient à son rôle ; vous choisissez la note et le type. (Une future mise à jour vous permettra de changer la direction vous-même.)

**Comment l'utiliser.** Sous le titre **Connexions suggérées**, vous verrez les notes apparentées classées de la plus proche à la plus éloignée, chacune avec ses termes en commun. Cliquez sur le bouton **Relier** d'un candidat → dans le petit menu **« Comment sont-ils liés ? »**, choisissez le type de relation → le lien typé est créé **instantanément** et la suggestion disparaît de la liste. Il figure alors dans les **propriétés** de la note et apparaît dans ses rétroliens/liens sortants et dans tout le graphe. Si rien ne convient vraiment, laissez-les — ou, dans le Réviseur, marquez la note comme **autonome** délibérée. Les Connexions suggérées proposent ; c'est vous qui décidez.

**Locale, privée, sans blocage.** Les suggestions sont calculées à la demande à partir de votre seule Bibliothèque — rien ne quitte votre appareil — et leur constitution ne bloque jamais votre frappe (vous verrez un bref « Recherche de notes liées… » pendant le travail). Les suggestions, les indices de termes en commun et les types de relation apparaissent tous dans la langue que vous avez choisie et se reflètent correctement pour les écritures de droite à gauche.

---

## 24. Couleurs cognitives et menus clic droit

### Style des propriétés (Concepteur de style)

Ouvrez le **Concepteur de style** (Paramètres → Apparence → ✦ Ouvrir le Concepteur de style, ou son propre onglet) et choisissez la catégorie **Propriétés** pour restyler les petites étiquettes à l'intérieur du frontmatter d'une note. Deux éléments : **Étiquettes de propriétés** (les pastilles ordinaires de type `tags` — Fond de l'étiquette, Texte de l'étiquette, Rayon de l'étiquette 0–20 px, Hauteur 14–32 px) et **Badges de taxonomie** (Arrière-plan, Texte, Rayon 0–20 px). Un aperçu en direct au centre se met à jour au fil de votre édition ; chaque valeur démarre exactement sur l'apparence d'aujourd'hui, donc rien ne change tant que vous ne touchez pas à un contrôle. Cliquez sur **Conserver** pour enregistrer pour cet Univers.

### Couleurs cognitives (Concepteur de style)

La catégorie **Couleurs cognitives** vous donne **une couleur partagée par état cognitif**, de sorte que chaque surface qui affiche cet état s'accorde. Cinq ensembles :

- **Maturité** — Graine, Jeune pousse, Persistante, Canonique, Flétrissante.
- **Confiance** — Hypothèse, Preuve, Établi, Contesté.
- **Origine** — Reçu, Découvert, Mixte, Aucune.
- **Étape** — Étincelle, Naissance, Croissance, Maturité, Dormance, Archivage.
- **Catégorie de correspondance** (pourquoi un résultat de recherche a correspondu) — Titre, Contenu, Étiquette, Wikilien, Propriété, Sémantique, Structuré.

Le comportement est **unifier à la demande** : rien ne change tant que vous ne choisissez pas une couleur. Chaque surface conserve sa couleur actuelle comme repli, et dès l'instant où vous définissez la couleur d'un état ici, **toutes** les surfaces qui affichent cet état — arborescence, onglets, inspecteur de note, surbrillance de recherche dans l'éditeur, badge de correspondance et surbrillance du résultat de recherche — adoptent votre couleur d'un coup. Laissez un état intact et il a exactement l'apparence d'avant. Cliquez sur **Conserver** pour enregistrer.

### Menus clic droit

Constellation vous offre un menu contextuel à trois endroits, chacun ne proposant que les actions qui conviennent là où vous avez cliqué :

- **Clic droit sur le corps de la note** — Lien / Lien externe ; **Format ▸** (Gras, Italique, Souligner, Barré, Surligné, Code en ligne, Math en ligne, Basculer commentaire, Exposant, Indice, Effacer le formatage) ; **Paragraphe ▸** (Liste à puces/numérotée/de tâches, H1–H6, Corps, Citation) ; **Insérer ▸** (Note de bas de page, Tableau, Encadré, Ligne horizontale, Bloc de code, Bloc mathématique, Image) ; Couper / Copier / Coller / Coller en texte brut / Tout sélectionner ; et **Style…** (ouvre le Concepteur de style sur la catégorie **Éditeur**).
- **Clic droit sur une ligne de propriété du frontmatter** — Copier la valeur, Copier le nom, Supprimer la propriété, Ajouter une propriété ; puis le même menu d'édition que pour le corps ; et **Style…** ouvrant le Concepteur de style sur la catégorie **Propriétés**.
- **Clic droit sur un résultat de recherche** — un sous-ensemble **sûr** : Ouvrir, Ouvrir dans un nouvel onglet, Révéler dans l'arborescence, Copier le lien, Copier le chemin, Ajouter un marque-page, Afficher dans l'explorateur, Ouvrir dans l'app par défaut, et **Style…** (la catégorie **Couleurs cognitives**). Par conception, il n'y a **ni Renommer, ni Déplacer, ni Supprimer** ici — le panneau de recherche ne conserve pas de copie à la seconde près de l'arborescence, donc les actions destructrices restent dans l'arborescence, où la vue est toujours à jour.

Chaque entrée **Style…** atterrit sur la catégorie de la chose sur laquelle vous avez fait un clic droit, de sorte que vous n'avez jamais à chercher les bons contrôles. Chaque élément de menu, nom de catégorie et libellé d'état apparaît dans la langue d'interface que vous avez choisie et se reflète pour les mises en page de droite à gauche.
