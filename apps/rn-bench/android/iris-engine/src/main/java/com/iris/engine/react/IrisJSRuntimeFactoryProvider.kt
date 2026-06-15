package com.iris.engine.react

import android.util.Log
import com.facebook.react.runtime.JSRuntimeFactory

object IrisJSRuntimeFactoryProvider {
  @JvmStatic
  fun create(): JSRuntimeFactory {
    Log.w("IrisEngine", "Creating Iris JSRuntimeFactory provider output")
    return IrisJSRuntimeFactory()
  }
}
