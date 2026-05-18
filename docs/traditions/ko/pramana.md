---
id: pramana
name: pramāṇa
family: indian-nyaya
shape: sectoral
version: 1
changelog:
  - 2026-05-18 initial (Phase ι.1)
translation_status: AI-generated 2026-05-18 — native-speaker review recommended
---

# pramāṇa

**계열**: 인도 니야야 · **형태**: 부채꼴(4사분면)

## 핵심 메타포

돔은 **타당한 앎의 네 사분면**으로 나뉘며, 각 사분면은 일종의 인식론적
근거를 가진 노트들을 담습니다. 지식은 *얼마나 성숙했는지*(Aristotelian)
가 아니라 *어떻게 알려졌는지*에 따라 분류됩니다: 직접 지각, 증거로
부터의 추론, 알려진 사례로부터의 유추, 신뢰할 수 있는 증언을 통해서.
pramāṇa들은 **종류이지 수준이 아닙니다** — 한 사분면에서 다른 사분면
으로 노트를 옮기는 것은 보증의 변화이며, Confidence의 변화가 아닙니다.

각 사분면 내에서 Aristotelian의 방사형 Stratum 인코딩이 유지되어,
노트의 이해 깊이가 보증 종류 내에서도 읽기 가능한 채로 유지됩니다.
§δ.2-fix-1(2026-05-17) 이후, 사분면은 원래의 NE/SE/SW/NW 대신 E/S/W/N
에 위치하여 수직 축을 Stratum 라벨 충돌에서 해방시킵니다.

## 범위

**이 전통을 사용해야 할 때.** 지식이 어떻게 *근거를 갖추고 있는지*를
한눈에 보고자 할 때 — 작업의 어느 정도가 직접 관찰에, 추론된 결론에,
비교에, 권위에 의존하는지의 비율. 인식론적 자기-감사에 유용: 증언에
과도하게 의존하고 있는가? 추론이 받을 자격이 없는 무게를 지고 있는가?

**이 전통을 사용하면 안 될 때.** 노트 전체에 걸쳐 보증이 변하지 않을
때 — 예를 들어, 모두 체험에 관한 Universe(모두 pratyakṣa) 또는 모두
인용 기반(모두 śabda)은 이 렌즈 아래에서 유용한 구조를 드러내지 않습
니다. 또한 깨끗한 지식-원천 분류를 허용하지 않는 콘텐츠(창작, 사변,
픽션)에도 부적합합니다.

## 적용 가능성

- 연구 프로젝트 전체에 걸친 인식론적 균형의 자기-감사.
- 1차 자료와 2차 자료를 한눈에 구별하기.
- 지식의 인지-행위 분석을 가르치기.

## 계보

고전 인도 니야야 — 인식이 발생하는 타당한 수단을 열거함으로써 인지를
분석한 형식 인도 인식론 학파. 4-pramāṇa 니야야 정전이 Constellation이
제공하는 버전(다른 인도 학파들은 수가 다릅니다 — 상키야는 3개, 미맘사
는 6개를 인정). 수트라 시대 인도부터 중세 주석까지; 오늘날에도 B. K.
Matilal, J. N. Mohanty 등의 저작을 통해 살아있는 전통입니다.

## 비평

4-pramāṇa 니야야 변종을 선택하는 것 자체가 학문적 입장 — 미맘사의
6-pramāṇa 관점(*arthāpatti* 가정과 *anupalabdhi* 비파악 추가)은 종교
계보 규칙(오리엔테이션 v2.09)에 의해 명시적으로 제외되었습니다,
베다 권위에 기반하기 때문에; 불교 Pramāṇavāda 전통(Dignāga, Dharmakīrti)
도 마찬가지로 제외되었습니다. 다른 인도-철학 계보의 사용자들은
Constellation 렌더링을 환원적이라고 느낄 수 있습니다.

## 인용

**1차 자료.** *Nyāya-Sūtra* 1.1.3(4 pramāṇa의 열거). Gautama, *The
Nyāya Sūtras of Gautama*, trans. Satisa Chandra Vidyābhūṣana, rev. ed.
Nandalal Sinha (Delhi: Motilal Banarsidass, 1990)에서 이용 가능.

**현대.** J. N. Mohanty, *Classical Indian Philosophy* (Lanham:
Rowman & Littlefield, 2000), 17–34; Bimal Krishna Matilal,
*Perception: An Essay on Classical Indian Theories of Knowledge*
(Oxford: Clarendon Press, 1986), ch. 1.

## 노트별 frontmatter

`pramana_kind: pratyaksha | anumana | upamana | shabda`. Rust 측
`LayoutCacheRow` 확장이 도착하면, 이 필드가 기본 배치(현재 모든 노트
→ `pratyaksha`)를 재정의합니다. 철학적 기본값은 방어 가능합니다: 모든
지식은 반성적으로 재분류되기 전까지 지각으로 시작합니다.
