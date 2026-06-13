package com.iris.engine.react

import com.facebook.react.runtime.JSRuntimeFactory

object IrisJSRuntimeFactoryProvider {
  @JvmStatic fun create(): JSRuntimeFactory = IrisJSRuntimeFactory()
}
