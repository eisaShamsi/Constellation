# Epistemic Metadata

> **번역 참고:** 이 도움말 항목은 `help.uConstellation.World/Epistemic Metadata/Epistemic Metadata.md`에 있는 정식 영어판에서 AI가 생성한 번역입니다. 원어민 검토는 대기 중입니다. 수정 사항은 프로젝트 저장소를 통해 제출해 주세요.

*(MIG-022 §A — 갭 분석 §6.1 스키마 확장)*

이 항목은 노트의 더 풍부한 인식론적 분류를 위해 Constellation이 새로 인식하는 소수의 **선택적 프런트매터 필드**에 대해 설명합니다. 이들은 갭 분석(`docs/epistemic-content-gap-analysis.md`) — Constellation 인식적 콘텐츠 엔진(CECE)이 분류에 사용하는 2축 모델 "Source × Content Type"이 당신이 "어떻게 알게 되었는지"에 대해 기록하고 싶을 모든 것을 다 포착하지 못한다는 인식 — 에 대한 응답으로 추가되었습니다.

이러한 필드는 **모두 선택적**입니다. 이들 없이 기존 노트는 변경 없이 작동합니다. 노트가 추가 신호로부터 이익을 얻는 종류의 지식인 경우에 직접 (또는 향후에는 구조화된 에디터를 통해) 추가합니다.

---

## 필드 목록

### `held_by` — *이는 누구의 입장인가?*

노트가 기술하는 입장을 가진 사람을 나타내는 짧은 문자열. 기본값은 `user`(당신 자신의 입장)입니다. 사용할 수 있는 다른 값:
- 학자 이름: `held_by: "al-Shāfiʿī"`
- 학파 (madhhab): `held_by: "Ḥanafī"`
- 역사적 인물: `held_by: "Aristotle"`

당신 자신이 아닌 *다른 사람의* 입장을 기록하는 노트를 작성할 때, `held_by` 필드가 그것을 나타냅니다. 이것 없이는 Constellation이 노트의 인식론적 상태가 당신 자신의 것이라고 암묵적으로 가정하는데 — 진지한 학술 작업에서는 종종 잘못된 가정입니다.

### `domain` — *이것은 어떤 주제 분야에 관한 것인가?*

학문 분야 태그 목록. 자유 형식의 `tags` 필드(폭소노미 / 무드 / 프로젝트)와 구별되며, `domain`은 검색과 필터링을 위한 구조화된 학문 분야/주제 필드입니다. 예:

```yaml
domain: [fiqh, ʿibādāt]
domain: [photography, optics]
domain: [overland-travel, mechanics]
```

`content_type: "proposition"`와 `source: "inference"`로 분류된 노트는 논리 정리(domain: `[logic, mathematics]`)일 수도 있고 법적 견해(domain: `[fiqh, ʿibādāt]`)일 수도 있습니다 — 동일한 인식론적 형태이지만 매우 다른 검색 컨텍스트입니다. `domain`은 그것이 어느 것인지 말할 수 있게 합니다.

### `function` — *이 노트는 무엇을 위한 것인가?*

노트의 의도된 용도를 식별하는 단일 문자열. 인식되는 값:

- `reference` — 필요할 때 읽기 (정의, 인용, 나중에 찾아볼 사실)
- `seed` — 키우기 (아직 개발 중인 초기 단계 아이디어)
- `actionable` — 이것으로 무언가 하기 (작업, 후속 조치, 내릴 결정)
- `shipped` — 완성품 (출판된 에세이, 전달된 분석, 닫힌 루프)

CECE의 콘텐츠 타입 축(어떤 종류의 지식인지를 나타냄)과 구별됩니다 — `function`은 노트로 무엇을 *할* 것인지를 나타냅니다.

### `provenance_civilization` — *어떤 전통의 어휘가 작동하고 있는가?*

노트 어휘의 문명적 발자취를 식별하는 선택적 문자열. 전통 특유의 코퍼스에 대한 검색에 유용. 예:

- `provenance_civilization: "sunni-usuli"` — Sunni *uṣūl al-fiqh* 전통 (al-Bukhārī, al-Ghazālī, al-Āmidī)
- `provenance_civilization: "analytic-western"` — 프레게 이후 분석 철학
- `provenance_civilization: "nyaya"` — pramāṇa 인식론의 인도 Nyāya 학파
- `provenance_civilization: "buddhist-pramana"` — 불교 인식론적 전통 (Dignāga, Dharmakīrti)

대부분의 노트는 이것을 필요로 하지 않습니다. 예를 들어 Sunni *uṣūl*과 분석적 영미 인식론에 모두 의존하는 노트가 있을 때, 주요 발자취를 기록해 두면 미래의 당신이 적절한 비교 자료를 검색하는 데 도움이 됩니다.

### `updated_at` — *당신의 입장이 마지막으로 바뀐 것은 언제인가?*

노트의 인식론적 내용에 대한 가장 최근의 의도적인 수정의 ISO 날짜. 파일 시스템 `modified` 타임스탬프(오타 수정에 대해서도 모든 저장을 잡음)와 구별됩니다. `updated_at`은 실제로 입장을 다시 생각했을 때 *당신이* 설정하는 타임스탬프입니다.

```yaml
updated_at: 2026-05-09
```

§6.3 시간 축의 나머지(노트 상태 이력)가 완료될 때 유용합니다 — 그때까지 이것은 "내 견해를 마지막으로 수정한 시간"을 기록하는 단일 스냅샷 필드입니다.

### `ikhtilāf` — *구조화된 학술적 불일치*

새 필드 중 가장 복잡한 것. 어떤 문제에 대한 학자 또는 학파 간의 구조화된 불일치 — *ikhtilāf* — 를 `{school, position}` 쌍 목록으로 기록합니다. Constellation은 이것을 편집하기 위한 사용자 정의 Properties 패널 위젯을 제공합니다. YAML을 직접 편집할 수도 있습니다.

