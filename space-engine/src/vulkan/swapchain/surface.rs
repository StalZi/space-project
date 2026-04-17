use anyhow::Result;
use ash::khr::surface;
use ash::{Entry, Instance, vk};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

pub struct EngineSurface {
    pub handle: vk::SurfaceKHR,
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl EngineSurface {
    pub fn new(
        entry: &Entry,
        instance: &Instance,
        phys_device: &vk::PhysicalDevice,
        surface_loader: &surface::Instance,
        window: &Window,
    ) -> Result<EngineSurface> {
        let raw_display_handle = window.display_handle().unwrap().as_raw();
        let raw_window_handle = window.window_handle().unwrap().as_raw();

        let surface = unsafe {
            ash_window::create_surface(entry, instance, raw_display_handle, raw_window_handle, None)
                .expect("Failed to create Vulkan Surface")
        };

        let capabilities = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(*phys_device, surface)
                .expect("Failed to get surface capabilities")
        };

        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(*phys_device, surface)
                .expect("Failed to get surface formats")
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(*phys_device, surface)
                .expect("Failed to get surface present modes")
        };

        Ok(Self {
            handle: surface,
            capabilities,
            formats,
            present_modes,
        })
    }
}
