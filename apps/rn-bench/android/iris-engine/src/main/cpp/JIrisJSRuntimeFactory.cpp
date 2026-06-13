#include "JIrisJSRuntimeFactory.h"

#include <android/log.h>
#include <cstdlib>
#include <memory>

namespace facebook::react {

jni::local_ref<JIrisJSRuntimeFactory::jhybriddata> JIrisJSRuntimeFactory::initHybrid(
    jni::alias_ref<jclass>) {
  return makeCxxInstance();
}

void JIrisJSRuntimeFactory::registerNatives() {
  registerHybrid({
      makeNativeMethod("initHybrid", JIrisJSRuntimeFactory::initHybrid),
  });
}

std::unique_ptr<JSRuntime> JIrisJSRuntimeFactory::createJSRuntime(
    std::shared_ptr<MessageQueueThread>) noexcept {
  __android_log_assert(
      "IrisRuntimeImplemented",
      "IrisEngine",
      "Iris JS runtime is not implemented yet. This AAR only validates the React Native JSRuntimeFactory ABI.");
  std::abort();
}

} // namespace facebook::react
