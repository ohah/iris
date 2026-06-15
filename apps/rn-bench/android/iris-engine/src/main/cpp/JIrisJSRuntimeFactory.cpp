#include "JIrisJSRuntimeFactory.h"

#include "IrisRuntime.h"

#include <android/log.h>

#include <memory>

namespace facebook::react {

jni::local_ref<JIrisJSRuntimeFactory::jhybriddata> JIrisJSRuntimeFactory::initHybrid(
    jni::alias_ref<jclass>) {
  __android_log_print(ANDROID_LOG_WARN, "IrisEngine", "Iris initHybrid");
  return makeCxxInstance();
}

void JIrisJSRuntimeFactory::registerNatives() {
  __android_log_print(ANDROID_LOG_WARN, "IrisEngine", "Iris registerNatives");
  registerHybrid({
      makeNativeMethod("initHybrid", JIrisJSRuntimeFactory::initHybrid),
  });
}

std::unique_ptr<JSRuntime> JIrisJSRuntimeFactory::createJSRuntime(
    std::shared_ptr<MessageQueueThread>) noexcept {
  __android_log_print(ANDROID_LOG_WARN, "IrisEngine", "Iris createJSRuntime");
  return std::make_unique<JSIRuntimeHolder>(
      std::make_unique<iris::runtime::IrisRuntime>());
}

} // namespace facebook::react
