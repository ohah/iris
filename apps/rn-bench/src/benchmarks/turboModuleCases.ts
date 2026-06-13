import type { BenchmarkCase } from "./types";
import type { IrisBenchTurboModule } from "../native/IrisBenchTurboModule";

const numberRoundTripsPerSample = 1_000;
const irisNumericWorkloadIterations = 600_000;
const stringRoundTripsPerSample = 500;

export function createTurboModuleBenchmarkCases(
  module: IrisBenchTurboModule | null,
): BenchmarkCase[] {
  if (!module) {
    return [];
  }

  return [
    {
      description: "Synchronous JS to TurboModule number round trips on the Hermes baseline.",
      id: "turbomodule-number-round-trip",
      label: "TurboModule number",
      measuredIterations: 20,
      run: () => {
        let checksum = 0;

        for (let index = 0; index < numberRoundTripsPerSample; index += 1) {
          checksum += module.echoNumber(index);
        }

        return {
          checksum,
          detail: `${numberRoundTripsPerSample} sync number round trips`,
        };
      },
      unit: "ms",
      warmupIterations: 5,
    },
    {
      description: "Synchronous JS to TurboModule string round trips on the Hermes baseline.",
      id: "turbomodule-string-round-trip",
      label: "TurboModule string",
      measuredIterations: 20,
      run: () => {
        let checksum = 0;

        for (let index = 0; index < stringRoundTripsPerSample; index += 1) {
          checksum += module.roundTripString(`iris-${index}`).length;
        }

        return {
          checksum,
          detail: `${stringRoundTripsPerSample} sync string round trips`,
        };
      },
      unit: "ms",
      warmupIterations: 5,
    },
    {
      description: "Single synchronous call into the Iris native module probe workload.",
      id: "iris-module-native-compute",
      label: "Iris module compute",
      measuredIterations: 15,
      run: () => ({
        checksum: module.runIrisNumericWorkload(irisNumericWorkloadIterations),
        detail: `${irisNumericWorkloadIterations} native math operations`,
      }),
      unit: "ms",
      warmupIterations: 3,
    },
  ];
}
