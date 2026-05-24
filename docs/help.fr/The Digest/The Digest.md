---
translation_status: AI-generated 2026-05-24 — native-speaker review recommended
language: fr
source: docs/help.uConstellation.World/The Digest/The Digest.md
aliases:
  - The Digest
  - Universe Digest
  - Digest
  - Digest pane
  - Digest de l'Univers
  - Digest
  - Panneau Digest
description: Le Digest de l'Univers est un panneau du dock gauche qui présente chaque note de votre base de connaissances au niveau résumé-titre — étagé Library → Dossier → Note — pour que vous puissiez parcourir tout l'Univers sans rien ouvrir. Cliquez sur une ligne pour la développer et voir le résumé complet en ligne. Le filtre restreint toute la liste ; le tri bascule entre récence (par défaut) et alphabétique. Lit les mêmes résumés que vous voyez partout ailleurs ; pas de calcul supplémentaire ; pas d'espace disque supplémentaire.
---

# Digest de l'Univers

> *Pensez au Digest comme à une table des matières pour votre esprit — non une liste de fichiers, une liste d'idées.*

Le **Digest de l'Univers** est l'endroit où parcourir l'ensemble de votre base de connaissances au niveau du *sens*. Au lieu d'une arborescence de fichiers (que des noms) ou de la Vue du Ciel (que des formes), le Digest vous montre, sous chaque note, **la seule phrase qui dit de quoi parle la note**. Tapez sur une ligne et le résumé complet de plusieurs phrases se développe en ligne. Vous pouvez lire la substance de cinquante notes en une minute, sans jamais en ouvrir une seule.

Il vit dans votre **dock gauche**, aux côtés de l'arborescence de Fichiers, du Navigateur de Notes et de la Vue du Ciel — l'une des quatre façons que Constellation vous offre pour naviguer.

---

## Pourquoi le Digest existe

Une arborescence de fichiers vous dit ce que vous *avez*. Une recherche vous dit ce que vous avez *demandé*. Le Digest vous dit ce que vous *savez*.

Quand votre Univers dépasse quelques centaines de notes, « ouvrir chacune pour se rappeler ce qu'elle dit » devient impossible. Il vous faut un moyen de lire l'**essence** de chaque note à la vitesse du défilement — et un moyen de développer toute essence dans le résumé complet à l'instant où vous voulez y réfléchir plus attentivement. C'est le Digest.

C'est le troisième pilier du Core Plug-In **Note Summary Creator (NSC)** :
- **Pilier 1** : un moteur de résumés (Phase 1 / MIG-043).
- **Pilier 2** : un service qui place le résumé partout où une note apparaît (Phase 2 / MIG-044 — Classificateur, Résultats de recherche, bandeau de l'Éditeur, Backlinks, Outgoing Links, l'Index, infobulle au survol de la Vue du Ciel).
- **Pilier 3** : cette vue — le Digest de l'Univers (Phase 3 / MIG-045).

---

## Ouvrir le Digest

Dans la **barre latérale gauche**, cliquez sur l'**icône du Digest de l'Univers** (une petite liste avec un cercle dans le coin) — c'est la quatrième icône de la rangée, à côté de l'arborescence de Fichiers / du Navigateur de Notes / de la Vue du Ciel. La barre latérale bascule vers le panneau Digest.

Pour revenir, cliquez sur l'une des trois autres icônes (ou appuyez sur **Échap**).

---

## Ce que vous voyez

De haut en bas :

1. **Barre d'outils.** Un champ de recherche + une petite icône d'horloge (le bascule de tri, par défaut « par récence »).
2. **En-têtes de Library.** Barres violettes en majuscules — une par library dans votre Univers. Chacune affiche le nom de la library et un décompte du nombre de notes qu'elle contient.
3. **En-têtes de dossier.** Petits libellés atténués — un par dossier *qui contient des notes*. Les notes qui vivent à la racine de la library n'obtiennent pas d'en-tête de dossier.
4. **Lignes de notes.** Chaque ligne a :
   - Un chevron (▶) sur la gauche — cliquez-le pour développer la ligne.
   - Le **nom de la note** dans la couleur d'accentuation interactive — cliquez-le pour **ouvrir la note** dans l'éditeur.
   - Une fine ligne en italique sous le nom — le **titre du résumé** (le même qui apparaît dans chaque autre surface de Phase 1/2).

---

## Développer une ligne pour lire le résumé complet

Cliquez sur le **chevron** (▶) à gauche d'une ligne — ou cliquez sur la **ligne en italique du titre** elle-même. Le chevron pivote vers ▼ et le **résumé complet de plusieurs phrases** apparaît en ligne sous le titre, s'enveloppant naturellement sur autant de lignes qu'il en a besoin.

Cliquez à nouveau sur le chevron (ou le titre) pour replier.

La division « cliquez sur le chevron pour développer, cliquez sur le nom pour ouvrir » garde les deux gestes distincts : vous pouvez développer pour *lire au sujet d'*une note, puis continuer à défiler au-delà ; ce n'est que lorsque vous cliquez sur le nom que la note s'ouvre vraiment et prend le focus.

---

## Filtrer

