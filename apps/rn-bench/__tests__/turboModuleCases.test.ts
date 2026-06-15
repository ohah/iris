import { createIrisBridgeBenchmarkCases } from "../src/benchmarks/irisBridgeCases";
import { createTurboModuleBenchmarkCases } from "../src/benchmarks/turboModuleCases";
import type { IrisBridgeTurboModule } from "../src/native/IrisBridgeTurboModule";
import type { IrisBenchTurboModule } from "../src/native/IrisBenchTurboModule";

const fakeModule: IrisBenchTurboModule = {
  echoNumber: (value) => value,
  getEngineFlavor: () => "jest",
  getIrisRuntimeLabel: () => "jest-iris-module",
  getRuntimeBackend: () => "jest-backend",
  getRuntimeLabel: () => "jest-turbomodule",
  noop: () => undefined,
  roundTripString: (value) => value,
  runIrisNumericWorkload: (iterations) => iterations,
};

const fakeBridgeModule: IrisBridgeTurboModule = {
  getRuntimeBackend: () => "jest-jsi-backend",
  getRuntimeLabel: () => "jest-jsi-fast-path",
};

const originalFastPath = globalThis.__irisBridgeFastPath;

afterEach(() => {
  globalThis.__irisBridgeFastPath = originalFastPath;
});

test("omits TurboModule benchmarks when the native module is unavailable", () => {
  expect(createTurboModuleBenchmarkCases(null)).toEqual([]);
});

test("builds TurboModule boundary benchmark cases", () => {
  const cases = createTurboModuleBenchmarkCases(fakeModule);

  expect(cases.map((benchmarkCase) => benchmarkCase.id)).toEqual([
    "turbomodule-number-round-trip",
    "turbomodule-string-round-trip",
    "iris-module-native-compute",
  ]);
  expect(cases[0].run().checksum).toBe(499500);
  expect(cases[1].run().detail).toBe("500 sync string round trips");
  expect(cases[2].run().checksum).toBe(600000);
});

test("omits Iris bridge benchmarks when the JSI fast path is unavailable", () => {
  globalThis.__irisBridgeFastPath = undefined;

  expect(createIrisBridgeBenchmarkCases(fakeBridgeModule)).toEqual([]);
});

test("builds Iris bridge JSI benchmark cases", () => {
  globalThis.__irisBridgeFastPath = {
    createNativeBuffer: (byteLength) => {
      const buffer = new ArrayBuffer(byteLength);
      const view = new Uint8Array(buffer);

      for (let index = 0; index < view.length; index += 1) {
        view[index] = index % 251;
      }

      return buffer;
    },
    createNativeRowsBuffer: (rowCount) => {
      const values = new Int32Array(rowCount * 4);

      for (let index = 0; index < rowCount; index += 1) {
        const base = index * 4;
        values[base] = index;
        values[base + 1] = index % 9;
        values[base + 2] = index % 5 === 0 ? 1 : 0;
        values[base + 3] = (index * 17) % 1024;
      }

      return values.buffer;
    },
    echoNumber: (value) => value,
    nativeCompute: (iterations) => iterations,
    roundTripString: (value) => value,
  };

  const cases = createIrisBridgeBenchmarkCases(fakeBridgeModule);

  expect(cases.map((benchmarkCase) => benchmarkCase.id)).toEqual([
    "iris-jsi-number-round-trip",
    "iris-jsi-string-round-trip",
    "iris-jsi-native-compute",
    "iris-jsi-facade-number-round-trip",
    "iris-jsi-facade-string-round-trip",
    "iris-jsi-facade-native-compute",
    "iris-jsi-columnar-object-traversal",
    "iris-jsi-native-buffer-read",
  ]);
  expect(cases[0].run().checksum).toBe(499500);
  expect(cases[1].run().detail).toBe("500 sync JSI string round trips");
  expect(cases[2].run().checksum).toBe(600000);
  expect(cases[3].run().checksum).toBe(499500);
  expect(cases[4].run().detail).toBe("500 sync JSI facade string round trips");
  expect(cases[5].run().checksum).toBe(600000);
  expect(cases[6].run().checksum).toBe(15661082);
  expect(cases[7].run().checksum).toBe(12338);
});
