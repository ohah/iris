# 벤치마크 하네스

Iris 벤치마크는 먼저 로컬 산출물 계약을 고정하고, 성능 예산 게이트는 나중에 별도 PR에서 승격한다.

## 로컬 명령

```sh
mise run bench-smoke
mise run bench-js
mise run bench-android-release
mise run bench-android-release-repeat
mise run bench-android-engine-compare-check
mise run bench-android-engine-compare-local-check
mise run bench-android-engine-compare
mise run bench-android-iris-bootstrap-local
mise run bench-android-local-performance
mise run bench-android-local-performance-report
mise run bench-extract-fixture
mise run bench-extract-release-fixture
mise run bench-extract-android-release-fixture
mise run rn-codegen
mise run rn-android-build-debug
mise run rn-android-build-iris-engine
mise run rn-android-build-iris-release
mise run rn-android-build-iris-release-local
mise run rn-android-build-engine-comparison
mise run rn-android-build-engine-comparison-local
mise run rn-ios-build-debug
mise run rn-android-build-release
mise run rn-ios-build-release
```

- `bench-smoke`는 짧은 반복으로 하네스와 JSON schema가 동작하는지 확인한다.
- `bench-js`는 Hermes 기준선 앱과 같은 JS benchmark case를 로컬 JavaScript runtime에서 실행한다.
- `bench-android-release`는 Android 물리 기기에서 release APK 설치, 앱 실행, `Run suite`, 로그 저장, release artifact 추출을 한 번에 수행한다.
- `bench-android-release-repeat`는 같은 release APK에서 3회 반복 측정하고 run별 report와 summary artifact를 남긴다.
- `bench-android-engine-compare-check`는 현재 HBC 비교 모드 preflight다. Hermes/Iris APK와 bundle을 확인해 두 release variant가 byte-identical Hermes bytecode를 입력으로 쓰는지 검증한다. Iris 전용 bundle pipeline이 들어오면 같은 앱 소스와 artifact manifest parity를 검증하도록 확장해야 한다.
- `bench-android-engine-compare-local-check`는 로컬 skeleton AAR로 Hermes/Iris APK를 빌드하고 같은 preflight만 실행한다. 이 경로는 성능 비교를 만들지 않는다.
- `bench-android-engine-compare`는 Hermes release APK와 Iris release APK를 같은 하네스로 순서대로 측정하고 비교 artifact를 남긴다. Iris APK가 없으면 실패한다.
- `bench-android-iris-bootstrap-local`은 로컬 skeleton Iris release APK를 물리 기기에서 실행하고 `iris-engine-bootstrap` artifact를 logcat에서 추출한다. 현재 case는 HBC metadata parse, static coverage scan, scalar execution frontier다.
- `bench-android-local-performance`는 로컬 skeleton 기준으로 Hermes release JS 기준선과 Iris native bootstrap/frontier/native mirror 측정을 연속 실행한 뒤 성능 리포트를 남긴다. strict engine ratio는 금지하고, case별 native mirror ratio만 별도 필드에 기록한다.
- `bench-android-local-performance-report`는 이미 생성된 Hermes/Iris summary에서 같은 성능 리포트만 다시 생성한다.
- `bench-extract-fixture`는 fixture 로그에서 Hermes report를 추출해 파서와 검증 규칙을 확인한다.
- `bench-extract-release-fixture`는 release/Hermes/New Architecture/TurboModule case 요구 조건을 확인한다.
- `bench-extract-android-release-fixture`는 Android logcat의 quoted JSON 형식과 RN 0.85 bridgeless Android의 TurboModule case 기준 검증을 확인한다.
- `rn-codegen`은 TurboModule spec이 RN codegen에서 생성되는지 로컬에서 확인한다.
- `rn-android-build-debug`와 `rn-ios-build-debug`는 네이티브 연결 확인용이며 성능 기준선으로 쓰지 않는다. Android debug는 Hermes flavor를 기준으로 빌드한다.
- `rn-android-build-release`와 `rn-ios-build-release`는 최적화된 RN/Hermes 기준선 수집 전 release 빌드를 확인한다. Android release는 Hermes flavor를 기준으로 빌드한다. iOS task는 simulator build 확인용이며 최종 성능 기준선은 물리 기기에서 남긴다.
- `rn-android-build-iris-engine`은 로컬 Iris Android 엔진 AAR skeleton을 빌드한다.
- `rn-android-build-iris-release`는 `IRIS_ENGINE_AAR`가 실제 RN 호환 Iris 엔진 artifact를 가리킬 때만 `irisRelease` APK를 빌드한다.
- `rn-android-build-iris-release-local`은 로컬 skeleton AAR로 `irisRelease` APK 빌드 계약을 검증한다. 이 APK는 Iris-owned JSI runtime과 RN 초기화용 host surface를 만들지만 bundle 실행에서 의도적으로 실패하므로 성능 비교값으로 쓰지 않는다.
- `rn-android-build-engine-comparison`은 실제 Iris AAR을 사용해 Hermes/Iris release APK를 모두 빌드한다.
- `rn-android-build-engine-comparison-local`은 로컬 skeleton AAR을 사용해 Hermes/Iris release APK를 모두 빌드하고 preflight 재현성을 확인한다.
- 이 명령들은 CI 필수 체크에 포함하지 않는다.

