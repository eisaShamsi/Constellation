# Pouls de révision

*(Le Réviseur — la liste de rappel de vos connaissances)*

Le Pouls de révision, c'est l'endroit où Constellation vous indique **quelles notes réclament votre attention dès maintenant, et pourquoi** — et vous prescrit la seule chose saine à faire pour chacune. Tous les autres panneaux répondent à une question qu'il vous faut aller poser (« comment se porte mon graphe ? à quoi cette note est-elle reliée ? »). Le Réviseur est le seul qui **vient à vous** : il fait remonter les notes qui ont dépéri, dérivé ou se sont détachées, classées par urgence, chacune accompagnée d'une raison en langage clair.

Voyez-le comme la **liste de rappel** d'un médecin. Il ne se contente pas de dire « quelque chose ne va pas » — il diagnostique le problème, prescrit le remède, vous dit à quel point c'est urgent et *pourquoi*, puis vous oriente vers les instruments plus profonds (l'éditeur, l'Inspecteur 360°, le Catalogueur) pour agir.

> **Deux surfaces, une seule idée.** Le **Réviseur** complet (l'icône d'horloge 🕐 dans le dock de gauche) est la file d'attente de *toute la bibliothèque*. L'onglet **Révision** de la barre latérale droite (la même icône 🕐) n'affiche que *la note que vous avez ouverte* — son propre état. Ils partagent le même moteur, donc une note affiche la même priorité dans les deux.

---

## Ouvrir le Réviseur

Cliquez sur l'icône **horloge 🕐** tout à gauche du dock. Le Réviseur remplit la fenêtre en deux colonnes :

- **À gauche — la file d'attente**, regroupée en six optiques (ci-dessous). Chaque optique est toujours listée ; une optique vide est grisée avec un **0**. Cliquez sur l'en-tête d'une optique pour la replier ou la déplier.
- **À droite — le volet de détail** de la note que vous avez sélectionnée : ce qui ne va pas, le remède, le degré d'urgence et les actions.

Cliquez sur n'importe quelle note de la file pour charger son détail à droite. Cliquez sur le **nom** d'une note (ou sur **Relier**) pour l'ouvrir dans l'éditeur — un bouton **‹ Réviseur** apparaît alors dans la barre d'onglets supérieure pour vous ramener directement là où vous étiez.

---

## Les six optiques

Une note peut figurer dans plusieurs optiques à la fois — chaque optique répond à une question différente, et elles ne sont jamais fondues en un score unique.

| Optique | Ce que cela signifie |
|---|---|
| 🥀 **Périmée** | Une note sur laquelle celle-ci *s'appuie* (un lien porteur — soutient, contredit, dérive-de, partie-de, remplace) a changé **après** votre dernière révision de cette note. Votre note ne se réconcilie peut-être plus avec elle. |
| 🔄 **À réviser** | L'intervalle de révision est écoulé — le moment de la relire et de confirmer qu'elle tient toujours. |
| 🧠 **Points de contrôle du modèle mental** | Une note que vous avez signalée comme une hypothèse ou un modèle. *Soutenez-vous toujours ce point de vue ?* |
| 🔗 **Orpheline — reliez-moi** | Une note au contenu réel à laquelle **rien ne renvoie encore**. Elle est en dehors de votre toile de pensée. Une orpheline est une **alarme**, pas du fouillis : reliez-la, ou marquez-la comme volontairement autonome. |
| ⚠ **Fragile — étayez-moi** | Beaucoup de notes s'appuient sur celle-ci, mais elle-même repose sur peu de soutien. Un point unique de défaillance — donnez-lui un terrain plus solide. |
| 📝 **Jamais révisée** | Une note présente dans votre bibliothèque depuis un certain temps, mais à laquelle vous n'avez jamais accordé une première lecture complète. |

---

## Le volet de détail : diagnostic → prescription

Quand vous sélectionnez une note, la colonne de droite se lit de haut en bas :

