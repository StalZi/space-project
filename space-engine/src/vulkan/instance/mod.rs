mod get_wm_name;
use std::ffi::CStr;

use ash::vk::API_VERSION_1_0;
use ash::{Entry, vk};
use get_wm_name::get_window_manager;
use winit::raw_window_handle::RawDisplayHandle;

use crate::logger::{LogLevel, Logger};

pub fn make_instance(
    entry: &Entry,
    raw_display_handle: &RawDisplayHandle,
    app_name: &str,
) -> Result<ash::Instance, anyhow::Error> {
    let logger = Logger::get_logger();

    logger.log(
        format!("Vulkan Instance app name: {}", app_name),
        LogLevel::Info,
    );

    let app_name = std::ffi::CString::new(app_name).unwrap();

    let api_ver = unsafe {
        entry
            .try_enumerate_instance_version()
            .expect("Failed to enumerate instance version")
    };
    let api_ver = api_ver.unwrap_or(API_VERSION_1_0);

    logger.log(
        format!(
            "Vulkan API Version: {}.{}.{}",
            vk::api_version_major(api_ver),
            vk::api_version_minor(api_ver),
            vk::api_version_patch(api_ver)
        ),
        LogLevel::Info,
    );

    let app_info = ash::vk::ApplicationInfo::default()
        .application_name(&app_name)
        .application_version(0)
        .engine_name(&app_name)
        .engine_version(0)
        .api_version(api_ver);

    let extensions = ash_window::enumerate_required_extensions(*raw_display_handle).unwrap();

    logger.log(
        format!("Running {}", get_window_manager(*raw_display_handle)),
        LogLevel::Info,
    );

    logger.log_list(
        "Instance Extensions:",
        extensions
            .iter()
            .map(|extension| unsafe { CStr::from_ptr(*extension).to_string_lossy() }),
        LogLevel::Info,
    );

    let create_info = ash::vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_extension_names(extensions);

    let instance = unsafe { entry.create_instance(&create_info, None)? };

    Ok(instance)
}
