# 필수 프로젝트 규칙

Iris에서 작업하는 모든 에이전트와 개발자는 이 규칙을 따른다.

## 기본 규칙

- `.mise.toml`에 지정된 Bun, Node.js, Rust 버전을 사용한다.
- Rust 포맷은 기본 `rustfmt`를 따른다. 별도 포맷 정책이 필요하면 먼저 문서화한다.
- JavaScript와 TypeScript lint/format은 Vite+와 VoidZero/Oxc 계열인 `oxlint`와 `oxfmt`를 사용한다.
- `main`에 직접 푸시하지 않는다. 항상 브랜치와 PR을 사용한다.
- 커밋 메시지는 conventional-commit prefix(`feat`/`fix`/`docs`/`test`/`refactor`/`build`/`ci`/`chore`)를 쓰고, prefix 뒤 제목과 본문은 한국어로 작성한다.
- PR 제목과 본문은 한국어로 작성한다. 외부 API 이름, 파일명, 명령어, 에러 메시지는 원문을 유지한다.
- 필요한 PR 라벨이 없으면 `mise run sync-labels`로 기본 라벨 세트를 생성하거나 갱신한다.
- 브랜치 이름은 `type/kebab-설명` 형식을 쓴다(예: `build/tooling-foundation`).

## Iris 호환성 원칙

- Iris v1의 최우선 목표는 React Native 최신 안정 버전과 Hermes V1의 관측 가능한 동작 호환성이다. 세부 기준은 [엔진 전략](engine-strategy.md)을 단일 출처로 둔다.
- 기존 React Native 앱 코드 수정을 요구하는 최적화는 기본 경로에 넣지 않는다.
- Hermes bytecode format 호환은 필수 목표가 아니다. Iris는 Hermes HBC 대신 자체 bundle pipeline, 자체 IR, QuickJS backend 같은 내부 구현을 선택할 수 있다.
- QuickJS, JavaScriptCore, Hermes, Lynx 등 외부 런타임은 동작 비교와 설계 참고 대상으로 사용한다. QuickJS는 React Native bootstrap/JSI 호환성 실험 backend로 허용하지만 Hermes 관측 가능 동작 shim과 검증 계획을 함께 둔다. V8은 iOS 동일 축이 없으므로 Hermes 대체 엔진 후보가 아니라 과거 사례 참고 대상으로만 다룬다. 코드 표현을 그대로 옮기지 않는다.
- 외부 런타임 소스는 `references/` 아래 read-only 참고 자료로만 사용한다. `references/`는 git에 커밋하지 않는 로컬 체크아웃이며, 무엇을 받는지는 [레퍼런스](references.md)를 단일 출처로 둔다(`mise run fetch-references`).
- 엔진, JSI, TurboModule, Fabric, Metro, source map, debugger 경계를 바꾸는 변경은 호환성 테스트와 벤치마크 계획을 먼저 남긴다.
- Iris 전용 bundle pipeline을 도입해도 앱 소스 코드는 Hermes 기준선과 같아야 한다. compiler, source hash, transform option, runtime backend는 benchmark artifact와 PR에 기록한다.
- 성능 주장은 수치로 검증한다. 기준선, 기기, OS, 빌드 타입, 반복 횟수, 측정 도구를 기록하지 않은 주장은 PR에서 성능 개선으로 쓰지 않는다.
- HBC strict microbenchmark, Iris bootstrap/frontier, native mirror 측정은 RN strict engine comparison과 구분한다. 분류 기준은 [엔진 전략](engine-strategy.md)의 benchmark 분류를 따른다.
- `core performance budget` 필수 체크는 항상 존재해야 한다. 실제 benchmark가 없는 단계에서는 placeholder 게이트임을 문서와 산출물에 명시한다.

## 의존성

- 런타임 의존성을 추가할 때는 왜 지금 필요한지 PR에 적는다.
- JSI/C++/Rust FFI 경계는 소유권, lifetime, thread affinity를 문서화한다.
- iOS/Android 플랫폼 의존성은 기본 RN 앱 호환성을 깨지 않는 opt-in 경로로 시작한다.

## 문서와 설명

- 새 코드에는 초보자가 의도를 이해할 수 있는 주석을 남긴다.
- 주석은 코드가 무엇을 하는지보다 왜 존재하는지를 설명해야 한다.
- 구현을 진행할 때마다 관련 문서가 실제 코드와 맞는지 확인한다.
- PR 설명, 문서 정합성, 전략 영향 평가는 [PR 체크리스트](pr-checklist.md)를 단일 출처로 둔다.
- GitHub PR에 Mermaid 흐름도를 넣을 때는 가능하면 세로 방향(`flowchart TD` 또는 `graph TD`)을 기본값으로 사용한다.

## 테스트와 관측 가능성

- 기능 구현 전에 가능한 한 검증 경로를 먼저 정한다.
- 자동 검증이 불가능하면 이유와 수동 검증 방법을 PR에 적는다.
- 벤치마크 산출물은 로컬 전용으로 두고, 회귀 fixture로 커밋할 때는 민감정보를 제거한다.
- `mise run check`가 통과하지 않는 변경은 완료로 보지 않는다.
