#include "JIrisJSRuntimeFactory.h"

#include <android/log.h>
#include <fbjni/fbjni.h>

JNIEXPORT jint JNICALL JNI_OnLoad(JavaVM* vm, void*) {
  __android_log_print(ANDROID_LOG_WARN, "IrisEngine", "libirisengine JNI_OnLoad");
  return facebook::jni::initialize(vm, [] {
    facebook::react::JIrisJSRuntimeFactory::registerNatives();
  });
}
