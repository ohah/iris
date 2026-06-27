import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { basename, dirname, resolve } from "node:path";

const root = resolve(import.meta.dir, "../..");
const defaultCaseDir = "tools/bench/strict-hbc/cases";
const defaultHbcDir = "artifacts/bench/strict-hbc";
const defaultHermesc = "apps/rn-bench/node_modules/hermes-compiler/hermesc/osx-bin/hermesc";
const defaultOutputPath = "artifacts/bench/strict-hbc-timing-profile.txt";
const defaultJsonOutputPath = "artifacts/bench/strict-hbc-timing-profile.json";

type CountEntry = {
  count: number;
};

type TimingEntry = CountEntry & {
  averageNs: number;
  maxNs: number;
  totalNs: number;
};

type OpcodeTimingEntry = TimingEntry & {
  name: string;
  opcode: number;
};

type InstructionTimingEntry = OpcodeTimingEntry & {
  offset: number;
};

type CategoryTimingEntry = TimingEntry & {
  category: string;
};

type PropertyAccessTimingEntry = TimingEntry & {
  role: string;
  baseKind: string;
  propertyName: string;
};

type IndexedAccessTimingEntry = TimingEntry & {
  role: string;
  baseKind: string;
  keyKind: string;
};

type CallTargetTimingEntry = TimingEntry & {
  role: string;
  target: string;
};

type OpcodeCountEntry = CountEntry & {
  name: string;
  opcode: number;
};

type InstructionOffsetCountEntry = OpcodeCountEntry & {
  offset: number;
};

type StringOperandCountEntry = CountEntry & {
  role: string;
  value: string;
};

type PropertyAccessCountEntry = CountEntry & {
  role: string;
  baseKind: string;
  propertyName: string;
};

type IndexedAccessCountEntry = CountEntry & {
  role: string;
  baseKind: string;
  keyKind: string;
};

type CallTargetCountEntry = CountEntry & {
  role: string;
  target: string;
};

type CallArgumentKindCountEntry = CountEntry & {
  role: string;
  target: string;
  argument: string;
  kind: string;
};

type ParsedTimingProfileReport = {
  status: string;
  functionId: number;
  value: string;
  error: string;
  declaredGlobals: number;
  totalInstructions: number;
  jumpsTaken: number;
  totalMeasuredNs: number;
  topOpcodeTimings: OpcodeTimingEntry[];
  topInstructionTimings: InstructionTimingEntry[];
  topCategoryTimings: CategoryTimingEntry[];
  topPropertyTimings: PropertyAccessTimingEntry[];
  topIndexedTimings: IndexedAccessTimingEntry[];
  topCallTargetTimings: CallTargetTimingEntry[];
  topOpcodes: OpcodeCountEntry[];
  topInstructionOffsets: InstructionOffsetCountEntry[];
  topStringOperands: StringOperandCountEntry[];
  topPropertyAccesses: PropertyAccessCountEntry[];
  topIndexedAccesses: IndexedAccessCountEntry[];
  topCallTargets: CallTargetCountEntry[];
  topCallArgumentKinds: CallArgumentKindCountEntry[];
};

type TimingProfileCaseArtifact = {
  id: string;
  sourcePath: string;
  hbcPath: string;
  report: string;
  parsed: ParsedTimingProfileReport;
};

function readArg(name: string) {
  const prefix = `${name}=`;
  return process.argv.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

function readArgs(name: string) {
  const prefix = `${name}=`;
  return process.argv
    .filter((arg) => arg.startsWith(prefix))
    .map((arg) => arg.slice(prefix.length));
}

function relativePath(path: string) {
  return path.replace(`${root}/`, "");
}

function shellLine(command: string, args: string[]) {
  return [command, ...args.map((arg) => (arg.includes(" ") ? JSON.stringify(arg) : arg))].join(" ");
}

function run(command: string, args: string[]) {
  console.log(`$ ${shellLine(command, args)}`);
  execFileSync(command, args, {
    cwd: root,
    stdio: "inherit",
  });
}

function capture(command: string, args: string[]) {
  console.log(`$ ${shellLine(command, args)}`);
  return execFileSync(command, args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
    stdio: ["ignore", "pipe", "inherit"],
  }).trim();
}

function resolveCargo() {
  const configuredCargo = readArg("--cargo");
  if (configuredCargo != null) {
    return configuredCargo.includes("/") ? resolve(root, configuredCargo) : configuredCargo;
  }

  const cargoHomePath = resolve(homedir(), ".cargo/bin/cargo");
  return existsSync(cargoHomePath) ? cargoHomePath : "cargo";
}

function resolveCasePath(input: string) {
  if (input.endsWith(".js") || input.includes("/")) {
    return resolve(root, input);
  }
  return resolve(root, defaultCaseDir, `${input}.js`);
}

