// offscreen 프레젠터의 플랫폼 무관 인터페이스. engine 은 이 표면만 부른다(offscreen 직접 참조 금지).
// 표면 계약(13): is_offscreen · logical_size · scale_of · create_surface · set_bounds · set_hidden ·
// destroy · popup_show · popup_rect · present · present_popup · log_once · FRAMES_PRESENTED.
//
// macOS  : 검증된 offscreen(reference/oracle)을 그대로 재노출 — 동작 0 변경.
// windows: 같은 표면을 D3D11 풀 → DirectComposition 으로 구현(구조는 offscreen.rs 미러).
// linux  : 같은 표면을 EGL/DMA-BUF → X11 로 구현(구조는 offscreen.rs 미러).
// 서피스 레지스트리·3버퍼 풀·팝업 서브레이어·present 스왑 규칙은 세 OS 동일, GPU/윈도우 API 만 교체한다.
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub(crate) use linux::*;
