# TurboModule 기준선

Iris 경로를 비교하기 전에는 React Native/Hermes의 최적화된 정상 경로를 기준으로 잡는다.

## 원칙

- Hermes를 끄지 않는다.
- New Architecture를 끄지 않는다.
- dev build 수치를 비교 기준으로 쓰지 않는다.
- release build에서 JS dev mode가 꺼진 상태를 기준선으로 남긴다.
- TurboModule 경계 측정은 native work가 거의 없는 `echoNumber`와 `roundTripString` 왕복 비용만 본다.

## 구현 경계

`IrisBenchTurboModule`은 React Native codegen 기반 TurboModule이다.

- JS spec: `apps/rn-bench/src/native/NativeIrisBenchTurboModule.ts`
- JS wrapper: `apps/rn-bench/src/native/IrisBenchTurboModule.ts`
- Android implementation: `apps/rn-bench/android/app/src/main/java/com/iris/bench/IrisBenchTurboModule.kt`
- iOS implementation: `apps/rn-bench/ios/IrisBench/IrisBenchTurboModule.mm`

앱 런타임에서 native module이 잡히면 benchmark suite에 다음 케이스가 추가된다.

- `turbomodule-number-round-trip`
- `turbomodule-string-round-trip`

## 로컬 검증

```sh
mise run rn-codegen
mise run rn-typecheck
mise run rn-test
mise run rn-android-build-debug
mise run rn-ios-pods
mise run rn-ios-build-debug
mise run rn-android-build-release
mise run rn-ios-build-release
mise run bench-extract-release-fixture
```

`rn-codegen`은 생성물을 `artifacts/codegen/rn-bench`에 남긴다. 이 디렉터리는 gitignore된다.
Debug build 명령은 TurboModule 네이티브 연결 확인용이다. 성능 기준선은 아래 release 절차로만 남긴다.

## 최적화 기준 측정

Android는 물리 기기의 release variant를 기준으로 실행한다.

```sh
mise run rn-android-build-release
cd apps/rn-bench
bun run react-native run-android --mode release --no-packager
```

iOS는 물리 기기의 Release configuration을 기준으로 실행한다. `rn-ios-build-release`는 simulator release build 확인용이며 최종 성능 기준선으로 쓰지 않는다.

```sh
mise run rn-ios-pods
mise run rn-ios-build-release
cd apps/rn-bench
bun run react-native run-ios --mode Release --no-packager
```

release 앱 로그를 `artifacts/bench/rn-release-hermes.log`에 남기고 앱에서 `Run suite`를 누른 뒤 Hermes artifact를 추출한다. Android는 별도 터미널에서 다음 로그 캡처를 실행한다.

```sh
mkdir -p artifacts/bench
adb logcat -c
adb logcat ReactNativeJS:I '*:S' | tee artifacts/bench/rn-release-hermes.log
```

iOS simulator는 booted simulator를 대상으로 다음 로그 캡처를 실행한다.

```sh
mkdir -p artifacts/bench
xcrun simctl spawn booted log stream --style compact --predicate 'eventMessage CONTAINS "IRIS_BENCHMARK_ARTIFACT"' | tee artifacts/bench/rn-release-hermes.log
```

로그 파일에 marker가 남으면 다음 명령으로 release artifact를 생성한다.

```sh
mise run bench-extract-hermes-release
```

release 추출은 `metadata.build.mode = release`, Hermes, New Architecture, TurboModule proxy, TurboModule number/string case를 모두 요구한다. 이 산출물을 Iris 경로와 비교할 때의 기준으로 사용한다.
