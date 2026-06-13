# React Native PoC

`apps/rn-bench`는 Iris 성능 실험의 첫 기준선 앱이다. 이 앱은 React Native 0.85, Hermes, New Architecture를 그대로 사용한다.

## 목적

- Hermes 순정 경로에서 JS 계산, JSON 객체 처리, 대량 리스트 렌더링 기준값을 얻는다.
- Iris native module probe는 같은 앱 안에서 확인한다.
- Iris 엔진 대체 비교는 같은 앱 소스의 엔진별 release variant로 A/B 비교한다.
- Android Iris 엔진은 `IRIS_ENGINE_AAR`의 `IrisJSRuntimeFactoryProvider`가 제공하는 `JSRuntimeFactory`로만 연결한다.
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
mise run rn-ios-pods
```

## 검증

```sh
mise run rn-typecheck
mise run rn-test
mise run rn-codegen
mise run rn-android-build-debug
mise run rn-android-build-iris-engine
mise run rn-android-build-iris-release
mise run rn-android-build-iris-release-local
mise run rn-ios-build-debug
mise run rn-android-build-release
mise run rn-ios-build-release
mise run bench-smoke
mise run bench-js
mise run bench-android-release
mise run bench-android-release-repeat
mise run bench-android-engine-compare
mise run bench-extract-fixture
mise run bench-extract-release-fixture
mise run bench-extract-android-release-fixture
mise run check
```

`mise run check`는 루트 Rust 검사와 함께 RN 앱 타입체크/Jest smoke test를 실행한다. Debug 네이티브 빌드는 TurboModule 연결 확인용이며 실제 기기 성능 측정은 CI 게이트가 아니라 별도 release 벤치마크 로그와 산출물로 관리한다.

## 측정 경계

현재 앱의 측정 버튼은 개발 중 빠르게 기준을 확인하기 위한 smoke benchmark다. `mise run bench-*` 명령은 같은 JS benchmark case를 로컬 산출물로 기록하지만 아직 CI 필수 체크는 아니다. 개발 중 Hermes 앱 smoke 기준선은 Metro 로그의 `IRIS_BENCHMARK_ARTIFACT`를 `mise run bench-extract-hermes`로 추출해 `artifacts/bench/hermes-baseline.json`에 남긴다.

릴리스 성능 주장을 하려면 물리 기기에서 실행한 release 앱 로그의 `IRIS_BENCHMARK_ARTIFACT`를 `mise run bench-extract-hermes-release`로 추출해 `artifacts/bench/hermes-release-baseline.json`에 남긴다. Android 반복 측정은 `mise run bench-android-release-repeat`로 run별 report와 `artifacts/bench/hermes-release-baseline-summary.json`을 남긴다. Hermes/Iris 엔진 대체 비교는 `IRIS_ENGINE_AAR`로 실제 Iris 엔진 APK를 빌드한 뒤 `mise run bench-android-engine-compare`로 남긴다. Iris AAR 계약은 `docs/iris-android-engine-contract.md`를 따른다. 이 추출은 release build, Hermes 또는 Iris 엔진 runtime, New Architecture, Iris module compute, TurboModule number/string case를 모두 요구한다. RN 0.85 bridgeless Android에서는 `global.__turboModuleProxy`가 노출되지 않을 수 있으므로 전역 proxy 플래그가 아니라 실제 Codegen TurboModule benchmark case 실행 여부를 기준으로 삼는다. 산출물에는 다음 조건을 기록해야 한다.

- 기기 모델, OS 버전, 빌드 타입
- React Native, Hermes, Iris commit
- 반복 횟수와 p50/p95
- cold start, TTI, dropped frames, JS long task, JSI transfer latency, memory

TurboModule 경계 측정은 `docs/turbomodule-baseline.md`의 release 기준을 따른다.
