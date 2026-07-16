// Chromium 서브프로세스 helper — renderer/GPU/utility 가 이 바이너리로 뜬다(cefsimple 의 분리-helper
// 패턴). execute_process 로 서브프로세스 역할을 수행한다. 브라우저(메인) 프로세스는 앱 본체(dylib 의
// initialize)가 담당하며 이 바이너리를 직접 실행하지 않는다.
//
// 렌더-사이드 메시지 라우터(CefMessageRouter): render_process_handler 로 각 V8 컨텍스트에 window.cefQuery
// 를 심고(on_context_created), 브라우저의 응답을 페이지 onSuccess/onFailure 로 되돌린다(on_process_message_
// received). 페이지↔호스트는 구조적 데이터 메시지(cefQuery)로 오가며 문자열 코드 주입(eval)이 없다.
// 브라우저-사이드(engine.rs)와 config(cefQuery 함수명)가 반드시 일치해야 라운드트립이 성립한다.
//
// framework 적재는 macOS 만: dist/soksak-sidecar-browser-chromium Helper.app/Contents/MacOS/<이 바이너리>
// 에서 LibraryLoader(helper=true)가 ../../../../Chromium Embedded Framework.framework 를 해소한다.
// linux/windows 는 libcef 를 빌드타임 링크하므로 런타임 적재가 불요하다(execute_process 만).

use cef::wrapper::message_router::{
    MessageRouterConfig, MessageRouterRendererSide, MessageRouterRendererSideHandlerCallbacks,
    RendererSideRouter,
};
use cef::*;
use std::sync::Arc;

// 렌더 프로세스 핸들러 — 세 콜백을 렌더-사이드 라우터로 전달한다. CEF 는 borrowed(Option<&mut T>) 로
// 주고 라우터는 owned(Option<T>) 를 원해 .map(|x| x.clone())(RcGuard refcount bump)로 다리를 놓는다.
// 두 비대칭: renderer 의 source_process 는 Option 으로 감싸고, 반환은 bool→c_int.
wrap_render_process_handler! {
    struct HelperRenderProcessHandler {
        router: Arc<RendererSideRouter>,
    }
    impl RenderProcessHandler {
        fn on_context_created(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            context: Option<&mut V8Context>,
        ) {
            self.router.on_context_created(
                browser.map(|b| b.clone()),
                frame.map(|f| f.clone()),
                context.map(|c| c.clone()),
            );
        }
        fn on_context_released(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            context: Option<&mut V8Context>,
        ) {
            self.router.on_context_released(
                browser.map(|b| b.clone()),
                frame.map(|f| f.clone()),
                context.map(|c| c.clone()),
            );
        }
        fn on_process_message_received(
            &self,
            browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            source_process: ProcessId,
            message: Option<&mut ProcessMessage>,
        ) -> ::std::os::raw::c_int {
            self.router.on_process_message_received(
                browser.map(|b| b.clone()),
                frame.map(|f| f.clone()),
                Some(source_process),
                message.map(|m| m.clone()),
            ) as ::std::os::raw::c_int
        }
    }
}

// CefApp — 렌더 프로세스에서 CEF 가 render_process_handler 를 조회한다. GPU/utility 서브프로세스는
// 이 핸들러를 쓰지 않는다(무해). 라우터 Arc 를 각 핸들러에 clone 해 한 인스턴스를 공유한다.
wrap_app! {
    struct HelperApp {
        router: Arc<RendererSideRouter>,
    }
    impl App {
        fn render_process_handler(&self) -> Option<RenderProcessHandler> {
            Some(HelperRenderProcessHandler::new(self.router.clone()))
        }
    }
}

fn main() {
    use cef::args::Args;

    // macOS 만 framework 를 런타임 적재한다(비-macOS 는 libcef 빌드타임 링크라 불요).
    #[cfg(target_os = "macos")]
    {
        use cef::library_loader::LibraryLoader;
        let exe = std::env::current_exe().expect("current_exe");
        let loader = LibraryLoader::new(&exe, /*helper=*/ true);
        if !loader.load() {
            eprintln!("[chromium-helper] framework 로드 실패");
            std::process::exit(1);
        }
    }
    let _ = cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);

    // 브라우저-사이드(engine.rs)와 동일 config — 기본 cefQuery/cefQueryCancel. 불일치 시 라운드트립 무성립.
    let router = <RendererSideRouter as MessageRouterRendererSide>::new(MessageRouterConfig::default());
    let mut app = HelperApp::new(router);

    let args = Args::new();
    let code = cef::execute_process(Some(args.as_main_args()), Some(&mut app), std::ptr::null_mut());
    std::process::exit(code);
}
