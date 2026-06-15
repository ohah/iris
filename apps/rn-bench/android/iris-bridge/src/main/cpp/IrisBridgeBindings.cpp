#include "IrisBridgeBindings.h"

#include <algorithm>
#include <cmath>
#include <string>
#include <vector>

namespace facebook::react {
namespace {

constexpr auto kFastPathGlobalName = "__irisBridgeFastPath";
constexpr auto kMaxNativeBufferByteLength = 16 * 1024 * 1024;
constexpr auto kMaxNativeRowCount = 1'000'000;
constexpr auto kNativeRowColumnCount = 4;

class IrisNativeBuffer final : public jsi::MutableBuffer {
 public:
  explicit IrisNativeBuffer(size_t byteLength) : bytes_(byteLength) {
    for (auto index = 0U; index < bytes_.size(); index += 1) {
      bytes_[index] = static_cast<uint8_t>(index % 251);
    }
  }

  size_t size() const override {
    return bytes_.size();
  }

  uint8_t* data() override {
    return bytes_.data();
  }

 private:
  std::vector<uint8_t> bytes_;
};

class IrisRowsBuffer final : public jsi::MutableBuffer {
 public:
  explicit IrisRowsBuffer(size_t rowCount) : values_(rowCount * kNativeRowColumnCount) {
    for (auto index = 0U; index < rowCount; index += 1) {
      const auto base = index * kNativeRowColumnCount;
      values_[base] = static_cast<int32_t>(index);
      values_[base + 1] = static_cast<int32_t>(index % 9);
      values_[base + 2] = index % 5 == 0 ? 1 : 0;
      values_[base + 3] = static_cast<int32_t>((index * 17) % 1024);
    }
  }

  size_t size() const override {
    return values_.size() * sizeof(int32_t);
  }

  uint8_t* data() override {
    return reinterpret_cast<uint8_t*>(values_.data());
  }

