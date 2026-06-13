# 레퍼런스

Iris는 React Native와 JavaScript runtime 생태계의 동작을 비교하되, 외부 소스의 코드 표현을 그대로 옮기지 않는다. 이 문서는 로컬 reference checkout의 단일 출처다.

`references/`는 git에 커밋하지 않는 로컬 디렉터리다(`.gitignore`). 각 개발자는 필요할 때 아래 명령으로 채운다.

```sh
mise run fetch-references
```

## 기본 레퍼런스

| 레퍼런스     | 무엇                                                        | 로컬 경로                 | 출처                                     |
| ------------ | ----------------------------------------------------------- | ------------------------- | ---------------------------------------- |
| React Native | RN 0.85+ engine integration, JSI, Fabric, TurboModules 기준 | `references/react-native` | https://github.com/facebook/react-native |
| Hermes       | Hermes V1 동작, bytecode/compiler/runtime 비교 기준         | `references/hermes`       | https://github.com/facebook/hermes       |
| Lynx         | ByteDance Lynx runtime/rendering/threading 구조 참고        | `references/lynx`         | https://github.com/lynx-family/lynx      |
| PrimJS       | Lynx의 JavaScript engine 계층 참고                          | `references/primjs`       | https://github.com/lynx-family/primjs    |
| QuickJS      | Iris QuickJS backend PoC 비교 기준                          | `references/quickjs`      | https://github.com/bellard/quickjs       |

## 선택 레퍼런스

WebKit/JavaScriptCore는 iOS JSC 동작을 이해하는 데 중요하지만 checkout 규모가 매우 크다. 기본 fetch 대상에는 넣지 않는다. 필요하면 별도 논의 후 `references/webkit` 같은 opt-in 경로로 받는다.

## 사용 규칙

허용한다:

- 공개 문서와 reference 동작을 비교해 Iris의 호환성 테스트와 벤치마크 가설을 세운다.
- React Native/Hermes/Lynx/PrimJS/QuickJS의 공개 API, build option, test fixture, architecture 문서를 읽는다.
- Iris 구현과 다른 런타임의 최종 동작을 비교한다.

허용하지 않는다:

- reference source의 자료구조 레이아웃, 함수 분해, iterator/control-flow 구조를 Iris에 그대로 옮긴다.
- 라이선스나 소유권이 불분명한 코드를 복사한다.
- reference checkout을 git에 커밋한다.

PR에서 reference를 근거로 썼다면 어느 reference의 어떤 공개 문서, 테스트, 동작 비교를 봤는지 적는다.
