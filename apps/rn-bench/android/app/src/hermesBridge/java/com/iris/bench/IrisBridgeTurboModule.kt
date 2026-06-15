package com.iris.bench

import com.facebook.proguard.annotations.DoNotStrip
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.annotations.ReactModule
import com.facebook.react.turbomodule.core.interfaces.BindingsInstallerHolder
import com.facebook.react.turbomodule.core.interfaces.TurboModuleWithJSIBindings
import com.facebook.soloader.SoLoader

@DoNotStrip
@ReactModule(name = IrisBridgeTurboModule.NAME)
class IrisBridgeTurboModule(reactContext: ReactApplicationContext) :
    NativeIrisBridgeTurboModuleSpec(reactContext), TurboModuleWithJSIBindings {

  init {
    SoLoader.loadLibrary("irisbridge")
  }

  @DoNotStrip override fun getRuntimeBackend(): String = BuildConfig.IRIS_RUNTIME_BACKEND

  @DoNotStrip override fun getRuntimeLabel(): String = "iris-jsi-fast-path"

  @DoNotStrip external override fun getBindingsInstaller(): BindingsInstallerHolder

  companion object {
    const val NAME = "IrisBridgeTurboModule"
  }
}
