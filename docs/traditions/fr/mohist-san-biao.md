---
id: mohist-san-biao
name: Mohist sān biǎo
family: chinese-pragmatist
shape: horizontal-bands
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# Mohist sān biǎo (三表)

**Famille** : pragmatiste chinoise · **Forme** : bandes horizontales (3 zones)

## Métaphore principale

Le dôme se divise en **trois zones horizontales empilées de haut en
bas**, une par standard mohiste pour évaluer une doctrine :

- **本 běn (racine)** — haut. Précédent historique des rois-sages :
  la doctrine a-t-elle une garantie dans la tradition héritée ?
- **原 yuán (origine)** — milieu. Preuve d'observation directe : les
  gens ordinaires voient-ils et entendent-ils que c'est ainsi ?
- **用 yòng (usage)** — bas. Bénéfice social pratique : l'adoption de
  cette doctrine améliore-t-elle la vie des gens ?

Une doctrine vaut d'être tenue seulement si elle passe les trois
tests — mais le rendu de Sight vous laisse voir les notes distribuées
à travers les trois pour avoir une idée de quel type-de-garantie fait
le plus de travail dans votre univers.

L'axe horizontal ne porte pas d'encodage spécifique — les trois
standards mohistes sont *catégoriels*, pas ordinaux, donc le
positionnement à l'intérieur d'une bande se fait par jitter
déterministe par note.

## Portée

**Quand utiliser cette tradition.** Lors du travail avec un contenu
où le test est *si une doctrine vaut d'être tenue*, pas quelle sorte
de garantie la sous-tend. Utile pour le contenu de politique,
d'éthique, d'applicat-empirique, et de décision-pratique où le
précédent historique / l'observation / le bénéfice sont les trois
axes de justification.

**Quand NE PAS utiliser cette tradition.** Lorsque le contenu n'a
pas de dimension doctrinale ou évaluative. Le contenu purement
descriptif, le travail créatif et les notes sur l'expérience
subjective s'ajustent mal.

## Applicabilité

- Propositions de politique et leurs justifications.
- Analyse éthique comparée (cette règle passe-t-elle les trois
  tests ?).
- Ingénierie et science appliquée où le bénéfice-aux-gens est
  explicite.

## Lignée

Épistémologie pragmatiste chinoise classique. Mòzǐ 墨子 (~Ve s. av.
J.-C.) fonda l'école mohiste, qui se présentait comme une
alternative critique au confucianisme. Les sān biǎo apparaissent dans
le chapitre « Anti-Fatalisme » comme le test que les mohistes
appliquaient à la doctrine fataliste héritée — et concluaient qu'elle
échouait aux trois tests. L'école a brièvement fleuri puis a été
éclipsée par l'ascension confucéenne et légiste ; elle survit comme
un texte canonique récupérable étudié aujourd'hui à travers des
éditions comme le *Mòzǐ jiāngǔ*.

## Critique

Les sān biǎo sont parfois critiqués comme une forme précoce de
pragmatisme qui confond garantie évidentielle avec utilité — le
critère du « bénéfice aux gens » en particulier est difficile à
formaliser. Les érudits modernes débattent aussi si sān biǎo est une
théorie épistémique pleinement développée ou un outil
polémique-rhétorique déployé dans un argument anti-fataliste
spécifique. Inclus dans la ligne de base curée sous la règle de
lignage religieux malgré son contexte théologique-céleste, car le
cœur méthodologique est séculier.

## Citation

**Primaire.** *Mòzǐ* 墨子, Book IX, "Fēi Mìng Shàng" 非命上
("Anti-Fatalism, Part I"). Critical edition: Sūn Yíràng, ed., *Mòzǐ
jiāngǔ* 墨子閒詁, 2 vols. (Beijing: Zhonghua Shuju, 1986). English:
Ian Johnston, trans., *The Mozi: A Complete Translation* (New York:
Columbia University Press, 2010).

**Moderne.** A. C. Graham, *Disputers of the Tao: Philosophical
Argument in Ancient China* (La Salle, IL: Open Court, 1989), ch. 1;
Chris Fraser, "Mohism," *Stanford Encyclopedia of Philosophy* (2020).

## Frontmatter par note

`mohist_zone: ben | yuan | yong`. Actuellement absent — les notes
sont distribuées par hash déterministe par notePath dans les trois
zones de sorte que la structure visuelle soit peuplée. Lorsque
l'extension `LayoutCacheRow` côté Rust arrivera, ce champ remplace
l'assignation par hash-bucket.
