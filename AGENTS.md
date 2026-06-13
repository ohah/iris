# Iris 에이전트 인덱스

이 문서는 Iris에서 작업하는 에이전트가 가장 먼저 읽는 진입점이다. 실제 규칙과 설명은 링크된 문서를 단일 출처로 둔다.

## 먼저 읽을 문서

- [개발 명령](docs/development-commands.md)
- [필수 프로젝트 규칙](docs/project-rules.md)
- [PR 체크리스트](docs/pr-checklist.md)
- [파일/폴더 구조](docs/project-structure.md)
- [레퍼런스](docs/references.md)
- [라벨 정책](docs/labels.md)
- [성능 예산](docs/performance-budget.md)

## 핵심 원칙

- Iris v1은 React Native 최신 안정 버전과 Hermes V1의 관측 가능한 동작 호환성을 최우선으로 둔다.
- 기존 React Native 앱 코드 수정을 요구하는 최적화는 기본 경로에 넣지 않는다.
- 성능 주장은 기준선, 기기, OS, 빌드 타입, 반복 횟수, 측정 도구가 있는 벤치마크로만 주장한다.
- JSI/C++/Rust 경계는 소유권, lifetime, thread affinity를 명시한다.
- 외부 런타임 소스는 `references/` 아래 read-only 참고 자료로만 두고 커밋하지 않는다.
- PR 제목/본문과 커밋 설명은 한국어로 작성한다. conventional-commit prefix와 기술 식별자는 원문을 유지한다.
- 필요한 PR 라벨이 없으면 `mise run sync-labels`로 기본 라벨 세트를 생성하거나 갱신한다.
- 구현이 문서화된 전략과 달라지거나 문서에 없는 아키텍처 결정이 필요하면 사용자에게 먼저 보고한다.
