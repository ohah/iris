# 벤치마크 하네스

Iris 벤치마크는 먼저 로컬 산출물 계약을 고정하고, 성능 예산 게이트는 나중에 별도 PR에서 승격한다.

## 로컬 명령

```sh
mise run bench-smoke
mise run bench-js
mise run bench-extract-fixture
mise run rn-codegen
mise run rn-android-build-debug
mise run rn-ios-build-debug
```

- `bench-smoke`는 짧은 반복으로 하네스와 JSON schema가 동작하는지 확인한다.
- `bench-js`는 Hermes 기준선 앱과 같은 JS benchmark case를 로컬 JavaScript runtime에서 실행한다.
- `bench-extract-fixture`는 fixture 로그에서 Hermes report를 추출해 파서와 검증 규칙을 확인한다.
- `rn-codegen`은 TurboModule spec이 RN codegen에서 생성되는지 로컬에서 확인한다.
- `rn-android-build-debug`와 `rn-ios-build-debug`는 네이티브 연결 확인용이며 성능 기준선으로 쓰지 않는다.
- 이 명령들은 CI 필수 체크에 포함하지 않는다.

## 산출물

```text
artifacts/bench/js-baseline-smoke.json
artifacts/bench/js-baseline.json
artifacts/bench/hermes-baseline-fixture.json
artifacts/bench/hermes-baseline.json
```

산출물 schema는 `iris.benchmark.v1`이며 다음 정보를 포함한다.

- app, platform, build, React Native, runtime metadata
- benchmark case id, 설명, warmup 횟수, measured 횟수
- sample, min, max, mean, p50, p95
- checksum과 detail

## 앱 내 실행

`apps/rn-bench`에서 `Run suite` 버튼을 누르면 같은 benchmark case를 Hermes 런타임에서 실행하고 `IRIS_BENCHMARK_ARTIFACT` 로그로 JSON report를 출력한다.

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

로그가 남으면 다음 명령으로 `artifacts/bench/hermes-baseline.json`을 생성한다.

```sh
mise run bench-extract-hermes
```

추출 도구는 `iris.benchmark.v1` schema, `rn-hermes-js-baseline` suite, Hermes runtime 여부, case별 sample/p50/p95 값을 검증한다.

이 값은 개발 중 빠른 기준선 확인용이다. 성능 주장은 release build, 동일 기기, 반복 측정, p50/p95, 기기 metadata가 모두 갖춰진 산출물에서만 한다.

TurboModule 경계 기준선은 `docs/turbomodule-baseline.md`의 release 측정 절차를 따른다.
