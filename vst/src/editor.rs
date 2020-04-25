use vst::editor::{KeyCode, KnobMode};
use flutter_winit::FlutterWindow;
use glutin::window::WindowBuilder;
use std::path::{Path, PathBuf};

pub struct Editor {
}

impl Editor {
    pub fn new() -> Self {
        Editor{}
    }
}

impl vst::editor::Editor for Editor {
    /// Get the size of the editor window.
    fn size(&self) -> (i32, i32) {
        (0, 0)
    }

    /// Get the coordinates of the editor window.
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    /// Editor idle call. Called by host.
    fn idle(&mut self) {}

    /// Called when the editor window is closed.
    fn close(&mut self) {}

    /// Called when the editor window is opened.
    ///
    /// `parent` is a window pointer that the new window should attach itself to.
    /// **It is dependent upon the platform you are targeting.**
    ///
    /// A few examples:
    ///
    ///  - On Windows, it should be interpreted as a `HWND`
    ///  - On Mac OS X (64 bit), it should be interpreted as a `NSView*`
    ///  - On X11 platforms, it should be interpreted as a `u32` (the ID number of the parent window)
    ///
    /// Return `true` if the window opened successfully, `false` otherwise.
    fn open(&mut self, parent: *mut std::ffi::c_void) -> bool {
        info!("open editor");

        // Open flutter window
        let assets_dir = std::env::var("FLUTTER_ASSET_DIR").expect("FLUTTER_ASSET_DIR");

        let mut args = Vec::with_capacity(3);

        if let Ok(observatory_port) = std::env::var("DART_OBSERVATORY_PORT") {
            args.push("--disable-service-auth-codes".to_string());
            args.push(format!("--observatory-port={}", observatory_port));
        }

        if let Ok(snapshot) = std::env::var("FLUTTER_AOT_SNAPSHOT") {
            if Path::new(&snapshot).exists() {
                args.push(format!("--aot-shared-library-name={}", snapshot));
            }
        }

        let window = WindowBuilder::new().with_title("Flutter App Demo");
        let flutter = FlutterWindow::new(window, PathBuf::from(assets_dir), args).unwrap();
        let flutter = flutter.with_resource_context().unwrap();

        flutter.start_engine().unwrap();

        flutter.run();

        info!("flutter is running");

        true
    }

    /// Return whether the window is currently open.
    fn is_open(&mut self) -> bool {
        info!("is_open");
        true
    }

    /// Set the knob mode for this editor (if supported by host).
    ///
    /// Return `true` if the knob mode was set.
    fn set_knob_mode(&mut self, mode: KnobMode) -> bool {
        false
    }

    /// Receive key up event. Return `true` if the key was used.
    fn key_up(&mut self, keycode: KeyCode) -> bool {
        false
    }

    /// Receive key down event. Return `true` if the key was used.
    fn key_down(&mut self, keycode: KeyCode) -> bool {
        false
    }
}