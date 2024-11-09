extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;
use nwg::Event;
use std::{cell::RefCell, ops::Deref, rc::Rc};
use zenoh::{Config, Wait};

use crate::driver::{self, Driver};
#[derive(Default)]
pub struct App {
    driver: RefCell<Option<Driver>>,
    window: nwg::Window,
    add_slider_btn: nwg::Button,
    layout: nwg::GridLayout,
    sliders: RefCell<Vec<nwg::TrackBar>>,
    handlers: RefCell<Vec<nwg::EventHandler>>,
}

pub struct AppUi {
    inner: Rc<App>,
    default_handler: RefCell<Vec<nwg::EventHandler>>,
}

impl App {
    fn init_driver(&self) {
        println!("Opening Zenoh Session...");
        let session = zenoh::open(Config::default()).wait().unwrap();
        println!("OK");

        let key_expr = "rt/joint_states".to_string();

        *self.driver.borrow_mut() = Some(driver::Driver::new(session, key_expr));
    }

    fn exit(&self) {
        let handlers = self.handlers.borrow();
        for handler in handlers.iter() {
            nwg::unbind_event_handler(&handler);
        }

        nwg::stop_thread_dispatch();
    }
    fn add_slider(app_rc: &Rc<App>) {
        let app = app_rc.clone();

        let mut sliders = app.sliders.borrow_mut();
        let mut handlers = app.handlers.borrow_mut();

        let mut new_slider = nwg::TrackBar::default();
        nwg::TrackBar::builder()
            .parent(&app.window)
            .range(Some(0..180))
            .build(&mut new_slider)
            .expect("Failed to build slider");

        let id = sliders.len();
        let evt_ui = Rc::downgrade(&app);

        app.layout.add_child(0, id as u32 + 1, &new_slider);

        let new_slider_handle = new_slider.handle;

        let handler = nwg::bind_event_handler(
            &new_slider_handle,
            &app_rc.window.handle,
            move |evt, _evt_data, handle| {
                if let Some(ui) = evt_ui.upgrade() {
                    match evt {
                        nwg::Event::OnHorizontalScroll => {
                            if handle == new_slider_handle {
                                if let Some(driver) = ui.driver.borrow().as_ref() {
                                    let angle = ui.sliders.borrow()[id].pos();
                                    println!("send code id:{},angle:{}", id, angle);
                                    driver.send_code(id as u8, angle as u8);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            },
        );

        sliders.push(new_slider);
        handlers.push(handler);
    }
}

impl nwg::NativeUi<AppUi> for App {
    fn build_ui(mut inital_state: App) -> Result<AppUi, nwg::NwgError> {
        nwg::Window::builder()
            .flags(
                nwg::WindowFlags::WINDOW
                    | nwg::WindowFlags::VISIBLE
                    | nwg::WindowFlags::RESIZABLE
                    | nwg::WindowFlags::MAXIMIZED,
            )
            .size((500, 300))
            .position((300, 300))
            .title("urdf controller")
            .build(&mut inital_state.window)?;

        nwg::Button::builder()
            .text("Add Slider")
            .parent(&inital_state.window)
            .build(&mut inital_state.add_slider_btn)?;

        nwg::GridLayout::builder()
            .parent(&inital_state.window)
            .child(0, 0, &inital_state.add_slider_btn)
            .build(&inital_state.layout)?;

        let ui = AppUi {
            inner: Rc::new(inital_state),
            default_handler: Default::default(),
        };

        let window_handles = [&ui.window.handle];

        for handle in window_handles.iter() {
            let evt_ui = Rc::downgrade(&ui.inner);
            let handle_events = move |evt, _evt_data, handle| {
                if let Some(evt_ui) = evt_ui.upgrade() {
                    match evt {
                        Event::OnInit => {
                            App::init_driver(&evt_ui);
                        }
                        Event::OnButtonClick => {
                            if &handle == &evt_ui.add_slider_btn {
                                App::add_slider(&evt_ui);
                            }
                        }
                        Event::OnWindowClose => {
                            if &handle == &evt_ui.window {
                                App::exit(&evt_ui);
                            }
                        }
                        _ => {}
                    }
                }
            };

            ui.default_handler
                .borrow_mut()
                .push(nwg::full_bind_event_handler(handle, handle_events));
        }

        return Ok(ui);
    }
}

impl<'a> Drop for AppUi {
    /// To make sure that everything is freed without issues, the default handler must be unbound.
    fn drop(&mut self) {
        let mut handlers = self.default_handler.borrow_mut();
        for handler in handlers.drain(0..) {
            nwg::unbind_event_handler(&handler);
        }
    }
}

impl<'a> Deref for AppUi {
    type Target = App;

    fn deref(&self) -> &App {
        &self.inner
    }
}
