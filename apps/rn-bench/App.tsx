import React, { useMemo, useState } from "react";
import {
  FlatList,
  Platform,
  Pressable,
  StatusBar,
  StyleSheet,
  Text,
  useColorScheme,
  View,
} from "react-native";
import { SafeAreaProvider, SafeAreaView } from "react-native-safe-area-context";
import {
  benchmarkCases,
  createRenderStressRows,
  type RenderStressRow,
} from "./src/benchmarks/cases";
import { runBenchmarkSuite } from "./src/benchmarks/harness";
import { createRuntimeMetadata } from "./src/benchmarks/metadata";
import { createTurboModuleBenchmarkCases } from "./src/benchmarks/turboModuleCases";
import type { BenchmarkCaseReport, BenchmarkSuiteReport } from "./src/benchmarks/types";
import {
  getIrisBenchTurboModule,
  isIrisBenchTurboModuleAvailable,
} from "./src/native/IrisBenchTurboModule";

type PlatformConstants = typeof Platform.constants & {
  Brand?: string;
  Model?: string;
  interfaceIdiom?: string;
  reactNativeVersion?: {
    major: number;
    minor: number;
    patch: number;
    prerelease?: string | null;
  };
};

const platformConstants = Platform.constants as PlatformConstants;

function formatReactNativeVersion() {
  const version = platformConstants.reactNativeVersion;

  if (!version) {
    return "0.85.0";
  }

  const prerelease = version.prerelease ? `-${version.prerelease}` : "";
  return `${version.major}.${version.minor}.${version.patch}${prerelease}`;
}

function readDeviceName() {
  return (
    platformConstants.Model ??
    platformConstants.Brand ??
    platformConstants.interfaceIdiom ??
    "unknown"
  );
}

function App() {
  const isDarkMode = useColorScheme() === "dark";

  return (
    <SafeAreaProvider>
      <StatusBar barStyle={isDarkMode ? "light-content" : "dark-content"} />
      <BenchmarkScreen />
    </SafeAreaProvider>
  );
}

function BenchmarkScreen() {
  const [itemCount, setItemCount] = useState(1_000);
  const [report, setReport] = useState<BenchmarkSuiteReport | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const rows = useMemo(() => createRenderStressRows(itemCount), [itemCount]);
  const nativeModule = useMemo(() => getIrisBenchTurboModule(), []);
  const appBenchmarkCases = useMemo(
    () => [...benchmarkCases, ...createTurboModuleBenchmarkCases(nativeModule)],
    [nativeModule],
  );
  const metadata = useMemo(
    () =>
      createRuntimeMetadata({
        appVersion: "0.0.1",
        device: readDeviceName(),
        os: Platform.OS,
        platformVersion: String(Platform.Version),
        reactNativeVersion: formatReactNativeVersion(),
      }),
    [],
  );
  const runtimeCards = [
    {
      label: "Hermes",
      value: metadata.runtime.hermes ? "enabled" : "missing",
    },
    {
      label: "Build",
      value: metadata.build.mode,
    },
    {
      label: "Fabric",
      value: metadata.runtime.fabric ? "ready" : "not detected",
    },
    {
      label: "Iris module",
      value:
        isIrisBenchTurboModuleAvailable() && nativeModule
          ? nativeModule.getIrisRuntimeLabel()
          : "not detected",
    },
    {
      label: "Rows",
      value: itemCount.toLocaleString(),
    },
  ];

  async function runSuite() {
    if (isRunning) {
      return;
    }

    setIsRunning(true);

    try {
      const nextReport = await runBenchmarkSuite(appBenchmarkCases, metadata, {
        yieldBetweenCases: true,
      });
      console.log("IRIS_BENCHMARK_ARTIFACT", JSON.stringify(nextReport));
      setReport(nextReport);
    } finally {
      setIsRunning(false);
    }
  }

  function expandList() {
    setItemCount((current) => (current >= 5_000 ? 1_000 : current + 1_000));
  }

  return (
    <SafeAreaView edges={["top", "left", "right"]} style={styles.screen}>
      <View style={styles.header}>
        <View>
          <Text style={styles.kicker}>Iris PoC baseline</Text>
          <Text style={styles.title}>React Native Hermes</Text>
        </View>
        <View style={styles.versionBadge}>
          <Text style={styles.versionText}>0.85</Text>
        </View>
      </View>

      <View style={styles.statusGrid}>
        {runtimeCards.map((card) => (
          <View key={card.label} style={styles.statusCard}>
            <Text style={styles.statusLabel}>{card.label}</Text>
            <Text style={styles.statusValue}>{card.value}</Text>
          </View>
        ))}
      </View>

      <View style={styles.actions}>
        <BenchmarkButton
          disabled={isRunning}
          label={isRunning ? "Running" : "Run suite"}
          onPress={runSuite}
        />
        <BenchmarkButton label="List load" onPress={expandList} />
      </View>

      <View style={styles.resultPanel}>
        <BenchmarkSummary isRunning={isRunning} report={report} />
      </View>

      <View style={styles.listHeader}>
        <Text style={styles.sectionTitle}>Render stress</Text>
        <Text style={styles.sectionMeta}>{itemCount.toLocaleString()} rows</Text>
      </View>

      <FlatList
        data={rows}
        initialNumToRender={24}
        keyExtractor={(item) => item.id}
        maxToRenderPerBatch={32}
        renderItem={({ item }) => <RenderRow row={item} />}
        style={styles.list}
        windowSize={8}
      />
    </SafeAreaView>
  );
}