function defaultCasePaths() {
  const caseDir = resolve(root, defaultCaseDir);
  return readdirSync(caseDir)
    .filter((entry) => entry.endsWith(".js"))
    .filter((entry) => !entry.endsWith("-lexical.js"))
    .filter((entry) => !entry.endsWith("-diagnostic.js"))
    .sort()
    .map((entry) => resolve(caseDir, entry));
}

function caseId(path: string) {
  return basename(path, ".js");
}

function countEntry(rawEntry: string) {
  const equalIndex = rawEntry.lastIndexOf("=");
  if (equalIndex < 0) {
    return null;
  }

  const count = Number(rawEntry.slice(equalIndex + 1));
  if (!Number.isFinite(count)) {
    return null;
  }

  return {
    key: rawEntry.slice(0, equalIndex),
    count,
  };
}

function timingEntry(rawEntry: string) {
  const equalIndex = rawEntry.lastIndexOf("=");
  if (equalIndex < 0) {
    return null;
  }

  const metrics = rawEntry
    .slice(equalIndex + 1)
    .split("|")
    .map((part) => part.split(":") as [string, string])
    .reduce<Record<string, number>>((accumulator, [name, rawValue]) => {
      const value = Number(rawValue);
      if (Number.isFinite(value)) {
        accumulator[name] = value;
      }
      return accumulator;
    }, {});

  if (
    metrics.count == null ||
    metrics.totalNs == null ||
    metrics.avgNs == null ||
    metrics.maxNs == null
  ) {
    return null;
  }

  return {
    key: rawEntry.slice(0, equalIndex),
    count: metrics.count,
    totalNs: metrics.totalNs,
    averageNs: metrics.avgNs,
    maxNs: metrics.maxNs,
  };
}

function splitProfileList(rawValue: string) {
  if (rawValue.trim() === "") {
    return [];
  }

  const entries: string[] = [];
  let start = 0;
  let quoted = false;
  let escaped = false;

  for (let index = 0; index < rawValue.length; index += 1) {
    const char = rawValue[index];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (char === "\\") {
      escaped = quoted;
      continue;
    }
    if (char === '"') {
      quoted = !quoted;
      continue;
    }
    if (char === "," && !quoted) {
      entries.push(rawValue.slice(start, index));
      start = index + 1;
    }
  }

  entries.push(rawValue.slice(start));
  return entries.filter((entry) => entry.length > 0);
}

function readField(report: string, startMarker: string, endMarker: string) {
  const start = report.indexOf(startMarker);
  if (start < 0) {
    throw new Error(`timing profile report is missing ${startMarker}`);
  }
  const valueStart = start + startMarker.length;
  const end = report.indexOf(endMarker, valueStart);
  if (end < 0) {
    throw new Error(`timing profile report is missing ${endMarker}`);
  }
  return report.slice(valueStart, end);
}

function readBracketList(report: string, name: string) {
  const marker = `${name}=[`;
  const start = report.indexOf(marker);
  if (start < 0) {
    throw new Error(`timing profile report is missing ${name}`);
  }
  const valueStart = start + marker.length;
  let quoted = false;
  let escaped = false;

  for (let index = valueStart; index < report.length; index += 1) {
    const char = report[index];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (char === "\\") {
      escaped = quoted;
      continue;
    }
    if (char === '"') {
      quoted = !quoted;
      continue;
    }
    if (char === "]" && !quoted) {
      return report.slice(valueStart, index);
    }
  }

  throw new Error(`timing profile report has an unterminated ${name} list`);
}

function parseOpcodeTimings(rawValue: string): OpcodeTimingEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = timingEntry(entry);
    if (parsed == null) {
      return [];
    }
    const match = parsed.key.match(/^(.*)\((\d+)\)$/);
    if (match == null) {
      return [];
    }
    return [
      {
        name: match[1],
        opcode: Number(match[2]),
        count: parsed.count,
        totalNs: parsed.totalNs,
        averageNs: parsed.averageNs,
        maxNs: parsed.maxNs,
      },
    ];
  });
}

function parseInstructionTimings(rawValue: string): InstructionTimingEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = timingEntry(entry);
    if (parsed == null) {
      return [];
    }
    const match = parsed.key.match(/^(.*)\((\d+)\)@(\d+)$/);
    if (match == null) {
      return [];
    }
    return [
      {
        name: match[1],
        opcode: Number(match[2]),
        offset: Number(match[3]),
        count: parsed.count,
        totalNs: parsed.totalNs,
        averageNs: parsed.averageNs,
        maxNs: parsed.maxNs,
      },
    ];
  });
}

