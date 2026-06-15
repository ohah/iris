import type { BenchmarkCase } from "./types";
import type { IrisBenchTurboModule } from "../native/IrisBenchTurboModule";

const numberRoundTripsPerSample = 1_000;
const irisNumericWorkloadIterations = 600_000;
const stringRoundTripsPerSample = 500;

export type NativeModuleBoundaryTarget = Pick<
  IrisBenchTurboModule,
  "echoNumber" | "roundTripString" | "runIrisNumericWorkload"
>;

type NativeModuleBoundaryCaseOptions = {
  descriptions: {
    nativeCompute: string;
    numberRoundTrip: string;
    stringRoundTrip: string;
  };
  details: {
    nativeCompute: string;
    numberRoundTrip: string;
    stringRoundTrip: string;
  };
  ids: {
    nativeCompute: string;
    numberRoundTrip: string;
    stringRoundTrip: string;
  };
  labels: {
    nativeCompute: string;
    numberRoundTrip: string;
    stringRoundTrip: string;
  };
};

export function createNativeModuleBoundaryBenchmarkCases(
  module: NativeModuleBoundaryTarget | null,
  options: NativeModuleBoundaryCaseOptions,
): BenchmarkCase[] {
  if (!module) {
    return [];
  }

  return [
    {
      description: options.descriptions.numberRoundTrip,
      id: options.ids.numberRoundTrip,
      label: options.labels.numberRoundTrip,
      measuredIterations: 20,
      run: () => {
        let checksum = 0;

        for (let index = 0; index < numberRoundTripsPerSample; index += 1) {
          checksum += module.echoNumber(index);
        }

        return {
          checksum,
          detail: `${numberRoundTripsPerSample} ${options.details.numberRoundTrip}`,
        };
      },
      unit: "ms",
      warmupIterations: 5,
    },
    {
      description: options.descriptions.stringRoundTrip,
      id: options.ids.stringRoundTrip,
      label: options.labels.stringRoundTrip,
      measuredIterations: 20,
      run: () => {
        let checksum = 0;

        for (let index = 0; index < stringRoundTripsPerSample; index += 1) {
          checksum += module.roundTripString(`iris-${index}`).length;
        }

        return {
          checksum,
          detail: `${stringRoundTripsPerSample} ${options.details.stringRoundTrip}`,
        };
      },
      unit: "ms",
      warmupIterations: 5,
    },
    {
      description: options.descriptions.nativeCompute,
      id: options.ids.nativeCompute,
      label: options.labels.nativeCompute,
      measuredIterations: 15,
      run: () => ({
        checksum: module.runIrisNumericWorkload(irisNumericWorkloadIterations),
        detail: `${irisNumericWorkloadIterations} ${options.details.nativeCompute}`,
      }),
      unit: "ms",
      warmupIterations: 3,
    },
  ];
}

export function createTurboModuleBenchmarkCases(
  module: IrisBenchTurboModule | null,
): BenchmarkCase[] {
  return createNativeModuleBoundaryBenchmarkCases(module, {
    descriptions: {
      nativeCompute: "Single synchronous call into the Iris native module probe workload.",
      numberRoundTrip: "Synchronous JS to TurboModule number round trips on the Hermes baseline.",
      stringRoundTrip: "Synchronous JS to TurboModule string round trips on the Hermes baseline.",
    },
    details: {
      nativeCompute: "native math operations",
      numberRoundTrip: "sync number round trips",
      stringRoundTrip: "sync string round trips",
    },
    ids: {
      nativeCompute: "iris-module-native-compute",
      numberRoundTrip: "turbomodule-number-round-trip",
      stringRoundTrip: "turbomodule-string-round-trip",
    },
    labels: {
      nativeCompute: "Iris module compute",
      numberRoundTrip: "TurboModule number",
      stringRoundTrip: "TurboModule string",
    },
  });
}