function BenchmarkSummary({
  isRunning,
  report,
}: {
  isRunning: boolean;
  report: BenchmarkSuiteReport | null;
}) {
  if (isRunning) {
    return <Text style={styles.emptyText}>Running warmup and measured iterations.</Text>;
  }

  if (!report) {
    return <Text style={styles.emptyText}>Run the suite to record a Hermes baseline report.</Text>;
  }

  return (
    <View>
      <View style={styles.reportHeader}>
        <Text style={styles.reportTitle}>{report.schemaVersion}</Text>
        <Text style={styles.reportMeta}>{report.summary.totalElapsedMs.toFixed(1)} ms total</Text>
      </View>

      {report.cases.map((result) => (
        <BenchmarkResultRow key={result.id} result={result} />
      ))}
    </View>
  );
}

function BenchmarkResultRow({ result }: { result: BenchmarkCaseReport }) {
  return (
    <View style={styles.resultRow}>
      <View style={styles.resultText}>
        <Text style={styles.resultLabel}>{result.label}</Text>
        <Text style={styles.resultDetail}>
          {result.detail}, p50 {result.stats.p50}
          {result.unit}, p95 {result.stats.p95}
          {result.unit}
        </Text>
      </View>
      <Text style={styles.resultValue}>{result.measuredIterations}x</Text>
    </View>
  );
}

type BenchmarkButtonProps = {
  disabled?: boolean;
  label: string;
  onPress: () => void;
};

function BenchmarkButton({ disabled = false, label, onPress }: BenchmarkButtonProps) {
  return (
    <Pressable
      accessibilityRole="button"
      disabled={disabled}
      onPress={onPress}
      style={({ pressed }) => [
        styles.button,
        pressed && styles.buttonPressed,
        disabled && styles.buttonDisabled,
      ]}
    >
      <Text style={styles.buttonText}>{label}</Text>
    </Pressable>
  );
}

