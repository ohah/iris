package com.iris.engine.react

import android.util.Log
import com.facebook.jni.HybridData
import com.facebook.jni.annotations.DoNotStrip
import com.facebook.react.runtime.JSRuntimeFactory
import com.facebook.soloader.SoLoader

class IrisJSRuntimeFactory : JSRuntimeFactory(initHybrid()) {
  init {
    Log.w("IrisEngine", "IrisJSRuntimeFactory created")
  }

  companion object {
    @JvmStatic
    @DoNotStrip
    private external fun initHybrid(): HybridData

    init {
      Log.w("IrisEngine", "Loading libirisengine")
      SoLoader.loadLibrary("irisengine")
      Log.w("IrisEngine", "Loaded libirisengine")
    }
  }
}