## 산출물

```text
artifacts/bench/js-baseline-smoke.json
artifacts/bench/js-baseline.json
artifacts/bench/hermes-baseline-fixture.json
artifacts/bench/hermes-release-baseline-fixture.json
artifacts/bench/hermes-android-release-baseline-fixture.json
artifacts/bench/hermes-baseline.json
artifacts/bench/hermes-release-baseline.json
artifacts/bench/hermes-release-baseline-run-*.json
artifacts/bench/hermes-release-baseline-summary.json
artifacts/bench/iris-release-baseline-run-*.json
artifacts/bench/iris-release-baseline-summary.json
artifacts/bench/iris-bootstrap-baseline-run-*.json
artifacts/bench/iris-bootstrap-baseline-summary.json
artifacts/bench/android-local-performance-report.json
artifacts/bench/android-engine-comparison.json
artifacts/bench/rn-release-hermes-run-*.log
artifacts/bench/rn-release-iris-run-*.log
artifacts/bench/rn-release-iris-bootstrap-run-*.log
```

산출물 schema는 `iris.benchmark.v1`이며 다음 정보를 포함한다.

- app, platform, build, React Native, runtime metadata
- benchmark case id, 설명, warmup 횟수, measured 횟수
- sample, min, max, mean, p50, p95
- checksum과 detail

## 앱 내 실행

`apps/rn-bench`에서 `Run suite` 버튼을 누르면 같은 benchmark case를 Hermes 런타임에서 실행하고 `IRIS_BENCHMARK_ARTIFACT` 로그로 JSON report를 출력한다.

## 엔진 비교 원칙

Iris가 Hermes를 대체하는 엔진으로 들어가는 비교는 독립 앱 두 개가 아니라 같은 `apps/rn-bench` 소스의 엔진별 release variant 두 개로 수행한다. 제품 목표는 Hermes HBC 형식 호환이 아니라 React Native/Hermes 관측 가능 동작 호환과 기존 앱 코드 마이그레이션 0이다. 기준은 다음과 같다.

- `Hermes release`와 `Iris release`는 같은 benchmark JS, 같은 RN 버전, 같은 native module surface를 사용한다.
- Iris 전용 bundle pipeline을 사용해도 앱 소스는 Hermes 기준선과 같아야 하며 compiler, source hash, transform option, runtime backend를 artifact에 기록해야 한다.
- HBC 비교 모드에서는 두 release variant의 generated `index.android.bundle`과 APK 내부 `assets/index.android.bundle`이 Hermes bytecode magic, bytecode version, source hash, file length, SHA-256까지 같아야 한다.
- Hermes APK에는 `libhermesvm.so`가 있어야 하고 `libirisengine.so`/`libjsc.so`가 없어야 한다.
- Iris APK에는 `libirisengine.so`가 있어야 하고 `libhermesvm.so`/`libjsc.so`가 없어야 한다. `libhermestooling.so`는 RN의 hermesc packaging 경로 때문에 허용한다.
- 앱 프로젝트를 복제하지 않는다. 복제 앱은 Gradle, RN, ProGuard, Codegen, assets 설정이 drift되어 비교 신뢰도를 떨어뜨린다.
- Iris 엔진 artifact가 준비되기 전에는 JSC나 Hermes를 `Iris release` 대체값으로 쓰지 않는다.
- QuickJS backend는 Iris-owned runtime 내부 구현으로 사용할 수 있다. 단 RN 앱 코드 수정을 요구하지 않고 Hermes 관측 가능 동작 shim과 검증 계획을 제공해야 한다.
- V8은 iOS 동일 비교축이 없으므로 기본 Hermes 대체 엔진 후보나 벤치마크 비교 대상으로 두지 않는다.
- Hermes/JSC fallback을 Iris 성능값으로 측정하지 않는다.
- 현재 `iris-module-native-compute`는 엔진 대체 성능값이 아니라 Hermes 앱 안에서 Iris native module 경로를 확인하는 probe다.

