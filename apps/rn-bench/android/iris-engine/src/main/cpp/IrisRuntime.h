#pragma once

#include <jsi/jsi.h>

#include <cstdint>
#include <memory>
#include <optional>
#include <string>
#include <unordered_map>
#include <vector>

namespace iris::runtime {

namespace jsi = facebook::jsi;

class IrisRuntime final : public jsi::Runtime {
 public:
  IrisRuntime();
  ~IrisRuntime() override = default;

  jsi::Value evaluateJavaScript(
      const std::shared_ptr<const jsi::Buffer>&,
      const std::string&) override;
  std::shared_ptr<const jsi::PreparedJavaScript> prepareJavaScript(
      const std::shared_ptr<const jsi::Buffer>&,
      std::string) override;
  jsi::Value evaluatePreparedJavaScript(
      const std::shared_ptr<const jsi::PreparedJavaScript>&) override;
  void queueMicrotask(const jsi::Function&) override;
  bool drainMicrotasks(int maxMicrotasksHint = -1) override;
  jsi::Object global() override;
  std::string description() override;
  bool isInspectable() override;

 protected:
  PointerValue* cloneSymbol(const PointerValue*) override;
  PointerValue* cloneBigInt(const PointerValue*) override;
  PointerValue* cloneString(const PointerValue*) override;
  PointerValue* cloneObject(const PointerValue*) override;
  PointerValue* clonePropNameID(const PointerValue*) override;

  jsi::PropNameID createPropNameIDFromAscii(const char*, size_t) override;
  jsi::PropNameID createPropNameIDFromUtf8(const uint8_t*, size_t) override;
  jsi::PropNameID createPropNameIDFromString(const jsi::String&) override;
  jsi::PropNameID createPropNameIDFromSymbol(const jsi::Symbol&) override;
  std::string utf8(const jsi::PropNameID&) override;
  bool compare(const jsi::PropNameID&, const jsi::PropNameID&) override;

  std::string symbolToString(const jsi::Symbol&) override;

  jsi::BigInt createBigIntFromInt64(int64_t) override;
  jsi::BigInt createBigIntFromUint64(uint64_t) override;
  bool bigintIsInt64(const jsi::BigInt&) override;
  bool bigintIsUint64(const jsi::BigInt&) override;
  uint64_t truncate(const jsi::BigInt&) override;
  jsi::String bigintToString(const jsi::BigInt&, int) override;

  jsi::String createStringFromAscii(const char*, size_t) override;
  jsi::String createStringFromUtf8(const uint8_t*, size_t) override;
  std::string utf8(const jsi::String&) override;

  jsi::Object createObject() override;
  jsi::Object createObject(std::shared_ptr<jsi::HostObject>) override;
  std::shared_ptr<jsi::HostObject> getHostObject(const jsi::Object&) override;
  jsi::HostFunctionType& getHostFunction(const jsi::Function&) override;

  bool hasNativeState(const jsi::Object&) override;
  std::shared_ptr<jsi::NativeState> getNativeState(const jsi::Object&) override;
  void setNativeState(
      const jsi::Object&,
      std::shared_ptr<jsi::NativeState>) override;

  jsi::Value getProperty(const jsi::Object&, const jsi::PropNameID&) override;
  jsi::Value getProperty(const jsi::Object&, const jsi::String&) override;
  bool hasProperty(const jsi::Object&, const jsi::PropNameID&) override;
  bool hasProperty(const jsi::Object&, const jsi::String&) override;
  void setPropertyValue(
      const jsi::Object&,
      const jsi::PropNameID&,
      const jsi::Value&) override;
  void setPropertyValue(
      const jsi::Object&,
      const jsi::String&,
      const jsi::Value&) override;

  bool isArray(const jsi::Object&) const override;
  bool isArrayBuffer(const jsi::Object&) const override;
  bool isFunction(const jsi::Object&) const override;
  bool isHostObject(const jsi::Object&) const override;
  bool isHostFunction(const jsi::Function&) const override;
  jsi::Array getPropertyNames(const jsi::Object&) override;

  jsi::WeakObject createWeakObject(const jsi::Object&) override;
  jsi::Value lockWeakObject(const jsi::WeakObject&) override;

