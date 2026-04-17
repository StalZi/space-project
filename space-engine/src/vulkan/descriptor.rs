use anyhow::Result;
use ash::{Device, vk};

pub fn create_descriptor_set_layout(
    device: &Device,
    bindings: &[vk::DescriptorSetLayoutBinding],
) -> Result<vk::DescriptorSetLayout> {
    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(bindings);
    let layout = unsafe { device.create_descriptor_set_layout(&create_info, None)? };
    Ok(layout)
}

pub fn create_descriptor_pool(
    device: &Device,
    pool_sizes: &[vk::DescriptorPoolSize],
    max_sets: u32,
) -> Result<vk::DescriptorPool> {
    let create_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(pool_sizes)
        .max_sets(max_sets);
    let pool = unsafe { device.create_descriptor_pool(&create_info, None)? };
    Ok(pool)
}

pub fn allocate_descriptor_set(
    device: &Device,
    descriptor_pool: vk::DescriptorPool,
    layouts: &[vk::DescriptorSetLayout],
) -> Result<vk::DescriptorSet> {
    let allocate_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(layouts);
    let sets = unsafe { device.allocate_descriptor_sets(&allocate_info)? };
    Ok(sets[0])
}

pub fn write_storage_buffer_descriptor(
    device: &Device,
    descriptor_set: vk::DescriptorSet,
    binding: u32,
    buffer: vk::Buffer,
    offset: vk::DeviceSize,
    range: vk::DeviceSize,
) {
    let buffer_info = vk::DescriptorBufferInfo::default()
        .buffer(buffer)
        .offset(offset)
        .range(range);
    let buffer_infos = [buffer_info];

    let write_descriptor_set = vk::WriteDescriptorSet::default()
        .dst_set(descriptor_set)
        .dst_binding(binding)
        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        .buffer_info(&buffer_infos);

    unsafe {
        device.update_descriptor_sets(&[write_descriptor_set], &[]);
    }
}