function RenderRow({ row }: { row: RenderStressRow }) {
  return (
    <View style={styles.row}>
      <Text style={styles.rowTitle}>{row.title}</Text>
      <Text style={styles.rowScore}>{row.score}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  actions: {
    flexDirection: "row",
    gap: 8,
    marginBottom: 12,
  },
  button: {
    alignItems: "center",
    backgroundColor: "#111827",
    borderRadius: 8,
    flex: 1,
    paddingHorizontal: 12,
    paddingVertical: 12,
  },
  buttonDisabled: {
    backgroundColor: "#6b7280",
  },
  buttonPressed: {
    backgroundColor: "#374151",
  },
  buttonText: {
    color: "#ffffff",
    fontSize: 14,
    fontWeight: "700",
  },
  emptyText: {
    color: "#667073",
    fontSize: 14,
  },
  header: {
    alignItems: "center",
    flexDirection: "row",
    justifyContent: "space-between",
    marginBottom: 16,
  },
  kicker: {
    color: "#0f766e",
    fontSize: 12,
    fontWeight: "700",
    letterSpacing: 0,
    textTransform: "uppercase",
  },
  list: {
    flex: 1,
  },
  listHeader: {
    alignItems: "center",
    flexDirection: "row",
    justifyContent: "space-between",
    marginBottom: 8,
  },
  reportHeader: {
    alignItems: "center",
    flexDirection: "row",
    justifyContent: "space-between",
    marginBottom: 6,
  },
  reportMeta: {
    color: "#667073",
    fontSize: 12,
    fontWeight: "700",
  },
  reportTitle: {
    color: "#0f766e",
    fontSize: 13,
    fontWeight: "800",
  },
  resultDetail: {
    color: "#667073",
    fontSize: 12,
    marginTop: 2,
  },
  resultLabel: {
    color: "#1f2528",
    fontSize: 14,
    fontWeight: "700",
  },
  resultPanel: {
    backgroundColor: "#ffffff",
    borderColor: "#d8dedb",
    borderRadius: 8,
    borderWidth: 1,
    marginBottom: 14,
    minHeight: 78,
    padding: 12,
  },
  resultRow: {
    alignItems: "center",
    borderBottomColor: "#edf0ee",
    borderBottomWidth: 1,
    flexDirection: "row",
    gap: 12,
    justifyContent: "space-between",
    paddingVertical: 8,
  },
  resultText: {
    flex: 1,
  },
  resultValue: {
    color: "#b45309",
    fontSize: 14,
    fontWeight: "800",
  },
  row: {
    alignItems: "center",
    backgroundColor: "#ffffff",
    borderColor: "#e3e7e4",
    borderRadius: 8,
    borderWidth: 1,
    flexDirection: "row",
    justifyContent: "space-between",
    marginBottom: 8,
    paddingHorizontal: 14,
    paddingVertical: 12,
  },
  rowScore: {
    color: "#0f766e",
    fontSize: 13,
    fontWeight: "800",
  },
  rowTitle: {
    color: "#1f2528",
    fontSize: 14,
    fontWeight: "600",
  },
  screen: {
    backgroundColor: "#f6f7f4",
    flex: 1,
    paddingHorizontal: 16,
    paddingTop: 14,
  },
  sectionMeta: {
    color: "#667073",
    fontSize: 13,
    fontWeight: "600",
  },
  sectionTitle: {
    color: "#1f2528",
    fontSize: 16,
    fontWeight: "800",
  },
  statusCard: {
    backgroundColor: "#ffffff",
    borderColor: "#d8dedb",
    borderRadius: 8,
    borderWidth: 1,
    flex: 1,
    minWidth: "47%",
    padding: 12,
  },
  statusGrid: {
    flexDirection: "row",
    flexWrap: "wrap",
    gap: 8,
    marginBottom: 12,
  },
  statusLabel: {
    color: "#667073",
    fontSize: 12,
    fontWeight: "700",
  },
  statusValue: {
    color: "#1f2528",
    fontSize: 15,
    fontWeight: "800",
    marginTop: 6,
  },
  title: {
    color: "#1f2528",
    fontSize: 27,
    fontWeight: "900",
    letterSpacing: 0,
    marginTop: 2,
  },
  versionBadge: {
    alignItems: "center",
    backgroundColor: "#dff5ef",
    borderColor: "#99d9c8",
    borderRadius: 8,
    borderWidth: 1,
    justifyContent: "center",
    minWidth: 58,
    paddingHorizontal: 12,
    paddingVertical: 8,
  },
  versionText: {
    color: "#0f766e",
    fontSize: 14,
    fontWeight: "900",
  },
});

export default App;
