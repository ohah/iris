#include <hermes/hermes.h>
#include <jsi/jsi.h>

#include <chrono>
#include <cmath>
#include <cstdint>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <memory>
#include <sstream>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace jsi = facebook::jsi;

namespace {

class VectorBuffer final : public jsi::Buffer {
 public:
  explicit VectorBuffer(std::vector<uint8_t> bytes) : bytes_(std::move(bytes)) {}

  size_t size() const override {
    return bytes_.size();
  }

  const uint8_t* data() const override {
    return bytes_.data();
  }

 private:
  std::vector<uint8_t> bytes_;
};

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

std::vector<uint8_t> readFile(const std::string& path) {
  std::ifstream input(path, std::ios::binary);
  if (!input) {
    throw std::runtime_error("failed to open " + path);
  }

  return std::vector<uint8_t>(
      std::istreambuf_iterator<char>(input), std::istreambuf_iterator<char>());
}

uint32_t parseIterations(const std::string& name, const std::string& value, bool allowZero) {
  size_t parsedLength = 0;
  const auto parsed = std::stoul(value, &parsedLength, 10);
  if (parsedLength != value.size()) {
    throw std::runtime_error(name + " must be an integer");
  }
  if (!allowZero && parsed == 0) {
    throw std::runtime_error(name + " must be greater than zero");
  }
  if (parsed > UINT32_MAX) {
    throw std::runtime_error(name + " is too large");
  }
  return static_cast<uint32_t>(parsed);
}

std::string valueJson(jsi::Runtime& runtime, const jsi::Value& value) {
  if (value.isBool()) {
    return value.getBool() ? "true" : "false";
  }
  if (value.isNull()) {
    return "null";
  }
  if (value.isNumber()) {
    const auto number = value.asNumber();
    if (!std::isfinite(number)) {
      return "\"" + jsonEscape(std::to_string(number)) + "\"";
    }

    std::ostringstream output;
    output << std::setprecision(15) << number;
    return output.str();
  }
  if (value.isString()) {
    return "\"" + jsonEscape(value.asString(runtime).utf8(runtime)) + "\"";
  }
  if (value.isUndefined()) {
    return "\"undefined\"";
  }
  return "\"object\"";
}

double elapsedMilliseconds(std::chrono::steady_clock::time_point start) {
  return std::chrono::duration<double, std::milli>(
             std::chrono::steady_clock::now() - start)
      .count();
}

std::string samplesJson(const std::vector<double>& samples) {
  std::ostringstream output;
  output << "[";
  for (size_t index = 0; index < samples.size(); ++index) {
    if (index > 0) {
      output << ",";
    }
    output << std::fixed << std::setprecision(6) << samples[index];
  }
  output << "]";
  return output.str();
}

std::string usage(const char* program) {
  return std::string("usage: ") + program + " [--warmup=N] [--iterations=N] <bundle.hbc>";
}

} // namespace

int main(int argc, char** argv) {
  try {
    uint32_t warmupIterations = 3;
    uint32_t measuredIterations = 20;
    std::string path;

    for (int index = 1; index < argc; ++index) {
      const std::string arg = argv[index];
      if (arg.rfind("--warmup=", 0) == 0) {
        warmupIterations = parseIterations("--warmup", arg.substr(9), true);
      } else if (arg.rfind("--iterations=", 0) == 0) {
        measuredIterations = parseIterations("--iterations", arg.substr(13), false);
      } else if (arg.rfind("--", 0) == 0) {
        throw std::runtime_error("unknown option: " + arg + "\n" + usage(argv[0]));
      } else if (path.empty()) {
        path = arg;
      } else {
        throw std::runtime_error("multiple HBC paths provided\n" + usage(argv[0]));
      }
    }

    if (path.empty()) {
      throw std::runtime_error(usage(argv[0]));
    }

    auto bytes = readFile(path);
    auto runtime = facebook::hermes::makeHermesRuntime();
    auto buffer = std::make_shared<VectorBuffer>(std::move(bytes));
    auto prepared = runtime->prepareJavaScript(buffer, path);
    jsi::Value value;

    for (uint32_t index = 0; index < warmupIterations; ++index) {
      value = runtime->evaluatePreparedJavaScript(prepared);
    }

    std::vector<double> samples;
    samples.reserve(measuredIterations);
    for (uint32_t index = 0; index < measuredIterations; ++index) {
      const auto start = std::chrono::steady_clock::now();
      value = runtime->evaluatePreparedJavaScript(prepared);
      samples.push_back(elapsedMilliseconds(start));
    }

    std::cout << "{\"engine\":\"hermes\",\"casePath\":\"" << jsonEscape(path)
              << "\",\"value\":" << valueJson(*runtime, value)
              << ",\"warmupIterations\":" << warmupIterations
              << ",\"measuredIterations\":" << measuredIterations
              << ",\"samplesMs\":" << samplesJson(samples) << "}\n";
    return 0;
  } catch (const std::exception& error) {
    std::cerr << error.what() << "\n";
    return 1;
  }
}
