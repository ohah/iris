package com.iris.engine.react

import com.facebook.jni.HybridData
import com.facebook.jni.annotations.DoNotStrip
import com.facebook.react.runtime.JSRuntimeFactory
import com.facebook.soloader.SoLoader

class IrisJSRuntimeFactory : JSRuntimeFactory(initHybrid()) {
  companion object {
    @JvmStatic
    @DoNotStrip
    private external fun initHybrid(): HybridData

    init {
      SoLoader.loadLibrary("irisengine")
    }
  }
}