Tapez dans le **champ de recherche** en haut. La liste se restreint à mesure que vous tapez — seules les notes dont le **nom, titre ou résumé complet** contient votre requête restent visibles. Les en-têtes de library et les en-têtes de dossier avec zéro note correspondante disparaissent entièrement (pas d'en-têtes vides).

Effacez le champ (bouton × ou retour arrière) pour restaurer la liste complète.

Le filtre est **instantané** — Constellation ne touche pas votre disque ni la base de données. Il lit les résumés déjà en mémoire, donc même un Univers de 10 000 notes filtre à la vitesse de la frappe.

---

## Tri : récence ou alphabétique

Cliquez sur l'**icône d'horloge** dans la barre d'outils pour basculer entre deux modes de tri :

- **Récence** (par défaut) — au sein de chaque dossier, les notes apparaissent dans l'ordre du **temps de création, les plus récentes en premier**. Les dossiers au sein d'une library sont triés par la note la plus récente qu'ils contiennent (pour que le dossier le plus actif apparaisse en premier). C'est le défaut parce qu'il fait apparaître *ce sur quoi vous avez travaillé récemment*.
- **Alphabétique** — dossiers triés par nom, notes au sein de chaque dossier triées par nom. Cliquez à nouveau pour revenir à la récence.

Le bascule est par session ; fermez et rouvrez le Digest et il revient à la récence.

---

## Fédération : les Univers enfants apparaissent en ligne

Si votre Univers a des **Univers enfants liés** (cUniverses), chaque library d'un Univers enfant apparaît dans le Digest comme **son propre en-tête de Library pair**, aux côtés des libraries de l'Univers parent. Le Digest est une vue unifiée de tout ce qui est atteignable depuis cet Univers, pas seulement des libraries qui vivent physiquement ici.

(Une future mise à jour de Constellation ajoutera un bascule marche/arrêt pour masquer temporairement les libraries d'Univers enfants du Digest ; pour l'instant elles apparaissent toujours.)

---

## Comment le Digest reste rapide sur d'énormes Univers

Le Digest est **virtualisé** : il ne rend que les lignes actuellement visibles dans votre fenêtre de défilement, pas l'arbre entier. Un Univers de 10 000 notes défile aussi fluidement qu'un de 50. À mesure que les lignes défilent à la vue, leurs résumés sont récupérés par lots depuis le cache en mémoire de Constellation (le même cache qui alimente toutes les autres surfaces de Phase 1/2 — pas de travail séparé, pas de stockage séparé).

Le Digest ne relit jamais vos notes depuis le disque. Il ne recalcule jamais les résumés. C'est une vue de **lecture** sur la même table `note_summaries` que le moteur peuple depuis la Phase 1.

---

## Flux de travail courants

**« Je veux voir sur quoi j'ai travaillé cette semaine. »**
Ouvrez le Digest avec tri = Récence (par défaut). Les notes les plus récemment créées apparaissent en haut de chaque library/dossier. Scannez les titres.

**« Je cherche une note à moitié remémorée sur X. »**
Ouvrez le Digest. Tapez X (un mot qui apparaîtrait dans le titre de la note, le titre du résumé ou le résumé complet). La liste se restreint aux candidats. Cliquez sur les chevrons pour lire les résumés complets ; cliquez sur le nom pour ouvrir le vainqueur.

**« Je veux écrire une revue descendante de ma Library. »**
Ouvrez le Digest, tri = Alphabétique. Parcourez les titres dans l'ordre. Cliquez sur les chevrons pour lire des résumés plus complets quand quelque chose vous accroche. Utilisez ceci comme épine dorsale d'une nouvelle note MOC (Map of Content).

**« J'explore un cUniverse fédéré pour la première fois. »**
Ouvrez le Digest. Défilez au-delà de vos propres libraries jusqu'aux libraries du cUniverse — ce sont des lignes paires. Lisez les titres pour apprendre ce que contient l'Univers lié, sans rien en ouvrir.

---

## Ce qui N'EST PAS dans le Digest

- **Menu contextuel par clic droit** sur les lignes — ouvrir dans un nouvel onglet, archiver, etc. (Pour v1, les actions primaires sont clic-nom-pour-ouvrir et clic-chevron-pour-développer. Une future mise à jour ajoutera un menu contextuel.)
- **Regroupements personnalisés** — Library → Dossier est le seul étagement pour v1. (Pas encore de « regrouper par étiquette » ou « regrouper par étape ».)
- **Glisser-pour-réordonner** — le Digest est en lecture seule ; le tri vient de règles, pas d'un ordre manuel.
- **Contrôles de classification de type Classificateur** — le Digest est une vue de *navigation* ; la classification vit dans le **Classificateur** (panneau séparé).

---

## Sujets associés

- **Résumés de notes** — d'où viennent les résumés, la règle de priorité (le vôtre gagne), et la liste complète des surfaces qui les affichent.
- **Le Classificateur** — la maison de *Générer tous les résumés* (précalculer chaque résumé de votre Library d'un coup pour que le Digest se remplisse instantanément).
- **Vue du Ciel** — la vue de la *forme* de vos connaissances (bulles + liens) ; le Digest est sa vue complémentaire du *sens*.
- **Formulation de la Connaissance** — pourquoi Constellation organise la connaissance par *connexion* et *résumé*, pas seulement par stockage de fichiers.
