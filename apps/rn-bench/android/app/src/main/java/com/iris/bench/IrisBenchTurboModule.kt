package com.iris.bench

import com.facebook.proguard.annotations.DoNotStrip
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.annotations.ReactModule
import kotlin.math.round
import kotlin.math.sin
import kotlin.math.sqrt

@DoNotStrip
@ReactModule(name = IrisBenchTurboModule.NAME)
class IrisBenchTurboModule(reactContext: ReactApplicationContext) :
    NativeIrisBenchTurboModuleSpec(reactContext) {

  @DoNotStrip override fun echoNumber(value: Double): Double = value

  @DoNotStrip override fun getIrisRuntimeLabel(): String = "iris-android-module"

  @DoNotStrip override fun getRuntimeLabel(): String = "android-turbomodule"

  @DoNotStrip override fun noop() = Unit

  @DoNotStrip override fun roundTripString(value: String): String = value

  @DoNotStrip
  override fun runIrisNumericWorkload(iterations: Double): Double {
    val boundedIterations = iterations.toInt().coerceAtLeast(0)
    var checksum = 0.0

    for (index in 0 until boundedIterations) {
      checksum += sqrt(((index % 1_000) + 1).toDouble()) * sin(index.toDouble())
    }

    return round(checksum * 1_000.0) / 1_000.0
  }

  companion object {
    const val NAME = "IrisBenchTurboModule"
  }
}
