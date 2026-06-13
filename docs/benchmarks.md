# 벤치마크 하네스

Iris 벤치마크는 먼저 로컬 산출물 계약을 고정하고, 성능 예산 게이트는 나중에 별도 PR에서 승격한다.

## 로컬 명령

```sh
mise run bench-smoke
mise run bench-js
mise run bench-android-release
mise run bench-android-release-repeat
mise run bench-android-engine-compare
mise run bench-extract-fixture
mise run bench-extract-release-fixture
mise run bench-extract-android-release-fixture
mise run rn-codegen
mise run rn-android-build-debug
mise run rn-android-build-iris-engine
mise run rn-android-build-iris-release
mise run rn-android-build-iris-release-local
mise run rn-ios-build-debug
mise run rn-android-build-release
mise run rn-ios-build-release
```

- `bench-smoke`는 짧은 반복으로 하네스와 JSON schema가 동작하는지 확인한다.
- `bench-js`는 Hermes 기준선 앱과 같은 JS benchmark case를 로컬 JavaScript runtime에서 실행한다.
- `bench-android-release`는 Android 물리 기기에서 release APK 설치, 앱 실행, `Run suite`, 로그 저장, release artifact 추출을 한 번에 수행한다.
- `bench-android-release-repeat`는 같은 release APK에서 3회 반복 측정하고 run별 report와 summary artifact를 남긴다.
- `bench-android-engine-compare`는 Hermes release APK와 Iris release APK를 같은 하네스로 순서대로 측정하고 비교 artifact를 남긴다. Iris APK가 없으면 실패한다.
- `bench-extract-fixture`는 fixture 로그에서 Hermes report를 추출해 파서와 검증 규칙을 확인한다.
- `bench-extract-release-fixture`는 release/Hermes/New Architecture/TurboModule case 요구 조건을 확인한다.
- `bench-extract-android-release-fixture`는 Android logcat의 quoted JSON 형식과 RN 0.85 bridgeless Android의 TurboModule case 기준 검증을 확인한다.
- `rn-codegen`은 TurboModule spec이 RN codegen에서 생성되는지 로컬에서 확인한다.
- `rn-android-build-debug`와 `rn-ios-build-debug`는 네이티브 연결 확인용이며 성능 기준선으로 쓰지 않는다. Android debug는 Hermes flavor를 기준으로 빌드한다.
- `rn-android-build-release`와 `rn-ios-build-release`는 최적화된 RN/Hermes 기준선 수집 전 release 빌드를 확인한다. Android release는 Hermes flavor를 기준으로 빌드한다. iOS task는 simulator build 확인용이며 최종 성능 기준선은 물리 기기에서 남긴다.
- `rn-android-build-iris-engine`은 로컬 Iris Android 엔진 AAR skeleton을 빌드한다.
- `rn-android-build-iris-release`는 `IRIS_ENGINE_AAR`가 실제 RN 호환 Iris 엔진 artifact를 가리킬 때만 `irisRelease` APK를 빌드한다.
- `rn-android-build-iris-release-local`은 로컬 skeleton AAR로 `irisRelease` APK 빌드 계약을 검증한다. 이 APK는 Iris-owned JSI runtime과 RN 초기화용 host surface를 만들지만 bundle 실행에서 의도적으로 실패하므로 성능 비교값으로 쓰지 않는다.
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
artifacts/bench/android-engine-comparison.json
artifacts/bench/rn-release-hermes-run-*.log
artifacts/bench/rn-release-iris-run-*.log
```

산출물 schema는 `iris.benchmark.v1`이며 다음 정보를 포함한다.

- app, platform, build, React Native, runtime metadata
- benchmark case id, 설명, warmup 횟수, measured 횟수
- sample, min, max, mean, p50, p95
- checksum과 detail

## 앱 내 실행

`apps/rn-bench`에서 `Run suite` 버튼을 누르면 같은 benchmark case를 Hermes 런타임에서 실행하고 `IRIS_BENCHMARK_ARTIFACT` 로그로 JSON report를 출력한다.

## 엔진 비교 원칙

Iris가 Hermes를 대체하는 엔진으로 들어가는 비교는 독립 앱 두 개가 아니라 같은 `apps/rn-bench` 소스의 엔진별 release variant 두 개로 수행한다. 기준은 다음과 같다.

- `Hermes release`와 `Iris release`는 같은 benchmark JS, 같은 RN 버전, 같은 native module surface를 사용한다.
- 앱 프로젝트를 복제하지 않는다. 복제 앱은 Gradle, RN, ProGuard, Codegen, assets 설정이 drift되어 비교 신뢰도를 떨어뜨린다.
- Iris 엔진 artifact가 준비되기 전에는 JSC나 Hermes를 `Iris release` 대체값으로 쓰지 않는다.
- V8은 iOS 동일 비교축이 없으므로 Hermes 대체 엔진 후보나 벤치마크 비교 대상으로 두지 않는다.
- `irisRelease`도 Hermes 기준선과 같은 hermesc bytecode bundle을 사용한다. plain JS bundle이나 JSC fallback을 Iris 성능값으로 측정하지 않는다.
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

이 APK는 `libirisengine.so`와 hermesc bytecode bundle이 같이 패키징되고 RN이 Iris-owned JSI runtime 객체 및 초기화용 host surface를 받는지 확인하기 위한 것이다. 아직 Iris runtime의 JS 실행 기능이 구현되지 않았으므로 벤치마크 비교값으로 쓰지 않는다.

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
mise run bench-android-engine-compare
```

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
