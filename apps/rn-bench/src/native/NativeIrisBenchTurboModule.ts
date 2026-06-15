import type { TurboModule } from "react-native";
import { TurboModuleRegistry } from "react-native";

export interface Spec extends TurboModule {
  readonly echoNumber: (value: number) => number;
  readonly getEngineFlavor: () => string;
  readonly getIrisRuntimeLabel: () => string;
  readonly getRuntimeBackend: () => string;
  readonly getRuntimeLabel: () => string;
  readonly noop: () => void;
  readonly roundTripString: (value: string) => string;
  readonly runIrisNumericWorkload: (iterations: number) => number;
}

export default TurboModuleRegistry.get<Spec>("IrisBenchTurboModule");
