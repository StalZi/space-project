use crate::logger::{LogLevel, Logger};

pub mod phys_device;
use phys_device::EnginePhysicalDevice;
use phys_device::queue_family::*;

use std::collections::HashSet;
use std::ffi::CStr;

use anyhow::Result;

use ash::khr::surface;
use ash::{Device, Instance};
use ash::{Entry, vk};

use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub struct EngineDevice {
    pub handle: Device,
    pub qfs: PolishedQueueFamilies,
    pub queues: Vec<vk::Queue>,
}

impl EngineDevice {
    pub fn new(
        entry: &Entry,
        instance: &Instance,
        raw_dh: RawDisplayHandle,
        raw_wh: RawWindowHandle,
        surface_loader: &surface::Instance,
        phys_device: &EnginePhysicalDevice,
    ) -> Result<EngineDevice> {
        let logger = Logger::get_logger();

        logger.log("Picking Queue Families", LogLevel::Info);
        let qfs = PolishedQueueFamilies::new(
            entry,
            instance,
            raw_dh,
            raw_wh,
            surface_loader,
            phys_device,
        )?;

        let queue_family_indicies = HashSet::from([
            qfs.graphics.index,
            qfs.present.index,
            qfs.compute.index,
            qfs.transfer.index,
        ]);

        // let features = phys_device.features;

        let queue_create_infos = queue_family_indicies
            .iter()
            .map(|&index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(index)
                    .queue_priorities(&[1.0])
            })
            .collect::<Vec<_>>();

        let extensions = [ash::khr::swapchain::NAME.as_ptr()];
        logger.log_list(
            "Device extensions:",
            extensions
                .iter()
                .map(|extension| unsafe { CStr::from_ptr(*extension).to_string_lossy() }),
            LogLevel::Verbose,
        );

        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);
        let mut vulkan_12_features =
            vk::PhysicalDeviceVulkan12Features::default().buffer_device_address(true);
        let device_create_info = vk::DeviceCreateInfo::default()
            // .enabled_features(&features)
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions)
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features);

        let handle =
            unsafe { instance.create_device(phys_device.handle, &device_create_info, None)? };

        let queues = queue_family_indicies
            .iter()
            .map(|index| unsafe {
                handle
                    .get_device_queue2(&vk::DeviceQueueInfo2::default().queue_family_index(*index))
            })
            .collect::<Vec<_>>();

        Ok(Self {
            handle,
            qfs,
            queues,
        })
    }
}