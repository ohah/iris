import type { TurboModule } from "react-native";
import { TurboModuleRegistry } from "react-native";

export interface Spec extends TurboModule {
  readonly getRuntimeBackend: () => string;
  readonly getRuntimeLabel: () => string;
}

export default TurboModuleRegistry.get<Spec>("IrisBridgeTurboModule");
