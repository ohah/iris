import React, { useMemo, useState } from "react";
import {
  FlatList,
  Pressable,
  StatusBar,
  StyleSheet,
  Text,
  useColorScheme,
  View,
} from "react-native";
import { SafeAreaProvider, SafeAreaView } from "react-native-safe-area-context";

type BenchResult = {
  detail: string;
  label: string;
  value: string;
};

type RuntimeGlobals = typeof globalThis & {
  HermesInternal?: unknown;
  __turboModuleProxy?: unknown;
  nativeFabricUIManager?: unknown;
  performance?: {
    now?: () => number;
  };
};

type Row = {
  id: string;
  score: string;
  title: string;
};

const runtime = globalThis as RuntimeGlobals;

function now() {
  return runtime.performance?.now?.() ?? Date.now();
}

function runComputeBench(): BenchResult {
  const startedAt = now();
  let checksum = 0;

  for (let index = 0; index < 600_000; index += 1) {
    checksum += Math.sqrt((index % 1_000) + 1) * Math.sin(index);
  }

  return {
    detail: `600k math ops, checksum ${checksum.toFixed(2)}`,
    label: "JS compute",
    value: `${(now() - startedAt).toFixed(1)} ms`,
  };
}

function runJsonBench(): BenchResult {
  const startedAt = now();
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
    detail: `${encoded.length.toLocaleString()} bytes, checksum ${checksum}`,
    label: "JSON round trip",
    value: `${(now() - startedAt).toFixed(1)} ms`,
  };
}

function createRows(count: number): Row[] {
  return Array.from({ length: count }, (_, index) => ({
    id: String(index),
    score: String((index * 37) % 997).padStart(3, "0"),
    title: `Render row ${index + 1}`,
  }));
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
  const [results, setResults] = useState<BenchResult[]>([]);
  const rows = useMemo(() => createRows(itemCount), [itemCount]);
  const runtimeCards = [
    {
      label: "Hermes",
      value: runtime.HermesInternal ? "enabled" : "missing",
    },
    {
      label: "TurboModule",
      value: runtime.__turboModuleProxy ? "ready" : "not detected",
    },
    {
      label: "Fabric",
      value: runtime.nativeFabricUIManager ? "ready" : "not detected",
    },
    {
      label: "Rows",
      value: itemCount.toLocaleString(),
    },
  ];

  function pushResult(result: BenchResult) {
    setResults((current) => [result, ...current].slice(0, 4));
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
        <BenchmarkButton label="Compute" onPress={() => pushResult(runComputeBench())} />
        <BenchmarkButton label="JSON" onPress={() => pushResult(runJsonBench())} />
        <BenchmarkButton label="List load" onPress={expandList} />
      </View>

      <View style={styles.resultPanel}>
        {results.length === 0 ? (
          <Text style={styles.emptyText}>Run a benchmark to record the Hermes baseline.</Text>
        ) : (
          results.map((result) => (
            <View key={`${result.label}-${result.value}-${result.detail}`} style={styles.resultRow}>
              <View>
                <Text style={styles.resultLabel}>{result.label}</Text>
                <Text style={styles.resultDetail}>{result.detail}</Text>
              </View>
              <Text style={styles.resultValue}>{result.value}</Text>
            </View>
          ))
        )}
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

type BenchmarkButtonProps = {
  label: string;
  onPress: () => void;
};

function BenchmarkButton({ label, onPress }: BenchmarkButtonProps) {
  return (
    <Pressable
      accessibilityRole="button"
      onPress={onPress}
      style={({ pressed }) => [styles.button, pressed && styles.buttonPressed]}
    >
      <Text style={styles.buttonText}>{label}</Text>
    </Pressable>
  );
}

function RenderRow({ row }: { row: Row }) {
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
