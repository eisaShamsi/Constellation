---
aliases:
  - Panneau Calendrier
  - Calendrier des notes quotidiennes
  - Calendriers culturels
description: Une vue mensuelle en plein écran à travers huit calendriers, avec des jours cliquables, la création de notes quotidiennes, les échéances de tâches et l'enregistrement de dates culturelles.
---

# Calendrier

Le **Calendrier** est une vue mensuelle en plein écran, ouverte depuis le **dock de gauche** (l'icône du calendrier). Les jours qui comportent des notes ou des tâches à échéance sont signalés par des **points** colorés. L'en-tête affiche le mois dans le calendrier que vous avez choisi ; si vous définissez un **calendrier secondaire**, un sous-titre en dessous indique la plage équivalente dans ce calendrier (par exemple, un mois grégorien affiche son intervalle hégirien, « Dhū al-Ḥijja 1447 – Muḥarram 1448 AH »).

## Cliquer sur un jour

Chaque cellule de jour est interactive :

| Action | Résultat |
|--------|----------|
| Cliquer sur l'espace vide (ou sur le numéro du jour) | Ouvre — ou crée — la **note quotidienne** de ce jour. Cliquer sur une date qui possède déjà une note quotidienne se contente de l'**ouvrir** ; jamais de créer un doublon. |
| Cliquer sur un point | Ouvre l'élément correspondant. Si un jour comporte plusieurs notes ou tâches, cliquer sur le point affiche une petite **liste** dans laquelle choisir. |
| Cliquer sur un point de tâche | Ouvre la note **positionnée sur la ligne de cette tâche**, prête à être modifiée. |

### Couleurs des points

| Couleur du point | Signification |
|------------------|---------------|
| Doré | La **note quotidienne** de ce jour |
| Violet | Une autre **note** modifiée (ou datée) ce jour-là |
| Rouge | Une **tâche** à échéance ce jour-là |

Toutes les couleurs des points — et chaque autre partie du calendrier — sont personnalisables dans la surface **Style Setter → Calendrier**.

> [!tip]
> Dans la liste des tâches, vous pouvez **cocher la case d'une tâche pour la terminer** directement depuis le calendrier — les tâches terminées disparaissent immédiatement. Seules les tâches qui portent leur propre échéance `📅 YYYY-MM-DD` apparaissent dans le calendrier (c'est la date qui les place sur un jour).

## Calendriers culturels (huit)

Dans **Paramètres → Calendrier**, vous pouvez définir le **système de calendrier**, et toute la grille du mois bascule vers celui-ci :

- **Grégorien**
- **Hégirien (islamique)** — un moteur astronomique précis ; les mois sacrés sont mis en évidence et les événements islamiques sont marqués.
- **Hégirien solaire (persan)**
- **Hébraïque**
- **Indien (Saka)**
- **Bouddhiste**
- **Chinois** — *luni-solaire*
- **Coréen** — *luni-solaire*

Chaque cellule affiche à la fois la date du calendrier choisi (en grand) et la date grégorienne (en petit), ainsi que la phase de la lune. Chaque en-tête de mois affiche le **nom** du mois, son **numéro entre parenthèses** et l'**année** — le numéro aide pour les calendriers dont l'ordre des mois est peu familier.

Les calendriers **chinois et coréen** sont *luni-solaires* : ils insèrent parfois un **mois intercalaire** (闰六月 / 윤6월), que le calendrier affiche comme sa propre page afin que la navigation ne saute ni ne double jamais ce mois.

Vous pouvez également choisir le **début de semaine** (dimanche/lundi) et activer ou désactiver la **colonne du numéro de semaine**.

### Options du calendrier hégirien

Sous **Paramètres → Calendrier → « Calendrier hégirien (islamique) »**, deux réglages supplémentaires sont disponibles :

- **Méthode de calcul** — **Astronomique (conjonction lunaire)**, qui suit la véritable nouvelle lune (la plus précise, par défaut), ou **Tabulaire (al-Tawfīqāt al-Ilhāmiyyah** — les concordances inspirées**)**, le cycle arithmétique classique.
- **Correction de mois** — décalez le début d'un mois hégirien de ±1 ou ±2 jours pour le faire coïncider avec une **observation locale de la lune**. Choisissez l'année et le mois hégiriens, sélectionnez un décalage, puis cliquez sur **Définir** ; la correction s'applique à ce mois et à tous les mois suivants. Vos corrections sont répertoriées (chacune pouvant être retirée), avec un bouton **Tout effacer**.

