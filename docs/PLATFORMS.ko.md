# 크로스플랫폼 렌더·프레젠테이션

Chromium 엔진 사이드카가 macOS·Windows·Linux에서 pane 콘텐츠를 렌더·present하는
방식. 이 크레이트의 플랫폼 작업 정본 규칙이다. 옛 노트에서 다른 규칙을 유도하지 않는다.

## 규칙

**사이드카가 네이티브 프레젠테이션을 소유한다.** 엔진은 CEF로 offscreen 렌더하고
(windowless + 공유 텍스처), 각 플랫폼이 그 공유 텍스처를 모듈 소유 네이티브 표면에
present한다. 프레젠테이션은 사이드카 안에 두며 코어로 옮기지 않는다 — 작동하는 macOS
경로를 불변으로 두는 한, 한 인터페이스 뒤의 per-OS 프레젠터가 유일하게 정합적인 형태다.

프레임 경로는 호스트 vtable·IPC·JS를 통과하지 않는다(SIDECARS.md §8): 픽셀은 CEF 공유
텍스처에서 네이티브 표면으로 직접 blit된다.

## 인터페이스와 per-OS 모듈

`presenter/mod.rs`는 플랫폼 무관 인터페이스(서피스 수명·bounds·hidden·popup·present).
`engine.rs`는 이 인터페이스만 부르고 플랫폼 모듈을 직접 부르지 않는다.

- `presenter/macos.rs` — 프로덕션. IOSurface → Metal blit → CALayer. 동결 레퍼런스
  경로이고 hand-rolled(raw Metal) 그대로 불변.
- `presenter/windows.rs`·`presenter/linux.rs` — CEF 공유 텍스처를 `cef` 크레이트의
  `osr_texture_import`(피처 `accelerated_osr`)로 `wgpu::Texture` 로 가져와, 네이티브
  child 창의 `wgpu::Surface` 에 렌더한다.

CEF `on_accelerated_paint`는 플랫폼별로 다른 핸들을 준다(macOS IOSurface·Windows
D3D11 `HANDLE`·Linux DMA-BUF planes). 세 네이티브 GPU 스택을 손으로 굴리는 대신 신규
플랫폼은 크레이트의 통합 임포터를 쓴다: `SharedTextureHandle::new(info).import_texture(&device)`
가 `wgpu::Texture` 를 반환한다(Linux DMA-BUF→Vulkan·Windows D3D11→Vulkan interop, CPU
폴백 포함). 그래서 Windows·Linux 는 present 메커니즘 하나(wgpu)를 공유하고, macOS 만
raw Metal(작동 경로=동결 레퍼런스)에 남는다. `present`는 `&AcceleratedPaintInfo` 를 받아
각자 소비한다(macOS 는 `shared_texture_io_surface`, wgpu 프레젠터는 info 전체를 임포터에).
wgpu 는 `cef` 크레이트가 쓰는 버전(29)에 핀하고 non-macOS 타깃에서만 켜, macOS dylib 은
wgpu 를 끌어오지 않는다.

## 오라클

`offscreen.rs`는 동결 레퍼런스(오라클)이고 `harness` 피처에서만 컴파일된다. 프로덕션
경로가 아니며 재활용하지 않는다 — 구현으로 재활용한 오라클은 아무것도 검증 못 한다.
`presenter/macos.rs`가 그 알고리즘을 재현하는 프로덕션 사본이고, 하니스가 프로덕션
출력 == 오라클 출력을 단언한다.

## 검증

- macOS·Linux는 로컬 컴파일: `cargo check --target <triple>`. exit 코드를 직접 잡는다 —
  `cargo check … | tail`은 파이프(tail)의 exit를 보고해 실패를 가린다.
- Windows는 CI 전용. `cef-dll-sys`가 CEF C++ 래퍼를 빌드할 때 리소스 컴파일러를
  요구하는데 macOS 크로스컴파일 환경엔 없다. Linux가 비-macOS 코드 정합성의 로컬 프록시.
- 네이티브 present는 각 OS 런타임에서 CI 검증(Linux는 xvfb). 컴파일만 되는 스텁은
  합격한 플랫폼이 아니다.

### 플랫폼 간 멱등