 private:
  std::vector<int32_t> values_;
};

double runNativeCompute(double iterations) {
  const auto boundedIterations = std::max(0, static_cast<int>(iterations));
  auto checksum = 0.0;

  for (auto index = 0; index < boundedIterations; index += 1) {
    checksum += std::sqrt(static_cast<double>((index % 1000) + 1)) *
        std::sin(static_cast<double>(index));
  }

  return std::round(checksum * 1000.0) / 1000.0;
}

jsi::Value echoNumber(
    jsi::Runtime& runtime,
    const jsi::Value* arguments,
    size_t count) {
  if (count == 0 || !arguments[0].isNumber()) {
    throw jsi::JSError(runtime, "echoNumber expects one number argument");
  }

  return jsi::Value(arguments[0].asNumber());
}

jsi::Value roundTripString(
    jsi::Runtime& runtime,
    const jsi::Value* arguments,
    size_t count) {
  if (count == 0 || !arguments[0].isString()) {
    throw jsi::JSError(runtime, "roundTripString expects one string argument");
  }

  auto value = arguments[0].asString(runtime).utf8(runtime);
  return jsi::String::createFromUtf8(runtime, value);
}

jsi::Value nativeCompute(
    jsi::Runtime& runtime,
    const jsi::Value* arguments,
    size_t count) {
  if (count == 0 || !arguments[0].isNumber()) {
    throw jsi::JSError(runtime, "nativeCompute expects one number argument");
  }

  return jsi::Value(runNativeCompute(arguments[0].asNumber()));
}

jsi::Value createNativeBuffer(
    jsi::Runtime& runtime,
    const jsi::Value* arguments,
    size_t count) {
  if (count == 0 || !arguments[0].isNumber()) {
    throw jsi::JSError(runtime, "createNativeBuffer expects one number argument");
  }

  const auto requestedByteLength = arguments[0].asNumber();
  if (requestedByteLength < 0 || requestedByteLength > kMaxNativeBufferByteLength) {
    throw jsi::JSError(runtime, "createNativeBuffer byte length is out of range");
  }

  const auto byteLength = static_cast<size_t>(requestedByteLength);
  return jsi::ArrayBuffer(runtime, std::make_shared<IrisNativeBuffer>(byteLength));
}

jsi::Value createNativeRowsBuffer(
    jsi::Runtime& runtime,
    const jsi::Value* arguments,
    size_t count) {
  if (count == 0 || !arguments[0].isNumber()) {
    throw jsi::JSError(runtime, "createNativeRowsBuffer expects one number argument");
  }

  const auto requestedRowCount = arguments[0].asNumber();
  if (requestedRowCount < 0 || requestedRowCount > kMaxNativeRowCount) {
    throw jsi::JSError(runtime, "createNativeRowsBuffer row count is out of range");
  }

  const auto rowCount = static_cast<size_t>(requestedRowCount);
  return jsi::ArrayBuffer(runtime, std::make_shared<IrisRowsBuffer>(rowCount));
}

void installFastPath(jsi::Runtime& runtime) {
  auto fastPath = jsi::Object(runtime);

  fastPath.setProperty(
      runtime,
      "echoNumber",
      jsi::Function::createFromHostFunction(
          runtime,
          jsi::PropNameID::forAscii(runtime, "echoNumber"),
          1,
          [](jsi::Runtime& runtime,
             const jsi::Value&,
             const jsi::Value* arguments,
             size_t count) {
            return echoNumber(runtime, arguments, count);
          }));
  fastPath.setProperty(
      runtime,
      "roundTripString",
      jsi::Function::createFromHostFunction(
          runtime,
          jsi::PropNameID::forAscii(runtime, "roundTripString"),
          1,
          [](jsi::Runtime& runtime,
             const jsi::Value&,
             const jsi::Value* arguments,
             size_t count) {
            return roundTripString(runtime, arguments, count);
          }));
  fastPath.setProperty(
      runtime,
      "nativeCompute",
      jsi::Function::createFromHostFunction(
          runtime,
          jsi::PropNameID::forAscii(runtime, "nativeCompute"),
          1,
          [](jsi::Runtime& runtime,
             const jsi::Value&,
             const jsi::Value* arguments,
             size_t count) {
            return nativeCompute(runtime, arguments, count);
          }));
  fastPath.setProperty(
      runtime,
      "createNativeBuffer",
      jsi::Function::createFromHostFunction(
          runtime,
          jsi::PropNameID::forAscii(runtime, "createNativeBuffer"),
          1,
          [](jsi::Runtime& runtime,
             const jsi::Value&,
             const jsi::Value* arguments,
             size_t count) {
            return createNativeBuffer(runtime, arguments, count);
          }));
  fastPath.setProperty(
      runtime,
      "createNativeRowsBuffer",
      jsi::Function::createFromHostFunction(
          runtime,
          jsi::PropNameID::forAscii(runtime, "createNativeRowsBuffer"),
          1,
          [](jsi::Runtime& runtime,
             const jsi::Value&,
             const jsi::Value* arguments,
             size_t count) {
            return createNativeRowsBuffer(runtime, arguments, count);
          }));

  runtime.global().setProperty(runtime, kFastPathGlobalName, std::move(fastPath));
}

} // namespace

void IrisBridgeBindings::registerNatives() {
  javaClassLocal()->registerNatives({
      makeNativeMethod("getBindingsInstaller", IrisBridgeBindings::getBindingsInstaller),
  });
}

jni::local_ref<BindingsInstallerHolder::javaobject> IrisBridgeBindings::getBindingsInstaller(
    jni::alias_ref<IrisBridgeBindings> /*jobj*/) {
  return BindingsInstallerHolder::newObjectCxxArgs(
      [](jsi::Runtime& runtime, const std::shared_ptr<CallInvoker>&) {
        installFastPath(runtime);
      });
}

} // namespace facebook::react
