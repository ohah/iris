import NativeIrisBenchTurboModule, { type Spec } from "./NativeIrisBenchTurboModule";

export type IrisBenchTurboModule = Spec;

export function getIrisBenchTurboModule(): IrisBenchTurboModule | null {
  return NativeIrisBenchTurboModule;
}

export function isIrisBenchTurboModuleAvailable() {
  return NativeIrisBenchTurboModule != null;
}
