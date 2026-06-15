export const strictRnBenchmarkSuite = {
  id: "rn-hermes-js-baseline",
  name: "React Native Hermes JS Baseline",
} as const;

export const strictRnBenchmarkCaseIds = [
  "js-compute",
  "json-round-trip",
  "object-traversal",
  "typed-array-copy",
  "turbomodule-number-round-trip",
  "turbomodule-string-round-trip",
  "iris-module-native-compute",
] as const;

export type StrictRnBenchmarkCaseId = (typeof strictRnBenchmarkCaseIds)[number];
