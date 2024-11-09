extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use nwg::NativeUi;
mod driver;
mod gui;

#[async_std::main]
async fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let _app = gui::App::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
