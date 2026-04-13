use crate::logger::{LogLevel, Logger};

pub mod swapchain;

pub mod pipeline;
pub mod buffer;
pub mod descriptor;

mod instance;
use instance::make_instance;

mod device;
use device::EngineDevice;
use device::phys_device::EnginePhysicalDevice;

use anyhow::Result;

use ash::khr;

use std::sync::Arc;

use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

const VKINSTANCEAPPNAME: &str = "Space Game";

pub struct VulkanContext {
    pub surface_loader: khr::surface::Instance,
    pub swapchain_loader: khr::swapchain::Device,
    pub device: EngineDevice,
    pub phys_device: EnginePhysicalDevice,
    pub instance: ash::Instance,

    pub entry: ash::Entry,
    logger: &'static Logger,
}

impl VulkanContext {
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let logger = Logger::get_logger();

        let entry = unsafe { ash::Entry::load()? };

        let raw_display_handle = window.as_ref().display_handle()?.as_raw();
        let raw_window_handle = window.as_ref().window_handle()?.as_raw();

        logger.log("Creating Vulkan Instance", LogLevel::Info);
        let instance = make_instance(&entry, &raw_display_handle, VKINSTANCEAPPNAME)?;
        logger.log("Succesfuly created Vulkan Instance", LogLevel::Success);

        logger.log("Creating Vulkan Device", LogLevel::Info);
        let surface_loader = khr::surface::Instance::new(&entry, &instance);

        let phys_device = EnginePhysicalDevice::new(&instance)?;

        let device = EngineDevice::new(&entry, &instance, raw_display_handle, raw_window_handle, &surface_loader, &phys_device)?;
        logger.log("Succesfuly created Vulkan Device", LogLevel::Success);

        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device.handle);



        Ok(Self { logger, entry, instance, surface_loader, device, phys_device, swapchain_loader })
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {

        unsafe { self.device.handle.destroy_device(None) };

        unsafe { self.instance.destroy_instance(None) };
    }
}