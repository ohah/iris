package com.iris.bench

import com.facebook.react.BaseReactPackage
import com.facebook.react.bridge.NativeModule
import com.facebook.react.bridge.ReactApplicationContext
import com.facebook.react.module.model.ReactModuleInfo
import com.facebook.react.module.model.ReactModuleInfoProvider

class IrisBenchPackage : BaseReactPackage() {
  override fun getModule(
      name: String,
      reactContext: ReactApplicationContext,
  ): NativeModule? =
      when (name) {
        IrisBenchTurboModule.NAME -> IrisBenchTurboModule(reactContext)
        IRIS_BRIDGE_MODULE_NAME -> createOptionalModule(IRIS_BRIDGE_MODULE_CLASS_NAME, reactContext)
        else -> null
      }

  override fun getReactModuleInfoProvider(): ReactModuleInfoProvider =
      ReactModuleInfoProvider {
        val modules =
            mutableMapOf(
            IrisBenchTurboModule.NAME to
                ReactModuleInfo(
                    IrisBenchTurboModule.NAME,
                    IrisBenchTurboModule::class.java.name,
                    canOverrideExistingModule = false,
                    needsEagerInit = false,
                    isCxxModule = false,
                    isTurboModule = true,
                ),
            )

        findOptionalModuleClass(IRIS_BRIDGE_MODULE_CLASS_NAME)?.let { moduleClass ->
          modules[IRIS_BRIDGE_MODULE_NAME] =
              ReactModuleInfo(
                  IRIS_BRIDGE_MODULE_NAME,
                  moduleClass.name,
                  canOverrideExistingModule = false,
                  needsEagerInit = false,
                  isCxxModule = false,
                  isTurboModule = true,
              )
        }

        modules
      }

  private fun createOptionalModule(
      className: String,
      reactContext: ReactApplicationContext,
  ): NativeModule? =
      runCatching {
            findOptionalModuleClass(className)
                ?.getConstructor(ReactApplicationContext::class.java)
                ?.newInstance(reactContext)
          }
          .getOrNull()

  private fun findOptionalModuleClass(className: String): Class<out NativeModule>? =
      runCatching { Class.forName(className).asSubclass(NativeModule::class.java) }.getOrNull()

  companion object {
    private const val IRIS_BRIDGE_MODULE_NAME = "IrisBridgeTurboModule"
    private const val IRIS_BRIDGE_MODULE_CLASS_NAME = "com.iris.bench.IrisBridgeTurboModule"
  }
}
