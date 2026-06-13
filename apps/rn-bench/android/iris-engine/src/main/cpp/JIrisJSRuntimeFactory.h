#pragma once

#include <fbjni/fbjni.h>
#include <jni.h>
#include <react/runtime/JSRuntimeFactory.h>

namespace facebook::react {

// React Native keeps this JNI wrapper header internal, but ReactInstance casts
// Java JSRuntimeFactory objects through this exact hybrid base.
class JJSRuntimeFactory : public jni::HybridClass<JJSRuntimeFactory>, public JSRuntimeFactory {
 public:
  static constexpr auto kJavaDescriptor = "Lcom/facebook/react/runtime/JSRuntimeFactory;";

 private:
  friend HybridBase;
};

class JIrisJSRuntimeFactory
    : public jni::HybridClass<JIrisJSRuntimeFactory, JJSRuntimeFactory> {
 public:
  static constexpr auto kJavaDescriptor = "Lcom/iris/engine/react/IrisJSRuntimeFactory;";

  static jni::local_ref<jhybriddata> initHybrid(jni::alias_ref<jclass>);

  static void registerNatives();

  std::unique_ptr<JSRuntime> createJSRuntime(
      std::shared_ptr<MessageQueueThread> msgQueueThread) noexcept override;

 private:
  friend HybridBase;
};

} // namespace facebook::react
