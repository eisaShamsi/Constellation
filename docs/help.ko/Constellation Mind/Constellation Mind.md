---
aliases:
  - Constellation 마인드
  - Constellation Mind
  - Mind
  - 로컬 LLM
  - 로컬 대규모 언어 모델
  - Fanar
  - AI 채팅
  - 개인 AI
description: Constellation Mind는 Constellation의 로컬 대규모 언어 모델(LLM) 레이어입니다 — 자신의 노트에 대해 채팅할 수 있는 AI로, 전적으로 사용자의 기기에서 실행됩니다. 0b 단계는 2026-05-24에 설정 → Mind에서 설치할 수 있는 아랍어 우선 모델 Fanar-1-9B와 함께 출시되었습니다. 채팅 표면은 1단계에서 도착합니다.
---

# Constellation Mind (عقل Constellation)

## 무엇입니까?

Constellation Mind는 Constellation의 로컬 대규모 언어 모델(LLM) 레이어입니다 — 사용자의 우주를 알고 사용자의 노트에 대해 대화할 수 있는 AI 어시스턴트로, **그 어느 것도 클라우드로 전송하지 않습니다**.

다른 모든 "노트용 AI" 도구와 구별되는 세 가지 특징이 있습니다:

1. **로컬 우선.** 모델은 사용자의 기기에서 실행됩니다. 노트는 절대 떠나지 않습니다. 클라우드 왕복이 없습니다 — 채팅은 로컬이며 오프라인 지원입니다.
2. **아랍어 우선.** 번들된 기본 모델은 **Fanar-1-9B**로, 카타르 컴퓨팅 연구소(QCRI)의 아랍어 중심, 수니파 인식 모델입니다. MSA + 걸프 방언 네이티브 능력; 영어는 두 번째 언어이지 유일한 언어가 아닙니다.
3. **출처 기반.** AI가 사용자의 노트에 대해 하는 모든 사실 주장은 소스 노트를 인용해야 합니다. 환각된 인용은 생성 후 검증기에 의해 포착됩니다(1단계).

## 오늘 출시되는 것(0b 단계 — 2026-05-24)

- **설정 → Mind 패널** — 설치 가능한 모델을 나열합니다(현재 Fanar 1.9B Q4_K_M, ~5 GiB만), 모델을 다운로드하고 검증하는 설치 버튼이 있습니다.
- **모델 설치** — GitHub Release에서 청크 다운로드(제3자 클라우드 없음), 각 청크와 조립된 전체에서 SHA-256 검증.
- **실제 추론 런타임** — `llama-cpp-2`(v1에서 CPU 전용)가 Q4_K_M GGUF를 로드하고 토큰을 스트리밍합니다.
- **아직 채팅 표면 없음** — 그것은 1단계(다음 마일스톤)입니다. 오늘 모델을 설치하고 검증할 수 있지만, 대화 UI는 MIG-048에서 도착합니다.

## Fanar를 설치하는 방법

1. **설정 → Mind**를 엽니다.
2. 카탈로그에서 **Fanar 1.9B (Q4_K_M)**을 찾습니다. 카드는 크기(5.01 GiB), 라이선스(방어적 Gemma 공지가 있는 Apache-2.0), 그리고 "활성으로 설정" 또는 "설치" 버튼을 보여줍니다.
3. **설치**를 클릭합니다. 진행률 표시줄은 세 단계로 다운로드 + SHA 검증 + 조립을 보여줍니다.
4. 배지가 **설치됨** + **활성**으로 바뀌면 모델이 준비된 것입니다. Fanar는 `<app-data>/Constellation/models/fanar-1-9b-q4km-v1.gguf`에 있으며 mmap으로 백업됩니다(RAM에 복사 없음).

그게 전부입니다. 1단계가 채팅 표면을 출시할 때까지 설치된 모델은 대기 상태에 있습니다.

## 1단계에서 올 것(다음 마일스톤)

- **채팅 표면** — 아랍어 또는 영어로 우주에 대해 Fanar와 대화할 수 있는 Constellation 패널(메시지별 RTL 인식).
- **읽기 도구** — Mind는 `search_notes`, `read_note`, `find_similar`, `list_recent`를 호출하여 응답을 실제 노트에 기반을 둘 수 있습니다.
- **인용 검증기** — 모든 주장은 실제 노트를 인용합니다; 조작된 `note:UUID` 참조는 사용자에게 도달하기 전에 거부됩니다.
- **앱 시작 시 사전 워밍** — Mind가 백그라운드에서 로드되어 첫 채팅에서 10초의 콜드 로드를 지불할 필요가 없습니다.
- **대화 기록** — 우주별로 저장되며, 노트로 승격 가능합니다.

