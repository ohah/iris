# 라벨 정책

Iris PR은 라벨을 하나 이상 가져야 한다. 필요한 라벨이 없으면 임시로 비슷한 라벨을 쓰지 말고 기본 라벨 세트를 먼저 동기화한다.

```sh
mise run sync-labels
```

## 기본 라벨

| 라벨           | 용도                                            |
| -------------- | ----------------------------------------------- |
| `ci`           | GitHub Actions, branch protection, release gate |
| `docs`         | 문서, 에이전트 가이드, PR 템플릿                |
| `tests`        | 테스트와 검증 경로                              |
| `tooling`      | mise, Bun, Vite+, Oxc, Cargo 같은 개발 도구     |
| `runtime`      | Iris runtime 전체 설계와 실행 경로              |
| `jsi`          | React Native JSI, C++/Rust FFI 경계             |
| `react-native` | React Native 앱, Fabric, TurboModules, Metro    |
| `hermes`       | Hermes 호환성, HBC, Hermes 기준선               |
| `quickjs`      | QuickJS backend 실험                            |
| `performance`  | 성능 예산, 회귀 방지, 측정 인프라               |
| `benchmark`    | 벤치마크 앱과 측정 시나리오                     |
| `rust`         | Rust crate, cxx, memory/lifetime 변경           |
| `dependencies` | 의존성 추가, 업데이트, lockfile 변경            |

## 운영 규칙

- PR 제목과 본문은 한국어로 작성한다.
- 라벨 이름은 GitHub 검색과 자동화를 위해 영어 kebab-case를 쓴다.
- 새 책임 영역이 반복적으로 등장하면 `docs/labels.md`와 `tools/sync-labels.sh`를 함께 수정한다.
- `enhancement` 같은 GitHub 기본 라벨은 쓸 수 있지만, Iris 책임 영역이 명확하면 위 라벨을 우선한다.
