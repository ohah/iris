export const benchmarkLaneIds = [
  "hermes-baseline",
  "hermes-iris-bridge",
  "quickjs-iris-bridge",
] as const;

export type BenchmarkLaneId = (typeof benchmarkLaneIds)[number];

export type BenchmarkLaneStatus = "measurable" | "partial" | "planned";

export type BenchmarkLane = {
  appId?: string;
  apkPath?: string;
  comparisonReason: string;
  currentMeasurement: string;
  engine: "hermes" | "quickjs";
  id: BenchmarkLaneId;
  label: string;
  logPath?: string;
  objective: string;
  reportPath?: string;
  requiredCapabilities: string[];
  runtimeBackend: string;
  status: BenchmarkLaneStatus;
  strictComparableWithBaseline: boolean;
  summaryPath?: string;
  suiteId?: string;
};

const lanes: Record<BenchmarkLaneId, BenchmarkLane> = {
  "hermes-baseline": {
    appId: "com.iris.bench.hermes",
    apkPath: "apps/rn-bench/android/app/build/outputs/apk/hermes/release/app-hermes-release.apk",
    comparisonReason:
      "Baseline lane. Ratios use this lane as the denominator on the same physical device and release build.",
    currentMeasurement: "Android release Hermes benchmark from apps/rn-bench.",
    engine: "hermes",
    id: "hermes-baseline",
    label: "Hermes baseline",
    logPath: "artifacts/bench/rn-release-hermes.log",
    objective:
      "React Native 0.85, New Architecture, official Hermes runtime, no Iris bridge changes.",
    reportPath: "artifacts/bench/hermes-release-baseline.json",
    requiredCapabilities: [
      "official RN Hermes runtime",
      "release build on physical Android device",
      "same benchmark cases and app source as every strict comparison lane",
    ],
    runtimeBackend: "rn-hermes",
    status: "measurable",
    strictComparableWithBaseline: true,
    summaryPath: "artifacts/bench/hermes-release-baseline-summary.json",
    suiteId: "rn-hermes-js-baseline",
  },
  "hermes-iris-bridge": {
    appId: "com.iris.bench.hermesbridge",
    apkPath:
      "apps/rn-bench/android/app/build/outputs/apk/hermesBridge/release/app-hermesBridge-release.apk",
    comparisonReason:
      "Current Hermes baseline ratios are diagnostic. The lane now has Iris-owned JSI HostFunction fast-path probes and native-owned ArrayBuffer handoff, but thread ownership is not proven yet.",
    currentMeasurement:
      "HermesBridge Android flavor keeps Hermes as the VM and installs Iris-owned JSI HostFunction fast-path probes, same-method JS facade probes, columnar object payload, and native-owned ArrayBuffer handoff measurements. Multi-threaded JSI transfer is not implemented yet.",
    engine: "hermes",
    id: "hermes-iris-bridge",
    label: "Hermes + Iris JSI bridge",
    logPath: "artifacts/bench/rn-release-hermes-iris-bridge.log",
    objective:
      "Keep Hermes as the JavaScript VM, but route Iris-owned zero-copy buffers, object transfer policy, and logic-thread scheduling through the JSI boundary.",
    reportPath: "artifacts/bench/hermes-iris-bridge-baseline.json",
    requiredCapabilities: [
      "Hermes remains the JavaScript VM",
      "Iris-owned JSI HostObject/HostFunction transfer path",
      "native-owned ArrayBuffer handoff proof where lifetime is explicit",
      "documented UI-thread and logic-thread ownership",
      "same RN benchmark suite and checksum as hermes-baseline",
    ],
    runtimeBackend: "iris-hermes-jsi-bridge",
    status: "partial",
    strictComparableWithBaseline: false,
    summaryPath: "artifacts/bench/hermes-iris-bridge-baseline-summary.json",
    suiteId: "rn-hermes-js-baseline",
  },
  "quickjs-iris-bridge": {
    appId: "com.iris.bench.iris",
    apkPath: "apps/rn-bench/android/app/build/outputs/apk/iris/release/app-iris-release.apk",
    comparisonReason:
      "Current ratios against Hermes are diagnostic only because QuickJS does not yet execute the RN JSI/Fabric/TurboModule workload.",
    currentMeasurement:
      "Android Iris AAR emits iris-qjs-backend-microbenchmark from QuickJS; JSI adapter work is still missing.",
    engine: "quickjs",
    id: "quickjs-iris-bridge",
    label: "QuickJS + Iris JSI bridge",
    logPath: "artifacts/bench/rn-release-iris-qjs.log",
    objective:
      "Use QuickJS as the JavaScript backend behind the same Iris JSI bridge policy planned for Hermes.",
    reportPath: "artifacts/bench/iris-qjs-android-baseline.json",
    requiredCapabilities: [
      "QuickJS executes RN bundle code through JSI",
      "Hermes observable behavior shim, including HermesInternal where RN expects it",
      "microtask, error stack, HostFunction, HostObject, ArrayBuffer, Fabric, and TurboModule parity",
      "same RN benchmark suite and checksum as hermes-baseline before strict ratios are allowed",
    ],
    runtimeBackend: "iris-qjs",
    status: "partial",
    strictComparableWithBaseline: false,
    summaryPath: "artifacts/bench/iris-qjs-android-baseline-summary.json",
    suiteId: "iris-qjs-backend-microbenchmark",
  },
};

export function allBenchmarkLanes() {
  return benchmarkLaneIds.map((id) => lanes[id]);
}

export function benchmarkLane(id: BenchmarkLaneId) {
  return lanes[id];
}
