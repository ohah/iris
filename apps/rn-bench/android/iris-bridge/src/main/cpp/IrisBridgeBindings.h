#pragma once

#include <ReactCommon/BindingsInstallerHolder.h>
#include <fbjni/fbjni.h>
#include <jsi/jsi.h>

namespace facebook::react {

class IrisBridgeBindings : public jni::JavaClass<IrisBridgeBindings> {
 public:
  static constexpr const char* kJavaDescriptor = "Lcom/iris/bench/IrisBridgeTurboModule;";

  IrisBridgeBindings() = default;

  static void registerNatives();

 private:
  static jni::local_ref<BindingsInstallerHolder::javaobject> getBindingsInstaller(
      jni::alias_ref<IrisBridgeBindings> jobj);
};

} // namespace facebook::react