Android는 `engine` flavor를 사용한다. 두 flavor는 같은 앱 소스에서 빌드되지만 설치 ID와 런처 이름이 달라 같은 물리 기기에 동시에 설치할 수 있다.

- `hermesRelease`: 기본 기준선이며 앱 ID는 `com.iris.bench.hermes`, 런처 이름은 `IrisBench Hermes`다.
- `irisRelease`: 앱 ID는 `com.iris.bench.iris`, 런처 이름은 `IrisBench Iris`다. `IRIS_ENGINE_AAR`가 실제 RN 호환 Iris 엔진 artifact를 가리킬 때만 빌드한다.

Iris release APK는 다음처럼 빌드한다.

```sh
IRIS_ENGINE_AAR=/absolute/path/to/iris-engine.aar mise run rn-android-build-iris-release
```

Iris AAR은 `docs/iris-android-engine-contract.md`의 `IrisJSRuntimeFactoryProvider.create(): JSRuntimeFactory` 계약을 제공해야 한다. 이 계약이 없으면 `irisRelease`는 컴파일 단계에서 실패한다.

로컬 skeleton AAR로 계약만 확인할 때는 다음 명령을 사용한다.

```sh
mise run rn-android-build-iris-release-local
```

이 APK는 `libirisengine.so`와 현재 skeleton의 HBC bootstrap artifact가 같이 패키징되고 RN이 Iris-owned JSI runtime 객체와 초기화용 host surface를 받는지 확인하기 위한 것이다. 아직 Iris runtime의 JS 실행 기능이 구현되지 않았으므로 Hermes와의 RN JS workload 비교값으로 쓰지 않는다. RN 0.85 기준 local skeleton HBC gap의 현재 coverage는 `supportedInstructions=2888/2888`, `supportedUniqueOpcodes=46/46`, `unsupportedUniqueOpcodes=0`, `firstUnsupported=none`이다. 현재 Rust scalar executor는 같은 bundle에서 bounded scalar execution을 시도하고, `SCALAR_EXECUTION_GLOBAL_STEP_LIMIT`에 도달하면 `status=frontier`와 차단 함수를 detail에 남긴다. 이는 module factory와 React 앱 실행 완료, Hermes/RN 실행 호환성 완료, 성능 우위를 의미하지 않는다.

로컬에서 APK 실행 없이 같은 report를 확인할 때는 다음 명령을 사용한다.

```sh
mise run bench-android-iris-hbc-gap-local
mise run bench-android-iris-hbc-exec-local
mise run bench-android-iris-hbc-trace-local
```

Iris native bootstrap 비용은 물리 기기에서 다음 명령으로 측정한다.

```sh
mise run bench-android-iris-bootstrap-local
```

이 명령은 `irisRelease` APK를 설치하고 앱 시작 중 `IRIS_BENCHMARK_ARTIFACT_CHUNK` logcat payload를 모아 `iris-engine-bootstrap` report를 만든다. 현재 bootstrap case는 `iris-hbc-metadata-parse`, `iris-hbc-static-coverage-scan`, `iris-hbc-scalar-execution-frontier`다. frontier case는 Rust scalar executor가 실제로 실행을 시도해 완료하거나 첫 의미론 차단점까지 간 시간을 측정한다. 현재 RN 0.85 local skeleton bundle에서는 `detail=status=frontier, error=Iris scalar executor function 3 exceeded step limit 50000`처럼 bounded execution frontier를 남긴다. RN JS bundle의 module factory와 React 앱을 끝까지 실행하는 단계는 아니므로 Hermes release의 `rn-hermes-js-baseline`과 strict ratio를 계산하지 않는다.

같은 artifact에는 다음 native mirror case도 포함한다. 이들은 Hermes JS/TurboModule case와 sample shape를 맞춘 C++ probe지만 JavaScript 실행이나 JS/TurboModule boundary를 통과하지 않는다.

- `iris-native-js-compute-mirror`
- `iris-native-json-round-trip-mirror`
- `iris-native-object-traversal-mirror`
- `iris-native-typed-array-copy-mirror`
- `iris-native-number-round-trip-mirror`
- `iris-native-string-round-trip-mirror`
- `iris-native-module-compute-mirror`

Hermes release 기준선과 Iris bootstrap/frontier 측정을 같은 물리 기기에서 한 번에 갱신하려면 다음 명령을 사용한다.