  jsi::Array createArray(size_t) override;
  jsi::ArrayBuffer createArrayBuffer(
      std::shared_ptr<jsi::MutableBuffer>) override;
  size_t size(const jsi::Array&) override;
  size_t size(const jsi::ArrayBuffer&) override;
  uint8_t* data(const jsi::ArrayBuffer&) override;
  jsi::Value getValueAtIndex(const jsi::Array&, size_t) override;
  void setValueAtIndexImpl(
      const jsi::Array&,
      size_t,
      const jsi::Value&) override;

  jsi::Function createFunctionFromHostFunction(
      const jsi::PropNameID&,
      unsigned int,
      jsi::HostFunctionType) override;
  jsi::Value call(
      const jsi::Function&,
      const jsi::Value&,
      const jsi::Value*,
      size_t) override;
  jsi::Value callAsConstructor(
      const jsi::Function&,
      const jsi::Value*,
      size_t) override;

  bool strictEquals(const jsi::Symbol&, const jsi::Symbol&) const override;
  bool strictEquals(const jsi::BigInt&, const jsi::BigInt&) const override;
  bool strictEquals(const jsi::String&, const jsi::String&) const override;
  bool strictEquals(const jsi::Object&, const jsi::Object&) const override;

  bool instanceOf(const jsi::Object&, const jsi::Function&) override;

  void setExternalMemoryPressure(const jsi::Object&, size_t) override;

 private:
  struct ObjectState {
    std::unordered_map<std::string, std::shared_ptr<jsi::Value>> properties;
    std::shared_ptr<jsi::HostObject> hostObject;
    std::optional<jsi::HostFunctionType> hostFunction;
    std::shared_ptr<jsi::NativeState> nativeState;
    std::vector<std::shared_ptr<jsi::Value>> elements;
    bool isArray{false};
  };

  struct HermesBytecodeMetadata {
    uint32_t version;
    uint32_t fileLength;
    uint32_t functionCount;
    uint32_t functionHeadersOffset;
    uint32_t functionHeadersSize;
    uint32_t stringCount;
    uint32_t stringStorageOffset;
    uint32_t stringStorageSize;
    uint32_t cjsModuleCount;
    uint32_t cjsModuleTableOffset;
    uint32_t cjsModuleTableSize;
    uint32_t functionBodiesOffset;
    uint32_t globalFunctionOffset;
    uint32_t globalFunctionSize;
    uint32_t globalFunctionName;
    uint32_t globalFunctionParamCount;
    uint32_t globalFunctionFrameSize;
    uint32_t globalInstructionCount;
  };

  struct IrisPreparedJavaScript final : public jsi::PreparedJavaScript {
    IrisPreparedJavaScript(
        std::shared_ptr<const jsi::Buffer> buffer,
        std::string sourceURL,
        HermesBytecodeMetadata metadata);

    std::shared_ptr<const jsi::Buffer> buffer;
    std::string sourceURL;
    HermesBytecodeMetadata metadata;
  };

  struct PointerState final : public PointerValue {
    enum class Kind {
      PropNameID,
      String,
      Object,
    };

    explicit PointerState(std::string text, Kind kind);
    explicit PointerState(std::shared_ptr<ObjectState> object);

    void invalidate() noexcept override;

    Kind kind;
    std::string text;
    std::shared_ptr<ObjectState> object;
  };

  PointerState& pointerState(const PointerValue*) const;
  PointerState& pointerState(const jsi::Object&) const;
  PointerState& pointerState(const jsi::String&) const;
  PointerState& pointerState(const jsi::PropNameID&) const;
  std::shared_ptr<ObjectState> objectState(const jsi::Object&) const;
  std::string propertyKey(const jsi::PropNameID&) const;
  std::string propertyKey(const jsi::String&) const;
  std::shared_ptr<jsi::Value> copyValue(const jsi::Value&);

  jsi::Object makeObject(std::shared_ptr<ObjectState>);
  jsi::Value makeObjectValue(std::shared_ptr<ObjectState>);
  jsi::Value makeFunctionValue(
      std::string name,
      unsigned int paramCount,
      jsi::HostFunctionType);

  void installBootstrapGlobals();
  HermesBytecodeMetadata validateHermesBytecodeBuffer(
      const std::shared_ptr<const jsi::Buffer>&,
      const std::string&) const;
  [[noreturn]] void abortBytecodeExecutionUnavailable(
      const char*,
      const HermesBytecodeMetadata&,
      const std::string&) const;
  [[noreturn]] void abortBundleContractViolation(const std::string&) const;

  std::shared_ptr<ObjectState> globalObject_;
};

} // namespace iris::runtime
