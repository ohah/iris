import NativeIrisBridgeTurboModule, { type Spec } from "./NativeIrisBridgeTurboModule";

export type IrisBridgeTurboModule = Spec;

export function getIrisBridgeTurboModule(): IrisBridgeTurboModule | null {
  return NativeIrisBridgeTurboModule;
}
