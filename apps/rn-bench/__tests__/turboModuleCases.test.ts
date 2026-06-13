import { createTurboModuleBenchmarkCases } from "../src/benchmarks/turboModuleCases";
import type { IrisBenchTurboModule } from "../src/native/IrisBenchTurboModule";

const fakeModule: IrisBenchTurboModule = {
  echoNumber: (value) => value,
  getIrisRuntimeLabel: () => "jest-iris-module",
  getRuntimeLabel: () => "jest-turbomodule",
  noop: () => undefined,
  roundTripString: (value) => value,
  runIrisNumericWorkload: (iterations) => iterations,
};

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
