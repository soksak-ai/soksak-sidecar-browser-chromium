# soksak-sidecar-browser-chromium

soksak의 **브라우저 ▸ Chromium** 탭 뒤에서 도는 번들 Chromium 엔진 — 코어가
인프로세스로 로드하는 엔진 사이드카(공유 네이티브 모듈)입니다.

## 무엇인가

- 브라우저 pane에 진짜 Chromium을 네이티브 속도로 렌더합니다. OS 웹뷰와 달리
  어느 설치에서나 동일한 엔진입니다.
- **앱이 아닙니다.** 자기 창도, 독 아이콘도, 실행할 것도 없습니다 — 브라우저
  플러그인이 열 때 코어가 로드하는 dylib이며, 프로세스 목록에 이 이름의 helper
  (renderer/GPU/network)들이 보이는 것은 Chromium의 표준 멀티프로세스 구조입니다.
- **손으로 설치할 것이 없습니다.** `soksak-plugin-browser-chromium` 플러그인이
  이 사이드카를 선언하면 soksak이 핀 고정된 릴리즈 아카이브를 자동으로 받아
  sha256 검증 후 설치합니다. 검증 실패 시 아무것도 설치되지 않습니다.

## 연결 방식

- **프로토콜** `soksak-spec-sidecar-browser` — 요청 `create / bounds / load / reload /
  back / forward / hidden / focus / close / popup-mode`, 이벤트 `nav / title /
  popup-url`. 브라우저 플러그인과 이 모듈 사이의 계약이며 코어는 해석 없이
  전달만 합니다.
- **호스팅 ABI** — export된 `soksak_sidecar_engine_*` C 심볼. 로드 시 코어가
  모듈의 자기보고 interface를 플러그인 선언과 대조하고 불일치면 거부합니다.
  한번 로드된 모듈은 앱 수명 동안 상주합니다.

## 개발

```sh
cargo build --release
./stage.sh ~/.soksak/sidecars/soksak-sidecar-browser-chromium/dist
```

스테이징은 OS 별로 동작합니다(`stage.sh` 가 단일 진실). macOS 에서는 dylib,
Chromium framework, helper `.app` 변형(base / Renderer / GPU / Plugin / Alerts —
Chromium 은 렌더러를 `… Helper (Renderer).app` 형제 번들에서 띄웁니다)을
배치하고, Linux 와 Windows 에서는 공유 라이브러리, `libcef` 와 리소스·로케일,
helper 바이너리를 배치합니다.

- 개발 스테이징: `./stage.sh dist` 로 identity 홈 `sidecars/` 에 배치 — 해석 경로는 이것뿐(env 바이너리 오버라이드 없음).
- 진단: `SOKSAK_SIDECAR_BROWSER_CHROMIUM_NO_TICK=1` (렌더 틱 비활성)

## 플랫폼

엔진은 macOS·Linux·Windows 에서 빌드되고 동작합니다. 하나의 인터페이스 뒤에
OS 별 프레젠터가 있습니다: macOS 는 Metal 로 present 하고, Linux 와 Windows 는
CEF 공유 텍스처를 wgpu 로 가져와 코어가 전달한 부모 핸들 아래의 child 창에
렌더합니다. 세 CI 워크플로우가 모든 플랫폼을 같은 기준으로 지킵니다 — 5타깃
빌드 매트릭스(컴파일·링크), 그리고 실제 present 프레임과 전체 입력 왕복을
단언하는 Linux(xvfb + 소프트웨어 Vulkan)·Windows(DX12 WARP) 온스크린 harness
실행입니다. 자세한 내용은 [docs/PLATFORMS.ko.md](docs/PLATFORMS.ko.md) 에 있습니다.

## 릴리즈

`main` 에서 수동 workflow dispatch 하면 각 플랫폼의 네이티브 러너에서 dist
아카이브를 빌드해 — darwin arm64/x64·linux arm64/x64·windows x64 — 다섯 개의
아카이브와 sha256 파일을 immutable 릴리즈 자산으로 발행합니다. 플러그인은
플랫폼별 URL 과 해시를 매니페스트에 핀합니다.

## 출처 표기

Chromium은 CEF(Chromium Embedded Framework)를 [`cef`](https://crates.io/crates/cef)
Rust 크레이트로 임베드합니다. Chromium과 CEF는 각 저작자의 BSD 라이선스입니다.