function parseCategoryTimings(rawValue: string): CategoryTimingEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = timingEntry(entry);
    if (parsed == null) {
      return [];
    }
    return [
      {
        category: parsed.key,
        count: parsed.count,
        totalNs: parsed.totalNs,
        averageNs: parsed.averageNs,
        maxNs: parsed.maxNs,
      },
    ];
  });
}

function parsePropertyAccessTimings(rawValue: string): PropertyAccessTimingEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = timingEntry(entry);
    if (parsed == null) {
      return [];
    }
    const parts = parsed.key.split(":");
    if (parts.length < 3) {
      return [];
    }
    return [
      {
        role: parts[0],
        baseKind: parts[1],
        propertyName: parts.slice(2).join(":"),
        count: parsed.count,
        totalNs: parsed.totalNs,
        averageNs: parsed.averageNs,
        maxNs: parsed.maxNs,
      },
    ];
  });
}

function parseIndexedAccessTimings(rawValue: string): IndexedAccessTimingEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = timingEntry(entry);
    if (parsed == null) {
      return [];
    }
    const [role, baseKind, keyKind] = parsed.key.split(":");
    if (role == null || baseKind == null || keyKind == null) {
      return [];
    }
    return [
      {
        role,
        baseKind,
        keyKind,
        count: parsed.count,
        totalNs: parsed.totalNs,
        averageNs: parsed.averageNs,
        maxNs: parsed.maxNs,
      },
    ];
  });
}

function parseCallTargetTimings(rawValue: string): CallTargetTimingEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = timingEntry(entry);
    if (parsed == null) {
      return [];
    }
    const separator = parsed.key.indexOf(":");
    if (separator < 0) {
      return [];
    }
    return [
      {
        role: parsed.key.slice(0, separator),
        target: parsed.key.slice(separator + 1),
        count: parsed.count,
        totalNs: parsed.totalNs,
        averageNs: parsed.averageNs,
        maxNs: parsed.maxNs,
      },
    ];
  });
}

function parseOpcodeCounts(rawValue: string): OpcodeCountEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = countEntry(entry);
    if (parsed == null) {
      return [];
    }
    const match = parsed.key.match(/^(.*)\((\d+)\)$/);
    if (match == null) {
      return [];
    }
    return [
      {
        name: match[1],
        opcode: Number(match[2]),
        count: parsed.count,
      },
    ];
  });
}

function parseInstructionOffsetCounts(rawValue: string): InstructionOffsetCountEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = countEntry(entry);
    if (parsed == null) {
      return [];
    }
    const match = parsed.key.match(/^(.*)\((\d+)\)@(\d+)$/);
    if (match == null) {
      return [];
    }
    return [
      {
        name: match[1],
        opcode: Number(match[2]),
        offset: Number(match[3]),
        count: parsed.count,
      },
    ];
  });
}

function parseStringOperandCounts(rawValue: string): StringOperandCountEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = countEntry(entry);
    if (parsed == null) {
      return [];
    }
    const separator = parsed.key.indexOf(":");
    if (separator < 0) {
      return [];
    }
    return [
      {
        role: parsed.key.slice(0, separator),
        value: parsed.key.slice(separator + 1),
        count: parsed.count,
      },
    ];
  });
}

function parsePropertyAccessCounts(rawValue: string): PropertyAccessCountEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = countEntry(entry);
    if (parsed == null) {
      return [];
    }
    const parts = parsed.key.split(":");
    if (parts.length < 3) {
      return [];
    }
    return [
      {
        role: parts[0],
        baseKind: parts[1],
        propertyName: parts.slice(2).join(":"),
        count: parsed.count,
      },
    ];
  });
}

function parseIndexedAccessCounts(rawValue: string): IndexedAccessCountEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = countEntry(entry);
    if (parsed == null) {
      return [];
    }
    const [role, baseKind, keyKind] = parsed.key.split(":");
    if (role == null || baseKind == null || keyKind == null) {
      return [];
    }
    return [{ role, baseKind, keyKind, count: parsed.count }];
  });
}

function parseCallTargetCounts(rawValue: string): CallTargetCountEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = countEntry(entry);
    if (parsed == null) {
      return [];
    }
    const separator = parsed.key.indexOf(":");
    if (separator < 0) {
      return [];
    }
    return [
      {
        role: parsed.key.slice(0, separator),
        target: parsed.key.slice(separator + 1),
        count: parsed.count,
      },
    ];
  });
}

function parseCallArgumentKindCounts(rawValue: string): CallArgumentKindCountEntry[] {
  return splitProfileList(rawValue).flatMap((entry) => {
    const parsed = countEntry(entry);
    if (parsed == null) {
      return [];
    }
    const [role, target, argument, kind] = parsed.key.split(":");
    if (role == null || target == null || argument == null || kind == null) {
      return [];
    }
    return [{ role, target, argument, kind, count: parsed.count }];
  });
}

