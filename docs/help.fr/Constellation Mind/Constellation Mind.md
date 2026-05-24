---
aliases:
  - Esprit de Constellation
  - Constellation Mind
  - Mind
  - LLM local
  - Grand Modèle de Langage local
  - Fanar
  - Chat IA
  - IA personnelle
description: Constellation Mind est la couche locale du Grand Modèle de Langage (LLM) de Constellation — une IA avec laquelle vous pouvez discuter de vos propres notes, fonctionnant entièrement sur votre appareil. La Phase 0b a été livrée le 2026-05-24 avec le modèle Fanar-1-9B prioritaire en arabe, installable depuis Paramètres → Mind. L'interface de chat arrive en Phase 1.
---

# Constellation Mind (عقل Constellation)

## Qu'est-ce que c'est ?

Constellation Mind est la couche locale du Grand Modèle de Langage (LLM) de Constellation — un assistant IA qui connaît votre Univers et peut converser avec vous au sujet de vos notes, **sans envoyer aucune d'elles dans le cloud**.

Trois choses le distinguent de tout autre outil « IA pour notes » :

1. **Local d'abord.** Le modèle s'exécute sur votre appareil. Vos notes ne le quittent jamais. Il n'y a pas d'aller-retour vers le cloud — le chat est local et capable de fonctionner hors ligne.
2. **Arabe d'abord.** Le modèle par défaut fourni est **Fanar-1-9B**, le modèle arabo-centré et conscient du contexte sunnite du Qatar Computing Research Institute. Compétence native en MSA et dialectes du Golfe ; l'anglais est la deuxième langue, pas la seule.
3. **Lié aux citations.** Chaque affirmation factuelle que l'IA fait à propos de vos notes doit citer la note source. Les citations hallucinées sont interceptées par un validateur post-génération (Phase 1).

## Ce qui est livré aujourd'hui (Phase 0b — 2026-05-24)

- **Panneau Paramètres → Mind** — liste les modèles installables (actuellement uniquement Fanar 1.9B Q4_K_M, ~5 Gio), avec un bouton Installer qui télécharge et vérifie le modèle.
- **Installation du modèle** — téléchargement par fragments depuis un GitHub Release (pas de cloud tiers), vérifié par SHA-256 par fragment et sur l'ensemble assemblé.
- **Runtime d'inférence réel** — `llama-cpp-2` (CPU uniquement en v1) charge le GGUF Q4_K_M et diffuse les tokens.
- **Pas encore d'interface de chat** — c'est la Phase 1 (le prochain jalon). Aujourd'hui, vous pouvez installer le modèle et le vérifier ; l'interface conversationnelle arrive avec MIG-048.

## Comment installer Fanar

1. Ouvrez **Paramètres → Mind**.
2. Trouvez **Fanar 1.9B (Q4_K_M)** dans le catalogue. La carte affiche la taille (5,01 Gio), la licence (Apache-2.0 avec avis défensifs Gemma) et un bouton « Définir comme actif » ou « Installer ».
3. Cliquez sur **Installer**. Une barre de progression montre le téléchargement + la vérification SHA + l'assemblage en trois phases.
4. Lorsque le badge passe à **Installé** + **Actif**, le modèle est prêt. Fanar réside dans `<app-data>/Constellation/models/fanar-1-9b-q4km-v1.gguf` et est adossé à mmap (pas de copie en RAM).

C'est tout. Jusqu'à ce que la Phase 1 livre l'interface de chat, le modèle installé est en attente.

## Ce qui arrive en Phase 1 (prochain jalon)

- **Interface de chat** — un panneau Constellation où vous parlez à Fanar de votre Univers en arabe ou en anglais (avec conscience RTL par message).
- **Outils de lecture** — Mind peut appeler `search_notes`, `read_note`, `find_similar`, `list_recent` pour ancrer ses réponses dans vos notes réelles.
- **Validateur de citations** — chaque affirmation cite une note réelle ; les références `note:UUID` fabriquées sont rejetées avant de vous atteindre.
- **Préchauffage au démarrage de l'application** — Mind se charge en arrière-plan pour que votre premier chat ne paie pas les 10 secondes de chargement à froid.
- **Historique des conversations** — sauvegardé par Univers ; promouvable en Note.

