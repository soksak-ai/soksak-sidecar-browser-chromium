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

스테이징은 dylib, Chromium framework, helper `.app` 변형(base / Renderer / GPU /
Plugin / Alerts — Chromium은 렌더러를 `… Helper (Renderer).app` 형제 번들에서
띄웁니다)을 배치합니다.

- 개발 스테이징: `./stage.sh dist` 로 identity 홈 `sidecars/` 에 배치 — 해석 경로는 이것뿐(env 바이너리 오버라이드 없음).
- 진단: `SOKSAK_SIDECAR_BROWSER_CHROMIUM_NO_TICK=1` (렌더 틱 비활성)

## 릴리즈

`v*` 태그를 push하면 dist 아카이브가 빌드되어 `…-darwin-arm64.tar.gz`와 sha256이
릴리즈 자산으로 올라갑니다. 플러그인은 그 URL과 해시를 매니페스트에 핀합니다.
현재 자산은 macOS arm64 대상이며 다른 플랫폼은 예정입니다.

## 출처 표기

Chromium은 CEF(Chromium Embedded Framework)를 [`cef`](https://crates.io/crates/cef)
Rust 크레이트로 임베드합니다. Chromium과 CEF는 각 저작자의 BSD 라이선스입니다.
