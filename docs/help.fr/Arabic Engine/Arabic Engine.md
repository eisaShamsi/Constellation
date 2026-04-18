# Moteur arabe

Constellation analyse les textes arabes avec un moteur morphologique a cinq couches, concu des la base pour cette application. Ce n'est pas le portage d'un stemmer existant — c'est un instrument natif qui comprend les racines arabes, les schemes, les noms propres, les emprunts et votre propre terminologie. Vous ne configurez jamais le moteur lui-meme ; il tourne silencieusement sous chaque recherche, chaque lien, chaque entree d'index. Ce que vous *pouvez* configurer — et ce que couvre ce sujet d'aide — est le seul endroit ou le moteur sollicite votre jugement : le panneau **Surcharges du moteur arabe** dans les Parametres.

---

## Pourquoi le moteur existe

L'arabe est une langue a schemes. Une seule racine comme ك‑ت‑ب (« ecrire ») engendre des dizaines de formes de surface — كاتب (ecrivain), مكتوب (ecrit), كتاب (livre), يكتب (il ecrit), كتبنا (nous avons ecrit) — qui devraient toutes se replier sur le meme noyau semantique lors d'une recherche. Un stemmer naif soit mutile ces formes (en sur-depouillant وائل en ائل, par exemple), soit manque entierement le lien entre elles. Le moteur de Constellation evite ces deux echecs en faisant passer chaque mot arabe par cinq couches, dans un ordre de priorite strict :

