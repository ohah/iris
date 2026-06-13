#!/usr/bin/env sh
# references/ (gitignored)를 채운다. 외부 런타임은 Iris 구현을 베끼는
# 대상이 아니라 React Native/Hermes 호환성과 성능 가설을 비교하는 참고 자료다.
set -eu

root="$(cd "$(dirname "$0")/.." && pwd)"
refs="$root/references"
mkdir -p "$refs"

clone_or_update() {
  name="$1"
  url="$2"
  dir="$refs/$name"

  if [ -d "$dir/.git" ]; then
    echo "update $name"
    git -C "$dir" fetch --depth 1 origin
    git -C "$dir" checkout --quiet FETCH_HEAD
  else
    echo "clone  $name"
    git clone --depth 1 --filter=blob:none "$url" "$dir"
  fi
}

clone_or_update "react-native" "https://github.com/facebook/react-native.git"
clone_or_update "hermes" "https://github.com/facebook/hermes.git"
clone_or_update "lynx" "https://github.com/lynx-family/lynx.git"
clone_or_update "primjs" "https://github.com/lynx-family/primjs.git"
clone_or_update "quickjs" "https://github.com/bellard/quickjs.git"

cat <<'MSG'

완료. references/ 는 gitignore된다.
기본 대상: react-native, hermes, lynx, primjs, quickjs
자세한 사용 규칙은 docs/references.md 를 본다.
MSG
