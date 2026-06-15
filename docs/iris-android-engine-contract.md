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
- skeleton runtime은 Hermes bytecode magic, version, fileLength와 구조화된 HBC metadata를 Rust `iris-hbc` parser로 확인한다. 아직 JS 실행 기능은 제공하지 않으므로 RN JS workload를 완료하지 못한다. 대신 앱 시작 중 HBC metadata parse, static coverage scan, scalar execution frontier를 `iris-engine-bootstrap` artifact로 logcat에 남긴다.
- Rust `iris-hbc` crate는 cross-platform HBC metadata parser와 초기 scalar executor subset의 기준 구현이다. Android skeleton은 같은 Rust parser를 `cxx`로 호출한다. 현재 이 `cxx` 경계는 HBC 바이트 버퍼 parse/scan용이며 JS object graph나 HostObject zero-copy 경계가 아니다.
- 현재 executor subset은 RN 0.85 local skeleton global HBC의 opcode gap을 닫기 위한 기준 구현이다. 지원 opcode는 `Mov`, `Ret`, `Throw`, `NewObject`, `NewObjectWithBuffer`, `NewObjectWithBufferLong`, `NewArray`, `NewArrayWithBuffer`, `NewArrayWithBufferLong`, `GetGlobalObject`, `CreateFunctionEnvironment`, `CreateEnvironment`, short/long `StoreToEnvironment`, short/long `StoreNPToEnvironment`, short/long `LoadFromEnvironment`, `DeclareGlobalVar`, `GetByIdShort`, `TryGetById`, `GetById`, `PutByIdLoose`, `PutByIdStrict`, `PutByValLoose`, `PutByValStrict`, `PutOwnBySlotIdx`, `PutOwnBySlotIdxLong`, `DefineOwnById`, `DefineOwnByIdLong`, `Call1`, `Call2`, `Call3`, `Call4`, `CallBuiltin`, `CallBuiltinLong`, `Construct`, `CreateClosure`, `CreateClosureLongIndex`, `CreateThisForNew`, `SelectObject`, short/long `Jmp`/`JmpTrue`/`JmpFalse`, `TypeOfIs`, `JmpTypeOfIs`, `JNotEqual`, `JNotEqualLong`, `JStrictEqual`, `JStrictEqualLong`, `JStrictNotEqual`, `JStrictNotEqualLong`, `LoadConstUInt8`, `LoadConstInt`, `LoadConstDouble`, `LoadConstString`, `LoadConstEmpty`, `LoadConstUndefined`, `LoadConstNull`, `LoadConstTrue`, `LoadConstFalse`, `LoadConstZero`, `LoadThisNS`다.
- `NewObject`는 opaque plain object handle을 발급하고 object별 property store를 만든다. `NewObjectWithBuffer*`는 Hermes object shape/key buffer와 serialized literal value buffer를 읽어 string-key property store와 slot-name mapping을 만든다. `PutOwnBySlotIdx*`는 slot mapping이 있는 object property를 갱신하고, `DefineOwnById*`는 string id 기반 own property write로 모델링한다.
- `NewArray*`는 opaque array handle을 발급하고 Hermes serialized literal buffer의 primitive/string 값을 array element store에 기록한다. 아직 `Array.prototype`, sparse holes, accessor, element kind 최적화, indexed property semantics 전체는 실행하지 않는다.
- `CreateFunctionEnvironment`와 `CreateEnvironment`는 opaque environment handle을 발급하고 environment slot store를 만든다. `CreateEnvironment`는 parent environment handle만 보존한다.
- `CreateClosure`는 function id와 optional parent environment를 보존하는 opaque bytecode function handle을 register에 쓴다. Function handle도 named property store를 갖는다. `CreateThisForNew`는 bytecode constructor와 일부 native constructor target에 대해 opaque `this` object handle을 발급한다. `Construct`는 현재 subset에서 constructor body를 실행하지 않고 `undefined` completion으로 모델링한다. `SelectObject`는 constructor return이 object면 return object를, 아니면 미리 만든 `this` object를 선택한다. 아직 prototype, derived constructor, real argument frame, `new.target` 의미는 실행하지 않는다.
- `LoadConstString`은 Hermes string table id 기반 opaque string handle을 register에 쓰므로 bytecode payload를 복사하지 않는다. `LoadThisNS`는 현재 top-level non-strict `this`를 global object handle로 매핑한다. `TypeOfIs`/`JmpTypeOfIs`는 Hermes `TypeOfIsTypes` mask의 `Undefined,Object,String,Boolean,Number,Function,Null` subset을 판정한다. equality jump는 scalar primitive와 opaque handle identity, loose `null == undefined`만 모델링한다.
- `GetGlobalObject`는 opaque global object handle을 register에 쓰고, `DeclareGlobalVar`는 Hermes string table의 UTF-8 이름을 zero-copy로 읽어 기록한다. `TryGetById`/`GetById`는 현재 global/plain object/array/function property subset을 지원하고, `PutByIdLoose`/`PutByIdStrict`는 global/plain object/array/function property write를 지원한다. `PutByValLoose`/`PutByValStrict`는 computed key가 string handle일 때만 같은 property store에 기록한다.
- `Call1`/`Call2`/`Call3`/`Call4`는 explicit argument register를 검증하고 `nativePerformanceNow`, `HermesInternal.concat`, `Object.defineProperty`, Metro `__d`/`__r`/`__registerSegment` host function handle을 실행한다. `Map`과 `Object`는 native constructor handle로 모델링한다. `CallBuiltin*`는 RN 0.85 local skeleton global HBC에서 관찰된 `HermesBuiltin.copyDataProperties`만 target/source object property copy로 모델링한다. 그 외 bytecode function call, generic `Call`, 다른 Hermes native builtin, Symbol/BigInt typeof mask와 UTF-16 global var 이름은 아직 실행하지 않는다.

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

