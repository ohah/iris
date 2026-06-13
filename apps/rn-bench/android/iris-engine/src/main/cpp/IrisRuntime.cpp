#include "IrisRuntime.h"

#include <android/log.h>
#include <cstdlib>

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

jsi::Value IrisRuntime::evaluateJavaScript(
    const std::shared_ptr<const jsi::Buffer>&,
    const std::string&) {
  abortUnimplemented("evaluateJavaScript");
}

std::shared_ptr<const jsi::PreparedJavaScript> IrisRuntime::prepareJavaScript(
    const std::shared_ptr<const jsi::Buffer>&,
    std::string) {
  abortUnimplemented("prepareJavaScript");
}

jsi::Value IrisRuntime::evaluatePreparedJavaScript(
    const std::shared_ptr<const jsi::PreparedJavaScript>&) {
  abortUnimplemented("evaluatePreparedJavaScript");
}

void IrisRuntime::queueMicrotask(const jsi::Function&) {
  abortUnimplemented("queueMicrotask");
}

bool IrisRuntime::drainMicrotasks(int) {
  abortUnimplemented("drainMicrotasks");
}

jsi::Object IrisRuntime::global() {
  abortUnimplemented("global");
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

jsi::Runtime::PointerValue* IrisRuntime::cloneString(const PointerValue*) {
  abortUnimplemented("cloneString");
}

jsi::Runtime::PointerValue* IrisRuntime::cloneObject(const PointerValue*) {
  abortUnimplemented("cloneObject");
}

jsi::Runtime::PointerValue* IrisRuntime::clonePropNameID(const PointerValue*) {
  abortUnimplemented("clonePropNameID");
}

jsi::PropNameID IrisRuntime::createPropNameIDFromAscii(const char*, size_t) {
  abortUnimplemented("createPropNameIDFromAscii");
}

jsi::PropNameID IrisRuntime::createPropNameIDFromUtf8(const uint8_t*, size_t) {
  abortUnimplemented("createPropNameIDFromUtf8");
}

jsi::PropNameID IrisRuntime::createPropNameIDFromString(const jsi::String&) {
  abortUnimplemented("createPropNameIDFromString");
}

jsi::PropNameID IrisRuntime::createPropNameIDFromSymbol(const jsi::Symbol&) {
  abortUnimplemented("createPropNameIDFromSymbol");
}

std::string IrisRuntime::utf8(const jsi::PropNameID&) {
  abortUnimplemented("utf8(PropNameID)");
}

bool IrisRuntime::compare(const jsi::PropNameID&, const jsi::PropNameID&) {
  abortUnimplemented("compare(PropNameID)");
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

jsi::String IrisRuntime::createStringFromAscii(const char*, size_t) {
  abortUnimplemented("createStringFromAscii");
}

jsi::String IrisRuntime::createStringFromUtf8(const uint8_t*, size_t) {
  abortUnimplemented("createStringFromUtf8");
}

std::string IrisRuntime::utf8(const jsi::String&) {
  abortUnimplemented("utf8(String)");
}

jsi::Object IrisRuntime::createObject() {
  abortUnimplemented("createObject");
}

jsi::Object IrisRuntime::createObject(std::shared_ptr<jsi::HostObject>) {
  abortUnimplemented("createObject(HostObject)");
}

std::shared_ptr<jsi::HostObject> IrisRuntime::getHostObject(
    const jsi::Object&) {
  abortUnimplemented("getHostObject");
}

jsi::HostFunctionType& IrisRuntime::getHostFunction(const jsi::Function&) {
  abortUnimplemented("getHostFunction");
}

bool IrisRuntime::hasNativeState(const jsi::Object&) {
  abortUnimplemented("hasNativeState");
}

std::shared_ptr<jsi::NativeState> IrisRuntime::getNativeState(
    const jsi::Object&) {
  abortUnimplemented("getNativeState");
}

void IrisRuntime::setNativeState(
    const jsi::Object&,
    std::shared_ptr<jsi::NativeState>) {
  abortUnimplemented("setNativeState");
}

jsi::Value IrisRuntime::getProperty(
    const jsi::Object&,
    const jsi::PropNameID&) {
  abortUnimplemented("getProperty(PropNameID)");
}

jsi::Value IrisRuntime::getProperty(const jsi::Object&, const jsi::String&) {
  abortUnimplemented("getProperty(String)");
}

bool IrisRuntime::hasProperty(
    const jsi::Object&,
    const jsi::PropNameID&) {
  abortUnimplemented("hasProperty(PropNameID)");
}

bool IrisRuntime::hasProperty(const jsi::Object&, const jsi::String&) {
  abortUnimplemented("hasProperty(String)");
}

void IrisRuntime::setPropertyValue(
    const jsi::Object&,
    const jsi::PropNameID&,
    const jsi::Value&) {
  abortUnimplemented("setPropertyValue(PropNameID)");
}

void IrisRuntime::setPropertyValue(
    const jsi::Object&,
    const jsi::String&,
    const jsi::Value&) {
  abortUnimplemented("setPropertyValue(String)");
}

bool IrisRuntime::isArray(const jsi::Object&) const {
  abortUnimplemented("isArray");
}

bool IrisRuntime::isArrayBuffer(const jsi::Object&) const {
  abortUnimplemented("isArrayBuffer");
}

bool IrisRuntime::isFunction(const jsi::Object&) const {
  abortUnimplemented("isFunction");
}

bool IrisRuntime::isHostObject(const jsi::Object&) const {
  abortUnimplemented("isHostObject");
}

bool IrisRuntime::isHostFunction(const jsi::Function&) const {
  abortUnimplemented("isHostFunction");
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

jsi::Array IrisRuntime::createArray(size_t) {
  abortUnimplemented("createArray");
}

jsi::ArrayBuffer IrisRuntime::createArrayBuffer(
    std::shared_ptr<jsi::MutableBuffer>) {
  abortUnimplemented("createArrayBuffer");
}

size_t IrisRuntime::size(const jsi::Array&) {
  abortUnimplemented("size(Array)");
}

size_t IrisRuntime::size(const jsi::ArrayBuffer&) {
  abortUnimplemented("size(ArrayBuffer)");
}

uint8_t* IrisRuntime::data(const jsi::ArrayBuffer&) {
  abortUnimplemented("data(ArrayBuffer)");
}

jsi::Value IrisRuntime::getValueAtIndex(const jsi::Array&, size_t) {
  abortUnimplemented("getValueAtIndex");
}

void IrisRuntime::setValueAtIndexImpl(
    const jsi::Array&,
    size_t,
    const jsi::Value&) {
  abortUnimplemented("setValueAtIndexImpl");
}

jsi::Function IrisRuntime::createFunctionFromHostFunction(
    const jsi::PropNameID&,
    unsigned int,
    jsi::HostFunctionType) {
  abortUnimplemented("createFunctionFromHostFunction");
}

jsi::Value IrisRuntime::call(
    const jsi::Function&,
    const jsi::Value&,
    const jsi::Value*,
    size_t) {
  abortUnimplemented("call");
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

bool IrisRuntime::strictEquals(const jsi::String&, const jsi::String&) const {
  abortUnimplemented("strictEquals(String)");
}

bool IrisRuntime::strictEquals(const jsi::Object&, const jsi::Object&) const {
  abortUnimplemented("strictEquals(Object)");
}

bool IrisRuntime::instanceOf(const jsi::Object&, const jsi::Function&) {
  abortUnimplemented("instanceOf");
}

void IrisRuntime::setExternalMemoryPressure(const jsi::Object&, size_t) {
  abortUnimplemented("setExternalMemoryPressure");
}

} // namespace iris::runtime
