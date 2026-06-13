import type { BenchmarkCase } from "./types";

function runComputeCase() {
  let checksum = 0;

  for (let index = 0; index < 600_000; index += 1) {
    checksum += Math.sqrt((index % 1_000) + 1) * Math.sin(index);
  }

  return {
    checksum: Number(checksum.toFixed(3)),
    detail: "600k math operations",
  };
}

function runJsonCase() {
  const payload = Array.from({ length: 8_000 }, (_, index) => ({
    active: index % 3 === 0,
    id: index,
    meta: {
      lane: index % 7,
      label: `group-${index % 11}`,
    },
    name: `item-${index}`,
    points: [index, index * 2, index * 3],
  }));

  const encoded = JSON.stringify(payload);
  const decoded = JSON.parse(encoded) as Array<{
    id: number;
    points: number[];
  }>;
  const checksum = decoded.reduce((total, item) => total + item.id + item.points[2], 0);

  return {
    checksum,
    detail: `${encoded.length} encoded bytes`,
  };
}

function runObjectTraversalCase() {
  const rows = Array.from({ length: 12_000 }, (_, index) => ({
    id: index,
    lane: index % 9,
    nested: {
      active: index % 5 === 0,
      score: (index * 17) % 1_024,
    },
  }));

  const checksum = rows.reduce((total, row) => {
    if (!row.nested.active) {
      return total + row.lane;
    }

    return total + row.id + row.nested.score;
  }, 0);

  return {
    checksum,
    detail: `${rows.length} objects traversed`,
  };
}

function runTypedArrayCopyCase() {
  const source = new Uint8Array(1_000_000);

  for (let index = 0; index < source.length; index += 1) {
    source[index] = index % 251;
  }

  const copy = new Uint8Array(source.length);
  copy.set(source);

  let checksum = 0;
  for (let index = 0; index < copy.length; index += 10_000) {
    checksum += copy[index];
  }

  return {
    checksum,
    detail: `${copy.byteLength} bytes copied`,
  };
}

export const benchmarkCases: BenchmarkCase[] = [
  {
    description: "CPU-bound JavaScript math loop for Hermes baseline timing.",
    id: "js-compute",
    label: "JS compute",
    measuredIterations: 15,
    run: runComputeCase,
    unit: "ms",
    warmupIterations: 3,
  },
  {
    description: "Large object array stringify/parse round trip.",
    id: "json-round-trip",
    label: "JSON round trip",
    measuredIterations: 15,
    run: runJsonCase,
    unit: "ms",
    warmupIterations: 3,
  },
  {
    description: "Nested object creation and traversal baseline.",
    id: "object-traversal",
    label: "Object traversal",
    measuredIterations: 15,
    run: runObjectTraversalCase,
    unit: "ms",
    warmupIterations: 3,
  },
  {
    description: "Uint8Array allocation and copy baseline before JSI buffer work.",
    id: "typed-array-copy",
    label: "TypedArray copy",
    measuredIterations: 15,
    run: runTypedArrayCopyCase,
    unit: "ms",
    warmupIterations: 3,
  },
];

export type RenderStressRow = {
  id: string;
  score: string;
  title: string;
};

export function createRenderStressRows(count: number): RenderStressRow[] {
  return Array.from({ length: count }, (_, index) => ({
    id: String(index),
    score: String((index * 37) % 997).padStart(3, "0"),
    title: `Render row ${index + 1}`,
  }));
}