예:

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

`ikhtilāf`를 가진 노트는 어떤 단일 인식론적 상태에도 있지 않습니다 — 그것은 여러 행위자 사이의 *구조화된 불일치*를 기록합니다. 이 필드 없이는 Constellation이 그러한 노트를 마치 이 입장 중 하나를 자체적으로 견지하는 것처럼 처리할 것이며 — 이는 잘못입니다.

Properties 패널은 각 행을 두 개의 입력(school + position)과 제거 버튼을 가진 에디터 카드로 렌더링하고, 하단에 "학파 추가" 버튼을 표시합니다.

### `warrant`와 `warrant_notes` — *파싱되지만 (당분간) 비활성*

두 필드는 파싱되어 디스크에 저장되지만, **현재로선 어떤 UI에도 표면화되지 않습니다**:

- `warrant: "mutawātir"` — 노트의 주장에 대한 근거(warrant)의 등급 라벨. Sunni *uṣūl* 계층은 *mutawātir / mashhūr / āḥād*을 사용하고 특히 하디스 내에서는 *ṣaḥīḥ / ḥasan / ḍaʿīf / mawḍūʿ*를 사용합니다. 다른 전통은 자체 등급 어휘가 있습니다.
- `warrant_notes: "transmitted by 30+ companions in al-Bukhārī"` — warrant 등급을 뒷받침하는 자유 텍스트.

이들은 **Constellation Warrant Research 워크스트림**이 분류기를 출시할 때 사용 준비가 됩니다 (수개월 연구 프로젝트; 갭 분석 §6.2 참조). 그때까지는 직접 채울 수 있고 데이터는 유지되지만 아무 것도 표시하지 않습니다. 미래의 warrant 인식 쿼리와 배지는 이 값을 직접 읽습니다.

---

## 이러한 필드가 나타나는 곳

노트의 프런트매터에 새 필드를 채우면 다른 모든 YAML 필드와 같은 방식으로 **Properties 패널**(오른쪽 사이드바)에 나타납니다 — 키당 한 행, 타입에 적합한 에디터와 함께:

- `held_by`, `function`, `provenance_civilization`, `warrant`, `warrant_notes` → 텍스트 입력
- `domain` → 태그 목록 (입력 + Enter로 추가, 각 태그의 ×로 제거)
- `updated_at` → 날짜 선택기
- `ikhtilāf` → `school` / `position` 행과 추가/제거 버튼이 있는 사용자 정의 위젯

---

## `supersedes`는 어떤가?

`supersedes`는 기술적으로 단일 노트의 속성이 아닌 *노트 간의 관계*입니다. Constellation은 이것을 YAML 스칼라가 아닌 **타입드 링크**로 처리합니다:

```markdown
This note replaces my earlier analysis: [[old-note-id|supersedes]]
```

위키링크의 `|supersedes` 접미사는 Constellation에 이것이 `supersedes` 종류의 타입드 링크임을 알려줍니다 — 별도의 알약 색상(슬레이트 블루-그레이)을 가지고, 다른 타입드 링크와 함께 Backlinks + Outgoing Links 패널에 표시되며, Living Link Architecture(가중치, 라이프사이클, 순회 횟수)에 참여합니다.

이는 노트 간 관계를 한 곳 — 타입드 링크 시스템 — 에 유지하여 타입드 링크와 프런트매터 스칼라 사이에 분할되지 않도록 합니다. `contradicts:`에도 동일하게 적용됩니다 (MIG-022 이전 어휘에서 이미 타입드 링크).

---

## 이것이 *아닌* 것

이러한 필드는 오늘날 CECE 분류에 의해 **소비되지 않습니다**. CECE는 Source × Content Type만 분류합니다; 새 메타데이터 필드는 사람 주도 검색, 미래의 warrant 인식 분류기, 그리고 시간 축(완성될 때)을 위해 기록됩니다.

특히:
- `function: "actionable"`은 Tasks 패널에 작업을 자동 생성하지 *않습니다*
- `held_by: "al-Shāfiʿī"`는 CECE가 노트를 분류하는 방식을 변경하지 *않습니다*
- `domain: [fiqh]`는 검색 쿼리에 그것을 포함하도록 작성하지 않는 한 검색 결과를 필터링하지 *않습니다*

필드는 **스키마**입니다 — 추가할 수 있는 인식된 어휘. 향후 MIG는 이를 소비하는 기능을 출시할 것입니다 (warrant 분류기, 시간적 쿼리, 도메인 인식 필터링 등).

---

## 작업 예제

새벽 단식 의무가 그날의 유효성에 중요한지에 대한 Sunni 학파의 입장을 기록하는 노트:

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

The classical Mālikī position (one niyyah for the month) is described
by [[Ibn-Rushd-bidayah|derives-from]] in the bidāyat al-mujtahid passage
on niyyah. My current view: [[ramadan-niyyah-personal|supersedes]]
my earlier note that conflated the Mālikī position with the Shāfiʿī one.
```

새 필드 7개 중 6개가 채워짐; `warrant_notes`는 생략 (아직 기록할 전승 세부 사항 없음); `supersedes`와 `derives-from`은 YAML 스칼라가 아닌 본문의 타입드 링크로.

---

*MIG-022 §A — 이 스키마 확장은 이 Constellation 빌드에 포함됩니다. Warrant Research 워크스트림(별도 Concept Paper, 수개월)은 `warrant` 필드를 소비하는 warrant 분류기를 출시합니다. 시간 축(MIG-023, 별도 Architect 사이클)은 `updated_at` 및 더 광범위한 노트 상태 이력을 소비합니다.*
