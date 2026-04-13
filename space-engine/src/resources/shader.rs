use ash::Device;
use ash::vk;

use anyhow::Result;

fn create_shader_module(device: &Device, code: &[u8]) -> Result<vk::ShaderModule> {
    let mut code = std::io::Cursor::new(code);
    let code = ash::util::read_spv(&mut code)?;
    let create_info = vk::ShaderModuleCreateInfo::default().code(&code);

    let shader_module = unsafe { device.create_shader_module(&create_info, None)? };

    Ok(shader_module)
}

pub fn load_shader_module(device: &Device, dir: &str, path: &str) -> Result<vk::ShaderModule> {
    let code = std::fs::read(format!("{}/{}", dir, path))?;
    create_shader_module(device, &code)
}