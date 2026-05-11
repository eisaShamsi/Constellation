# Métadonnées Épistémiques

> **Note de traduction :** Cette rubrique d'aide est une traduction
> générée par IA à partir de la version canonique en anglais à
> `help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md`.
> La relecture par un locuteur natif est en attente. Veuillez signaler
> les corrections via le dépôt du projet.

*(MIG-022 §A — extensions de schéma issues de l'analyse des lacunes §6.1)*

Cette rubrique décrit un petit ensemble de **champs frontmatter optionnels** que Constellation reconnaît désormais pour une classification épistémique plus riche de vos notes. Ils ont été ajoutés en réponse à l'analyse des lacunes (`docs/epistemic-content-gap-analysis.md`) — la reconnaissance que le modèle à deux axes Source × Content Type contre lequel le Constellation Epistemic Content Engine (CECE) classifie ne couvre pas tout ce que vous pourriez vouloir consigner sur la manière dont vous en êtes venu à savoir ce que vous savez.

Ces champs sont **tous optionnels**. Les notes existantes sans eux fonctionnent sans changement. Vous les ajoutez à la main (ou, à l'avenir, via un éditeur structuré) lorsqu'une note est le genre de connaissance qui bénéficie du signal supplémentaire.

---

## Les champs

### `held_by` — *de qui est cette position ?*

Une courte chaîne indiquant qui détient la position décrite par la note. Par défaut, `user` (votre propre position). D'autres valeurs que vous pourriez utiliser :
- Le nom d'un savant : `held_by: "al-Shāfiʿī"`
- Une école : `held_by: "Ḥanafī"`
- Une figure historique : `held_by: "Aristotle"`

Lorsque vous écrivez une note qui consigne *la position de quelqu'un d'autre* plutôt que la vôtre, `held_by` est le champ qui le dit. Sans lui, Constellation suppose tacitement que l'état épistémique de la note est le vôtre — ce qui, pour un travail académique sérieux, est souvent erroné.

### `domain` — *de quel sujet s'agit-il ?*

Une liste d'étiquettes disciplinaires. Distinct de votre champ libre `tags` (folksonomie / humeur / projet), `domain` est le champ structuré de discipline/sujet pour la récupération et le filtrage. Exemples :

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

Une note classée comme `content_type: "proposition"` ET `source: "inference"` pourrait être un théorème logique (domain : `[logic, mathematics]`) ou une opinion juridique (domain : `[fiqh, ʿibādāt]`) — même forme épistémique, contextes de récupération très différents. `domain` vous permet de dire lequel.

### `function` — *à quoi sert cette note ?*

Une seule chaîne identifiant l'usage prévu de la note. Valeurs reconnues :

- `reference` — à lire en cas de besoin (une définition, une citation, un fait que vous consulterez plus tard)
- `seed` — à incuber (une idée à un stade précoce que vous développez encore)
- `actionable` — faire quelque chose avec ceci (une tâche, un suivi, une décision à prendre)
- `shipped` — produit fini (un essai publié, une analyse livrée, une boucle bouclée)

Distinct de l'axe content-type de CECE (qui dit de quel TYPE de connaissance il s'agit) — `function` dit ce que vous FEREZ avec la note.

### `provenance_civilization` — *quel vocabulaire traditionnel est à l'œuvre ?*

Une chaîne optionnelle identifiant l'empreinte civilisationnelle du vocabulaire de la note. Utile pour la récupération sur des corpus spécifiques à une tradition. Exemples :

- `provenance_civilization: "sunni-usuli"` — tradition sunnite *uṣūl al-fiqh* (al-Bukhārī, al-Ghazālī, al-Āmidī)
- `provenance_civilization: "analytic-western"` — philosophie analytique post-Frege
- `provenance_civilization: "nyaya"` — école indienne Nyāya d'épistémologie pramāṇa
- `provenance_civilization: "buddhist-pramana"` — tradition épistémologique bouddhiste (Dignāga, Dharmakīrti)

La plupart des notes n'en ont pas besoin. Lorsque vous avez, par exemple, une note qui s'appuie à la fois sur le *uṣūl* sunnite ET sur l'épistémologie analytique anglo-américaine, consigner l'empreinte primaire aide votre futur vous-même à retrouver le matériel comparable approprié.

### `updated_at` — *quand votre position a-t-elle changé pour la dernière fois ?*

Date ISO de la révision délibérée la plus récente du contenu épistémique de la note. Distinct de l'horodatage `modified` du système de fichiers (qui capte chaque enregistrement, même les corrections de coquilles) ; `updated_at` est l'horodatage que VOUS fixez lorsque vous avez réellement repensé la position.

```yaml
updated_at: 2026-05-09
```

Utile lorsque le reste de l'axe temporel §6.3 atterrira (historique des états de la note) — d'ici là, c'est un champ d'instantané unique qui consigne « la dernière fois que j'ai révisé mon point de vue ».

### `ikhtilāf` — *désaccord savant structuré*

Le plus complexe des nouveaux champs. Consigne l'*ikhtilāf* — le désaccord structuré entre savants ou écoles sur une question — sous forme de liste de paires `{school, position}`. Constellation fournit un widget personnalisé du panneau Propriétés pour l'éditer ; vous pouvez aussi modifier le YAML directement.

Exemple :

```yaml
ikhtilāf:
  - school: Ḥanafī
    position: permissible
  - school: Mālikī
    position: discouraged
  - school: Shāfiʿī
    position: permissible with conditions
  - school: Ḥanbalī
    position: forbidden
```

Une note avec `ikhtilāf` n'est dans aucun état épistémique unique — elle consigne un *désaccord structuré* entre plusieurs agents. Sans ce champ, Constellation traiterait une telle note comme si elle tenait elle-même l'une de ces positions, ce qui est faux.

Le panneau Propriétés rend chaque ligne comme une carte d'éditeur avec deux entrées (school + position) plus un bouton de suppression, et un bouton « Ajouter une école » en bas.

### `warrant` et `warrant_notes` — *parsés mais inertes (pour l'instant)*

Deux champs sont parsés et stockés sur disque mais **pas encore exposés dans aucune UI** :

- `warrant: "mutawātir"` — une étiquette de degré pour la garantie de l'affirmation de la note. La hiérarchie sunnite *uṣūl* utilise *mutawātir / mashhūr / āḥād* et au sein du hadith spécifiquement *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ*. D'autres traditions ont leurs propres vocabulaires de notation.
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — texte libre étayant le degré de garantie.

Ceux-ci sont prêts à l'emploi lorsque le **Constellation Warrant Research workstream** livrera son classificateur (projet de recherche de plusieurs mois ; voir l'analyse des lacunes §6.2). D'ici là, vous pouvez les remplir à la main et les données persistent ; rien ne les affiche. Les futures requêtes et badges sensibles à la garantie liront ces valeurs directement.

