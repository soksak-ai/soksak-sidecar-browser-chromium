// offscreen 프레젠터 — Windows 구현. CEF 가 offscreen 렌더한 공유 텍스처(D3D11 shared HANDLE)를
// 모듈 소유 child 표면의 DirectComposition visual 로 present 한다. 구조는 offscreen.rs(macOS reference)를
// 미러한다: 서피스 레지스트리 · 3버퍼 풀 · 팝업 서브레이어 · present 스왑. GPU/윈도우 API 만 D3D11/DComp
// 로 교체하고 상태 계약(논리크기·scale·hidden·popup)은 동일하게 유지한다.
//
// 진행 단계: 레지스트리·상태 부기는 완결(is_offscreen/logical_size/scale_of/set_bounds/set_hidden/
// destroy/popup_*). 네이티브 child 창 생성·D3D11 풀·present blit 은 Phase C/D(CI 검증). 미구현 경로는
// 조용히 성공하지 않고 log_once 로 표식한다(SIDECARS.md P 규칙: 조용한 강등 금지).

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

// present 완료 프레임 총계 — stats.dbg 로 노출(reference 와 동일 표면). Phase C/D present 가 증가시킨다.
pub(crate) static FRAMES_PRESENTED: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

// 엔진 id 하나의 present 상태(플랫폼 무관 부기). hidden/popup 은 Phase C/D present 가 소비한다.
#[allow(dead_code)] // hidden·popup_shown·popup_rect: Phase C/D present 경로에서 소비
struct Surf {
    scale: f32,
    log_w: i32,
    log_h: i32,
    hidden: bool,
    popup_shown: bool,
    popup_rect: (i32, i32, i32, i32),
}

static SURFS: LazyLock<Mutex<HashMap<u32, Surf>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn is_offscreen(id: u32) -> bool {
    SURFS.lock().map(|m| m.contains_key(&id)).unwrap_or(false)
}

pub(crate) fn logical_size(id: u32) -> Option<(i32, i32)> {
    SURFS.lock().ok().and_then(|m| m.get(&id).map(|s| (s.log_w, s.log_h)))
}

pub(crate) fn scale_of(id: u32) -> Option<f32> {
    SURFS.lock().ok().and_then(|m| m.get(&id).map(|s| s.scale))
}

pub(crate) fn create_surface(id: u32, parent: usize, _x: i32, _y: i32, w: i32, h: i32, scale: f32) {
    if parent == 0 {
        return;
    }
    if let Ok(mut m) = SURFS.lock() {
        m.insert(
            id,
            Surf { scale, log_w: w.max(1), log_h: h.max(1), hidden: false, popup_shown: false, popup_rect: (0, 0, 0, 0) },
        );
    }
    log_once(id, "windows offscreen child 표면 생성 미구현 (Phase C/D: D3D11/DirectComposition)");
}

pub(crate) fn set_bounds(id: u32, _x: i32, _y: i32, w: i32, h: i32) {
    if let Ok(mut m) = SURFS.lock() {
        if let Some(s) = m.get_mut(&id) {
            s.log_w = w.max(1);
            s.log_h = h.max(1);
        }
    }
}

pub(crate) fn set_hidden(id: u32, hidden: bool) {
    if let Ok(mut m) = SURFS.lock() {
        if let Some(s) = m.get_mut(&id) {
            s.hidden = hidden;
        }
    }
}

pub(crate) fn destroy(id: u32) {
    if let Ok(mut m) = SURFS.lock() {
        m.remove(&id);
    }
}

pub(crate) fn popup_show(id: u32, show: bool) {
    if let Ok(mut m) = SURFS.lock() {
        if let Some(s) = m.get_mut(&id) {
            s.popup_shown = show;
        }
    }
}

pub(crate) fn popup_rect(id: u32, x: i32, y: i32, w: i32, h: i32) {
    if let Ok(mut m) = SURFS.lock() {
        if let Some(s) = m.get_mut(&id) {
            s.popup_rect = (x, y, w, h);
        }
    }
}

pub(crate) fn present(id: u32, info: &cef::AcceleratedPaintInfo) {
    let _ = info; // Phase C/D: info.shared_texture_handle (D3D11 HANDLE) → OpenSharedResource1 → DComp swap
    log_once(id, "windows present 미구현 (Phase C/D: D3D11 shared HANDLE → DirectComposition swap)");
}

pub(crate) fn present_popup(id: u32, info: &cef::AcceleratedPaintInfo) {
    let _ = info;
    log_once(id, "windows 팝업 present 미구현 (Phase C/D)");
}

// id 별 1회 에러 로그 — 조용한 강등 금지(스펙 P 규칙), 프레임마다 폭주 금지. reference 와 동일.
static LOGGED: LazyLock<Mutex<std::collections::HashSet<(u32, &'static str)>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));
pub(crate) fn log_once(id: u32, msg: &'static str) {
    if LOGGED.lock().map(|mut s| s.insert((id, msg))).unwrap_or(false) {
        eprintln!("[chromium] offscreen(id={id}): {msg}");
    }
}
