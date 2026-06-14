#include "IrisRuntime.h"
#include "rust/cxx.h"
#include "iris_hbc.h"

#include <android/log.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <functional>
#include <iomanip>
#include <numeric>
#include <sstream>
#include <stdexcept>
#include <utility>

namespace iris::runtime {

namespace {

[[noreturn]] void abortUnimplemented(const char* operation) {
  __android_log_assert(
      "IrisRuntimeImplemented",
      "IrisEngine",
      "Iris JSI operation is not implemented yet: %s. This is an Iris-owned Runtime scaffold, not a Hermes/JSC fallback.",
      operation);
  std::abort();
}

double elapsedMilliseconds(std::chrono::steady_clock::time_point start) {
  const auto end = std::chrono::steady_clock::now();
  return std::chrono::duration<double, std::milli>(end - start).count();
}

std::string formatDouble(double value) {
  std::ostringstream output;
  output << std::fixed << std::setprecision(3) << value;
  return output.str();
}

struct MetricStats {
  double min{0.0};
  double max{0.0};
  double mean{0.0};
  double p50{0.0};
  double p95{0.0};
};

MetricStats summarizeSamples(std::vector<double> samples) {
  std::sort(samples.begin(), samples.end());
  const auto sum = std::accumulate(samples.begin(), samples.end(), 0.0);
  const auto percentile = [&samples](double rank) {
    const auto index = static_cast<size_t>(
        std::ceil(rank * static_cast<double>(samples.size()))) -
        1;
    return samples[std::min(index, samples.size() - 1)];
  };

  return MetricStats{
      samples.front(),
      samples.back(),
      sum / static_cast<double>(samples.size()),
      percentile(0.50),
      percentile(0.95)};
}

std::string samplesJson(const std::vector<double>& samples) {
  std::ostringstream output;
  output << "[";
  for (size_t index = 0; index < samples.size(); ++index) {
    if (index > 0) {
      output << ",";
    }
    output << formatDouble(samples[index]);
  }
  output << "]";
  return output.str();
}

std::string jsonEscape(const std::string& value) {
  std::ostringstream output;
  for (const char character : value) {
    switch (character) {
      case '\\':
        output << "\\\\";
        break;
      case '"':
        output << "\\\"";
        break;
      case '\n':
        output << "\\n";
        break;
      case '\r':
        output << "\\r";
        break;
      case '\t':
        output << "\\t";
        break;
      default:
        output << character;
        break;
    }
  }
  return output.str();
}

std::string benchmarkCaseJson(
    const char* id,
    const char* label,
    const char* description,
    const std::string& detail,
    const std::string& checksumJson,
    const std::vector<double>& samples,
    size_t warmupIterations) {
  const auto stats = summarizeSamples(samples);
  std::ostringstream output;
  output << "{\"id\":\"" << id << "\",\"label\":\"" << label
         << "\",\"description\":\"" << description << "\",\"detail\":\""
         << jsonEscape(detail) << "\",\"checksum\":" << checksumJson
         << ",\"warmupIterations\":" << warmupIterations
         << ",\"measuredIterations\":" << samples.size()
         << ",\"unit\":\"ms\",\"stats\":{\"min\":" << formatDouble(stats.min)
         << ",\"max\":" << formatDouble(stats.max)
         << ",\"mean\":" << formatDouble(stats.mean)
         << ",\"p50\":" << formatDouble(stats.p50)
         << ",\"p95\":" << formatDouble(stats.p95)
         << ",\"samples\":" << samplesJson(samples) << "}}";
  return output.str();
}

std::string benchmarkCaseJson(
    const char* id,
    const char* label,
    const char* description,
    const std::string& detail,
    uint64_t checksum,
    const std::vector<double>& samples,
    size_t warmupIterations) {
  return benchmarkCaseJson(
      id,
      label,
      description,
      detail,
      std::to_string(checksum),
      samples,
      warmupIterations);
}

double roundToThreeDecimals(double value) {
  return std::round(value * 1'000.0) / 1'000.0;
}

struct NativeBenchmarkResult {
  std::string checksumJson;
  std::string detail;
};

struct NativeMeasuredCase {
  std::string json;
  size_t measuredIterations{0};
  double totalElapsedMs{0.0};
};

double runNativeComputeChecksum(size_t iterations) {
  double checksum = 0.0;

  for (size_t index = 0; index < iterations; ++index) {
    checksum +=
        std::sqrt(static_cast<double>((index % 1'000) + 1)) *
        std::sin(static_cast<double>(index));
  }

  return roundToThreeDecimals(checksum);
}

uint64_t parseUnsignedInteger(const std::string& value, size_t& cursor) {
  uint64_t parsed = 0;

  while (cursor < value.size() && value[cursor] >= '0' && value[cursor] <= '9') {
    parsed = parsed * 10 + static_cast<uint64_t>(value[cursor] - '0');
    ++cursor;
  }

  return parsed;
}

NativeBenchmarkResult runNativeJsonMirrorCase() {
  std::string encoded;
  encoded.reserve(900'000);
  encoded.push_back('[');
  for (size_t index = 0; index < 8'000; ++index) {
    if (index > 0) {
      encoded.push_back(',');
    }
    encoded += "{\"active\":";
    encoded += index % 3 == 0 ? "true" : "false";
    encoded += ",\"id\":";
    encoded += std::to_string(index);
    encoded += ",\"meta\":{\"lane\":";
    encoded += std::to_string(index % 7);
    encoded += ",\"label\":\"group-";
    encoded += std::to_string(index % 11);
    encoded += "\"},\"name\":\"item-";
    encoded += std::to_string(index);
    encoded += "\",\"points\":[";
    encoded += std::to_string(index);
    encoded.push_back(',');
    encoded += std::to_string(index * 2);
    encoded.push_back(',');
    encoded += std::to_string(index * 3);
    encoded += "]}";
  }
  encoded.push_back(']');

  uint64_t checksum = 0;
  size_t cursor = 0;
  while ((cursor = encoded.find("\"id\":", cursor)) != std::string::npos) {
    cursor += 5;
    const auto id = parseUnsignedInteger(encoded, cursor);
    const auto pointsCursor = encoded.find("\"points\":[", cursor);
    if (pointsCursor == std::string::npos) {
      break;
    }
    cursor = pointsCursor + 10;
    (void)parseUnsignedInteger(encoded, cursor);
    ++cursor;
    (void)parseUnsignedInteger(encoded, cursor);
    ++cursor;
    const auto thirdPoint = parseUnsignedInteger(encoded, cursor);
    checksum += id + thirdPoint;
  }

  return NativeBenchmarkResult{
      std::to_string(checksum),
      std::to_string(encoded.size()) +
          " encoded bytes, native JSON-shape encode/scan mirror"};
}

NativeBenchmarkResult runNativeObjectTraversalMirrorCase() {
  struct Row {
    uint32_t id;
    uint32_t lane;
    uint32_t score;
    bool active;
  };

  std::vector<Row> rows;
  rows.reserve(12'000);
  for (uint32_t index = 0; index < 12'000; ++index) {
    rows.push_back(Row{
        index,
        index % 9,
        (index * 17) % 1'024,
        index % 5 == 0});
  }

  uint64_t checksum = 0;
  for (const auto& row : rows) {
    if (!row.active) {
      checksum += row.lane;
      continue;
    }

    checksum += row.id + row.score;
  }

  return NativeBenchmarkResult{
      std::to_string(checksum),
      std::to_string(rows.size()) + " native structs traversed"};
}

NativeBenchmarkResult runNativeTypedArrayCopyMirrorCase() {
  std::vector<uint8_t> source(1'000'000);
  for (size_t index = 0; index < source.size(); ++index) {
    source[index] = static_cast<uint8_t>(index % 251);
  }

  std::vector<uint8_t> copy(source.size());
  std::copy(source.begin(), source.end(), copy.begin());

  uint64_t checksum = 0;
  for (size_t index = 0; index < copy.size(); index += 10'000) {
    checksum += copy[index];
  }

  return NativeBenchmarkResult{
      std::to_string(checksum),
      std::to_string(copy.size()) + " bytes copied with native vector"};
}

NativeBenchmarkResult runNativeNumberRoundTripMirrorCase() {
  double checksum = 0.0;
  for (size_t index = 0; index < 1'000; ++index) {
    checksum += static_cast<double>(index);
  }

  return NativeBenchmarkResult{
      formatDouble(checksum),
      "1000 native number identity calls, no JS/TurboModule boundary"};
}

NativeBenchmarkResult runNativeStringRoundTripMirrorCase() {
  uint64_t checksum = 0;
  for (size_t index = 0; index < 500; ++index) {
    const auto value = "iris-" + std::to_string(index);
    checksum += value.size();
  }

  return NativeBenchmarkResult{
      std::to_string(checksum),
      "500 native string identity calls, no JS/TurboModule boundary"};
}

NativeMeasuredCase measureNativeMirrorCase(
    const char* id,
    const char* label,
    const char* description,
    size_t warmupIterations,
    size_t measuredIterations,
    const std::function<NativeBenchmarkResult()>& runCase) {
  NativeBenchmarkResult result{"0", "not run"};
  std::vector<double> samples;
  samples.reserve(measuredIterations);

  for (size_t index = 0; index < warmupIterations; ++index) {
    result = runCase();
  }
  for (size_t index = 0; index < measuredIterations; ++index) {
    const auto start = std::chrono::steady_clock::now();
    result = runCase();
    samples.push_back(elapsedMilliseconds(start));
  }

  return NativeMeasuredCase{
      benchmarkCaseJson(
          id,
          label,
          description,
          result.detail,
          result.checksumJson,
          samples,
          warmupIterations),
      samples.size(),
      std::accumulate(samples.begin(), samples.end(), 0.0)};
}

} // namespace

IrisRuntime::IrisPreparedJavaScript::IrisPreparedJavaScript(
    std::shared_ptr<const jsi::Buffer> buffer,
    std::string sourceURL,
    HermesBytecodeMetadata metadata)
    : buffer(std::move(buffer)),
      sourceURL(std::move(sourceURL)),
      metadata(metadata) {}

IrisRuntime::PointerState::PointerState(std::string text, Kind kind)
    : kind(kind), text(std::move(text)) {}

IrisRuntime::PointerState::PointerState(std::shared_ptr<ObjectState> object)
    : kind(Kind::Object), object(std::move(object)) {}

void IrisRuntime::PointerState::invalidate() noexcept {
  delete this;
}

IrisRuntime::IrisRuntime() : globalObject_(std::make_shared<ObjectState>()) {
  installBootstrapGlobals();
}

IrisRuntime::PointerState& IrisRuntime::pointerState(
    const PointerValue* pointer) const {
  if (pointer == nullptr) {
    throw jsi::JSINativeException("IrisRuntime received a null JSI pointer");
  }
  return *const_cast<PointerState*>(static_cast<const PointerState*>(pointer));
}

IrisRuntime::PointerState& IrisRuntime::pointerState(
    const jsi::Object& object) const {
  return pointerState(getPointerValue(object));
}

IrisRuntime::PointerState& IrisRuntime::pointerState(
    const jsi::String& string) const {
  return pointerState(getPointerValue(string));
}

IrisRuntime::PointerState& IrisRuntime::pointerState(
    const jsi::PropNameID& name) const {
  return pointerState(getPointerValue(name));
}

std::shared_ptr<IrisRuntime::ObjectState> IrisRuntime::objectState(
    const jsi::Object& object) const {
  auto& pointer = pointerState(object);
  if (pointer.kind != PointerState::Kind::Object || pointer.object == nullptr) {
    throw jsi::JSINativeException("IrisRuntime expected a JS object pointer");
  }
  return pointer.object;
}

std::string IrisRuntime::propertyKey(const jsi::PropNameID& name) const {
  auto& pointer = pointerState(name);
  if (pointer.kind != PointerState::Kind::PropNameID) {
    throw jsi::JSINativeException("IrisRuntime expected a property name id");
  }
  return pointer.text;
}

std::string IrisRuntime::propertyKey(const jsi::String& string) const {
  auto& pointer = pointerState(string);
  if (pointer.kind != PointerState::Kind::String) {
    throw jsi::JSINativeException("IrisRuntime expected a string");
  }
  return pointer.text;
}

std::shared_ptr<jsi::Value> IrisRuntime::copyValue(const jsi::Value& value) {
  return std::make_shared<jsi::Value>(*this, value);
}

jsi::Object IrisRuntime::makeObject(std::shared_ptr<ObjectState> object) {
  return make<jsi::Object>(new PointerState(std::move(object)));
}

jsi::Value IrisRuntime::makeObjectValue(std::shared_ptr<ObjectState> object) {
  return jsi::Value(makeObject(std::move(object)));
}

jsi::Value IrisRuntime::makeFunctionValue(
    std::string,
    unsigned int,
    jsi::HostFunctionType function) {
  auto object = std::make_shared<ObjectState>();
  object->hostFunction = std::move(function);
  return makeObjectValue(std::move(object));
}

void IrisRuntime::installBootstrapGlobals() {
  auto objectConstructor = std::make_shared<ObjectState>();
  objectConstructor->properties["defineProperty"] = copyValue(
      makeFunctionValue(
          "defineProperty",
          3,
          [](jsi::Runtime& runtime,
             const jsi::Value&,
             const jsi::Value* args,
             size_t count) -> jsi::Value {
            if (count < 3) {
              throw jsi::JSINativeException(
                  "Iris Object.defineProperty requires target, key, and descriptor");
            }

            auto target = args[0].asObject(runtime);
            std::string key = args[1].asString(runtime).utf8(runtime);
            auto descriptor = args[2].asObject(runtime);
            auto value = descriptor.getProperty(runtime, "value");
            target.setProperty(runtime, key.c_str(), value);
            return jsi::Value(runtime, target);
          }));

  globalObject_->properties["Object"] = copyValue(
      makeObjectValue(std::move(objectConstructor)));
}

IrisRuntime::HermesBytecodeMetadata IrisRuntime::validateHermesBytecodeBuffer(
    const std::shared_ptr<const jsi::Buffer>& buffer,
    const std::string& sourceURL) const {
  if (!buffer) {
    abortBundleContractViolation(
        "Iris received a null JavaScript buffer for " + sourceURL);
  }

  const size_t size = buffer->size();
  const uint8_t* data = buffer->data();
  if (data == nullptr) {
    abortBundleContractViolation(
        "Iris expected Hermes bytecode for " + sourceURL +
        ", but the buffer has a null data pointer.");
  }

  try {
    const auto hbcBytes = rust::Slice<const uint8_t>(data, size);
    const auto metadata = iris::hbc::parse_hbc_metadata(hbcBytes);
    return HermesBytecodeMetadata{
        metadata.version,
        metadata.file_length,
        metadata.function_count,
        metadata.function_headers_offset,
        metadata.function_headers_size,
        metadata.string_count,
        metadata.string_storage_offset,
        metadata.string_storage_size,
        metadata.cjs_module_count,
        metadata.cjs_module_table_offset,
        metadata.cjs_module_table_size,
        metadata.function_bodies_offset,
        metadata.global_function_offset,
        metadata.global_function_size,
        metadata.global_function_name,
        metadata.global_function_param_count,
        metadata.global_function_frame_size,
        metadata.global_instruction_count,
        "not scanned"};
  } catch (const rust::Error& error) {
    abortBundleContractViolation(
        "Iris expected Hermes bytecode for " + sourceURL +
        ", but Rust HBC metadata parsing failed: " + error.what() +
        ". Plain JS, JSC, V8, or QuickJS fallback is not allowed.");
  }
}

void IrisRuntime::abortBytecodeExecutionUnavailable(
    const char* operation,
    const HermesBytecodeMetadata& metadata,
    const std::string& sourceURL) const {
  __android_log_assert(
      "IrisBytecodeExecutionUnavailable",
      "IrisEngine",
      "Iris %s prepared Hermes bytecode v%u (%u bytes, %u functions, %u strings, functionHeaders=%u+%u, stringStorage=%u+%u, cjsModules=%u, cjsTable=%u+%u, functionBodies=%u, globalFunction=%u+%u name=%u params=%u frame=%u instructions=%u, source=%s). %s. Bytecode execution is not implemented yet. This is an Iris-owned Runtime scaffold, not a Hermes/JSC fallback.",
      operation,
      metadata.version,
      metadata.fileLength,
      metadata.functionCount,
      metadata.stringCount,
      metadata.functionHeadersOffset,
      metadata.functionHeadersSize,
      metadata.stringStorageOffset,
      metadata.stringStorageSize,
      metadata.cjsModuleCount,
      metadata.cjsModuleTableOffset,
      metadata.cjsModuleTableSize,
      metadata.functionBodiesOffset,
      metadata.globalFunctionOffset,
      metadata.globalFunctionSize,
      metadata.globalFunctionName,
      metadata.globalFunctionParamCount,
      metadata.globalFunctionFrameSize,
      metadata.globalInstructionCount,
      sourceURL.c_str(),
      metadata.executionGap.c_str());
  std::abort();
}

void IrisRuntime::abortBundleContractViolation(const std::string& message)
    const {
  __android_log_assert(
      "IrisBundleContractViolation",
      "IrisEngine",
      "%s",
      message.c_str());
  std::abort();
}

void IrisRuntime::emitBootstrapBenchmarkArtifact(
    const std::shared_ptr<const jsi::Buffer>& buffer,
    const HermesBytecodeMetadata& metadata,
    const std::string& sourceURL) {
  if (emittedBootstrapBenchmarkArtifact_) {
    return;
  }
  emittedBootstrapBenchmarkArtifact_ = true;

  const auto hbcBytes =
      rust::Slice<const uint8_t>(buffer->data(), buffer->size());
  constexpr size_t warmupIterations = 1;
  constexpr size_t metadataIterations = 5;
  constexpr size_t coverageWarmupIterations = 0;
  constexpr size_t coverageIterations = 1;
  constexpr size_t executionWarmupIterations = 1;
  constexpr size_t executionIterations = 3;
  std::vector<double> metadataSamples;
  std::vector<double> coverageSamples;
  std::vector<double> executionSamples;
  std::string coverageDetail = "not scanned";
  std::string executionDetail = "not executed";
  metadataSamples.reserve(metadataIterations);
  coverageSamples.reserve(coverageIterations);
  executionSamples.reserve(executionIterations);

  try {
    for (size_t index = 0; index < warmupIterations; ++index) {
      (void)iris::hbc::parse_hbc_metadata(hbcBytes);
    }
    for (size_t index = 0; index < metadataIterations; ++index) {
      const auto start = std::chrono::steady_clock::now();
      (void)iris::hbc::parse_hbc_metadata(hbcBytes);
      metadataSamples.push_back(elapsedMilliseconds(start));
    }

    for (size_t index = 0; index < coverageWarmupIterations; ++index) {
      (void)iris::hbc::describe_hbc_execution_gap(hbcBytes);
    }
    for (size_t index = 0; index < coverageIterations; ++index) {
      const auto start = std::chrono::steady_clock::now();
      coverageDetail =
          std::string(iris::hbc::describe_hbc_execution_gap(hbcBytes));
      coverageSamples.push_back(elapsedMilliseconds(start));
    }

    for (size_t index = 0; index < executionWarmupIterations; ++index) {
      (void)iris::hbc::describe_hbc_scalar_execution(hbcBytes);
    }
    for (size_t index = 0; index < executionIterations; ++index) {
      const auto start = std::chrono::steady_clock::now();
      executionDetail =
          std::string(iris::hbc::describe_hbc_scalar_execution(hbcBytes));
      executionSamples.push_back(elapsedMilliseconds(start));
    }
  } catch (const rust::Error& error) {
    abortBundleContractViolation(
        "Iris bootstrap benchmark failed while handling Hermes bytecode for " +
        sourceURL + ": " + error.what());
  }

  const std::string detail =
      std::to_string(metadata.fileLength) + " bytes, " +
      std::to_string(metadata.functionCount) + " functions, " +
      std::to_string(metadata.stringCount) + " strings, " +
      std::to_string(metadata.globalInstructionCount) +
      " global instructions";
  const auto metadataCase = benchmarkCaseJson(
      "iris-hbc-metadata-parse",
      "Iris HBC metadata parse",
      "Parse Hermes bytecode metadata.",
      detail,
      static_cast<uint64_t>(metadata.fileLength),
      metadataSamples,
      warmupIterations);
  const auto coverageCase = benchmarkCaseJson(
      "iris-hbc-static-coverage-scan",
      "Iris HBC static coverage scan",
      "Scan global Hermes bytecode opcodes.",
      coverageDetail,
      static_cast<uint64_t>(metadata.globalInstructionCount),
      coverageSamples,
      coverageWarmupIterations);
  const auto executionCase = benchmarkCaseJson(
      "iris-hbc-scalar-execution-frontier",
      "Iris HBC scalar execution frontier",
      "Execute the current Rust scalar subset until completion or the first semantic frontier.",
      executionDetail,
      static_cast<uint64_t>(metadata.globalInstructionCount),
      executionSamples,
      executionWarmupIterations);
  const std::vector<NativeMeasuredCase> nativeMirrorCases{
      measureNativeMirrorCase(
          "iris-native-js-compute-mirror",
          "Iris native JS compute mirror",
          "Native C++ mirror of the Hermes JS compute arithmetic loop. This does not execute JavaScript.",
          3,
          15,
          [] {
            return NativeBenchmarkResult{
                formatDouble(runNativeComputeChecksum(600'000)),
                "600k native math operations, no JS bytecode execution"};
          }),
      measureNativeMirrorCase(
          "iris-native-json-round-trip-mirror",
          "Iris native JSON mirror",
          "Native C++ JSON-shape encode/scan mirror of the Hermes JSON round trip. This is not JSON.parse.",
          3,
          15,
          runNativeJsonMirrorCase),
      measureNativeMirrorCase(
          "iris-native-object-traversal-mirror",
          "Iris native object traversal mirror",
          "Native C++ struct creation/traversal mirror of the Hermes object traversal case.",
          3,
          15,
          runNativeObjectTraversalMirrorCase),
      measureNativeMirrorCase(
          "iris-native-typed-array-copy-mirror",
          "Iris native typed array copy mirror",
          "Native C++ byte vector allocation/copy mirror of the Hermes TypedArray case.",
          3,
          15,
          runNativeTypedArrayCopyMirrorCase),
      measureNativeMirrorCase(
          "iris-native-number-round-trip-mirror",
          "Iris native number round trip mirror",
          "Native C++ number identity loop matching the TurboModule sample count. This has no JS/TurboModule boundary.",
          5,
          20,
          runNativeNumberRoundTripMirrorCase),
      measureNativeMirrorCase(
          "iris-native-string-round-trip-mirror",
          "Iris native string round trip mirror",
          "Native C++ string identity loop matching the TurboModule sample count. This has no JS/TurboModule boundary.",
          5,
          20,
          runNativeStringRoundTripMirrorCase),
      measureNativeMirrorCase(
          "iris-native-module-compute-mirror",
          "Iris native module compute mirror",
          "Native C++ mirror of IrisBenchTurboModule.runIrisNumericWorkload. This has no JS/TurboModule boundary.",
          3,
          15,
          [] {
            return NativeBenchmarkResult{
                formatDouble(runNativeComputeChecksum(600'000)),
                "600000 native math operations, no JS/TurboModule boundary"};
          })};

  const auto totalElapsedMs =
      std::accumulate(metadataSamples.begin(), metadataSamples.end(), 0.0) +
      std::accumulate(coverageSamples.begin(), coverageSamples.end(), 0.0) +
      std::accumulate(executionSamples.begin(), executionSamples.end(), 0.0) +
      std::accumulate(
          nativeMirrorCases.begin(),
          nativeMirrorCases.end(),
          0.0,
          [](double total, const NativeMeasuredCase& benchmarkCase) {
            return total + benchmarkCase.totalElapsedMs;
          });
  const auto totalMeasuredIterations =
      metadataSamples.size() + coverageSamples.size() +
      executionSamples.size() +
      std::accumulate(
          nativeMirrorCases.begin(),
          nativeMirrorCases.end(),
          static_cast<size_t>(0),
          [](size_t total, const NativeMeasuredCase& benchmarkCase) {
            return total + benchmarkCase.measuredIterations;
          });

  std::ostringstream artifact;
  artifact
      << "{\"schemaVersion\":\"iris.benchmark.v1\","
      << "\"createdAt\":\"native-logcat\","
      << "\"artifact\":{\"kind\":\"runtime-log\",\"generatedBy\":\"IrisRuntime::emitBootstrapBenchmarkArtifact\",\"path\":\"logcat:IrisEngine\"},"
      << "\"suite\":{\"id\":\"iris-engine-bootstrap\",\"name\":\"Iris engine bootstrap\"},"
      << "\"metadata\":{\"app\":{\"name\":\"IrisBench\",\"version\":\"0.0.1\"},"
      << "\"build\":{\"commit\":\"unknown\",\"mode\":\"release\",\"source\":\"native-iris-runtime\"},"
      << "\"platform\":{\"device\":\"android-physical\",\"os\":\"android\",\"version\":\"unknown\"},"
      << "\"reactNative\":{\"version\":\"0.85.0\"},"
      << "\"runtime\":{\"fabric\":true,\"hermes\":false,\"hermesVersion\":\"none\",\"jsEngine\":\"iris-skeleton\",\"newArchitecture\":true,\"turboModuleProxy\":true}},"
      << "\"cases\":[" << metadataCase << "," << coverageCase << ","
      << executionCase;
  for (const auto& nativeMirrorCase : nativeMirrorCases) {
    artifact << "," << nativeMirrorCase.json;
  }
  artifact << "],"
           << "\"summary\":{\"caseCount\":" << (3 + nativeMirrorCases.size())
           << ",\"measuredIterations\":" << totalMeasuredIterations
           << ",\"totalElapsedMs\":" << formatDouble(totalElapsedMs) << "}}";

  const auto artifactText = artifact.str();
  constexpr size_t logChunkSize = 800;
  const auto chunkCount =
      (artifactText.size() + logChunkSize - 1) / logChunkSize;
  for (size_t index = 0; index < chunkCount; ++index) {
    const auto offset = index * logChunkSize;
    const auto chunk = artifactText.substr(offset, logChunkSize);
    __android_log_print(
        ANDROID_LOG_INFO,
        "IrisEngine",
        "IRIS_BENCHMARK_ARTIFACT_CHUNK %zu/%zu %s",
        index + 1,
        chunkCount,
        chunk.c_str());
  }
  __android_log_print(
      ANDROID_LOG_WARN,
      "IrisEngine",
      "Iris emitted bootstrap benchmark for Hermes bytecode v%u from %s. Full bytecode execution is still unavailable, so this is not an RN JS workload benchmark.",
      metadata.version,
      sourceURL.c_str());
}

jsi::Value IrisRuntime::evaluateJavaScript(
    const std::shared_ptr<const jsi::Buffer>& buffer,
    const std::string& sourceURL) {
  auto metadata = validateHermesBytecodeBuffer(buffer, sourceURL);
  emitBootstrapBenchmarkArtifact(buffer, metadata, sourceURL);
  return jsi::Value::undefined();
}

std::shared_ptr<const jsi::PreparedJavaScript> IrisRuntime::prepareJavaScript(
    const std::shared_ptr<const jsi::Buffer>& buffer,
    std::string sourceURL) {
  auto metadata = validateHermesBytecodeBuffer(buffer, sourceURL);
  return std::make_shared<IrisPreparedJavaScript>(
      buffer, std::move(sourceURL), metadata);
}

jsi::Value IrisRuntime::evaluatePreparedJavaScript(
    const std::shared_ptr<const jsi::PreparedJavaScript>& js) {
  auto prepared = dynamic_cast<const IrisPreparedJavaScript*>(js.get());
  if (prepared == nullptr) {
    abortBundleContractViolation(
        "Iris evaluatePreparedJavaScript received a script prepared by a"
        " different runtime.");
  }
  emitBootstrapBenchmarkArtifact(
      prepared->buffer, prepared->metadata, prepared->sourceURL);
  return jsi::Value::undefined();
}

void IrisRuntime::queueMicrotask(const jsi::Function&) {
  abortUnimplemented("queueMicrotask");
}

bool IrisRuntime::drainMicrotasks(int) {
  return true;
}

jsi::Object IrisRuntime::global() {
  return makeObject(globalObject_);
}

std::string IrisRuntime::description() {
  return "IrisRuntime(Android JSI scaffold)";
}

bool IrisRuntime::isInspectable() {
  return false;
}

jsi::Runtime::PointerValue* IrisRuntime::cloneSymbol(const PointerValue*) {
  abortUnimplemented("cloneSymbol");
}

jsi::Runtime::PointerValue* IrisRuntime::cloneBigInt(const PointerValue*) {
  abortUnimplemented("cloneBigInt");
}

jsi::Runtime::PointerValue* IrisRuntime::cloneString(
    const PointerValue* pointer) {
  auto& state = pointerState(pointer);
  return new PointerState(state.text, PointerState::Kind::String);
}

jsi::Runtime::PointerValue* IrisRuntime::cloneObject(
    const PointerValue* pointer) {
  auto& state = pointerState(pointer);
  return new PointerState(state.object);
}

jsi::Runtime::PointerValue* IrisRuntime::clonePropNameID(
    const PointerValue* pointer) {
  auto& state = pointerState(pointer);
  return new PointerState(state.text, PointerState::Kind::PropNameID);
}

jsi::PropNameID IrisRuntime::createPropNameIDFromAscii(
    const char* str,
    size_t length) {
  return make<jsi::PropNameID>(
      new PointerState(std::string(str, length), PointerState::Kind::PropNameID));
}

jsi::PropNameID IrisRuntime::createPropNameIDFromUtf8(
    const uint8_t* utf8,
    size_t length) {
  return make<jsi::PropNameID>(new PointerState(
      std::string(reinterpret_cast<const char*>(utf8), length),
      PointerState::Kind::PropNameID));
}

jsi::PropNameID IrisRuntime::createPropNameIDFromString(
    const jsi::String& string) {
  return make<jsi::PropNameID>(
      new PointerState(propertyKey(string), PointerState::Kind::PropNameID));
}

jsi::PropNameID IrisRuntime::createPropNameIDFromSymbol(const jsi::Symbol&) {
  abortUnimplemented("createPropNameIDFromSymbol");
}

std::string IrisRuntime::utf8(const jsi::PropNameID& name) {
  return propertyKey(name);
}

bool IrisRuntime::compare(
    const jsi::PropNameID& left,
    const jsi::PropNameID& right) {
  return propertyKey(left) == propertyKey(right);
}

std::string IrisRuntime::symbolToString(const jsi::Symbol&) {
  abortUnimplemented("symbolToString");
}

jsi::BigInt IrisRuntime::createBigIntFromInt64(int64_t) {
  abortUnimplemented("createBigIntFromInt64");
}

jsi::BigInt IrisRuntime::createBigIntFromUint64(uint64_t) {
  abortUnimplemented("createBigIntFromUint64");
}

bool IrisRuntime::bigintIsInt64(const jsi::BigInt&) {
  abortUnimplemented("bigintIsInt64");
}

bool IrisRuntime::bigintIsUint64(const jsi::BigInt&) {
  abortUnimplemented("bigintIsUint64");
}

uint64_t IrisRuntime::truncate(const jsi::BigInt&) {
  abortUnimplemented("truncate(BigInt)");
}

jsi::String IrisRuntime::bigintToString(const jsi::BigInt&, int) {
  abortUnimplemented("bigintToString");
}

jsi::String IrisRuntime::createStringFromAscii(
    const char* str,
    size_t length) {
  return make<jsi::String>(
      new PointerState(std::string(str, length), PointerState::Kind::String));
}

jsi::String IrisRuntime::createStringFromUtf8(
    const uint8_t* utf8,
    size_t length) {
  return make<jsi::String>(new PointerState(
      std::string(reinterpret_cast<const char*>(utf8), length),
      PointerState::Kind::String));
}

std::string IrisRuntime::utf8(const jsi::String& string) {
  return propertyKey(string);
}

jsi::Object IrisRuntime::createObject() {
  return makeObject(std::make_shared<ObjectState>());
}

jsi::Object IrisRuntime::createObject(std::shared_ptr<jsi::HostObject> hostObject) {
  auto object = std::make_shared<ObjectState>();
  object->hostObject = std::move(hostObject);
  return makeObject(std::move(object));
}

std::shared_ptr<jsi::HostObject> IrisRuntime::getHostObject(
    const jsi::Object& object) {
  auto state = objectState(object);
  if (!state->hostObject) {
    throw jsi::JSINativeException("IrisRuntime object is not a HostObject");
  }
  return state->hostObject;
}

jsi::HostFunctionType& IrisRuntime::getHostFunction(
    const jsi::Function& function) {
  auto state = objectState(function);
  if (!state->hostFunction) {
    throw jsi::JSINativeException("IrisRuntime object is not a HostFunction");
  }
  return *state->hostFunction;
}

bool IrisRuntime::hasNativeState(const jsi::Object& object) {
  return objectState(object)->nativeState != nullptr;
}

std::shared_ptr<jsi::NativeState> IrisRuntime::getNativeState(
    const jsi::Object& object) {
  return objectState(object)->nativeState;
}

void IrisRuntime::setNativeState(
    const jsi::Object& object,
    std::shared_ptr<jsi::NativeState> state) {
  objectState(object)->nativeState = std::move(state);
}

jsi::Value IrisRuntime::getProperty(
    const jsi::Object& object,
    const jsi::PropNameID& name) {
  auto state = objectState(object);
  auto key = propertyKey(name);
  auto property = state->properties.find(key);
  if (property != state->properties.end()) {
    return jsi::Value(*this, *property->second);
  }
  if (state->hostObject) {
    return state->hostObject->get(*this, name);
  }
  return jsi::Value::undefined();
}

jsi::Value IrisRuntime::getProperty(
    const jsi::Object& object,
    const jsi::String& name) {
  auto key = propertyKey(name);
  auto propName = createPropNameIDFromUtf8(
      reinterpret_cast<const uint8_t*>(key.data()), key.size());
  return getProperty(object, propName);
}

bool IrisRuntime::hasProperty(
    const jsi::Object& object,
    const jsi::PropNameID& name) {
  auto state = objectState(object);
  auto key = propertyKey(name);
  if (state->properties.find(key) != state->properties.end()) {
    return true;
  }
  if (!state->hostObject) {
    return false;
  }
  return !state->hostObject->get(*this, name).isUndefined();
}

bool IrisRuntime::hasProperty(const jsi::Object& object, const jsi::String& name) {
  auto key = propertyKey(name);
  auto propName = createPropNameIDFromUtf8(
      reinterpret_cast<const uint8_t*>(key.data()), key.size());
  return hasProperty(object, propName);
}

void IrisRuntime::setPropertyValue(
    const jsi::Object& object,
    const jsi::PropNameID& name,
    const jsi::Value& value) {
  auto state = objectState(object);
  if (state->hostObject) {
    state->hostObject->set(*this, name, value);
    return;
  }
  state->properties[propertyKey(name)] = copyValue(value);
}

void IrisRuntime::setPropertyValue(
    const jsi::Object& object,
    const jsi::String& name,
    const jsi::Value& value) {
  auto key = propertyKey(name);
  auto propName = createPropNameIDFromUtf8(
      reinterpret_cast<const uint8_t*>(key.data()), key.size());
  setPropertyValue(object, propName, value);
}

bool IrisRuntime::isArray(const jsi::Object& object) const {
  return objectState(object)->isArray;
}

bool IrisRuntime::isArrayBuffer(const jsi::Object&) const {
  return false;
}

bool IrisRuntime::isFunction(const jsi::Object& object) const {
  return objectState(object)->hostFunction.has_value();
}

bool IrisRuntime::isHostObject(const jsi::Object& object) const {
  return objectState(object)->hostObject != nullptr;
}

bool IrisRuntime::isHostFunction(const jsi::Function& function) const {
  return objectState(function)->hostFunction.has_value();
}

jsi::Array IrisRuntime::getPropertyNames(const jsi::Object&) {
  abortUnimplemented("getPropertyNames");
}

jsi::WeakObject IrisRuntime::createWeakObject(const jsi::Object&) {
  abortUnimplemented("createWeakObject");
}

jsi::Value IrisRuntime::lockWeakObject(const jsi::WeakObject&) {
  abortUnimplemented("lockWeakObject");
}

jsi::Array IrisRuntime::createArray(size_t length) {
  auto object = std::make_shared<ObjectState>();
  object->isArray = true;
  object->elements.resize(length);
  return make<jsi::Array>(new PointerState(std::move(object)));
}

jsi::ArrayBuffer IrisRuntime::createArrayBuffer(
    std::shared_ptr<jsi::MutableBuffer>) {
  abortUnimplemented("createArrayBuffer");
}

size_t IrisRuntime::size(const jsi::Array& array) {
  return objectState(array)->elements.size();
}

size_t IrisRuntime::size(const jsi::ArrayBuffer&) {
  abortUnimplemented("size(ArrayBuffer)");
}

uint8_t* IrisRuntime::data(const jsi::ArrayBuffer&) {
  abortUnimplemented("data(ArrayBuffer)");
}

jsi::Value IrisRuntime::getValueAtIndex(const jsi::Array& array, size_t index) {
  auto state = objectState(array);
  if (index >= state->elements.size() || !state->elements[index]) {
    return jsi::Value::undefined();
  }
  return jsi::Value(*this, *state->elements[index]);
}

void IrisRuntime::setValueAtIndexImpl(
    const jsi::Array& array,
    size_t index,
    const jsi::Value& value) {
  auto state = objectState(array);
  if (index >= state->elements.size()) {
    state->elements.resize(index + 1);
  }
  state->elements[index] = copyValue(value);
}

jsi::Function IrisRuntime::createFunctionFromHostFunction(
    const jsi::PropNameID&,
    unsigned int paramCount,
    jsi::HostFunctionType function) {
  auto value = makeFunctionValue("", paramCount, std::move(function));
  return std::move(value).getObject(*this).getFunction(*this);
}

jsi::Value IrisRuntime::call(
    const jsi::Function& function,
    const jsi::Value& jsThis,
    const jsi::Value* args,
    size_t count) {
  auto state = objectState(function);
  if (!state->hostFunction) {
    abortUnimplemented("call(non-host-function)");
  }
  return (*state->hostFunction)(*this, jsThis, args, count);
}

jsi::Value IrisRuntime::callAsConstructor(
    const jsi::Function&,
    const jsi::Value*,
    size_t) {
  abortUnimplemented("callAsConstructor");
}

bool IrisRuntime::strictEquals(const jsi::Symbol&, const jsi::Symbol&) const {
  abortUnimplemented("strictEquals(Symbol)");
}

bool IrisRuntime::strictEquals(const jsi::BigInt&, const jsi::BigInt&) const {
  abortUnimplemented("strictEquals(BigInt)");
}

bool IrisRuntime::strictEquals(
    const jsi::String& left,
    const jsi::String& right) const {
  return propertyKey(left) == propertyKey(right);
}

bool IrisRuntime::strictEquals(
    const jsi::Object& left,
    const jsi::Object& right) const {
  return objectState(left) == objectState(right);
}

bool IrisRuntime::instanceOf(const jsi::Object&, const jsi::Function&) {
  abortUnimplemented("instanceOf");
}

void IrisRuntime::setExternalMemoryPressure(const jsi::Object&, size_t) {
  // Iris does not have a GC heap yet. Keep this as a no-op until object memory
  // accounting exists, so RN HostObjects can still be registered.
}

} // namespace iris::runtime
