#!/usr/bin/env sh
set -eu

if [ "$#" -ne 3 ]; then
  echo "usage: $0 <android-build-dir> <ndk-version> <min-sdk>" >&2
  exit 2
fi

ANDROID_BUILD_DIR=$1
NDK_VERSION=$2
MIN_SDK=$3

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

if [ -n "${IRIS_ANDROID_NDK_HOME:-}" ]; then
  NDK_HOME=$IRIS_ANDROID_NDK_HOME
elif [ -n "${ANDROID_HOME:-}" ] && [ -d "$ANDROID_HOME/ndk/$NDK_VERSION" ]; then
  NDK_HOME="$ANDROID_HOME/ndk/$NDK_VERSION"
elif [ -n "${ANDROID_SDK_ROOT:-}" ] && [ -d "$ANDROID_SDK_ROOT/ndk/$NDK_VERSION" ]; then
  NDK_HOME="$ANDROID_SDK_ROOT/ndk/$NDK_VERSION"
elif [ -n "${ANDROID_NDK_HOME:-}" ]; then
  NDK_HOME=$ANDROID_NDK_HOME
elif [ -n "${ANDROID_NDK_ROOT:-}" ]; then
  NDK_HOME=$ANDROID_NDK_ROOT
else
  ANDROID_SDK=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}
  if [ -z "$ANDROID_SDK" ]; then
    echo "ANDROID_HOME, ANDROID_SDK_ROOT, or ANDROID_NDK_HOME must be set" >&2
    exit 1
  fi
  NDK_HOME="$ANDROID_SDK/ndk/$NDK_VERSION"
fi

if [ ! -d "$NDK_HOME" ]; then
  echo "Android NDK not found: $NDK_HOME" >&2
  exit 1
fi

TOOLCHAIN_ROOT="$NDK_HOME/toolchains/llvm/prebuilt"
if [ -d "$TOOLCHAIN_ROOT/darwin-x86_64" ]; then
  HOST_TAG=darwin-x86_64
elif [ -d "$TOOLCHAIN_ROOT/darwin-aarch64" ]; then
  HOST_TAG=darwin-aarch64
elif [ -d "$TOOLCHAIN_ROOT/linux-x86_64" ]; then
  HOST_TAG=linux-x86_64
else
  echo "Unsupported Android NDK host under $TOOLCHAIN_ROOT" >&2
  exit 1
fi

TOOLCHAIN_BIN="$TOOLCHAIN_ROOT/$HOST_TAG/bin"
LLVM_AR="$TOOLCHAIN_BIN/llvm-ar"
LLVM_RANLIB="$TOOLCHAIN_BIN/llvm-ranlib"

if command -v rustup >/dev/null 2>&1; then
  RUSTUP=rustup
elif [ -n "${HOME:-}" ] && [ -x "$HOME/.cargo/bin/rustup" ]; then
  RUSTUP="$HOME/.cargo/bin/rustup"
else
  echo "rustup not found. Install Rust through mise or rustup before building Iris Android." >&2
  exit 1
fi

CXXBRIDGE=${CXXBRIDGE:-"$REPO_ROOT/target/cargo-tools/bin/cxxbridge"}
if ! command -v "$CXXBRIDGE" >/dev/null 2>&1; then
  cargo install --locked cxxbridge-cmd --version 1.0.194 --root "$REPO_ROOT/target/cargo-tools"
fi

GENERATED_DIR="$ANDROID_BUILD_DIR/generated/rust"
GENERATED_CXX_DIR="$GENERATED_DIR/cxxbridge"
GENERATED_INCLUDE_DIR="$GENERATED_DIR/include/rust"
mkdir -p "$GENERATED_CXX_DIR" "$GENERATED_INCLUDE_DIR"

"$CXXBRIDGE" "$REPO_ROOT/crates/iris-hbc/src/lib.rs" -i iris_hbc.h -o "$GENERATED_CXX_DIR/iris_hbc.cc"
"$CXXBRIDGE" "$REPO_ROOT/crates/iris-hbc/src/lib.rs" --header -o "$GENERATED_CXX_DIR/iris_hbc.h"
"$CXXBRIDGE" --header -o "$GENERATED_INCLUDE_DIR/cxx.h"

installed_targets=$("$RUSTUP" target list --installed)
for target in \
  aarch64-linux-android \
  armv7-linux-androideabi \
  i686-linux-android \
  x86_64-linux-android
do
  if ! printf '%s\n' "$installed_targets" | grep -qx "$target"; then
    "$RUSTUP" target add "$target"
  fi
done

build_abi() {
  abi=$1
  target=$2
  clang_prefix=$3
  rust_target_env=$(printf '%s' "$target" | tr '[:lower:]-' '[:upper:]_')
  rust_target_env_lower=$(printf '%s' "$target" | tr '-' '_')
  cc="$TOOLCHAIN_BIN/${clang_prefix}${MIN_SDK}-clang"
  cxx="$TOOLCHAIN_BIN/${clang_prefix}${MIN_SDK}-clang++"

  env \
    "AR_$rust_target_env=$LLVM_AR" \
    "AR_$rust_target_env_lower=$LLVM_AR" \
    "CC_$rust_target_env=$cc" \
    "CC_$rust_target_env_lower=$cc" \
    "CXX_$rust_target_env=$cxx" \
    "CXX_$rust_target_env_lower=$cxx" \
    "RANLIB_$rust_target_env=$LLVM_RANLIB" \
    "RANLIB_$rust_target_env_lower=$LLVM_RANLIB" \
    "CARGO_TARGET_${rust_target_env}_LINKER=$cc" \
    cargo build --manifest-path "$REPO_ROOT/Cargo.toml" --package iris-hbc --release --target "$target"

  mkdir -p "$ANDROID_BUILD_DIR/rust/$abi"
  cp "$REPO_ROOT/target/$target/release/libiris_hbc.a" "$ANDROID_BUILD_DIR/rust/$abi/libiris_hbc.a"
}

build_abi arm64-v8a aarch64-linux-android aarch64-linux-android
build_abi armeabi-v7a armv7-linux-androideabi armv7a-linux-androideabi
build_abi x86 i686-linux-android i686-linux-android
build_abi x86_64 x86_64-linux-android x86_64-linux-android
