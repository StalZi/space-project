use crate::logger::{LogLevel, Logger};

pub mod shader;

pub mod mesh;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use mesh::Mesh;

use crate::resources::mesh::{GeoSurface, MeshBuffers, Vertex};
use crate::utils::{ColorRGBA, Point3D, PointUV};
use crate::vulkan::VulkanContext;
use crate::vulkan::buffer::create_mesh_buffers;

#[derive(Default, Debug)]
pub struct MeshesInfo {
    path: String,
    pub meshes: HashMap<String, Mesh>,
}

pub struct ResourceManager {
    context: Arc<VulkanContext>,
    command_pool: vk::CommandPool,
    meshes_info: MeshesInfo,
    logger: &'static Logger,
}

impl ResourceManager {
    pub fn new(context: Arc<VulkanContext>) -> Result<Self> {
        let logger = Logger::get_logger();
        let command_pool = (unsafe {
            context.device.handle.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(context.device.qfs.transfer.index)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )
        })?;

        Ok(Self {
            context,
            command_pool,
            meshes_info: MeshesInfo::default(),
            logger,
        })
    }

    pub fn get_or_init_meshes(
        &mut self,
        path: &String,
        allocator: &mut Allocator,
    ) -> Result<&MeshesInfo> {
        if self.meshes_info.path != *path {
            self.meshes_info.meshes = self.load_gltf_meshes(allocator, path)?;
            self.meshes_info.path = path.clone();
        };

        Ok(&self.meshes_info)
    }

    pub fn load_gltf_meshes(
        &self,
        allocator: &mut Allocator,
        file_path: &String,
    ) -> Result<HashMap<String, Mesh>> {
        self.logger
            .log(format!("Loading mesh {}", file_path), LogLevel::Info);
        let (document, buffers, _images) = gltf::import(file_path)?; // Returns document, buffer data, and image data
        let mut meshes = HashMap::new();

        for mesh in document.meshes() {
            let mut mesh_asset = Mesh {
                name: mesh.name().unwrap_or("Unnamed").to_string(),
                surfaces: Vec::new(),
                buffers: MeshBuffers::default(),
            };

            let mut indices = Vec::new();
            let mut vertices: Vec<Vertex> = Vec::new();

            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let initial_vertex_count = vertices.len();

                // --- Load Vertex Positions ---
                if let Some(positions) = reader.read_positions() {
                    let positions: Vec<_> = positions.collect();
                    vertices.resize(initial_vertex_count + positions.len(), Vertex::default());

                    for (i, pos) in positions.into_iter().enumerate() {
                        vertices[initial_vertex_count + i].position = Point3D::from_arr(pos); // [f32; 3]
                    }
                }

                // --- Load Indices ---
                if let Some(indices_accessor) = primitive.indices() {
                    let index_count = indices_accessor.count(); // Get count from accessor
                    let start_index = indices.len() as u32;

                    indices.reserve(index_count);

                    // Now create the reader and process
                    if let Some(index_reader) = reader.read_indices() {
                        match index_reader {
                            gltf::mesh::util::ReadIndices::U8(iter) => {
                                for idx in iter {
                                    indices.push(idx as u32 + initial_vertex_count as u32);
                                }
                            }
                            gltf::mesh::util::ReadIndices::U16(iter) => {
                                for idx in iter {
                                    indices.push(idx as u32 + initial_vertex_count as u32);
                                }
                            }
                            gltf::mesh::util::ReadIndices::U32(iter) => {
                                for idx in iter {
                                    indices.push(idx + initial_vertex_count as u32);
                                }
                            }
                        }
                    }

                    mesh_asset.surfaces.push(GeoSurface {
                        start_index,
                        count: index_count as u32,
                    });
                }

                // --- Load Normals (if present) ---
                if let Some(normals) = reader.read_normals() {
                    for (i, normal) in normals.enumerate() {
                        vertices[initial_vertex_count + i].normal = Point3D::from_arr(normal);
                    }
                }

                // --- Load UVs (if present) ---
                if let Some(uvs) = reader.read_tex_coords(0) {
                    for (i, uv) in uvs.into_f32().enumerate() {
                        vertices[initial_vertex_count + i].uv = PointUV::from_arr(uv);
                    }
                }

                // --- Load Colors (if present) ---
                if let Some(colors) = reader.read_colors(0) {
                    for (i, color) in colors.into_rgba_f32().enumerate() {
                        vertices[initial_vertex_count + i].color = ColorRGBA::from_arr(color);
                    }
                }
            }

            // ... optional normal-as-color override ...

            if !vertices.is_empty() {
                let mut min_pos = [f32::MAX; 3];
                let mut max_pos = [f32::MIN; 3];

                for v in &vertices {
                    min_pos[0] = min_pos[0].min(v.position.x);
                    min_pos[1] = min_pos[1].min(v.position.y);
                    min_pos[2] = min_pos[2].min(v.position.z);

                    max_pos[0] = max_pos[0].max(v.position.x);
                    max_pos[1] = max_pos[1].max(v.position.y);
                    max_pos[2] = max_pos[2].max(v.position.z);
                }

                let size = [
                    max_pos[0] - min_pos[0],
                    max_pos[1] - min_pos[1],
                    max_pos[2] - min_pos[2],
                ];

                println!("=== MESH: {} ===", mesh.name().unwrap_or("Unnamed"));
                println!("  Vertices count: {}", vertices.len());
                println!("  Min position: {:?}", min_pos);
                println!("  Max position: {:?}", max_pos);
                println!("  Size: {:?}", size);
            }
            mesh_asset.buffers = create_mesh_buffers(
                &self.context.device.handle,
                &self.command_pool,
                &self.context.device.queues[self.context.device.qfs.transfer.index as usize],
                allocator,
                &mesh_asset.name,
                &vertices,
                &indices,
            )?;
            meshes.insert(mesh.name().unwrap_or("Unnamed").to_string(), mesh_asset);
        }
        Ok(meshes)
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        for mesh in self.meshes_info.meshes.values() {
            unsafe {
                self.context
                    .device
                    .handle
                    .destroy_buffer(mesh.buffers.index_buffer.buffer, None);
                self.context
                    .device
                    .handle
                    .destroy_buffer(mesh.buffers.vertex_buffer.buffer, None);
            }
        }
        unsafe {
            self.context
                .device
                .handle
                .destroy_command_pool(self.command_pool, None)
        };
    }
}
