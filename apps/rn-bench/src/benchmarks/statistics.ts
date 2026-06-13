import type { BenchmarkStats } from "./types";

function round(value: number) {
  return Number(value.toFixed(3));
}

function percentile(sortedSamples: number[], percentileValue: number) {
  if (sortedSamples.length === 0) {
    return 0;
  }

  const rank = Math.ceil((percentileValue / 100) * sortedSamples.length) - 1;
  const index = Math.max(0, Math.min(sortedSamples.length - 1, rank));
  return sortedSamples[index];
}

export function summarizeSamples(samples: number[]): BenchmarkStats {
  const sortedSamples = [...samples].sort((left, right) => left - right);
  const total = sortedSamples.reduce((sum, value) => sum + value, 0);

  return {
    max: round(sortedSamples[sortedSamples.length - 1] ?? 0),
    mean: round(total / (sortedSamples.length || 1)),
    min: round(sortedSamples[0] ?? 0),
    p50: round(percentile(sortedSamples, 50)),
    p95: round(percentile(sortedSamples, 95)),
    samples: samples.map(round),
  };
}