```sh
mise run bench-android-local-performance
```

이 명령은 `hermes-release-baseline-summary.json`, `iris-bootstrap-baseline-summary.json`, `android-local-performance-report.json`을 생성한다. `android-local-performance-report.json`은 두 측정이 서로 다른 suite임을 명시하고 `ratioAllowed=false`로 기록한다. 또한 `caseComparisons`에는 Hermes case와 Iris native mirror case의 p50/p95 ratio를 `strictComparable=false`, `nativeMirrorComparable=true`로 남긴다. 이미 summary가 있으면 다음 명령으로 리포트만 다시 쓸 수 있다.

```sh
mise run bench-android-local-performance-report
```

자동화 스크립트는 다른 엔진 APK가 생겼을 때 다음처럼 같은 하네스에 연결한다.

```sh
bun run tools/bench/run-android-release-benchmark.ts \
  --app-id=com.iris.bench.iris \
  --apk=apps/rn-bench/android/app/build/outputs/apk/iris/release/app-iris-release.apk \
  --report-output=artifacts/bench/iris-release-baseline.json \
  --log-output=artifacts/bench/rn-release-iris.log \
  --allow-non-hermes
```

두 엔진 APK가 모두 준비되면 다음 명령으로 같은 물리 기기에서 순서대로 측정한다.

```sh
IRIS_ENGINE_AAR=/absolute/path/to/iris-engine.aar mise run rn-android-build-engine-comparison
mise run bench-android-engine-compare-check
mise run bench-android-engine-compare
```

로컬 skeleton으로는 다음 preflight까지만 실행한다.

```sh
mise run bench-android-engine-compare-local-check
```

`bench-android-engine-compare`는 측정 전에 APK/runtime/bundle preflight를 먼저 수행한다. 현재 구현은 HBC 비교 모드 preflight이므로 HBC parity를 검사한다. Iris 전용 bundle pipeline이 들어오면 source manifest와 Iris artifact parity를 검사해야 한다. preflight가 통과해도 Iris runtime이 bundle 실행 단계에서 실패하면 비교 artifact를 쓰지 않는다. 그 경우는 성능 열위가 아니라 호환성 차단으로 분류한다.

비교 artifact는 두 summary의 반복 횟수, suite id/name, benchmark case set, 단위가 모두 같을 때만 생성한다. 이 조건이 깨지면 ratio를 계산하지 않고 실패시킨다.

Metro 로그를 파일로 남긴 뒤 Hermes baseline artifact로 추출한다.

```sh
mkdir -p artifacts/bench
mise run rn-start 2>&1 | tee artifacts/bench/metro-hermes.log
```

별도 터미널에서 앱을 실행하고 `Run suite`를 누른다.

```sh
mise run rn-ios
mise run rn-android
```

개발 중 smoke 로그가 남으면 다음 명령으로 `artifacts/bench/hermes-baseline.json`을 생성한다.

```sh
mise run bench-extract-hermes
```

추출 도구는 `iris.benchmark.v1` schema, `rn-hermes-js-baseline` suite, Hermes runtime 여부, case별 sample/p50/p95 값을 검증한다.

이 값은 개발 중 빠른 기준선 확인용이다. 성능 주장은 release build, 동일 물리 기기, 반복 측정, p50/p95, 기기 metadata가 모두 갖춰진 산출물에서만 한다. release 로그는 `artifacts/bench/rn-release-hermes.log`에 남긴 뒤 다음 명령으로 검증한다.

```sh
mise run bench-extract-hermes-release
```

Android 물리 기기에서는 다음 명령으로 release 빌드부터 산출물 추출까지 자동화한다.

```sh
mise run bench-android-release
```

반복 측정이 필요한 성능 판단은 다음 명령을 기준으로 한다.

```sh
mise run bench-android-release-repeat
```

release 추출은 다음 조건을 모두 요구한다.

- `metadata.build.mode`가 `release`
- Hermes runtime
- New Architecture
- `iris-module-native-compute`
- `turbomodule-number-round-trip`
- `turbomodule-string-round-trip`

RN 0.85 bridgeless Android에서는 `global.__turboModuleProxy`가 노출되지 않을 수 있다. 따라서 release 추출은 전역 proxy 플래그가 아니라 실제 Codegen TurboModule number/string benchmark case가 포함됐는지로 TurboModule 경계를 검증한다.

TurboModule 경계 기준선은 `docs/turbomodule-baseline.md`의 release 측정 절차를 따른다.