1. **Couche 0 — Surcharges utilisateur** (c'est celle que vous controlez)
2. **Couche 2 — Liste protegee** (environ 1 200 noms propres, lieux, emprunts et mots-outils curatives a la main et qui ne doivent jamais etre touches)
3. **Couche 3 — FST generatif** (un transducteur a etats finis compile qui fait correspondre environ 7 000 racines x 158 schemes a leur vocabulaire de surface complet)
4. **Couche 3b — Cascade** (reparations phonologiques : assimilation, racines faibles, placement de la hamza)
5. **Couche 5 — Heuristique** (le repli indulgent — un decoupeur d'affixes conservateur qui ne se declenche que lorsque toutes les autres couches ont renonce a repondre)

Une etape de classement (Couche 4) choisit la meilleure analyse lorsqu'il y en a plusieurs. Le classement place vos surcharges au-dessus de tout le reste.

---

## Fonction : Surcharges du moteur arabe

### Ce que c'est

Le panneau Surcharges est une petite table dans les Parametres ou vous dites au moteur, avec vos propres mots, comment analyser certaines formes de surface arabes. Chaque surcharge comporte :

- **Forme de surface** — le mot arabe exactement tel que vous le tapez (par ex. وائل).
- **Lemme** — la forme canonique que le moteur doit renvoyer (par ex. وائل).
- **Racine** — optionnelle. Trois ou quatre consonnes si le mot possede une racine classique.
- **Scheme** — optionnel. Une etiquette libre (par ex. `فاعل`) si vous voulez consigner le schema morphologique.
- **Categorie grammaticale** — Nom propre / Nom / Adjectif / Adverbe / Verbe / Particule / Etranger / Inconnu.
- **Note** — optionnelle. Une ligne de contexte pour votre vous futur.

### Pourquoi c'est important

Tout reseau de connaissances contient des termes que le moteur ne peut pas connaitre par dictionnaire : vos propres neologismes, les noms de votre ville, les acronymes de votre domaine, les emprunts que vos collegues preferent ecrire d'une maniere particuliere. Sans surcharges, le moteur appliquerait son analyse generique a ces formes de surface, et vos resultats de recherche se fragmenteraient autour de legeres variations. Une surcharge est la reponse souveraine — elle l'emporte sur le FST generatif, la cascade et le repli heuristique. Le classement de la Couche 4 donne aux surcharges l'origine la plus haute et une confiance de 1,0, de sorte qu'elles ne sont jamais ecartees au profit d'une autre analyse.

Les surcharges vivent dans un unique fichier JSON a `<votre Univers>/.constellation/arabic-overrides.json`. Le fichier est en texte brut, trie alphabetiquement et ecrit de maniere atomique (via une paire `.tmp` + renommage), si bien qu'une coupure de courant pendant une modification ne peut pas le corrompre. Il vous appartient — vous pouvez le gerer sous controle de version, le diffuser ou le partager entre appareils.

### Comment l'utiliser

**Etape 1 : ouvrir le panneau**

Cliquez sur l'icone d'engrenage dans la barre d'outils en haut a droite (ou appuyez sur `Ctrl + ,` / `Cmd + ,`) pour ouvrir les Parametres. Dans la barre laterale gauche, selectionnez **Surcharges arabes** — elle se trouve a cote de **Langue**. Si vous ne la voyez pas, faites defiler la barre laterale.

**Etape 2 : ajouter votre premiere surcharge**

Cliquez sur **Ajouter une surcharge**. Un formulaire apparait avec six champs (forme de surface, lemme, racine, scheme, categorie grammaticale, note). Tapez la forme de surface exactement comme vous l'ecrivez dans vos notes — le moteur normalise les diacritiques et les variantes d'alif en interne, inutile donc de les reproduire a l'identique. Renseignez le lemme attendu. Laissez la racine et le scheme vides si vous ne les connaissez pas ; le moteur utilisera la surcharge malgre tout. Choisissez une categorie grammaticale dans le menu deroulant ou laissez **Inconnu**. Cliquez sur **Enregistrer**.

**Etape 3 : observer la banniere de reindexation**

Des que vous enregistrez, le panneau affiche **Reindexation…** et le moteur balaie chaque note de l'Univers actif dont le texte contient cette forme de surface. Chaque note correspondante est retokenisee sous le nouveau verdict de surcharge. Lorsque le balayage se termine — en general en moins d'une seconde sur un Univers typique — la banniere passe a **N note(s) reindexee(s)** et disparait automatiquement apres trois secondes. Pas besoin de redemarrer l'application, pas besoin de reconstruire un index.

**Etape 4 : verifier dans la recherche**

Ouvrez le hub de recherche (`Ctrl + K` / `Cmd + K`) et tapez la forme de surface. Les resultats devraient maintenant refleter le lemme que vous avez specifie : les requetes sur le lemme trouvent la forme de surface, et les requetes sur la forme de surface trouvent d'autres flexions du lemme.

**Etape 5 : supprimer une surcharge**

Cliquez sur le bouton **x** sur la ligne de la surcharge. L'entree est retiree du disque immediatement, et le meme balayage de reindexation s'execute en sens inverse — les notes qui contenaient la forme de surface sont retokenisees avec l'analyse generique du moteur. La banniere indique combien de notes ont ete touchees.

### Interaction avec la Liste protegee

La Liste protegee (Couche 2) contient deja environ 1 200 formes de surface courantes qu'il ne faut jamais depouiller — des noms comme وائل, des lieux comme فلسطين, des emprunts comme إنترنت. Inutile de les ajouter vous-meme ; le moteur les embarque. Utilisez le panneau Surcharges pour des formes *personnelles* a votre Univers — votre propre terminologie, des noms locaux, des emprunts specifiques a un domaine, ou les cas ou vous n'etes pas d'accord avec la lecture automatique du moteur.

### Interaction entre Univers

Chaque Univers a son propre fichier de surcharges. Changer d'Univers remplace l'ensemble de surcharges actif en memoire — le moteur recharge le JSON depuis le dossier `.constellation/` du nouvel Univers. Si le fichier est absent (Univers tout neuf), le moteur considere l'ensemble comme vide. Si le fichier est malformatte, le moteur journalise un avertissement et se replie sur un ensemble vide plutot que de refuser de charger.

### Ce qui se passe si vous editez le fichier a la main

Vous le pouvez. Le format du fichier est :

```json
[
  {
    "surface": "وائل",
    "lemma": "وائل",
    "root": null,
    "pattern": null,
    "pos": "ProperNoun",
    "note": "Prenom — ne jamais depouiller"
  }
]
```

Gardez les entrees triees alphabetiquement par forme de surface pour obtenir des diffs git lisibles. Le moteur retrie a chaque enregistrement, donc un reordonnancement manuel ne survivra pas a une modification via l'interface.

---

## Glossaire

- **Forme de surface** — un mot arabe tel qu'il est ecrit, avec les clitiques attaches (par ex. الكتاب, بالكتاب, كتبنا).
- **Lemme** — la forme de citation d'un mot, depouillee de sa flexion (par ex. كتاب).
- **Racine** — le noyau semantique de 3 ou 4 consonnes partage par une famille de mots (par ex. ك‑ت‑ب).
- **Scheme** — le patron de voyelles et d'affixes qui, combine a une racine, produit une forme de surface (par ex. فاعل → كاتب).
- **FST** — un transducteur a etats finis. Le moteur en utilise un pour faire correspondre efficacement racines x schemes a leur vocabulaire de surface complet.
- **Cascade** — la couche de reparation phonologique qui gere l'assimilation, les consonnes faibles et le placement de la hamza.
- **Surcharge** — votre propre verdict sur la maniere dont une forme de surface specifique doit etre analysee ; l'emporte sur toutes les autres couches.
