import type { BenchmarkMetadata } from "./types";

type RuntimeGlobals = typeof globalThis & {
  HermesInternal?: {
    getRuntimeProperties?: () => {
      "OSS Release Version"?: string;
    };
  };
  __DEV__?: boolean;
  __turboModuleProxy?: unknown;
  nativeFabricUIManager?: unknown;
};

type RuntimeMetadataInput = {
  appVersion: string;
  device: string;
  os: string;
  platformVersion: string;
  reactNativeVersion: string;
};

const runtime = globalThis as RuntimeGlobals;

export function createRuntimeMetadata(input: RuntimeMetadataInput): BenchmarkMetadata {
  const hermesVersion =
    runtime.HermesInternal?.getRuntimeProperties?.()?.["OSS Release Version"] ?? "unknown";
  const turboModuleProxy = Boolean(runtime.__turboModuleProxy);
  const fabric = Boolean(runtime.nativeFabricUIManager);

  return {
    app: {
      name: "IrisBench",
      version: input.appVersion,
    },
    build: {
      commit: "unknown",
      mode: runtime.__DEV__ === true ? "development" : "release",
      source: "runtime-not-embedded",
    },
    platform: {
      device: input.device,
      os: input.os,
      version: input.platformVersion,
    },
    reactNative: {
      version: input.reactNativeVersion,
    },
    runtime: {
      fabric,
      hermes: Boolean(runtime.HermesInternal),
      hermesVersion,
      jsEngine: runtime.HermesInternal ? "hermes" : "unknown",
      newArchitecture: turboModuleProxy || fabric,
      turboModuleProxy,
    },
  };
}
