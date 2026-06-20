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
mise run bench-strict-hbc-engine-compare-smoke
mise run bench-strict-hbc-engine-compare
mise run bench-strict-hbc-engine-compare-repeat
mise run bench-strict-hbc-compare-artifacts
mise run bench-strict-hbc-compare-gate
mise run bench-strict-hbc-call-targets
mise run bench-strict-hbc-global-access
mise run bench-strict-hbc-math-lookup
mise run bench-strict-hbc-profile
mise run bench-strict-hbc-source-shape
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
- `bench-strict-hbc-engine-compare-smoke`와 `bench-strict-hbc-engine-compare`는 같은 Hermes bytecode 파일을 Hermes runner와 Iris scalar executor에 실행하는 host-side strict HBC 비교다. 정식 비교 명령은 `--rounds=2`로 Hermes-first/Iris-first 실행 순서와 case 순서를 교차한다. `--sample-inner-iterations=N`은 sub-ms case에서 한 measured sample 안에 같은 HBC 실행을 N번 묶어 timer jitter 비중을 줄인다. `--iris-fast-paths=off`는 Iris exact/shape fast path를 끄고 준비된 함수의 일반 scalar interpreter 경로만 재서 남은 opcode 실행 비용을 분리하는 진단 옵션이다. RN release app, JSI, Fabric, TurboModule 경계 비교값은 아니다.
- `bench-strict-hbc-engine-compare-repeat`는 같은 strict HBC 비교를 여러 번 실행하고 첫 noisy run을 제외한 stability summary를 만든다. 기본 안정성 기준은 Iris p50 relative spread지만, sub/low-ms case는 `--max-absolute-spread-ms=N`을 명시해 절대 spread 완화 기준을 artifact에 남길 수 있다.
- `bench-strict-hbc-compare-artifacts`는 이미 생성된 strict HBC single artifact 또는 repeat summary 두 개를 비교해 Iris p50/p95와 Hermes 대비 ratio 변화를 출력한다.
- `bench-strict-hbc-compare-gate`는 같은 비교를 수행하되 checksum mismatch, unstable repeat summary, threshold 초과 회귀를 exit code 실패로 만든다.
- `bench-strict-hbc-global-access`는 같은 계산을 전역 `var` read/write와 top-level lexical binding으로 각각 실행해 global property 접근 비용만 분리하는 diagnostic 비교다.
- `bench-strict-hbc-math-lookup`은 native Math call 없이 반복 `Math.sin/sqrt` lookup 비용만 분리하는 diagnostic 비교다.
- `bench-strict-hbc-profile`은 strict HBC case별 Iris scalar executor 동적 opcode/property/call hot path를 텍스트와 JSON으로 출력한다. 기본 JSON 출력은 `artifacts/bench/strict-hbc-profile.json`이며 `--json-output=...`으로 바꿀 수 있다. 성능 ratio가 아니라 최적화 후보 선정용 계측이다.
- `bench-strict-hbc-source-shape`는 전역 `var` 기반 case와 top-level lexical binding case를 같은 HBC 비교 하네스에서 실행해 source shape가 Iris/Hermes ratio에 주는 영향을 분리한다. 기본 strict 비교에는 lexical diagnostic case를 자동 포함하지 않는다.
- `bench-android-engine-compare-check`는 측정 전 Hermes/Iris APK 존재, APK runtime boundary, generated-vs-packaged HBC 일치, Hermes/Iris packaged HBC bytecode parity를 확인한다.
- `bench-android-engine-compare-local-check`는 로컬 skeleton APK까지 빌드한 뒤 같은 preflight만 실행한다.
- `bench-android-engine-compare`는 Hermes/Iris release APK를 같은 물리 기기에서 순서대로 측정한다.
- Android strict 측정에서 앱이 `Run suite` UI까지 도달하지 못하면 runner는 실패시키되 지정된 `--log-output`에 startup log를 남긴다. 이 실패는 성능 열위가 아니라 RN JS startup/runtime compatibility blocker다.
- `bench-android-iris-hbc-gap-local`은 로컬 skeleton Iris release HBC bundle을 빌드한 뒤 현재 Iris scalar executor coverage와 첫 미지원 opcode를 출력한다.
- `bench-android-iris-hbc-exec-local`은 같은 HBC bundle을 Rust scalar executor subset으로 실행해 완료 여부를 출력한다.
- `bench-android-iris-hbc-trace-local`은 같은 HBC bundle을 실행하며 완료 trace 또는 첫 semantic frontier trace를 출력한다.
- `bench-android-iris-bootstrap-local`은 로컬 skeleton Iris release APK를 물리 기기에서 실행하고 Iris native bootstrap artifact를 추출한다. 현재 측정 범위는 HBC metadata parse, static coverage scan, relaxed/strict scalar execution이며 RN JS workload 비교값은 아니다.
- `bench-android-local-performance`는 로컬 skeleton 기준 Hermes release JS 기준선과 Iris native bootstrap/scalar execution/native mirror 측정을 연속 실행하고 `android-local-performance-report.json`을 생성한다. strict engine ratio는 기록하지 않고, case별 native mirror ratio만 `strictComparable=false`로 기록한다.
- `bench-android-local-performance-report`는 이미 생성된 Hermes/Iris summary에서 같은 리포트만 다시 생성한다.
- `bench-extract-hermes-release`는 release, Hermes, New Architecture, Iris module compute, TurboModule 경계 case를 모두 요구한다.
- 벤치마크 하네스는 `mise run bench-*` 명령으로 로컬에서만 실행한다. 아직 CI 필수 체크에는 넣지 않는다.
- `mise run check`가 PR 전 기본 검증 경로이며 RN 타입체크와 smoke test를 포함한다.
