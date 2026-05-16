---
translation_status: AI-generated 2026-05-16 — native-speaker review recommended
language: fr
source: docs/help.uConstellation.World/Constellation Nervous System/Constellation Nervous System.md
aliases:
  - Constellation Nervous System
  - CNS
  - Système Nerveux Constellation
description: Constellation Nervous System (CNS) est la vue de traversée-de-connexions de votre univers. Il analyse le graphe de liens entre vos notes et fait apparaître les métriques de Santé de l'Univers, les communautés, les ponts principaux entre clusters, et les « Angles Morts » des écarts structurels. CNS est la vue complémentaire à Constellation Sight — si Sight est la forme sensorielle de votre univers, CNS en est les connexions neurales.
---

# Constellation Nervous System (CNS)

## Qu'est-ce que c'est ?

**Constellation Nervous System** est la vue de **traversée-de-connexions** de votre univers. Tandis que Constellation Sight montre la *forme* de vos notes (strate × temps × encodage de canal), CNS montre le *câblage* — le graphe de liens typés qui les connecte et les motifs structurels cachés dans ce graphe.

Il répond : **« Comment les idées dans mon univers sont-elles connectées, et où sont les écarts ? »**

La vue est construite autour de quatre surfaces analytiques :
- **Santé de l'Univers** — scores globaux et par métrique pour comment connecté, équilibré et modulaire est votre savoir.
- **Communautés** — groupes de notes densément interconnectées (« clusters idéologiques »).
- **Ponts Principaux** — les quelques notes qui lient des communautés autrement séparées (« connecteurs porteurs »).
- **Angles Morts** — écarts structurels où vous attendriez des connexions mais n'en avez pas encore.

Le nom « Nervous System » est anatomique : les nerfs sont des voies de connexion portant signaux entre parties distantes d'un organisme. CNS traite votre graphe de liens typés de la même manière.

## Pourquoi est-ce important ?

La plupart des apps de notes traitent les liens comme de la plomberie (sauter d'ici à là-bas). Constellation les traite comme **architecture de savoir** :

- Une note avec beaucoup de liens entrants est **porteuse** — de nombreuses idées dépendent d'elle.
- Une note qui ponte deux communautés est un **point de synthèse**.
- Une communauté avec un lien interne faible est **fragile**.
- Un « Angle Mort » est un endroit où la structure DEVRAIT avoir une connexion mais ne l'a pas — une hypothèse à explorer.

## Comment l'ouvrir

1. Cliquez sur l'**icône neurone** (petite forme de cellule nerveuse ramifiée — corps cellulaire au milieu avec trois branches dendritiques et terminaux synaptiques) dans le dock à gauche.
2. CNS s'ouvre en superposition pleine fenêtre, style puits de gravité — graphe forcé où chaque note est un nœud et chaque lien typé une arête.
3. Pour fermer : cliquez sur **(×)** en haut, ou appuyez sur **Esc**.

## Ce que vous voyez

### La Carte Santé de l'Univers

Panneau résumé montrant la santé de connectivité globale de votre univers, avec rondelle dorée d'un score composite (ex. **91 / 100**) et quatre métriques :

- **Modularity** — propreté du clustering en communautés distinctes.
- **Dominance** — si une communauté domine l'univers.
- **Entropy** — variété des tailles de communautés.
- **Connectivity** — moyenne de liens par note.

Chaque métrique a une pilule colorée : **HEALTHY** (vert) / **CAUTION** (jaune) / **IMBALANCED** (rouge).

### Le Puits de Gravité

Visualisation principale : les notes flottent comme nœuds, les liens les rapprochent, la répulsion les éloigne. Communautés s'auto-organisent en clusters.

- **Taille du nœud** = nombre de liens.
- **Couleur du nœud** = appartenance à la communauté.
- **Arête** = lien typé entre deux notes.

### Ponts Principaux

Liste des notes qui lient le plus de communautés distinctes — ce sont vos points de synthèse.

### Communautés

Liste des clusters détectés.

### Angles Morts (Écarts Structurels)

Connexions manquantes suggérées — paires de notes que l'algorithme pense DEVRAIENT être liées.

## Interaction

CNS utilise un schéma **clic-simple-prévisualise / double-clic-ouvre** (différent du clic-simple-ouvre de Sight) :

| Geste | Effet |
|---|---|
| **Clic simple sur nœud** | Le sélectionne. Panneau droit avec titre, communauté, rang de centralité, liens entrants/sortants. La note N'EST PAS ouverte. |
| **Double clic sur nœud** | Ouvre la note dans l'éditeur. Bouton **« Return to CNS »** apparaît. |
| **Survol nœud** | Infobulle avec titre. |
| **Clic sur zone vide** | Efface la sélection. |
| **Molette** | Zoom in/out. |
| **Clic + glisser** | Pan. |
| **Clic sur communauté dans la liste** | Met en évidence les notes dans le puits. |
| **Clic sur entrée Pont Principal** | Focalise sur la note pont. |
| **Esc** | Ferme CNS. |

Le clic-simple-prévisualise est délibéré : il vous permet de scanner les détails de nombreuses notes (et leurs connexions) sans vous engager à en ouvrir chacune dans l'éditeur.

## Quand CNS Est le Plus Utile

- **Auditer votre densité de connexion** — Universe Health donne une lecture d'un coup d'œil.
- **Trouver vos points de synthèse** — Top Bridges montre les notes faisant le plus de travail architectural.
- **Découvrir des communautés que vous ne connaissiez pas** — clusters émergents du graphe.
- **Combler les Angles Morts** — quand le graphe suggère deux notes DEVRAIENT être liées mais ne le sont pas.
- **Planifier la réorganisation** — communautés correspondent naturellement à la structure de dossiers.

## CNS vs Sight — Quand Utiliser Quoi

- **Sight** = « Comment mon univers est-il FAÇONNÉ ? » Analyse spatiale / catégorielle.
- **CNS** = « Comment mon univers est-il CONNECTÉ ? » Analyse réseau / topologique.

Ce sont complémentaires : Sight lit la surface ; CNS lit le câblage dessous.

## Surfaces Associées

- **Constellation Sight** — la visualisation sœur (icône œil dans le dock).
- **Sky View** — aussi vue graphe, mais construite différemment.
- **Panneaux Backlinks / Outgoing Links** — listes de connexion par note.
