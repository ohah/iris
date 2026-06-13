# 성능 예산

`core performance budget`은 branch protection에 걸리는 필수 체크다.

현재 Iris에는 React Native PoC 앱과 로컬 JS 벤치마크 하네스가 있지만 JSI adapter와 기기 벤치마크 하네스가 아직 없으므로, 이 체크는 성능 수치를 주장하지 않는다. 대신 성능 예산 하네스가 존재하고 산출물을 남기며, 실제 runtime 예산이 들어오기 전까지 필수 체크 이름을 안정화한다.

## 현재 게이트

```sh
mise run perf
```

현재 게이트는 다음을 검증한다.

- `artifacts/perf/core-budget.json` 산출물을 생성한다.
- 아직 runtime budget이 없음을 명시한다.
- 실제 성능 개선 주장을 하지 않는다.

## 실제 예산으로 승격할 때

Iris JSI adapter와 기기 벤치마크 하네스가 추가되면 이 문서를 수정하고 다음 지표를 최소 예산으로 둔다.

- cold start p50/p95
- time to interactive p50/p95
- FlatList dropped frame과 blank area 비율
- JS long task p95
- GC pause p95
- JSI transfer latency p50/p95
- peak RSS와 JS heap

측정값은 기기, OS, 빌드 타입, 반복 횟수와 함께 기록한다.
