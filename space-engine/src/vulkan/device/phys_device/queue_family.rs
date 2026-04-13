use anyhow::Result;
use ash::khr::surface;
use ash::{Entry, Instance};
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::vulkan::device::phys_device::EnginePhysicalDevice;

pub struct PolishedQueueFamilies {
    pub graphics: EngineQueueFamily,
    pub present: EngineQueueFamily,
    pub compute: EngineQueueFamily,
    pub transfer: EngineQueueFamily,
}

impl PolishedQueueFamilies {
    pub fn new(
        entry: &Entry,
        instance: &Instance,
        raw_dh: RawDisplayHandle,
        raw_wh: RawWindowHandle,
        surface_loader: &surface::Instance,
        phys_device: &EnginePhysicalDevice,
    ) -> Result<Self> {
        // Create a dummy Vulkan surface for the checking compatibility of the physical device with the window. We will destroy this surface later and create a new one for each window.
        let dummy_surface = unsafe {
            ash_window::create_surface(entry, instance, raw_dh, raw_wh, None)
                .expect("Failed to create Vulkan Surface")
        };

        // We pick the required queue families for the physical device using the dummy surface to check for present support. We will use these queue families to create the logical device.
        let qfs = Self {
            graphics: pick_graphics_queue_family(&phys_device.queue_families)
                .expect("Failed to find graphics queue family"),
            present: pick_present_queue_family(
                &phys_device.queue_families,
                surface_loader,
                dummy_surface,
                phys_device.handle,
            )
            .expect("Failed to find present queue family"),
            compute: pick_compute_queue_family(&phys_device.queue_families)
                .expect("Failed to find compute queue family"),
            transfer: pick_transfer_queue_family(&phys_device.queue_families)
                .expect("Failed to find transfer queue family"),
        };

        // Dropping the dummy surface.
        unsafe {
            surface_loader.destroy_surface(dummy_surface, None);
        }

        Ok(qfs)
    }
}

#[derive(Clone)]
pub struct EngineQueueFamily {
    pub index: u32,
    pub properties: ash::vk::QueueFamilyProperties,
}

pub fn pick_present_queue_family(
    queue_families: &[EngineQueueFamily],
    surface_loader: &surface::Instance,
    surface: ash::vk::SurfaceKHR,
    phys_device: ash::vk::PhysicalDevice,
) -> Option<EngineQueueFamily> {
    queue_families
        .iter()
        .find(|qf| unsafe {
            surface_loader
                .get_physical_device_surface_support(phys_device, qf.index, surface)
                .unwrap_or(false)
        })
        .cloned()
}

pub fn pick_graphics_queue_family(
    queue_families: &[EngineQueueFamily],
) -> Option<EngineQueueFamily> {
    queue_families
        .iter()
        .find(|qf| {
            qf.properties
                .queue_flags
                .contains(ash::vk::QueueFlags::GRAPHICS)
        })
        .cloned()
}

pub fn pick_compute_queue_family(
    queue_families: &[EngineQueueFamily],
) -> Option<EngineQueueFamily> {
    queue_families
        .iter()
        .find(|qf| {
            qf.properties
                .queue_flags
                .contains(ash::vk::QueueFlags::COMPUTE)
        })
        .cloned()
}

pub fn pick_transfer_queue_family(
    queue_families: &[EngineQueueFamily],
) -> Option<EngineQueueFamily> {
    queue_families
        .iter()
        .find(|qf| {
            qf.properties
                .queue_flags
                .contains(ash::vk::QueueFlags::TRANSFER)
        })
        .cloned()
}
