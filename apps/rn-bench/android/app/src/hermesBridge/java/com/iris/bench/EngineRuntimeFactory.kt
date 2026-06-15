package com.iris.bench

import com.facebook.react.runtime.JSRuntimeFactory
import com.facebook.react.runtime.hermes.HermesInstance

internal object EngineRuntimeFactory {
  fun create(): JSRuntimeFactory = HermesInstance()
}

