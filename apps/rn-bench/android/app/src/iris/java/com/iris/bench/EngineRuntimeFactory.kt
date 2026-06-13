package com.iris.bench

import com.facebook.react.runtime.JSRuntimeFactory
import com.iris.engine.react.IrisJSRuntimeFactoryProvider

internal object EngineRuntimeFactory {
  fun create(): JSRuntimeFactory = IrisJSRuntimeFactoryProvider.create()
}
