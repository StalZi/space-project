use winit::raw_window_handle::RawDisplayHandle;

pub fn get_window_manager<'a>(display_handle: RawDisplayHandle) -> &'a str {
    match display_handle {
        RawDisplayHandle::Windows(_) => "Windows",
        RawDisplayHandle::Wayland(_) => "Wayland",
        RawDisplayHandle::Xlib(_) => "Xlib",
        RawDisplayHandle::Xcb(_) => "Xcb",
        RawDisplayHandle::Android(_) => "Android",
        RawDisplayHandle::AppKit(_) | RawDisplayHandle::UiKit(_) => "Apple Kit",
        _ => "N/A",
    }
}
