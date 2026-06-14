# 개발 명령

Iris는 `mise`를 개발 명령의 단일 진입점으로 둔다.

## 준비

```sh
mise install
mise run install
mise run fetch-references
mise run sync-labels
```

## 자주 쓰는 명령

```sh
mise run fmt
mise run fmt-check
mise run lint
mise run vp-check
mise run fetch-references
mise run sync-labels
mise run perf
mise run bench-smoke
mise run bench-js
mise run bench-android-release
mise run bench-android-release-repeat
mise run bench-android-engine-compare-check
mise run bench-android-engine-compare-local-check
mise run bench-android-engine-compare
mise run bench-android-iris-hbc-gap-local
mise run bench-android-iris-hbc-exec-local
mise run bench-android-iris-hbc-trace-local
mise run bench-android-iris-bootstrap-local
mise run bench-extract-fixture
mise run bench-extract-hermes
mise run bench-extract-release-fixture
mise run bench-extract-hermes-release
mise run rn-start
mise run rn-ios
mise run rn-android
mise run rn-ios-pods
mise run rn-ios-build-debug
mise run rn-ios-build-release
mise run rn-android-build-debug
mise run rn-android-build-release
mise run rn-android-build-iris-engine
mise run rn-android-build-iris-release
mise run rn-android-build-iris-release-local
mise run rn-android-build-engine-comparison
mise run rn-android-build-engine-comparison-local
mise run rn-codegen
mise run rn-typecheck
mise run rn-test
mise run rust-check
mise run test
mise run check
```

## 원칙

- JavaScript와 TypeScript는 Vite+와 VoidZero/Oxc 계열인 `oxlint`와 `oxfmt`를 사용한다.
- Rust는 별도 `rustfmt.toml` 없이 기본 `rustfmt` 스타일을 사용한다.
- Bun은 `package.json`의 `packageManager`와 `.mise.toml`의 `bun = "1.3.14"`로 고정한다.
- Vite+는 로컬 `vite-plus` 패키지와 `bunx vp`로 실행한다.
- React Native PoC 앱은 `apps/rn-bench`에 두고 `mise run rn-*` 명령으로 실행한다.
- `rn-ios-build-debug`와 `rn-android-build-debug`는 네이티브 연결 확인용 로컬 빌드이며 성능 기준선으로 쓰지 않는다. Android debug는 Hermes flavor를 기준으로 한다.
- `rn-ios-build-release`와 `rn-android-build-release`는 최적화된 RN/Hermes 기준선 수집 전 release 빌드가 가능한지 확인한다. Android release는 Hermes flavor를 기준으로 한다.
- `rn-android-build-iris-engine`은 로컬 Iris Android 엔진 AAR skeleton을 빌드한다.
- `rn-android-build-iris-release`는 `IRIS_ENGINE_AAR`가 실제 RN 호환 Iris 엔진 artifact를 가리킬 때만 Iris release APK를 빌드한다. AAR 계약은 `docs/iris-android-engine-contract.md`를 따른다.
- `rn-android-build-iris-release-local`은 로컬 skeleton AAR을 `irisRelease`에 주입해 앱과 엔진 AAR 계약, Iris-owned JSI runtime 생성, RN 초기화용 host surface, Hermes bytecode header 검증 경계를 확인한다.
- `rn-android-build-engine-comparison`은 Hermes release와 실제 Iris release APK를 모두 빌드한다. 실제 비교 실행 전 기본 빌드 경로다.
- `rn-android-build-engine-comparison-local`은 Hermes release와 로컬 skeleton 기반 Iris release를 모두 빌드한다. 성능값이 아니라 비교 preflight 검증용이다.
- `bench-android-release-repeat`는 Android 물리 기기의 release APK에서 3회 반복 측정 summary를 남긴다.
- `bench-android-engine-compare-check`는 측정 전 Hermes/Iris APK 존재, APK runtime boundary, generated-vs-packaged HBC 일치, Hermes/Iris packaged HBC bytecode parity를 확인한다.
- `bench-android-engine-compare-local-check`는 로컬 skeleton APK까지 빌드한 뒤 같은 preflight만 실행한다.
- `bench-android-engine-compare`는 Hermes/Iris release APK를 같은 물리 기기에서 순서대로 측정한다.
- `bench-android-iris-hbc-gap-local`은 로컬 skeleton Iris release HBC bundle을 빌드한 뒤 현재 Iris scalar executor coverage와 첫 미지원 opcode를 출력한다.
- `bench-android-iris-hbc-exec-local`은 같은 HBC bundle을 Rust scalar executor subset으로 실행해 완료 여부를 출력한다.
- `bench-android-iris-hbc-trace-local`은 같은 HBC bundle을 실행하며 완료 trace 또는 첫 semantic frontier trace를 출력한다.
- `bench-android-iris-bootstrap-local`은 로컬 skeleton Iris release APK를 물리 기기에서 실행하고 Iris native bootstrap artifact를 추출한다. 현재 측정 범위는 HBC metadata parse, static coverage scan, scalar execution frontier이며 RN JS workload 비교값은 아니다.
- `bench-android-local-performance`는 로컬 skeleton 기준 Hermes release JS 기준선과 Iris native bootstrap/frontier/native mirror 측정을 연속 실행하고 `android-local-performance-report.json`을 생성한다. strict engine ratio는 기록하지 않고, case별 native mirror ratio만 `strictComparable=false`로 기록한다.
- `bench-android-local-performance-report`는 이미 생성된 Hermes/Iris summary에서 같은 리포트만 다시 생성한다.
- `bench-extract-hermes-release`는 release, Hermes, New Architecture, Iris module compute, TurboModule 경계 case를 모두 요구한다.
- 벤치마크 하네스는 `mise run bench-*` 명령으로 로컬에서만 실행한다. 아직 CI 필수 체크에는 넣지 않는다.
- `mise run check`가 PR 전 기본 검증 경로이며 RN 타입체크와 smoke test를 포함한다.
