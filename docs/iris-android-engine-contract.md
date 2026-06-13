# Iris Android 엔진 계약

Iris Android 엔진은 React Native 앱 포크가 아니라 `apps/rn-bench`의 `irisRelease` flavor에 주입되는 AAR artifact로 연결한다. 앱은 Hermes/JSC fallback을 만들지 않는다. AAR이 계약을 만족하지 못하면 `irisRelease` 빌드나 앱 시작이 실패해야 한다.

## 로컬 skeleton

현재 로컬 skeleton은 `apps/rn-bench/android/iris-engine`에 있다.

```sh
mise run rn-android-build-iris-engine
mise run rn-android-build-iris-release-local
```

- `rn-android-build-iris-engine`은 `iris-engine-release.aar`를 만든다.
- `rn-android-build-iris-release-local`은 그 AAR을 Gradle `irisEngineAar` property로 주입해 `irisRelease` APK를 빌드한다.
- skeleton AAR에는 provider class와 `libirisengine.so`만 들어간다. `libreactnative.so`, `libjsi.so`, `libfbjni.so`는 앱의 RN dependency에서 온다.
- skeleton은 `facebook::react::JSIRuntimeHolder`에 담긴 Iris-owned `jsi::Runtime` 객체를 반환한다.
- skeleton runtime은 RN 초기화가 요구하는 최소 JSI host surface를 제공한다. 현재 포함된 범위는 global object, string/property key, plain object property storage, host object/function registration, array slot storage, native state, empty microtask drain이다.
- skeleton runtime은 Hermes bytecode header magic, version, fileLength를 확인한다. 아직 JS 실행 기능은 제공하지 않으므로 RN이 유효한 Hermes bytecode를 `evaluateJavaScript(...)`나 `evaluatePreparedJavaScript(...)`로 넘기면 bytecode 실행기 미구현 메시지로 실패한다.
- Rust `iris-hbc` crate는 cross-platform HBC metadata parser의 기준 구현이다. Android skeleton의 C++ header 검증은 현재 연결 경계이며, 다음 runtime 단계에서 `cxx`를 통해 Rust parser로 수렴시킨다.

## 앱 연결점

React Native 0.85 Android New Architecture의 연결점은 `DefaultReactHost.getDefaultReactHost(..., jsRuntimeFactory)`다.

- `hermesRelease`는 `com.facebook.react.runtime.hermes.HermesInstance()`를 직접 넘긴다.
- `irisRelease`는 `IRIS_ENGINE_AAR`로 들어온 AAR의 provider에서 `JSRuntimeFactory`를 받아 넘긴다.
- `MainApplication`은 flavor별 `EngineRuntimeFactory`만 호출한다. 앱 소스는 엔진별로 복제하지 않는다.

## AAR Java/Kotlin ABI

`IRIS_ENGINE_AAR`는 다음 public class를 제공해야 한다.

```kotlin
package com.iris.engine.react

import com.facebook.react.runtime.JSRuntimeFactory

object IrisJSRuntimeFactoryProvider {
  @JvmStatic
  fun create(): JSRuntimeFactory
}
```

`create()`는 Hermes나 JSC를 감싼 fallback이 아니라 Iris 엔진의 RN runtime factory를 반환해야 한다. 이 class나 method가 없으면 `irisRelease` Kotlin compile이 실패한다.

## Native ABI

반환되는 `JSRuntimeFactory`는 React Native의 `com.facebook.react.runtime.JSRuntimeFactory`를 상속해야 한다. 내부적으로는 RN의 `HybridData`를 만들고, C++ 쪽에서 `facebook::react::JSRuntimeFactory::createJSRuntime(std::shared_ptr<MessageQueueThread>)`를 구현해 Iris `facebook::jsi::Runtime`을 반환해야 한다.

엔진 AAR은 자체 native library를 포함하고 필요한 `SoLoader.loadLibrary(...)`를 provider 또는 factory 초기화 경로에서 수행해야 한다. 앱은 Iris native library 이름을 알지 않는다.

## Bundle 계약

Iris는 Hermes 대체 엔진으로 비교한다. 따라서 `irisRelease`도 Hermes 기준선과 같은 RN release bundle pipeline을 탄다.

- `irisRelease`의 `index.android.bundle`은 hermesc가 만든 bytecode bundle이다.
- plain JS bundle이나 JSC fallback을 Iris 성능값으로 측정하지 않는다.
- V8은 iOS 동일 비교축이 없으므로 Iris/Hermes 대체 성능 비교 대상에 포함하지 않는다.
- Iris가 이 bytecode header를 검증한 뒤 아직 실행할 수 없다면 실패가 맞다. 성공한 것처럼 우회하지 않는다.
- skeleton `irisRelease` APK에는 `libhermesvm.so`가 없어야 한다. `libhermestooling.so`는 RN의 hermesc bytecode/tooling packaging 경로 때문에 남을 수 있지만 runtime factory와 runtime object는 Iris AAR에서 온다.
- skeleton의 실기기 smoke 기준은 RN 초기화가 Iris runtime host surface를 통과하고 Hermes bytecode header를 확인한 뒤 Iris bytecode execution 미구현 메시지로 실패하는 것이다.

## 호환성 기준

Iris의 목표는 React Native Hermes 100% 호환이다. 최소 기준은 다음이다.

- RN 0.85 New Architecture, Fabric, TurboModule 경로가 Hermes와 같은 JS surface로 동작한다.
- RN core가 참조하는 Hermes observable surface를 제공한다. 예: `HermesInternal`, stack 형식, Promise rejection tracker, microtask 동작.
- benchmark artifact는 release build, New Architecture, TurboModule number/string case, Iris native module probe를 모두 포함해야 한다.
- `bench-android-engine-compare`는 같은 물리 기기에서 `com.iris.bench.hermes`와 `com.iris.bench.iris`를 순서대로 측정한다.