Iris는 Hermes 대체 엔진으로 비교하지만, 제품 목표는 Hermes HBC 형식 호환이 아니라 React Native/Hermes 관측 가능 동작 호환이다. 따라서 `irisRelease`는 Hermes 기준선과 같은 앱 소스를 사용해야 하지만, 내부 bundle artifact는 Hermes HBC가 아니어도 된다.

- `irisRelease`는 같은 RN 앱 소스, 같은 benchmark case, 같은 native module surface를 사용해야 한다.
- Iris 전용 bundle pipeline을 쓰는 경우 compiler, source hash, transform option, runtime backend를 artifact에 기록한다.
- Hermes HBC를 쓰는 경로는 strict HBC microbenchmark 또는 기존 skeleton/bootstrap 비교용 baseline으로만 고정한다.
- Hermes, JSC, 또는 다른 런타임 fallback을 Iris 성능값으로 측정하지 않는다.
- QuickJS backend는 Iris-owned runtime 내부 구현으로 사용할 수 있지만, RN 앱 코드 수정을 요구하지 않고 Hermes 관측 가능 동작 shim과 검증 계획을 제공해야 한다.
- V8은 iOS 동일 비교축이 없으므로 Iris/Hermes 대체 성능 비교 대상에 포함하지 않는다.
- Iris가 선택한 bundle artifact를 아직 실행할 수 없다면 실패가 맞다. 성공한 것처럼 우회하지 않는다.
- skeleton `irisRelease` APK에는 `libhermesvm.so`가 없어야 한다. `libhermestooling.so`는 RN의 hermesc bytecode/tooling packaging 경로 때문에 남을 수 있지만 runtime factory와 runtime object는 Iris AAR에서 온다.
- 현재 `bench-android-engine-compare-check`는 HBC 비교 모드의 preflight라서 실제 APK 안의 `assets/index.android.bundle`도 검증한다. Iris 전용 bundle pipeline이 들어오면 같은 앱 소스와 artifact manifest parity를 확인하는 preflight로 확장해야 한다.
- `bench-android-engine-compare-check`는 Hermes APK에 `libhermesvm.so`가 있고 `libirisengine.so`/`libjsc.so`가 없는지, Iris APK에 `libirisengine.so`가 있고 `libhermesvm.so`/`libjsc.so`가 없는지도 확인한다.
- skeleton의 smoke 기준은 RN 초기화가 Iris runtime host surface를 통과하고 Hermes bytecode를 확인한 뒤 `iris-engine-bootstrap` benchmark artifact를 앱 로그에 출력하는 것이다. RN 0.85 local skeleton HBC gap 기준 coverage는 `supportedInstructions=2903/2903`, `supportedUniqueOpcodes=46/46`, `unsupportedUniqueOpcodes=0`, `firstUnsupported=none`이다. 현재 `iris-hbc-scalar-execution-frontier` case는 `detail=status=frontier, error=Iris scalar executor function 3 exceeded step limit 50000`처럼 bounded scalar execution frontier를 남긴다. artifact에는 Hermes benchmark case와 sample shape를 맞춘 `iris-native-*-mirror` C++ probe도 포함한다. 이 mirror case는 JavaScript 실행이나 JS/TurboModule boundary를 통과하지 않으므로 strict engine comparison 값이 아니다. 이는 module factory와 React 앱 실행 완료나 Hermes/RN 실행 호환성 완료를 의미하지 않는다. detail이 `status=frontier`이면 그 값은 성공 workload 시간이 아니라 첫 의미론 차단점까지의 실행 시간이다.

## 호환성 기준

Iris의 목표는 React Native Hermes 관측 가능 동작 100% 호환이다. Hermes HBC 형식 호환은 필수 목표가 아니다. 최소 기준은 다음이다.

- 기존 RN 앱 코드 수정을 요구하지 않는다.
- RN 0.85 New Architecture, Fabric, TurboModule 경로가 Hermes와 같은 JS surface로 동작한다.
- RN core가 참조하는 Hermes observable surface를 제공한다. 예: `HermesInternal`, stack 형식, Promise rejection tracker, microtask 동작.
- Hermes와 비교하는 RN JS workload benchmark artifact는 release build, New Architecture, TurboModule number/string case, Iris native module probe를 모두 포함해야 한다. 현재 skeleton의 `iris-engine-bootstrap` artifact는 이 기준을 만족하는 비교 artifact가 아니라 HBC bootstrap/frontier 측정 artifact다.
- `bench-android-engine-compare`는 같은 물리 기기에서 `com.iris.bench.hermes`와 `com.iris.bench.iris`를 순서대로 측정한다.