전체 아키텍처는 `docs/Constellation-Mind-Concept-Paper-v1.1.md`를, 단계별 로드맵은 `docs/Constellation-Mind-Implementation-Plan-v1.0.md`를 참조하세요.

## 나중에 올 것

- **2단계 — 쓰기 도구**(Mind가 명시적 승인 하에 편집 / 새 노트 / 링크를 제안합니다).
- **2.5단계 — RoutedProvider + Jais**(G42/MBZUAI의 두 번째 모델 Jais-2-8B가 Fanar와 함께 공동 기본값으로 합류하며, Mind는 요청에 따라 둘 사이를 라우팅합니다).
- **3단계 — 자동 분류 + 스마트 링크**(Mind가 노트 저장 시 패싯과 링크를 제안합니다).
- **4단계 — 능력 도구**(음성 → 노트, OCR → 노트, 번역).
- **5단계 — 클라우드 옵트인**(자신의 Anthropic / OpenAI 키, 우주별 비용 한도 및 턴별 송신 로그 포함).

## 개인정보 보호 및 데이터 흐름

- **모델 설치 시에만 아웃바운드 HTTP** — Constellation은 이 저장소의 [`models/*` GitHub Releases](https://github.com/eisaShamsi/Constellation/releases)에서 모델 파일을 다운로드합니다. 텔레메트리 없음. 클라우드 추론 없음(아직 — 그것은 5단계이며, 사용자의 명시적 옵트인이 있는 경우에만).
- **디스크 상:** 모델 GGUF + 보유한 모델과 활성 모델을 추적하는 `installed_models.json` 레지스트리.
- **런타임 시:** 로드된 모델 파일은 메모리 매핑되며, 프롬프트와 응답은 RAM에만 존재합니다.

## 라이선스

각 모델은 GitHub Release에서 자체 LICENSE.txt를 옆에 가지고 있습니다. Fanar의 경우:

- **Apache License 2.0**(Fanar-1-9B-Instruct 저장소에서 QCRI가 선언한 라이선스).
- **Gemma 사용 약관** — Fanar는 `google/gemma-2-9b`의 지속적인 사전 훈련입니다; QCRI가 결과를 Apache-2.0 단독으로 다시 레이블링하더라도 Constellation은 Gemma 공지를 방어적으로 출하합니다.
- **Fanar 인용**(Fanar Team 2025, arXiv:2501.13944).
- **Constellation 재배포 공지** — Constellation의 GitHub Release의 GGUF는 QCRI의 업스트림 safetensors의 양자화로, `.github/workflows/model-pipeline.yml`에 의해 생성되고 원본 LICENSE와 함께 Apache-2.0 하에 배포됩니다.

전체 LICENSE.txt는 각 모델의 릴리스 옆에 존재합니다: <https://github.com/eisaShamsi/Constellation/releases/tag/models/fanar-1-9b-q4km-v1>.

## 문제 해결

**설치 버튼 대신 "아직 준비되지 않음" 배지.** 번들된 카탈로그에는 해당 모델의 자리 표시자 SHA-256이 있습니다. 정상적인 Constellation 설치에서는 발생하지 않아야 합니다; 이를 본다면, 카탈로그가 해당 모델 버전에 대해 업데이트되지 않은 것입니다. issue를 여세요.

**설치가 "파트 X/Y 다운로드 중"에서 멈춥니다.** 네트워크 문제. 설정 → Mind에서 취소하고 설치를 다시 트리거하세요 — 부분 청크는 자동으로 정리됩니다.

**설치가 성공하지만 파일 SHA-256이 일치하지 않습니다.** 다운로드 시 비트 플립. 재설치하면 새로운 것을 가져옵니다.

**채팅 표면 누락.** 1단계(MIG-048)는 아직 출시되지 않았습니다. 모델은 오늘 설치하고 검증할 수 있으며, 대화 UI는 다음 릴리스에서 도착합니다.

---

*하위 주제는 1단계 출시와 함께 이 폴더에 합류합니다: 채팅 UI 둘러보기, 인용 칩 탭 동작, 다중 모델 선택기, 두 번째 화면에서 긴 채팅 렌더링.*
