# PR 체크리스트

모든 PR은 기능 구현 여부와 상관없이 이 문서의 관점으로 평가한다.

GitHub PR 본문은 `.github/pull_request_template.md`를 사용한다. 이 문서는 템플릿에 들어가는 질문의 기준과 해석을 설명하는 단일 출처다.

## PR 메타데이터 (필수)

모든 PR은 다음을 반드시 갖춘다.

- 라벨: 하나 이상 단다.
- assignee: `ohah`로 지정한다.
- PR 제목과 본문: 한국어로 작성한다.
- 커밋 제목과 본문: conventional-commit prefix는 유지하고 설명은 한국어로 작성한다.
- 필요한 라벨이 없으면 `mise run sync-labels`로 기본 라벨 세트를 생성하거나 갱신한 뒤 사용한다.

```sh
# 새 PR
gh pr create --assignee ohah --label <label> ...

# 이미 만든 PR
gh pr edit <번호> --add-assignee ohah --add-label <label>
```

이 규칙은 `.github/workflows/pr-metadata.yml`가 확인한다. 체크 실패가 실제로 머지를 막으려면 GitHub branch protection에서 `PR metadata / require label and assignee=ohah`를 required check로 지정한다.

## PR 설명 필수 항목

```text
의도:
  이 PR이 해결하려는 문제

구현:
  어떤 책임 영역을 변경했는지

문서 정합성:
  관련 문서를 함께 수정했는지, 수정하지 않았다면 왜 필요 없었는지

호환성 근거:
  React Native/Hermes 관측 가능 동작과 앱 코드 마이그레이션 0 원칙을 유지하는지, 바뀐다면 왜 opt-in인지

전략 영향 평가:
  기존 Iris 전략과 충돌하는 부분이 있는지

테스트:
  실행한 명령과 결과

E2E/관측 가능성:
  어떤 snapshot, trace, benchmark, artifact 경로가 있는지

한계:
  자동 검증이 안 된 영역, 실제 구현 중 발견한 한계, 수동 검증 방법

사용자 논의 필요 여부:
  전략 수정이 필요한지, 문서에 없는 결정을 사용자와 논의했는지
```

## 전략 영향 평가

PR 설명에는 다음 질문에 대한 답이 있어야 한다.

- 이 PR은 기존 Iris 전략을 유지하는가?
- 필요한 라벨이 없을 때 임의 라벨 대신 기본 라벨 세트를 생성했는가?
- Hermes V1 관측 가능 동작 호환성과 기존 RN 앱 코드 무수정 원칙을 해치지 않는가?
- Hermes HBC 형식 호환을 제품 목표처럼 가정하지 않았는가?
- Iris 전용 bundle pipeline이나 backend를 바꿨다면 compiler/source hash/transform/runtime backend를 기록했는가?
- React Native 앱 코드 수정을 요구하지 않는가?
- JSI/C++/Rust lifetime과 thread affinity가 문서화되어 있는가?
- 테스트, E2E, 벤치마크 전략에 빈틈이 생기지 않았는가?
- 구현과 문서가 같은 상태를 설명하는가?
- 새 의존성이 추가되었다면 왜 지금 필요한가?
- 메모리/zero-copy 전략을 성급하게 복잡하게 만들지 않았는가?
- RN, TurboModule, Fabric, Metro, debugger, source map 경계를 나중에 막지 않는가?

## 서술 수준과 다이어그램

PR 본문은 리뷰어가 변경 코드를 직접 열지 않고도 "무엇을, 왜, 어떻게" 바꿨는지 재구성할 수 있을 만큼 자세히 적는다.

- PR 제목, PR 본문, 커밋 설명은 한국어로 작성한다. 기술 식별자, 파일명, API, 명령어, 에러 메시지는 원문을 유지한다.
- 위 필수 항목을 전부 채우고, 해당 없는 항목도 `N/A`와 이유를 적는다.
- 변경한 파일, 함수, 타입, ABI 버전을 실제 식별자 이름으로 짚는다.
- 동작을 바꿨다면 Before와 After를 대비해 적는다.
- 성능 주장은 측정값과 측정 조건을 함께 적는다.
- 성능 측정이 RN strict engine comparison, HBC strict microbenchmark, bootstrap/frontier, native mirror 중 무엇인지 명시한다.
- Android runtime 비교라면 `hermes-baseline`, `hermes-iris-bridge`, `quickjs-iris-bridge` 중 어떤 lane인지와 strict ratio 허용 여부를 명시한다.
- 구조, 흐름, 상태 전이가 있으면 GitHub가 렌더링하는 Mermaid 다이어그램을 가능하면 포함한다.
- GitHub PR의 Mermaid 흐름도는 가능하면 세로 방향(`flowchart TD` 또는 `graph TD`)으로 그린다. 가로 방향이 더 명확한 경우에만 `LR`을 사용하고 이유를 본문에 적는다.

Mermaid 문법은 GitHub에서 실제로 렌더돼야 한다.

- 노드 라벨 안에 escape된 따옴표(`\"`)를 넣지 않는다.
- 빈 노드(`X[ ]`/`X[]`)를 만들지 않는다.
- 라벨에 특수문자(`()` `:` `#` `/`)가 있으면 `["..."]`로 인용한다.
