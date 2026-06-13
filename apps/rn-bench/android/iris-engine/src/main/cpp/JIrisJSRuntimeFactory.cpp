#include "JIrisJSRuntimeFactory.h"

#include "IrisRuntime.h"

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
  return std::make_unique<JSIRuntimeHolder>(
      std::make_unique<iris::runtime::IrisRuntime>());
}

} // namespace facebook::react