Voir `docs/Constellation-Mind-Concept-Paper-v1.1.md` pour l'architecture complète et `docs/Constellation-Mind-Implementation-Plan-v1.0.md` pour la feuille de route phase par phase.

## Ce qui arrive plus tard

- **Phase 2 — Outils d'écriture** (Mind propose des modifications / nouvelles notes / liens sous votre approbation explicite).
- **Phase 2.5 — RoutedProvider + Jais** (un second modèle, Jais-2-8B de G42/MBZUAI, rejoint Fanar en tant que co-défaut ; Mind les route en fonction de la requête).
- **Phase 3 — Auto-classification + liaison intelligente** (Mind propose des facettes et des liens à la sauvegarde de la note).
- **Phase 4 — Outils de capacité** (voix → note, OCR → note, traduction).
- **Phase 5 — Adhésion au cloud** (votre propre clé Anthropic / OpenAI, avec plafond de coût par Univers et journal de sortie par tour).

## Confidentialité et flux de données

- **HTTP sortant uniquement lors de l'installation d'un modèle** — Constellation télécharge les fichiers de modèle depuis les [`models/*` GitHub Releases](https://github.com/eisaShamsi/Constellation/releases) de ce dépôt. Pas de télémétrie. Pas d'inférence cloud (pour l'instant — c'est la Phase 5, et uniquement avec votre adhésion explicite).
- **Sur disque :** le GGUF du modèle + un registre `installed_models.json` qui suit les modèles que vous avez et celui qui est actif.
- **Au runtime :** le fichier de modèle chargé est mappé en mémoire ; vos invites et réponses ne vivent qu'en RAM.

## Licences

Chaque modèle porte sa propre LICENSE.txt à côté de lui dans le GitHub Release. Pour Fanar :

- **Apache License 2.0** (la licence déclarée par QCRI sur le dépôt Fanar-1-9B-Instruct).
- **Conditions d'utilisation de Gemma** — Fanar est un pré-entraînement continu de `google/gemma-2-9b` ; Constellation livre les avis Gemma défensivement même si QCRI réétiquette le résultat sous Apache-2.0 seul.
- **Citation Fanar** (Fanar Team 2025, arXiv:2501.13944).
- **Avis de redistribution Constellation** — le GGUF sur le GitHub Release de Constellation est une quantification des safetensors en amont de QCRI, produit par `.github/workflows/model-pipeline.yml` et distribué sous Apache-2.0 avec la LICENSE originale qui voyage.

La LICENSE.txt complète vit à côté de chaque modèle dans sa version : <https://github.com/eisaShamsi/Constellation/releases/tag/models/fanar-1-9b-q4km-v1>.

## Dépannage

**Badge « Pas encore prêt » au lieu du bouton Installer.** Le catalogue fourni a un SHA-256 espace réservé pour ce modèle. Cela ne devrait pas se produire sur une installation normale de Constellation ; si vous le voyez, le catalogue n'a pas été mis à jour pour cette version du modèle. Ouvrez une issue.

**L'installation se bloque à « Téléchargement partie X/Y ».** Problème de réseau. Annulez depuis Paramètres → Mind, redéclenchez l'installation — les fragments partiels sont nettoyés automatiquement.

**L'installation réussit, le SHA-256 du fichier ne correspond pas.** Un bit retourné au téléchargement. La réinstallation récupérera un fichier frais.

**Interface de chat manquante.** La Phase 1 (MIG-048) n'a pas encore été livrée. Le modèle peut être installé et vérifié aujourd'hui ; l'interface conversationnelle arrive dans la prochaine version.

---

*Les sous-sujets rejoindront ce dossier au fur et à mesure de la livraison de la Phase 1 : visite guidée de l'interface de chat, comportement de tap sur les puces de citation, sélecteur multi-modèles, rendu des longues conversations sur le second écran.*