function parseTimingProfileReport(report: string): ParsedTimingProfileReport {
  return {
    status: readField(report, "status=", ", function="),
    functionId: Number(readField(report, ", function=", ", value=")),
    value: readField(report, ", value=", ", error="),
    error: readField(report, ", error=", ", declaredGlobals="),
    declaredGlobals: Number(readField(report, ", declaredGlobals=", ", totalInstructions=")),
    totalInstructions: Number(readField(report, ", totalInstructions=", ", jumpsTaken=")),
    jumpsTaken: Number(readField(report, ", jumpsTaken=", ", totalMeasuredNs=")),
    totalMeasuredNs: Number(readField(report, ", totalMeasuredNs=", ", topOpcodeTimings=[")),
    topOpcodeTimings: parseOpcodeTimings(readBracketList(report, "topOpcodeTimings")),
    topInstructionTimings: parseInstructionTimings(
      readBracketList(report, "topInstructionTimings"),
    ),
    topCategoryTimings: parseCategoryTimings(readBracketList(report, "topCategoryTimings")),
    topPropertyTimings: parsePropertyAccessTimings(readBracketList(report, "topPropertyTimings")),
    topIndexedTimings: parseIndexedAccessTimings(readBracketList(report, "topIndexedTimings")),
    topCallTargetTimings: parseCallTargetTimings(readBracketList(report, "topCallTargetTimings")),
    topOpcodes: parseOpcodeCounts(readBracketList(report, "topOpcodes")),
    topInstructionOffsets: parseInstructionOffsetCounts(
      readBracketList(report, "topInstructionOffsets"),
    ),
    topStringOperands: parseStringOperandCounts(readBracketList(report, "topStringOperands")),
    topPropertyAccesses: parsePropertyAccessCounts(readBracketList(report, "topPropertyAccesses")),
    topIndexedAccesses: parseIndexedAccessCounts(readBracketList(report, "topIndexedAccesses")),
    topCallTargets: parseCallTargetCounts(readBracketList(report, "topCallTargets")),
    topCallArgumentKinds: parseCallArgumentKindCounts(
      readBracketList(report, "topCallArgumentKinds"),
    ),
  };
}

function compileHbc(hermesc: string, sourcePath: string, hbcDir: string) {
  const outputPath = resolve(hbcDir, `${caseId(sourcePath)}.hbc`);
  mkdirSync(dirname(outputPath), { recursive: true });
  run(hermesc, ["-O", "-emit-binary", "-out", outputPath, sourcePath]);
  return outputPath;
}

const hermesc = resolve(root, readArg("--hermesc") ?? defaultHermesc);
if (!existsSync(hermesc)) {
  throw new Error(`hermesc does not exist: ${hermesc}`);
}

const hbcDir = resolve(root, readArg("--hbc-dir") ?? defaultHbcDir);
const outputPath = resolve(root, readArg("--output") ?? defaultOutputPath);
const jsonOutputPath = resolve(root, readArg("--json-output") ?? defaultJsonOutputPath);
const caseInputs = readArgs("--case");
const casePaths = caseInputs.length > 0 ? caseInputs.map(resolveCasePath) : defaultCasePaths();
const cargo = resolveCargo();

run(cargo, ["build", "--release", "-p", "iris-hbc", "--bin", "hbc-timing-profile"]);

const lines = [
  "Strict HBC scalar execution timing profile",
  "Generated by tools/bench/run-strict-hbc-timing-profile.ts",
  "",
];
const cases: TimingProfileCaseArtifact[] = [];

for (const sourcePath of casePaths) {
  if (!existsSync(sourcePath)) {
    throw new Error(`strict HBC case does not exist: ${sourcePath}`);
  }
  const hbcPath = compileHbc(hermesc, sourcePath, hbcDir);
  const id = caseId(sourcePath);
  const report = capture(resolve(root, "target/release/hbc-timing-profile"), [hbcPath]);
  lines.push(`== ${id} ==`, report, "");
  cases.push({
    id,
    sourcePath: relativePath(sourcePath),
    hbcPath: relativePath(hbcPath),
    report,
    parsed: parseTimingProfileReport(report),
  });
  console.log(`${id}: time-profiled`);
}

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, `${lines.join("\n")}\n`);
console.log(`Strict HBC timing profile artifact: ${outputPath}`);

mkdirSync(dirname(jsonOutputPath), { recursive: true });
writeFileSync(
  jsonOutputPath,
  `${JSON.stringify(
    {
      schemaVersion: "iris.benchmark.strict-hbc-timing-profile.v1",
      generatedBy: "tools/bench/run-strict-hbc-timing-profile.ts",
      createdAt: new Date().toISOString(),
      cases,
    },
    null,
    2,
  )}\n`,
);
console.log(`Strict HBC timing profile JSON artifact: ${jsonOutputPath}`);
