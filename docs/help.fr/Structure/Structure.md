# Structure

*(La colonne vertébrale compositionnelle — où cette note se situe dans l'ensemble de l'œuvre)*

Constellation vous offre déjà huit **liens de pensée** — *appuie, contredit, cause, illustre, généralise, dérive-de, fait-partie-de, remplace* — le vocabulaire que vous employez pour relier une idée à une autre. Les **liens structurels** sont d'une nature délibérément différente. Ils ne relient pas une idée à une idée ; ils déploient la **forme ordonnée d'une œuvre** que vous construisez à partir de vos notes : Livre → Partie → Chapitre → Scène, ou tout plan de type Carte de Contenu. Le panneau **Structure** est l'endroit où vous lisez cette forme.

L'unique question à laquelle Structure répond est : **« Où cette note se situe-t-elle dans l'ensemble de l'œuvre ? »** — et *non* « comment cette idée se rattache-t-elle à celle-là ». Cette seconde question relève des panneaux Rétroliens (Backlinks) et Liens sortants, et Structure ne leur marche pas sur les pieds.

---

## Pourquoi les liens structurels sont tenus à l'écart de votre pensée

Un placement structurel relève de l'**écriture d'auteur, non d'une affirmation à juger**. Placer une scène sous un chapitre, ou un chapitre sous un livre, est une décision sur la *forme de votre manuscrit* — ce n'est pas une preuve, pas un argument, pas quelque chose qui puisse être contredit ou devenir plus certain avec le temps.

Les liens structurels sont donc délibérément invisibles pour toute mesure de pensée, de maturité et de connexion :

- Ils ne comptent **pas** comme des connexions dans les rétroliens ou les liens sortants d'une note.
- Ils n'élèvent **pas** la maturité d'une note.
- Ils n'apparaissent **pas** dans la Vue Étoiles ni dans le graphe.

Une table des matières ne devrait pas faire paraître une note plus « connectée » qu'elle ne l'est. Vos liens de pensée et le plan de votre manuscrit sont deux choses distinctes, et Constellation les garde ainsi.

---

## Les deux sortes — vous ne saisissez jamais qu'un seul côté

Vous déclarez la structure depuis l'extrémité qui vous arrange, et Constellation déduit l'inverse pour vous. Vous n'avez jamais à entretenir les deux côtés.

| Propriété | Ce que cela signifie |
|---|---|
| **`parent`** | La place de *cette note* sous un parent. (Un chapitre indique la partie à laquelle il appartient.) |
| **`contains`** | La liste ordonnée des enfants de *cette note*. (Un livre énumère ses parties, dans l'ordre de lecture.) |

Déclarer le `parent` d'un enfant et l'énumérer dans une liste `contains` sont deux façons de dire la même chose. Utilisez celle qui correspond à votre manière de penser — du haut vers le bas (un livre qui *contient* ses parties) ou du bas vers le haut (un chapitre qui nomme son *parent*).

---

## Créer un lien structurel — pas à pas

Vous créez la structure dans les **Propriétés** d'une note — l'onglet Propriétés dans la barre latérale de droite, ou le bloc de propriétés en haut de la note.

1. Cliquez sur **+ Ajouter une propriété**.
2. Pour la clé, saisissez **`parent`** ou **`contains`**.
3. Dans la valeur, saisissez le **nom de la note cible** — juste le nom, par exemple `Part I - The Cartographer`. **Vous ne saisissez pas les crochets.** Constellation enveloppe le nom dans un `[[link]]` pour vous, automatiquement. (Si vous collez un nom qui comporte déjà des crochets, il est nettoyé en un unique `[[name]]` — jamais un double `[[[ ]]]`.)
4. Pour **`contains`**, ajoutez chaque enfant comme sa propre puce — saisissez un nom, appuyez sur Entrée, saisissez le suivant. **L'ordre dans lequel vous les ajoutez est l'ordre de lecture** du plan.

> **Ils se renomment en toute sécurité.** Renommez un chapitre et sa place dans la structure suit automatiquement — le lien pointe vers la note elle-même, et non vers un morceau de texte figé. Vous n'avez jamais à traquer et corriger un plan après un renommage.

---

## Lire le panneau Structure

Ouvrez l'onglet **Structure** dans la barre latérale de droite — juste après l'onglet Rétroliens.

- **Le plan.** Coiffé du titre **OUTLINE** (Plan) avec un décompte, le panneau affiche l'**œuvre entière** sous forme d'arborescence indentée à puces sarcelle — chaque descendant de l'œuvre, dans l'ordre — pas seulement les propres enfants de la note ouverte. Ainsi, même lorsque vous vous tenez sur une seule scène, vous voyez le livre entier autour d'elle.
- **« Vous êtes ici. »** La note que vous consultez actuellement est **mise en évidence** au sein du plan, de sorte que vous savez toujours où vous vous situez.
- **Le fil d'Ariane.** En haut, un fil d'Ariane sarcelle affiche le chemin remontant la colonne vertébrale — par exemple *The Atlas of Lost Places › Part I › Chapter 1*. Cliquez sur n'importe quel maillon (ou n'importe quelle ligne du plan) pour sauter directement à cette note.
- **Whole work ⇄ This note** (Œuvre entière ⇄ Cette note). Une bascule en haut à droite alterne entre l'œuvre entière et la seule branche de la note ouverte. Elle n'apparaît que lorsque la note possède un parent (sinon les deux vues seraient identiques).

> **Une boucle ne le bloque jamais.** Si la structure boucle accidentellement sur elle-même — le parent de la note A est B, et le parent de B est A — le plan dessine la chaîne puis s'arrête proprement, marquant le point de coupure par un petit **↻**. Survolez-le pour une explication en une ligne.

---

## Quand deux notes revendiquent le même enfant — « Contested »

La structure est censée être un arbre net, donc un enfant ne devrait avoir qu'un seul parent. Si deux notes revendiquent toutes deux le même enfant — l'une via le propre **`parent`** de l'enfant, l'autre via sa liste **`contains`** — Constellation ne choisit **pas** silencieusement l'une et n'abandonne pas l'autre. Au lieu de cela, cette ligne est signalée **Contested** (En litige) avec un badge ambre **⚠** nommant l'autre revendicateur, afin que vous puissiez voir le conflit et décider.

Deux boutons à un clic le résolvent :

- **Keep** (Conserver) — conserver le parent déclaré par l'enfant lui-même. (Cette note renonce à sa revendication sur l'enfant.)
- **Move here** (Déplacer ici) — accepter cette note comme parent. (Le `parent` de l'enfant bascule vers cette note.)

L'un ou l'autre choix met à jour les fichiers de notes directement et rafraîchit le plan. **Rien n'est jamais modifié sans votre clic** — Constellation signale le conflit et attend votre décision.

---

## Bon à savoir

- **Local et privé.** Le plan est lu à la demande à partir de vos propres notes ; rien n'est envoyé où que ce soit.
- **Rapide sur les grandes œuvres.** Les longs plans (au-delà d'environ 50 lignes) obtiennent leur propre barre de défilement et n'affichent que les lignes visibles à l'écran, de sorte qu'un grand manuscrit s'ouvre et défile en douceur.
- **Il parle votre langue.** Les libellés du panneau, le fil d'Ariane et les boutons de résolution apparaissent tous dans la langue d'interface que vous avez choisie et se reflètent correctement pour les langues qui se lisent de droite à gauche. Les *clés* de propriété `parent` / `contains` restent en anglais canonique dans le fichier (de sorte que la structure se lit de la même manière dans toutes les langues), tandis que leurs libellés de pastille affichés à l'écran sont localisés.
