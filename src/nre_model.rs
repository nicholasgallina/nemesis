use crate::nre_device::NreDevice;
use ash::vk;
use tobj;

pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::size_of::<[f32; 3]>() as u32,
            },
        ]
    }
}

pub struct NreModel {
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    vertex_count: u32,
}

impl NreModel {
    pub fn new(device: &NreDevice, vertices: &[Vertex]) -> Self {
        let (vertex_buffer, vertex_buffer_memory) = Self::create_vertex_buffer(device, vertices);
        Self {
            vertex_buffer,
            vertex_buffer_memory,
            vertex_count: vertices.len() as u32,
        }
    }

    fn create_vertex_buffer(
        device: &NreDevice,
        vertices: &[Vertex],
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let size = (std::mem::size_of::<Vertex>() * vertices.len()) as u64;
        let buffer_info = vk::BufferCreateInfo {
            size,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer = unsafe { device.device().create_buffer(&buffer_info, None).unwrap() };
        let mem_requirements = unsafe { device.device().get_buffer_memory_requirements(buffer) };
        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: mem_requirements.size,
            memory_type_index: Self::find_memory_type(device, mem_requirements.memory_type_bits),
            ..Default::default()
        };
        let memory = unsafe { device.device().allocate_memory(&alloc_info, None).unwrap() };

        unsafe {
            device
                .device()
                .bind_buffer_memory(buffer, memory, 0)
                .unwrap()
        };

        let data_ptr = unsafe {
            device
                .device()
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
                .unwrap()
        };

        unsafe {
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr(),
                data_ptr as *mut Vertex,
                vertices.len(),
            );
            device.device().unmap_memory(memory);
        }

        (buffer, memory)
    }

    pub fn find_memory_type(device: &NreDevice, type_filter: u32) -> u32 {
        let props = unsafe {
            device
                .instance()
                .get_physical_device_memory_properties(device.physical_device())
        };
        let required =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        for i in 0..props.memory_type_count {
            let type_match = (type_filter & (1 << i)) != 0;
            let prop_match = props.memory_types[i as usize]
                .property_flags
                .contains(required);
            if type_match && prop_match {
                return i;
            }
        }
        panic!("no suitable memory type found");
    }

    pub fn vertex_buffer(&self) -> vk::Buffer {
        self.vertex_buffer
    }

    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    pub fn from_obj(device: &NreDevice, path: &str) -> Self {
        let (models, _) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
        )
        .unwrap();

        let mesh = &models[0].mesh;
        let mut vertices = vec![];

        for i in 0..mesh.indices.len() {
            let idx = mesh.indices[i] as usize;
            let pos = [
                mesh.positions[idx * 3],
                mesh.positions[idx * 3 + 1],
                mesh.positions[idx * 3 + 2],
            ];
            let normal = if mesh.normals.is_empty() {
                [0.0, 1.0, 0.0]
            } else {
                let nidx = mesh.normal_indices[i] as usize;
                [
                    mesh.normals[nidx * 3],
                    mesh.normals[nidx * 3 + 1],
                    mesh.normals[nidx * 3 + 2],
                ]
            };
            vertices.push(Vertex {
                position: pos,
                normal,
            });
        }

        Self::new(device, &vertices)
    }
}
