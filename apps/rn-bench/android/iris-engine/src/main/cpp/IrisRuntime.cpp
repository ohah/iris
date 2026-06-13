#include "IrisRuntime.h"
#include "rust/cxx.h"
#include "iris_hbc.h"

#include <android/log.h>

#include <cstdint>
#include <cstdlib>
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
    const auto metadata = iris::hbc::parse_hbc_metadata(
        rust::Slice<const uint8_t>(data, size));
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
        metadata.global_function_frame_size};
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
      "Iris %s prepared Hermes bytecode v%u (%u bytes, %u functions, %u strings, functionHeaders=%u+%u, stringStorage=%u+%u, cjsModules=%u, cjsTable=%u+%u, functionBodies=%u, globalFunction=%u+%u name=%u params=%u frame=%u, source=%s), but bytecode execution is not implemented yet. This is an Iris-owned Runtime scaffold, not a Hermes/JSC fallback.",
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
      sourceURL.c_str());
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

jsi::Value IrisRuntime::evaluateJavaScript(
    const std::shared_ptr<const jsi::Buffer>& buffer,
    const std::string& sourceURL) {
  auto metadata = validateHermesBytecodeBuffer(buffer, sourceURL);
  abortBytecodeExecutionUnavailable(
      "evaluateJavaScript", metadata, sourceURL);
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
  abortBytecodeExecutionUnavailable(
      "evaluatePreparedJavaScript", prepared->metadata, prepared->sourceURL);
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
