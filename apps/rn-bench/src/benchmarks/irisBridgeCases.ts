import type { IrisBridgeTurboModule } from "../native/IrisBridgeTurboModule";
import type { BenchmarkCase } from "./types";

const numberRoundTripsPerSample = 1_000;
const irisNumericWorkloadIterations = 600_000;
const nativeBufferByteLength = 1_000_000;
const nativeBufferStride = 10_000;
const nativeObjectRowColumns = 4;
const nativeObjectRowCount = 12_000;
const stringRoundTripsPerSample = 500;

type IrisBridgeFastPath = {
  readonly createNativeBuffer: (byteLength: number) => ArrayBuffer;
  readonly createNativeRowsBuffer: (rowCount: number) => ArrayBuffer;
  readonly echoNumber: (value: number) => number;
  readonly nativeCompute: (iterations: number) => number;
  readonly roundTripString: (value: string) => string;
};

declare global {
  var __irisBridgeFastPath: IrisBridgeFastPath | undefined;
}

function getFastPath(module: IrisBridgeTurboModule | null): IrisBridgeFastPath | null {
  if (!module) {
    return null;
  }

  module.getRuntimeLabel();

  return globalThis.__irisBridgeFastPath ?? null;
}

function createFacade(fastPath: IrisBridgeFastPath) {
  return {
    echoNumber: (value: number) => fastPath.echoNumber(value),
    roundTripString: (value: string) => fastPath.roundTripString(value),
    runIrisNumericWorkload: (iterations: number) => fastPath.nativeCompute(iterations),
  };
}

export function createIrisBridgeBenchmarkCases(
  module: IrisBridgeTurboModule | null,
): BenchmarkCase[] {
  const fastPath = getFastPath(module);

  if (!fastPath) {
    return [];
  }

  const facade = createFacade(fastPath);

  return [
    {
      description: "Synchronous JS to Iris JSI host function number round trips.",
      id: "iris-jsi-number-round-trip",
      label: "Iris JSI number",
      measuredIterations: 20,
      run: () => {
        let checksum = 0;

        for (let index = 0; index < numberRoundTripsPerSample; index += 1) {
          checksum += fastPath.echoNumber(index);
        }

        return {
          checksum,
          detail: `${numberRoundTripsPerSample} sync JSI number round trips`,
        };
      },
      unit: "ms",
      warmupIterations: 5,
    },
    {
      description: "Synchronous JS to Iris JSI host function string round trips.",
      id: "iris-jsi-string-round-trip",
      label: "Iris JSI string",
      measuredIterations: 20,
      run: () => {
        let checksum = 0;

        for (let index = 0; index < stringRoundTripsPerSample; index += 1) {
          checksum += fastPath.roundTripString(`iris-${index}`).length;
        }

        return {
          checksum,
          detail: `${stringRoundTripsPerSample} sync JSI string round trips`,
        };
      },
      unit: "ms",
      warmupIterations: 5,
    },
    {
      description: "Single synchronous Iris JSI host function native compute workload.",
      id: "iris-jsi-native-compute",
      label: "Iris JSI compute",
      measuredIterations: 15,
      run: () => ({
        checksum: fastPath.nativeCompute(irisNumericWorkloadIterations),
        detail: `${irisNumericWorkloadIterations} JSI native math operations`,
      }),
      unit: "ms",
      warmupIterations: 3,
    },
    {
      description:
        "Synchronous JS facade to Iris JSI host function number round trips with the TurboModule method shape.",
      id: "iris-jsi-facade-number-round-trip",
      label: "Iris facade number",
      measuredIterations: 20,
      run: () => {
        let checksum = 0;

        for (let index = 0; index < numberRoundTripsPerSample; index += 1) {
          checksum += facade.echoNumber(index);
        }

        return {
          checksum,
          detail: `${numberRoundTripsPerSample} sync JSI facade number round trips`,
        };
      },
      unit: "ms",
      warmupIterations: 5,
    },
    {
      description:
        "Synchronous JS facade to Iris JSI host function string round trips with the TurboModule method shape.",
      id: "iris-jsi-facade-string-round-trip",
      label: "Iris facade string",
      measuredIterations: 20,
      run: () => {
        let checksum = 0;

        for (let index = 0; index < stringRoundTripsPerSample; index += 1) {
          checksum += facade.roundTripString(`iris-${index}`).length;
        }

        return {
          checksum,
          detail: `${stringRoundTripsPerSample} sync JSI facade string round trips`,
        };
      },
      unit: "ms",
      warmupIterations: 5,
    },
    {
      description: "Single synchronous JS facade call into the Iris JSI native compute workload.",
      id: "iris-jsi-facade-native-compute",
      label: "Iris facade compute",
      measuredIterations: 15,
      run: () => ({
        checksum: facade.runIrisNumericWorkload(irisNumericWorkloadIterations),
        detail: `${irisNumericWorkloadIterations} JSI facade native math operations`,
      }),
      unit: "ms",
      warmupIterations: 3,
    },
    {
      description:
        "Native-owned columnar object payload traversal through an Iris JSI ArrayBuffer.",
      id: "iris-jsi-columnar-object-traversal",
      label: "Iris JSI columnar objects",
      measuredIterations: 15,
      run: () => {
        const buffer = fastPath.createNativeRowsBuffer(nativeObjectRowCount);
        const rows = new Int32Array(buffer);
        let checksum = 0;

        for (let rowIndex = 0; rowIndex < nativeObjectRowCount; rowIndex += 1) {
          const base = rowIndex * nativeObjectRowColumns;
          const id = rows[base];
          const lane = rows[base + 1];
          const active = rows[base + 2] === 1;
          const score = rows[base + 3];

          checksum += active ? id + score : lane;
        }

        return {
          checksum,
          detail: `${nativeObjectRowCount} native-owned columnar rows traversed`,
        };
      },
      unit: "ms",
      warmupIterations: 3,
    },
    {
      description: "Native-owned ArrayBuffer handoff through Iris JSI without a JS copy.",
      id: "iris-jsi-native-buffer-read",
      label: "Iris JSI native buffer",
      measuredIterations: 15,
      run: () => {
        const buffer = fastPath.createNativeBuffer(nativeBufferByteLength);
        const view = new Uint8Array(buffer);
        let checksum = 0;

        for (let index = 0; index < view.length; index += nativeBufferStride) {
          checksum += view[index];
        }

        return {
          checksum,
          detail: `${buffer.byteLength} native-owned bytes read without JS copy`,
        };
      },
      unit: "ms",
      warmupIterations: 3,
    },
  ];
}