---

## Où ces champs apparaissent

Lorsque vous remplissez l'un des nouveaux champs dans le frontmatter d'une note, ils apparaissent dans le **panneau Propriétés** (barre latérale droite) de la même manière que tout autre champ YAML — une ligne par clé, avec l'éditeur approprié au type :

- `held_by`, `function`, `provenance_civilization`, `warrant`, `warrant_notes` → entrée texte
- `domain` → liste d'étiquettes (ajouter en tapant + Entrée, supprimer avec le × sur chaque étiquette)
- `updated_at` → sélecteur de date
- `ikhtilāf` → widget personnalisé avec lignes `school` / `position` + boutons ajouter/supprimer

---

## Et `supersedes` ?

`supersedes` est techniquement une *relation entre notes* plutôt qu'une propriété d'une note unique. Constellation le gère comme un **lien typé**, non comme un scalaire YAML :

```markdown
Cette note remplace mon analyse antérieure : [[old-note-id|supersedes]]
```

Le suffixe `|supersedes` sur le wikilink dit à Constellation qu'il s'agit d'un lien typé de la sorte `supersedes` — il reçoit une couleur de pastille distincte (bleu-gris ardoise), apparaît dans les panneaux Backlinks + Outgoing Links aux côtés d'autres liens typés, et participe à la Living Link Architecture (poids, cycle de vie, comptes de traversée).

Cela maintient les relations note-à-note en un seul endroit — le système de liens typés — plutôt que de les répartir entre liens typés et scalaires de frontmatter. La même chose s'applique à `contradicts:` (déjà un lien typé dans le vocabulaire pré-MIG-022).

---

## Ce que ce n'est PAS

Ces champs ne sont **PAS** consommés par la classification CECE aujourd'hui. CECE classifie uniquement sur Source × Content Type ; les nouveaux champs de métadonnées sont enregistrés pour la récupération pilotée par l'humain, les futurs classificateurs sensibles à la garantie, et l'axe temporel (lorsqu'il atterrira).

En particulier :
- `function: "actionable"` ne crée PAS automatiquement une tâche dans le panneau Tâches
- `held_by: "al-Shāfiʿī"` ne change PAS la façon dont CECE classifie la note
- `domain: [fiqh]` ne filtre PAS vos résultats de recherche à moins que vous n'écriviez la requête de recherche pour l'inclure

Les champs sont du **schéma** — un vocabulaire reconnu de champs que vous pouvez ajouter. De futurs MIGs livreront des fonctionnalités qui les consomment (classificateur de garantie, requêtes temporelles, filtrage sensible au domaine, etc.).

---

## Un exemple détaillé

Une note consignant les positions des écoles sunnites sur la question de savoir si l'aube qui se lève importe pour la validité du jour de jeûne obligatoire :

```yaml
---
title: Niyyah for Ramadan fasting
held_by: user
domain: [fiqh, ʿibādāt, sawm]
function: reference
provenance_civilization: sunni-usuli
updated_at: 2026-05-09
warrant: mashhūr
ikhtilāf:
  - school: Ḥanafī
    position: night-before niyyah valid; same-day niyyah valid before zawāl
  - school: Mālikī
    position: night-before niyyah required; one general niyyah for the month suffices
  - school: Shāfiʿī
    position: night-before niyyah required for each obligatory fast
  - school: Ḥanbalī
    position: night-before niyyah required for each obligatory fast
---

La position Mālikī classique (une niyyah pour le mois) est décrite
par [[Ibn-Rushd-bidayah|derives-from]] dans le passage sur la niyyah dans
bidāyat al-mujtahid. Mon point de vue actuel : [[ramadan-niyyah-personal|supersedes]]
ma note antérieure qui confondait la position Mālikī avec la position Shāfiʿī.
```

Six des sept nouveaux champs renseignés ; `warrant_notes` omis (pas encore de détail de chaîne de transmission à consigner) ; `supersedes` et `derives-from` comme liens typés dans le corps, non comme scalaires YAML.

---

*MIG-022 §A — les extensions de schéma atterrissent dans ce build de Constellation. Le Warrant Research workstream (Concept Paper séparé, plusieurs mois) livre le classificateur de garantie qui consomme le champ `warrant`. L'axe temporel (MIG-023, cycle Architect séparé) consomme `updated_at` plus l'historique plus large des états de note.*