Les deux réglages (ainsi que vos corrections) sont enregistrés **avec votre univers**, et voyagent donc d'un appareil à l'autre.

### Options d'affichage chinois et coréen

La Corée utilise le calendrier lunaire chinois, de sorte que les deux partagent des dates identiques — ce qui les distingue, c'est l'**écriture** et l'**année**. Lorsque l'un ou l'autre est votre calendrier principal ou secondaire, **Paramètres → Calendrier** affiche deux réglages supplémentaires :

- **Affichage de l'année** — chinois : le cycle sexagésimal 丙午年, l'année simple, ou les deux ; coréen : l'ère **Dangi** 단기 4359, l'année, ou le sexagésimal 병오년.
- **Noms des mois** — *écriture native* (五月 / 5월), ou *phonétique* — la prononciation du mois écrite dans votre propre langue (anglais « Wǔyuè / Owol » ; arabe « وُو-يوي / أوه-وُل »).

## Personnaliser le calendrier

Ouvrez le **Style Setter** (dock de gauche, ou **Paramètres → Style Setter**) et choisissez la surface **Calendrier** pour restyler chaque élément — chacun possède sa propre **couleur et taille de texte** (numéros des jours, la date de renvoi, la pastille du mois, les en-têtes des jours de la semaine, les numéros de semaine, le glyphe lunaire, la mise en évidence d'Aujourd'hui, les lignes de la grille, et les points de note/tâche/événement), ainsi que la **police** du calendrier. Un aperçu en direct et en pleine taille se met à jour à mesure que vous modifiez ; cliquez sur **Conserver** pour appliquer.

## Notes quotidiennes

Le Calendrier sert pleinement les notes quotidiennes : cliquez sur n'importe quel jour pour l'ouvrir, ou exécutez la commande **« Note quotidienne »** (palette de commandes) pour sauter à aujourd'hui.

> [!tip]
> **Les noms de fichiers des notes quotidiennes restent toujours grégoriens** (`YYYY-MM-DD`) quel que soit le calendrier affiché — ainsi vos fichiers restent portables et se trient correctement. La date culturelle est affichée dans le calendrier, et peut être enregistrée dans le frontmatter de la note (ci-dessous).

## Enregistrer une date culturelle dans une note

Deux outils optionnels écrivent la date culturelle dans les **propriétés** d'une note (le nom de fichier reste toujours grégorien `YYYY-MM-DD`) :

- **Tampon hégirien des notes quotidiennes** — *Paramètres → Calendrier → « Tamponner la date hégirienne dans les notes quotidiennes. »* Lorsqu'il est activé (disponible uniquement tant que le calendrier hégirien est votre calendrier **principal ou secondaire**), chaque **nouvelle** note quotidienne reçoit une ligne `hijri:`, par exemple `hijri: 1448-01-06`. Les notes que vous possédez déjà ne sont jamais touchées.
- **« + Hijri » dans les Propriétés d'une note** — ouvrez les **Propriétés** de n'importe quelle note, survolez la date, et un petit bouton **« + Hijri »** apparaît (en plus de « + Jalali », « + Hébraïque », etc. — **un bouton par calendrier non grégorien que vous avez sélectionné**). Cliquez dessus et Constellation lit la date grégorienne de la note et ajoute l'équivalent, par exemple `jalali: 1405-03-30`. Le bouton coréen écrit l'année **Dangi** ; un **mois intercalaire** chinois/coréen est signalé par un `L` (par exemple `chinese: 2025-06L-17`). Si la note n'a pas de propriété de date, c'est la date de création du fichier qui est utilisée.

> [!tip] Prise en charge RTL
> La grille du calendrier respecte la direction du texte courante. Dans les langues RTL (arabe, hébreu, persan, ourdou), la disposition du calendrier s'ajuste en conséquence.
