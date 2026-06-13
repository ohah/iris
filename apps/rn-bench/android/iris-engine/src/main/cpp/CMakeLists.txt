cmake_minimum_required(VERSION 3.22.1)

project(irisengine)

find_package(ReactAndroid REQUIRED CONFIG)
find_package(fbjni REQUIRED CONFIG)

add_library(
  irisengine
  SHARED
  JIrisJSRuntimeFactory.cpp
  OnLoad.cpp
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

target_link_libraries(
  irisengine
  ReactAndroid::reactnative
  ReactAndroid::jsi
  fbjni::fbjni
  log
)
