use anyhow::Result;
use ash::vk::{BufferDeviceAddressInfo, CommandPool};
use ash::{Device, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};

use crate::resources::mesh::{MeshBuffers, Vertex};

#[derive(Default, Debug)]
pub struct AllocatedBuffer {
    pub buffer: vk::Buffer,
    pub size: vk::DeviceSize,
    pub allocation: Allocation,
}

pub fn create_buffer(
    device: &Device,
    allocator: &mut Allocator,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    memory_location: MemoryLocation,
    name: &str,
) -> Result<AllocatedBuffer> {
    let buffer_create_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None)? };
    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let allocation = allocator.allocate(
        &(AllocationCreateDesc {
            name,
            requirements,
            location: memory_location,
            linear: true,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        }),
    )?;

    unsafe {
        device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset())?;
    }

    Ok(AllocatedBuffer {
        buffer,
        size,
        allocation,
    })
}

pub fn required_buffer_size<T>(capacity: usize) -> vk::DeviceSize {
    (capacity as vk::DeviceSize) * (std::mem::size_of::<T>() as vk::DeviceSize)
}

pub fn ensure_buffer_capacity<T>(
    device: &Device,
    allocator: &mut Allocator,
    allocated_buffer: &mut AllocatedBuffer,
    required_capacity: usize,
    usage: vk::BufferUsageFlags,
    memory_location: MemoryLocation,
    name: &str,
) -> Result<()> {
    let required_size = required_buffer_size::<T>(required_capacity);
    if allocated_buffer.size == required_size {
        return Ok(());
    }

    let new_allocated_buffer = create_buffer(
        device,
        allocator,
        required_size,
        usage,
        memory_location,
        name,
    )?;
    unsafe {
        device.device_wait_idle()?;
        device.destroy_buffer(allocated_buffer.buffer, None);
    }

    *allocated_buffer = new_allocated_buffer;

    Ok(())
}

pub fn create_mesh_buffers(
    device: &Device,
    command_pool: &CommandPool,
    transfer_queue: &vk::Queue,
    allocator: &mut Allocator,
    name: &str,
    vertices: &[Vertex],
    indices: &[u32],
) -> Result<MeshBuffers> {
    let vertex_buffer_size = required_buffer_size::<Vertex>(vertices.len());
    let index_buffer_size = required_buffer_size::<u32>(indices.len());

    let vertex_buffer = create_buffer(
        device,
        allocator,
        vertex_buffer_size,
        vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryLocation::GpuOnly,
        format!("{} Mesh Vertex Buffer", name).as_str(),
    )?;

    let vertex_buffer_address = unsafe {
        device.get_buffer_device_address(
            &(BufferDeviceAddressInfo {
                buffer: vertex_buffer.buffer,
                ..Default::default()
            }),
        )
    };

    let index_buffer = create_buffer(
        device,
        allocator,
        index_buffer_size,
        vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::INDEX_BUFFER,
        MemoryLocation::GpuOnly,
        format!("{} Mesh Vertex Buffer", name).as_str(),
    )?;

    let staging_buffer = create_buffer(
        device,
        allocator,
        vertex_buffer_size + index_buffer_size,
        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
        MemoryLocation::CpuToGpu,
        format!("{} Mesh Staging Buffer", name).as_str(),
    )?;

    unsafe {
        let data_ptr = staging_buffer.allocation.mapped_ptr().unwrap().as_ptr() as *mut u8;

        // Copy vertex data
        let vertices_ptr = vertices.as_ptr() as *const u8;
        std::ptr::copy_nonoverlapping(vertices_ptr, data_ptr, vertex_buffer_size as usize);

        // Copy index data
        let indices_ptr = indices.as_ptr() as *const u8;
        std::ptr::copy_nonoverlapping(
            indices_ptr,
            data_ptr.add(vertex_buffer_size as usize),
            index_buffer_size as usize,
        );
    }

    copy_buffer(
        device,
        command_pool,
        transfer_queue,
        staging_buffer.buffer,
        vertex_buffer.buffer,
        vertex_buffer_size,
        0,
    )?;

    copy_buffer(
        device,
        command_pool,
        transfer_queue,
        staging_buffer.buffer,
        index_buffer.buffer,
        index_buffer_size,
        vertex_buffer_size, // Offset in staging buffer
    )?;

    unsafe {
        device.destroy_buffer(staging_buffer.buffer, None);
    }
    allocator.free(staging_buffer.allocation)?;

    Ok(MeshBuffers {
        index_buffer,
        vertex_buffer,
        vertex_buffer_address,
    })
}

pub fn copy_buffer(
    device: &Device,
    command_pool: &CommandPool,
    transfer_queue: &vk::Queue,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    dst_size: u64,
    offset: u64,
) -> Result<()> {
    unsafe {
        // Allocate a command buffer
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(*command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = device.allocate_command_buffers(&allocate_info)?[0];

        // Begin recording
        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        device.begin_command_buffer(command_buffer, &begin_info)?;

        // Copy the buffer
        let copy_region = vk::BufferCopy::default()
            .src_offset(offset)
            .dst_offset(0)
            .size(dst_size);
        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]);

        // End recording
        device.end_command_buffer(command_buffer)?;

        // Submit the command buffer
        device.queue_submit(
            *transfer_queue,
            &[vk::SubmitInfo::default().command_buffers(&[command_buffer])],
            vk::Fence::null(),
        )?;
        device.queue_wait_idle(*transfer_queue)?;

        // Cleanup command buffer
        device.free_command_buffers(*command_pool, &[command_buffer]);

        Ok(())
    }
}
