use ash::vk::DeviceAddress;

use crate::utils::{ColorRGBA, Point3D, PointUV};
use crate::vulkan::buffer::AllocatedBuffer;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Point3D, // 12 bytes, offset 0
    _pad1: f32,            // 4 bytes
    pub normal: Point3D,   // 12 bytes, offset 16
    _pad2: f32,            // 4 bytes
    pub color: ColorRGBA,  // 16 bytes, offset 32
    pub uv: PointUV,       // 8 bytes, offset 48
    _pad3: [f32; 2],       // 8 bytes, offset 56
                           // Total: 64 bytes
}

#[derive(Default, Debug)]
pub struct MeshBuffers {
    pub index_buffer: AllocatedBuffer,
    pub vertex_buffer: AllocatedBuffer,
    pub vertex_buffer_address: DeviceAddress,
}

#[derive(Default, Debug)]
pub struct GeoSurface {
    pub start_index: u32,
    pub count: u32,
}

#[derive(Default, Debug)]
pub struct Mesh {
    pub name: String,
    pub buffers: MeshBuffers,
    pub surfaces: Vec<GeoSurface>,
}
