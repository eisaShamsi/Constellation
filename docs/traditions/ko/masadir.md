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

**계열**: 수니파 이슬람 *uṣūl* · **형태**: 부채꼴(4사분면 + 4확장 칩)

## 핵심 메타포

돔은 수니파 *uṣūl al-fiqh*에서의 **권위 있는 증명의 네 원천**으로
나뉩니다: 꾸란, 순나, ijmāʿ(학자 합의), qiyās(유추 추론). 각각은
하나의 증명의 다른 *정도*가 아니라 다른 *종류*의 증명이며, 따라서
레이아웃은 부채꼴(범주적 슬라이스)이지 동심원(등급화된 깊이)이 아닙
니다. 돔 아래에는 네 개의 보조 원천이 칩으로 자리 잡습니다: *istiḥsān*
(법학적 선호), *istiṣḥāb*(연속성의 추정), *maṣlaḥa mursalah*(제한
없는 공공 이익), *ʿurf*(관습적 실천).

pramāṇa처럼 사분면은 +π/4 회전되었습니다(§θ-fix-1, 2026-05-18) —
수직 축을 Stratum 라벨에서 해방시키기 위해 — 그래서 기하학적 위치는
원래 문서화된 NE/SE/SW/NW 대신 이제 E/S/W/N입니다.

## 범위

**이 전통을 사용해야 할 때.** 수니파 이슬람 법-학문 추론으로 분석되
거나 분석될 수 있는 콘텐츠를 다룰 때. 도출에서 증명-종류의 균형을
보는 데 유용: 당신의 논증은 꾸란에 깊이 뿌리내리고 있는가? 합의에
의존하는가? qiyās가 대부분의 일을 하는가? 네 확장 칩은 고전 uṣūl이
헤드라인 네 원천 이상을 인정한다는 시각적 상기입니다.

**이 전통을 사용하면 안 될 때.** 비이슬람 콘텐츠에는 사분면 라벨이
의미가 없습니다. 프레임워크는 또한 특히 수니파 — 십이 이맘 시아 uṣūl
은 qiyās를 ʿaql(이성)로 대체하며, 종교 계보 규칙(오리엔테이션 v2.09)에
따라 의도적으로 포함되지 않았습니다. 신비주의, 철학, 문학 콘텐츠는
부적합합니다.

## 적용 가능성

- 수니 fiqh 도출, *uṣūl al-fiqh* 강의, fatwa 분석.
- 법-학문 저술에서 교차-원천 균형 감사.
- 고전 이슬람 법학의 증명-종류 구조를 가르치기.

## 계보

고전 수니파 uṣūl al-fiqh — 이슬람 법 추론의 원천과 방법의 학. 4원천
정전은 수니 4학파(하나피, 말리키, 샤피이, 한발리)에 걸쳐 관습적이며,
각 원천이 어떻게 가중되는지에 대한 내부 변이가 있습니다. Constellation
렌더링은 알-가잘리 『*Mustaṣfā*』 노선을 따릅니다.

## 비평

ijmāʿ를 *naṣṣ*(텍스트로 전달된) 클러스터가 아닌 *ijtihādī*(추론-도출)
클러스터에 배치하는 것은, ijmāʿ를 결속력 있게 전달된 것으로 다루는
Ashʿarī/Māturīdī kalām에 의해 논쟁됩니다. Constellation은 Mustaṣfā-
정렬 독해를 제공합니다; 대안적 kalām 독해는 v4.1 다듬기 목표입니다.
4원천 정전은 또한 4학파에 걸친 교리적 차이를 평탄화합니다 — 하나피
특정 또는 말리키 특정 변종 레지스터는 나중에 추가될 수 있습니다.

시아 uṣūl의 제외는 제품 설계 선택(오리엔테이션 v2.09의 종교 계보
규칙)이지, 학문적 판단이 아닙니다.

## 인용

**1차 자료.** Abū Ḥāmid al-Ghazālī, *al-Mustaṣfā min ʿilm al-uṣūl*,
ed. Ḥamza ibn Zuhayr Ḥāfiẓ (Medina: al-Jāmiʿa al-Islāmiyya, 1413/1993).

**현대.** Franz Rosenthal, *Knowledge Triumphant: The Concept of
Knowledge in Medieval Islam* (Leiden: Brill, 1970); Wael B. Hallaq,
*A History of Islamic Legal Theories* (Cambridge: Cambridge University
Press, 1997).

## 노트별 frontmatter

`masadir_source: quran | sunnah | ijma | qiyas`. Rust 측 `LayoutCacheRow`
확장이 도착하면, 이 필드가 기본 배치(현재 모든 노트 → 꾸란)를 재정의
합니다. 확장-칩 원천(`istihsan | istishab | maslaha | urf`)에 대한
노트별 옵트인은 후속 작업입니다.
