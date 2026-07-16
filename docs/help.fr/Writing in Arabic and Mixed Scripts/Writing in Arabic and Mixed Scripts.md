# Écrire en arabe et en écritures mixtes

L'éditeur de Constellation est conçu « langue d'abord » : l'arabe, l'hébreu, le persan, l'ourdou et les notes bilingues ne sont pas un ajout après coup — le curseur, la sélection et la direction de chaque paragraphe suivent les mêmes règles que Microsoft Word sous Windows, si bien que vos automatismes restent valables. Ce sujet couvre tout ce qui touche à l'*écriture* en texte de droite à gauche et en texte mixte : comment le curseur se déplace, comment sélectionner par mot, phrase, ligne, paragraphe ou écran, et comment forcer la direction d'un paragraphe lorsque la détection automatique ne correspond pas à votre intention.

(Pour la façon dont Constellation *comprend* l'arabe — racines, recherche et moteur morphologique — consultez le sujet **Moteur arabe**.)

---

## Comment le curseur se déplace

- **Les flèches déplacent le curseur d'un caractère du texte, dans l'ordre de lecture** — jamais d'une position à l'écran. En arabe pur ou en anglais pur, le résultat correspond exactement à la flèche pressée. À une couture entre l'arabe et l'anglais (une phrase arabe contenant un mot anglais, par exemple), le curseur parcourt chaque caractère dans l'ordre d'écriture et « saute » visiblement la couture — ce saut est correct : c'est lui qui empêche le curseur de sembler coincé à la frontière.
- **Home** va au **début** de lecture de la ligne — le bord *droit* d'une ligne arabe. **End** va à la **fin** de lecture — le bord *gauche*. Maintenez **Shift** avec l'une ou l'autre pour sélectionner jusqu'à ce bord.
- **Enter** sur une ligne arabe place le curseur de la nouvelle ligne à **droite** — la position d'écriture naturelle.
- Un **mot latin en fin de ligne arabe** conserve un curseur net et stable au lieu de perdre sa direction.

Chacune de ces règles fonctionne à l'identique dans l'éditeur standard, en mode Focus et dans la vue de fusion des conflits.

---

## Sélectionner par unité

Chaque unité de texte a son sélecteur rapide, quelle que soit la langue et quel que soit le mélange :

| Unité | Comment |
|---|---|
| **Mot** | Double-cliquez dessus |
| **Phrase** | **Ctrl+clic** n'importe où dedans — ou **Ctrl+Shift+S** avec le curseur à l'intérieur |
| **Ligne** | **Ctrl+L** |
| **Paragraphe** | **Ctrl+Shift+L** — ou triple-clic |
| **Écran entier** | **Shift+Page Down** / **Shift+Page Up** |
| **Tout** | **Ctrl+A** |

Détails utiles :

- **La sélection de phrase comprend la ponctuation arabe.** Elle termine une phrase sur **؟ ۔ !** et sur le point — mais le point-virgule arabe **؛** marque une pause *à l'intérieur* d'une phrase, et la sélection le dépasse donc, à juste titre. Les nombres décimaux comme 3.14 ne coupent jamais une phrase.
- Un **paragraphe** est un bloc de texte bordé d'une ligne vide au-dessus et au-dessous — exactement comme dans Word. Les sélections de ligne et de paragraphe épousent le texte : sur une ligne arabe, le surlignage s'arrête aux mots au lieu de s'étirer sur le vide à gauche.
- Ctrl+clic *remplace* l'ancien geste « ajouter un curseur » sur cette touche — désormais, ce clic déclenche la sélection de phrase.

## Se déplacer par paragraphe

- **Ctrl+↓** saute au début du paragraphe **suivant** ; **Ctrl+↑** au début du paragraphe **courant** (appuyez de nouveau pour le précédent). Ajoutez **Shift** pour sélectionner paragraphe par paragraphe au fil des sauts. C'est la convention de Word, et « suivant » signifie simplement plus bas dans la page — le comportement est identique dans les notes en arabe, en anglais ou mixtes.

---

## Forcer la direction d'un paragraphe

Constellation détecte automatiquement la direction de chaque ligne d'après ses premières lettres. C'est presque toujours le bon choix — mais il arrive que vous vouliez passer outre : un paragraphe arabe qui s'ouvre sur un nom de marque anglais, ou un paragraphe majoritairement anglais que vous voulez lire de droite à gauche.

**Appuyez puis relâchez Ctrl+Shift du côté DROIT de votre clavier** → le paragraphe où se trouve le curseur devient **100 % droite-à-gauche**.
**Appuyez puis relâchez Ctrl+Shift du côté GAUCHE** → **100 % gauche-à-droite**.

C'est la convention de Microsoft Word. À savoir :

- **Le basculement se déclenche au relâchement** — appuyez sur les deux touches ensemble, relâchez, et n'appuyez sur rien d'autre entre-temps. C'est pourquoi Ctrl+Shift+S, Ctrl+Shift+L et tous les autres raccourcis continuent de fonctionner normalement : dès qu'une troisième touche entre en jeu, le changement de direction se retire.
- **C'est un forçage absolu** — il l'emporte sur la détection automatique et s'applique au paragraphe entier (ou à chaque paragraphe touché par une sélection).
- **Il est enregistré dans le texte lui-même** sous forme de caractère de direction invisible : il survit à la fermeture de la note, au redémarrage de l'application et à la synchronisation — et il voyage même avec le texte si vous le collez dans Word ou Obsidian.
- **Un seul Ctrl+Z l'annule.** Appuyer deux fois du même côté n'a aucun effet supplémentaire.
- **Le Markdown reste intact.** Les listes restent des listes, les titres restent des titres, les citations restent des citations. Les blocs de code, les tableaux et les traits horizontaux sont volontairement laissés tels quels. Une ligne qui *commence* par un #tag garde sa direction automatique (une marque forcée y casserait le tag) — le reste du paragraphe bascule néanmoins.

---

## Polices et interface

- **Polices de script** : configurez indépendamment les polices arabes, hébraïques et CJK dans **Paramètres → Langue**.
- **Barres d'outils de script** : boutons de symboles et de ponctuation propres à chaque langue.
- **Surlignage du tashkeel** : activez ou désactivez le surlignage des diacritiques arabes depuis la barre d'outils de l'éditeur.
- Choisir l'arabe ou l'hébreu comme langue d'interface fait basculer toute l'application en RTL.

---

## Glossaire

- **Ordre de lecture** — l'ordre dans lequel les caractères sont écrits et lus, indépendamment de leur position à l'écran.
- **Couture** — la frontière entre un segment droite-à-gauche et un segment gauche-à-droite sur une même ligne.
- **Forçage absolu** — une direction que vous imposez explicitement et qui l'emporte sur la détection automatique par les premières lettres.
- **Marque de direction** — le caractère invisible (RLM/LRM) qui enregistre votre choix dans le texte lui-même.
