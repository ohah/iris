import type { TurboModule } from "react-native";
import { TurboModuleRegistry } from "react-native";

export interface Spec extends TurboModule {
  readonly echoNumber: (value: number) => number;
  readonly getRuntimeLabel: () => string;
  readonly noop: () => void;
  readonly roundTripString: (value: string) => string;
}

export default TurboModuleRegistry.get<Spec>("IrisBenchTurboModule");
