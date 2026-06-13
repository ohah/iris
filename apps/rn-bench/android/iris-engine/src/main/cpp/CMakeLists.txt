cmake_minimum_required(VERSION 3.22.1)

project(irisengine)

find_package(ReactAndroid REQUIRED CONFIG)
find_package(fbjni REQUIRED CONFIG)

if(NOT DEFINED IRIS_RUST_BUILD_DIR)
  message(FATAL_ERROR "IRIS_RUST_BUILD_DIR is required")
endif()

if(NOT DEFINED IRIS_RUST_GENERATED_DIR)
  message(FATAL_ERROR "IRIS_RUST_GENERATED_DIR is required")
endif()

set(IRIS_HBC_BRIDGE_SOURCE "${IRIS_RUST_GENERATED_DIR}/cxxbridge/iris_hbc.cc")
set(IRIS_HBC_BRIDGE_INCLUDE_DIR "${IRIS_RUST_GENERATED_DIR}/cxxbridge")
set(IRIS_HBC_RUST_INCLUDE_DIR "${IRIS_RUST_GENERATED_DIR}/include")
set(IRIS_HBC_STATICLIB "${IRIS_RUST_BUILD_DIR}/${ANDROID_ABI}/libiris_hbc.a")

foreach(IRIS_REQUIRED_FILE IN ITEMS "${IRIS_HBC_BRIDGE_SOURCE}" "${IRIS_HBC_STATICLIB}")
  if(NOT EXISTS "${IRIS_REQUIRED_FILE}")
    message(FATAL_ERROR "Missing Iris Rust build output: ${IRIS_REQUIRED_FILE}")
  endif()
endforeach()

add_library(iris_hbc STATIC IMPORTED GLOBAL)
set_target_properties(
  iris_hbc
  PROPERTIES
  IMPORTED_LOCATION "${IRIS_HBC_STATICLIB}"
)

add_library(
  irisengine
  SHARED
  IrisRuntime.cpp
  JIrisJSRuntimeFactory.cpp
  OnLoad.cpp
  "${IRIS_HBC_BRIDGE_SOURCE}"
)

target_compile_features(irisengine PRIVATE cxx_std_20)
target_compile_options(
  irisengine
  PRIVATE
  -Wall
  -Werror
  -fexceptions
  -frtti
)
target_compile_definitions(
  irisengine
  PRIVATE
  FOLLY_NO_CONFIG=1
  FOLLY_HAVE_CLOCK_GETTIME=1
  FOLLY_USE_LIBCPP=1
  FOLLY_CFG_NO_COROUTINES=1
  FOLLY_MOBILE=1
  FOLLY_HAVE_RECVMMSG=1
  FOLLY_HAVE_PTHREAD=1
  FOLLY_HAVE_XSI_STRERROR_R=1
  HERMES_V1_ENABLED=1
  RN_SERIALIZABLE_STATE
)
target_include_directories(
  irisengine
  PRIVATE
  "${IRIS_HBC_BRIDGE_INCLUDE_DIR}"
  "${IRIS_HBC_RUST_INCLUDE_DIR}"
)

target_link_libraries(
  irisengine
  iris_hbc
  ReactAndroid::reactnative
  ReactAndroid::jsi
  fbjni::fbjni
  log
)
