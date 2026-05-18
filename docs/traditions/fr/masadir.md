---
id: masadir
name: masādir
family: sunni-islamic-usul
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# masādir

**Famille** : uṣūl islamique sunnite · **Forme** : sectorielle (4 quadrants + 4 chips d'extension)

## Métaphore principale

Le dôme se divise en **quatre sources de preuve autoritative** dans
l'*uṣūl al-fiqh* sunnite : Coran, sunnah, ijmāʿ (consensus des
savants) et qiyās (raisonnement analogique). Chacune est un *type*
différent de preuve — pas un degré différent d'une preuve — et donc
la disposition est sectorielle (tranches catégorielles), pas
concentrique (profondeur graduée). Sous le dôme, quatre sources
supplémentaires se posent comme chips : *istiḥsān* (préférence
juridique), *istiṣḥāb* (présomption de continuité), *maṣlaḥa
mursalah* (intérêt public non restreint) et *ʿurf* (pratique
coutumière).

Comme pour pramāṇa, les quadrants ont été tournés de +π/4 (§θ-fix-1,
2026-05-18) pour dégager l'axe vertical des étiquettes de strate —
donc les positions géométriques sont maintenant E/S/O/N au lieu des
originellement documentés NE/SE/SO/NO.

## Portée

**Quand utiliser cette tradition.** Lors du travail avec un contenu
qui est ou pourrait être analysé comme raisonnement islamique
sunnite légal-savant. Utile pour voir l'équilibre des types-de-preuve
à travers une dérivation : votre argument est-il fortement
coranique-fondé ? S'appuie-t-il sur le consensus ? Le qiyās fait-il
l'essentiel du travail ? Les quatre chips d'extension sont des
rappels visuels que l'uṣūl classique reconnaît plus que les quatre
sources principales.

**Quand NE PAS utiliser cette tradition.** Pour un contenu
non-islamique, les étiquettes des quadrants n'ont aucun sens. Le
cadre est aussi spécifiquement sunnite — l'uṣūl chiite duodécimain
remplace le qiyās par ʿaql (raison) et est délibérément non inclus
par la règle de lignage religieux (orientation v2.09). Un contenu
mystique, philosophique et littéraire s'ajuste mal.

## Applicabilité

- Dérivation de fiqh sunnite, cours d'*uṣūl al-fiqh*, analyse de
  fatwa.
- Audit d'équilibre entre les sources dans l'écriture
  légale-savante.
- Enseignement de la structure types-de-preuve de la jurisprudence
  islamique classique.

## Lignée

Uṣūl al-fiqh sunnite classique — la science des sources et des
méthodes du raisonnement légal islamique. Le canon à quatre sources
est conventionnel à travers les quatre madhāhib sunnites (Hanafi,
Maliki, Shafiʿi, Hanbali), avec une variation interne sur la manière
dont chaque source est pondérée. Le rendu de Constellation suit la
ligne du *Mustaṣfā* d'al-Ghazālī.

## Critique

Le placement de l'ijmāʿ dans l'amas *ijtihādī* (raisonnement-dérivé)
plutôt que dans l'amas *naṣṣ* (textuellement-transmis) est contesté
par le kalām ashʿarī/māturīdī, qui traite l'ijmāʿ comme
liant-transmis. Constellation diffuse la lecture alignée sur le
Mustaṣfā ; la lecture alternative kalām est une cible de polissage
v4.1. Le canon à quatre sources aplatit également les différences
doctrinales entre les quatre madhāhib — un registre variant
spécifique-Hanafi ou spécifique-Maliki pourrait être ajouté plus
tard.

L'exclusion de l'uṣūl chiite est un choix de conception de produit
(règle de lignage religieux de l'orientation v2.09), pas un jugement
académique.

## Citation

**Primaire.** Abū Ḥāmid al-Ghazālī, *al-Mustaṣfā min ʿilm al-uṣūl*,
ed. Ḥamza ibn Zuhayr Ḥāfiẓ (Medina: al-Jāmiʿa al-Islāmiyya, 1413/1993).

**Moderne.** Franz Rosenthal, *Knowledge Triumphant: The Concept of
Knowledge in Medieval Islam* (Leiden: Brill, 1970); Wael B. Hallaq,
*A History of Islamic Legal Theories* (Cambridge: Cambridge University
Press, 1997).

## Frontmatter par note

`masadir_source: quran | sunnah | ijma | qiyas`. Lorsque l'extension
`LayoutCacheRow` côté Rust arrivera, ce champ remplace le placement
par défaut (actuellement toutes les notes → Coran). L'adhésion par
note via `istihsan | istishab | maslaha | urf` pour les sources des
chips d'extension est un suivi.