터미널 계약(정규형 투영 + 오라클)을 옮긴 두 평면:

- 제어면(canonical): 각 프레젠터의 프레임경로 결정(surface scale·present coded size·
  colorspace·popup rect)을 정규형으로 투영해 cross-OS byte-exact 대조 — 세 프레젠터가
  같은 결정을 해야 한다.
- 데이터면(fidelity): 각 OS에서 present된 표면 == CEF 원본 프레임(프레젠터는 픽셀 무손실
  도관). CEF가 원본을 cross-OS 동일하게 보장한다.

## 상태

- **macOS**: 프로덕션 present(raw Metal, `presenter/macos.rs`)는 harness 런타임 검증됨
  (프레임 present + 입력, IME 포함).
- **Linux**: `presenter/linux.rs` — 부모 XID 아래 X11 child 창(`x11-dl`), 그 위 `wgpu::Surface`,
  `osr_texture_import` → textured-quad 렌더 → present — 구현 완료, linux 타깃 클린 컴파일.
  온스크린(child 창이 부모 아래 뜨고 프레임 보임)은 CI xvfb 검증이며 컴파일이 아니다.
- **Windows**: `presenter/windows.rs`(같은 wgpu present + HWND child 창)는 미작성.
  macOS 에서 컴파일 불가(CEF C++ 래퍼가 Windows 리소스 컴파일러 필요)라 CI 에서 작성·검증.
- Linux/Windows CEF 적재(`libcef.{so,dll}`)는 여전히 스텁 — macOS `.framework` 경로만 배선.

Linux 는 가속(`on_accelerated_paint`→import→present)·CPU 폴백(`on_paint`→업로드→`present_cpu`)
두 경로가 배선됨. SW GL CI(lavapipe)엔 하드웨어 DMA-BUF 가 없어 CEF 가 CPU 경로를 타므로, 그 폴백이
CI 에서 프레임이 뜨게 하는 관건이다.

## 검증 게이트

| 범위 | 검증 방법 | 위치 |
|---|---|---|
| macOS 프로덕션 present | harness 가 페이지 렌더 — 프레임+입력(IME 포함) | 로컬(harness 실행) |
| macOS/Linux 코드 정합 | `cargo check --target <triple>`(exit 직접 잡기) | 로컬 |
| Windows 코드 | `cargo check`(CEF C++ 래퍼가 Windows RC 필요) | CI 전용 |
| 온스크린 렌더(Linux/Windows) | xvfb 에서 harness 빌드·실행, 프레임 단언 | CI 전용 |
| cross-OS 멱등 | 제어면 canonical 레코드 + per-OS 데이터면 fidelity | 구축 예정 |

## 디버깅

앱 없이 엔진 구동:

    cargo run --release --features harness --bin harness -- <dist-dir> offscreen

`make sidecar-chromium`(또는 `stage.sh <dist-dir>`)가 CEF 프레임워크·helper 번들을 스테이징한다.
PASS(exit 0)=cefQuery 왕복 + offscreen 이면 present 경로·입력까지 성공. 프레임은
`stats.dbg.framesPresented`, 각 프레젠터는 첫 present 를 1회 로그. 흔한 실패: macOS 는 blank=
`Helper (Renderer).app` 변형 누락, Linux 는 프레임 0=child 창이 부모 아래 미맵 또는 CPU 폴백 미가동.

## 로드맵

- **A/B — 완료**: presenter 인터페이스+오라클 분리, 크레이트 게이트 해제, macOS·Linux 클린 컴파일.
- **C/D — Linux 완료, Windows CI**: `presenter/linux.rs` 가속·CPU 폴백 두 경로 구현·클린 컴파일,
  `presenter/windows.rs`(HWND child, 같은 wgpu 파이프라인)는 CI 에서 작성·컴파일. 온스크린은 CI xvfb.
- **E**: Linux/Windows CEF 적재(`libcef.{so,dll}`).
- **F**: 코어가 프레젠터에 per-OS 부모 핸들(X11 XID / HWND)을 macOS NSView 경로 옆에 넘김;
  5타깃 릴리스 매트릭스 CI.
- **멱등**: 제어면 canonical 투영 cross-OS 대조 + per-OS 데이터면 fidelity.