1. **Titre + résumé.** Un résumé d'une ou deux phrases de la note s'affiche toujours ici (selon vos réglages de résumé) afin que vous sachiez *ce que* l'on vous demande de revoir.
2. **Le diagnostic** — le « pourquoi maintenant » en langage clair, par ex. *« dérive-de 'Preuve' a changé le 2026-06-12. »*
3. **La prescription** — la seule chose saine à faire, par ex. *« Révisez-la au regard de 'Preuve' — réconciliez votre position ou mettez-la à jour. »* Pour une orpheline : *« Reliez-la à une note connexe — ou marquez-la comme délibérément autonome. »*
4. **Priorité** — un nombre de 0 à 100, présenté sous forme de **recette** (voir ci-dessous).
5. **Faits** — sa maturité (graine / jeune pousse / persistante / canonique / fanaison), ses connexions (« 12 entrants · 4 sortants »), et la date de votre dernière révision.
6. **Actions** + **passages de relais** (ci-dessous).

---

## Une priorité que vous pouvez lire — et outrepasser

Le nombre de priorité n'est pas une boîte noire. Il est calculé à partir de la situation de la note et présenté comme une **barre découpée selon ses raisons**, chacune étiquetée et s'additionnant exactement au nombre — par exemple :

> **63**  ·  *Pression temporelle +31 · Sollicitée +14 · Maturité +10 · …*

Il combine deux choses, la classique distinction urgent/important :

- **Urgence** — à quel point elle est en retard ou périmée, et à quel point le changement a été perturbateur (une *contradiction* qui survient sous vos pieds est plus urgente qu'une note de soutien qui se déplace simplement).
- **Importance** — combien de notes en dépendent, son degré de maturité, et si elle est fragile.

Vous gardez toujours la main sur le nombre. **Faites glisser le curseur** pour fixer votre propre priorité ; elle est alors marquée **« manuel »**, vous montre quelle serait la valeur *calculée*, et propose **« Rétablir la valeur calculée »** pour rendre la main au moteur. Votre choix manuel tient jusqu'à ce que vous le réinitialisiez.

---

## Agir sur une note

Chaque optique propose les verbes adaptés :

- **✓ Révisée** — vous l'avez relue ; elle est confirmée et reprogrammée sur l'échelle de 1·3·7·14·30 jours. (C'est la *seule* action qui fait avancer la « dernière révision » — le simple fait d'ouvrir une note ne compte pas comme une révision.)
- **🔗 Relier** (orphelines) — ouvre la note pour que vous puissiez ajouter un lien.
- **👁 Reporter 7 j** — masque pendant une semaine une note échue par le temps. (Le report ne s'applique qu'aux optiques fondées sur le temps ; une note périmée ou fragile n'est pas quelque chose qu'une semaine d'attente règle.)
- **🗄️ Écarter** — cesser de suivre cette note pour la révision. Pour une orpheline, l'intitulé devient **« Marquer comme autonome »** — *cette note est faite pour être seule, ce n'est pas une orpheline.*

Les **passages de relais** en bas vous amènent aux instruments plus profonds sans rompre le fil de votre pensée : **Ouvrir dans l'éditeur**, **Contexte complet (360°)** (le tableau structurel complet de la note) et **Classer** (le Catalogueur). Le Réviseur fait le tri ; ceux-ci expliquent.

---

## L'onglet Révision propre à la note

Ouvrez n'importe quelle note, puis l'onglet **🕐 Révision** de la barre latérale droite. Il affiche l'état de *cette* note — à réviser / périmée / un point de contrôle / jamais révisée — avec le même curseur de priorité (calculé par défaut, modifiable, avec réinitialisation) et les mêmes actions ✓ / Reporter / Écarter. C'est la vue, note par note, de tout ce qui précède.

---

## Réglages — le délai de grâce avant péremption

Par défaut, une note est signalée **Périmée** le lendemain du changement d'une dépendance. Si c'est trop empressé, allez dans **Réglages → Révision** et augmentez le **délai de grâce** (en jours, minimum 1) : un changement de dépendance ne signale alors la note qu'une fois ce nombre de jours écoulé depuis votre dernière révision. Gardez-le patient si vous faites beaucoup de petites retouches.
