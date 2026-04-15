use crate::nre_device::NreDevice;
use ash::vk;
use tobj;

// !struct
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

// IMPL
impl Vertex {
    pub fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    // FUNC: _ -> Vertex Input Attribute Description
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

// molecular data structures

// !struct
pub struct Atom {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
    pub element: String,
}

// !struct
pub struct AtomInstance {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
}

// IMPL Atom Instance
impl AtomInstance {
    pub fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 1,
            stride: std::mem::size_of::<AtomInstance>() as u32,
            input_rate: vk::VertexInputRate::INSTANCE,
        }]
    }

    pub fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 2,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 3,
                format: vk::Format::R32_SFLOAT,
                offset: 12,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 4,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 16,
            },
        ]
    }
}

// !struct
pub struct Bond {
    pub atom_a: usize,
    pub atom_b: usize,
}

// !struct
pub struct MoleculeData {
    pub atoms: Vec<Atom>,
    pub bonds: Vec<Bond>,
    pub center_of_mass: [f32; 3],
}

fn element_properties(element: &str) -> (f32, [f32; 3]) {
    match element {
        "H" => (1.20, [1.00, 1.00, 1.00]), // white
        "C" => (1.70, [0.50, 0.50, 0.50]), // grey
        "N" => (1.55, [0.13, 0.47, 0.71]), // blue
        "O" => (1.52, [0.84, 0.18, 0.18]), // red
        "S" => (1.80, [1.00, 0.78, 0.20]), // yellow
        "P" => (1.80, [1.00, 0.50, 0.00]), // orange
        _ => (1.50, [0.70, 0.70, 0.70]),   // fallback: grey
    }
}

// !struct
pub struct NreModel {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub vertex_count: u32,
    pub instance_buffer: Option<vk::Buffer>,
    pub instance_buffer_memory: Option<vk::DeviceMemory>,
    pub instance_count: u32,
    device: ash::Device,
}
// !impl
impl NreModel {
    pub fn new(device: &NreDevice, vertices: &[Vertex]) -> Self {
        let (vertex_buffer, vertex_buffer_memory) = Self::create_vertex_buffer(device, vertices);
        Self {
            vertex_buffer,
            vertex_buffer_memory,
            vertex_count: vertices.len() as u32,
            device: device.device().clone(),
            instance_buffer: None,
            instance_buffer_memory: None,
            instance_count: 0,
        }
    }

    // !fn
    pub fn instance_buffer(&self) -> Option<vk::Buffer> {
        self.instance_buffer
    }

    // !fn
    pub fn instance_count(&self) -> u32 {
        self.instance_count
    }

    // !fn
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

    // FUNC: device + path -> NreModel
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

    // FUNC: path -> MoleculeData
    pub fn from_pdb(path: &str) -> MoleculeData {
        let contents = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("ERROR: can't read PDB file: {}", path));

        let mut atoms: Vec<Atom> = Vec::new();
        let mut bonds: Vec<Bond> = Vec::new();

        for line in contents.lines() {
            if line.starts_with("ATOM  ") || line.starts_with("HETATM") {
                let x: f32 = line[30..38].trim().parse().unwrap_or(0.0);
                let y: f32 = line[38..46].trim().parse().unwrap_or(0.0);
                let z: f32 = line[46..54].trim().parse().unwrap_or(0.0);

                let element = if line.len() >= 78 {
                    line[76..78].trim().to_uppercase()
                } else {
                    line[12..14]
                        .trim()
                        .chars()
                        .filter(|c| c.is_alphabetic())
                        .collect::<String>()
                        .to_uppercase()
                };

                let (radius, color) = element_properties(&element);

                atoms.push(Atom {
                    position: [x, y, z],
                    radius,
                    color,
                    element,
                });
            }

            if line.starts_with("CONECT") {
                let parts: Vec<usize> = line[6..]
                    .split_whitespace()
                    .filter_map(|s| s.parse::<usize>().ok())
                    .collect();

                if parts.len() >= 2 {
                    let origin = parts[0] - 1;
                    for &partner in &parts[1..] {
                        let partner_idx = partner - 1;

                        if origin < partner_idx {
                            bonds.push(Bond {
                                atom_a: origin,
                                atom_b: partner_idx,
                            });
                        }
                    }
                }
            }
        }
        let center_of_mass = if atoms.is_empty() {
            [0.0, 0.0, 0.0]
        } else {
            let sum = atoms.iter().fold([0.0f32; 3], |acc, a| {
                [
                    acc[0] + a.position[0],
                    acc[1] + a.position[1],
                    acc[2] + a.position[2],
                ]
            });
            let n = atoms.len() as f32;
            [sum[0] / n, sum[1] / n, sum[2] / n]
        };

        MoleculeData {
            atoms,
            bonds,
            center_of_mass,
        }
    }

    // !fn
    pub fn from_molecule(device: &NreDevice, molecule: &MoleculeData) -> Self {
        let instances: Vec<AtomInstance> = molecule
            .atoms
            .iter()
            .map(|a| AtomInstance {
                position: a.position,
                radius: a.radius,
                color: a.color,
            })
            .collect();

        let size: u64 = (std::mem::size_of::<AtomInstance>() * instances.len()) as u64;

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
        } as *mut AtomInstance;

        unsafe {
            std::ptr::copy_nonoverlapping(
                instances.as_ptr(),
                data_ptr as *mut AtomInstance,
                instances.len(),
            );
            device.device().unmap_memory(memory);
        }

        Self {
            vertex_buffer: vk::Buffer::null(),
            vertex_buffer_memory: vk::DeviceMemory::null(),
            vertex_count: 0,
            instance_buffer: Some(buffer),
            instance_buffer_memory: Some(memory),
            instance_count: instances.len() as u32,
            device: device.device().clone(),
        }
    }
}

// override! DROP
impl Drop for NreModel {
    fn drop(&mut self) {
        unsafe {
            if self.vertex_count > 0 {
                self.device.destroy_buffer(self.vertex_buffer, None);
                self.device.free_memory(self.vertex_buffer_memory, None);
            }

            if let Some(buf) = self.instance_buffer {
                self.device.destroy_buffer(buf, None);
            }

            if let Some(mem) = self.instance_buffer_memory {
                self.device.free_memory(mem, None);
            }
        }
    }
}
