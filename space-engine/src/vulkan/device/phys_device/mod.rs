use crate::logger::{LogLevel, Logger};

pub mod queue_family;
use anyhow::Result;
use ash::{Instance, vk};
use queue_family::EngineQueueFamily;

pub struct EnginePhysicalDevice {
    pub handle: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub queue_families: Vec<EngineQueueFamily>,
}

impl EnginePhysicalDevice {
    pub fn new(instance: &Instance) -> Result<Self> {
        let logger = Logger::get_logger();

        let (handle, properties) = EnginePhysicalDevice::pick_physical_device(logger, instance)
            .expect("Failed to get physical device");
        let features = unsafe { instance.get_physical_device_features(handle) };
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(handle) };
        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(handle) };
        let queue_families = queue_family_properties
            .iter()
            .enumerate()
            .map(|(index, properties)| EngineQueueFamily {
                index: index as u32,
                properties: *properties,
            })
            .collect();

        logger.log(
            format!(
                "Physical device name: {:?}",
                properties.device_name_as_c_str()?
            ),
            LogLevel::Info,
        );

        logger.log(
            format!("Picked Properties: {:?}", properties),
            LogLevel::Verbose,
        );

        logger.log(
            format!("Queue family properties: {:?}", queue_family_properties),
            LogLevel::Verbose,
        );

        logger.log(
            format!("Memory properties: {:?}", memory_properties),
            LogLevel::Verbose,
        );

        logger.log(
            format!(
                "Physical device memory: {:?}GB",
                memory_properties.memory_heaps[0].size / 1024 / 1024 / 1000
            ),
            LogLevel::Info,
        );

        Ok(Self {
            handle,
            properties,
            memory_properties,
            features,
            queue_families,
        })
    }

    pub fn pick_physical_device(
        logger: &'static Logger,
        instance: &Instance,
    ) -> Result<(vk::PhysicalDevice, vk::PhysicalDeviceProperties)> {
        // Get the list of physical devices available on the system.
        let phys_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };

        logger.log_list(
            "Available devices:",
            phys_devices.iter().map(|phys_dev| unsafe {
                (instance
                    .get_physical_device_properties(*phys_dev)
                    .device_name_as_c_str())
                .unwrap()
                .to_string_lossy()
                .into_owned()
            }),
            LogLevel::Verbose,
        );

        // We will pick the first discrete GPU we find. If there are no discrete GPUs, we will just pick the first one.
        let mut phys_device_index = 0;
        let mut properties: vk::PhysicalDeviceProperties = Default::default();
        for (index, phys_device) in phys_devices.iter().enumerate() {
            properties = unsafe { instance.get_physical_device_properties(*phys_device) };
            if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                phys_device_index = index;
                break;
            };
        }

        Ok((phys_devices[phys_device_index], properties))
    }
}
