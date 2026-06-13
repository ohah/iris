# React Native PoC

`apps/rn-bench`는 Iris 성능 실험의 첫 기준선 앱이다. 이 앱은 React Native 0.85, Hermes, New Architecture를 그대로 사용한다.

## 목적

- Hermes 순정 경로에서 JS 계산, JSON 객체 처리, 대량 리스트 렌더링 기준값을 얻는다.
- 이후 Iris JSI/Rust 경로를 붙일 때 같은 앱 안에서 A/B 비교한다.
- React Native/Hermes 100% 호환 규칙을 검증 가능한 기준으로 고정한다.

## 실행

```sh
mise run install
mise run rn-start
mise run rn-ios
mise run rn-android
```

iOS 네이티브 빌드 전에는 CocoaPods 설치가 필요하다.

```sh
cd apps/rn-bench/ios
bundle install
bundle exec pod install
```

## 검증

```sh
mise run rn-typecheck
mise run rn-test
mise run bench-smoke
mise run bench-js
mise run check
```

`mise run check`는 루트 Rust 검사와 함께 RN 앱 타입체크/Jest smoke test를 실행한다. 실제 기기 성능 측정은 CI 게이트가 아니라 별도 벤치마크 로그와 산출물로 관리한다.

## 측정 경계

현재 앱의 측정 버튼은 개발 중 빠르게 기준을 확인하기 위한 smoke benchmark다. `mise run bench-*` 명령은 같은 JS benchmark case를 로컬 산출물로 기록하지만 아직 CI 필수 체크는 아니다. 릴리스 성능 주장을 하려면 다음 조건을 별도 산출물에 기록해야 한다.

- 기기 모델, OS 버전, 빌드 타입
- React Native, Hermes, Iris commit
- 반복 횟수와 p50/p95
- cold start, TTI, dropped frames, JS long task, JSI transfer latency, memory
