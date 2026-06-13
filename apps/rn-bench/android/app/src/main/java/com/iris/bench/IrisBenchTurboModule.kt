package com.iris.bench

import com.facebook.proguard.annotations.DoNotStrip
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.annotations.ReactModule

@DoNotStrip
@ReactModule(name = IrisBenchTurboModule.NAME)
class IrisBenchTurboModule(reactContext: ReactApplicationContext) :
    NativeIrisBenchTurboModuleSpec(reactContext) {

  @DoNotStrip override fun echoNumber(value: Double): Double = value

  @DoNotStrip override fun getRuntimeLabel(): String = "android-turbomodule"

  @DoNotStrip override fun noop() = Unit

  @DoNotStrip override fun roundTripString(value: String): String = value

  companion object {
    const val NAME = "IrisBenchTurboModule"
  }
}
