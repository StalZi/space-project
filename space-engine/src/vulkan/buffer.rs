use ash::{Device, vk};
use anyhow::Result;
use gpu_allocator::{MemoryLocation, vulkan::{Allocator, Allocation, AllocationCreateDesc, AllocationScheme}};

pub fn create_buffer(
    device: &Device,
    allocator: &mut Allocator,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    memory_location: MemoryLocation,
    name: &str,
) -> Result<(vk::Buffer, Allocation)> {
    let buffer_create_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None)? };
    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let allocation = allocator.allocate(&AllocationCreateDesc {
        name,
        requirements,
        location: memory_location,
        linear: true,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged,
    })?;

    unsafe {
        device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;
    }

    Ok((buffer, allocation))
}

pub fn required_buffer_size<T>(capacity: usize) -> vk::DeviceSize {
    capacity as vk::DeviceSize * std::mem::size_of::<T>() as vk::DeviceSize
}

pub fn ensure_buffer_capacity<T>(
    device: &Device,
    allocator: &mut Allocator,
    buffer: &mut vk::Buffer,
    allocation: &mut Allocation,
    buffer_size: &mut vk::DeviceSize,
    required_capacity: usize,
    usage: vk::BufferUsageFlags,
    memory_location: MemoryLocation,
    name: &str,
) -> Result<()> {
    let required_size = required_buffer_size::<T>(required_capacity);
    if *buffer_size == required_size {
        return Ok(());
    }

    let (new_buffer, new_allocation) = create_buffer(
        device,
        allocator,
        required_size,
        usage,
        memory_location,
        name,
    )?;

    unsafe {
        device.destroy_buffer(*buffer, None);
    }

    *buffer = new_buffer;
    *allocation = new_allocation;
    *buffer_size = required_size;

    Ok(())
}
