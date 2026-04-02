// descriptor set, set layout, pool, sets
use crate::nre_device::NreDevice;
use crate::nre_model::NreModel;
use ash::vk;
use glam;

// STRUCT -> Descriptor Set Layout
pub struct NreDescriptorSetLayout {
    layout: vk::DescriptorSetLayout,
}

// STRUCT -> Descriptor Pool
pub struct NreDescriptorPool {
    pool: vk::DescriptorPool,
}

// STRUCT -> Uniform Buffer
pub struct NreUniformBuffer {
    buffers: Vec<vk::Buffer>,
    memories: Vec<vk::DeviceMemory>,
    mapped: Vec<*mut std::ffi::c_void>,
}

// IMPL -> Descriptor Set Layout
impl NreDescriptorSetLayout {
    //
    pub fn new(device: &NreDevice) -> Self {
        //
        let layout_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        };
        //
        let layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            binding_count: 1,
            p_bindings: &layout_binding,
            ..Default::default()
        };
        //
        let layout = unsafe {
            device
                .device()
                .create_descriptor_set_layout(&layout_create_info, None)
                .unwrap()
        };
        //
        Self { layout }
    }
    //
    pub fn layout(&self) -> vk::DescriptorSetLayout {
        self.layout
    }
}

// IMPL -> Descriptor Pool
impl NreDescriptorPool {
    //
    pub fn new(device: &NreDevice) -> Self {
        let pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 2,
        };
        let pool_create_info = vk::DescriptorPoolCreateInfo {
            pool_size_count: 1,
            p_pool_sizes: &pool_size,
            max_sets: 2,
            ..Default::default()
        };
        //
        let pool = unsafe {
            device
                .device()
                .create_descriptor_pool(&pool_create_info, None)
                .unwrap()
        };
        //
        Self { pool }
    }
    //
    pub fn pool(&self) -> vk::DescriptorPool {
        self.pool
    }
    //
    pub fn allocate_descriptor_sets(
        &self,
        device: &NreDevice,
        layout: vk::DescriptorSetLayout,
    ) -> Vec<vk::DescriptorSet> {
        let layouts = vec![layout, layout];
        let alloc_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: self.pool,
            descriptor_set_count: 2,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };
        unsafe {
            device
                .device()
                .allocate_descriptor_sets(&alloc_info)
                .unwrap()
        }
    }
}

// IMPL -> Uniform Buffer
impl NreUniformBuffer {
    //
    pub fn new(device: &NreDevice) -> Self {
        let mut buffers = vec![];
        let mut memories = vec![];
        let mut mapped = vec![];
        for i in 0..2 {
            let buffer_info = vk::BufferCreateInfo {
                size: std::mem::size_of::<glam::Mat4>() as u64,
                usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            };
            //
            let buffer = unsafe { device.device().create_buffer(&buffer_info, None).unwrap() };
            let mem_requirements =
                unsafe { device.device().get_buffer_memory_requirements(buffer) };
            let alloc_info = vk::MemoryAllocateInfo {
                allocation_size: mem_requirements.size,
                memory_type_index: NreModel::find_memory_type(
                    device,
                    mem_requirements.memory_type_bits,
                ),
                ..Default::default()
            };
            let memory = unsafe { device.device().allocate_memory(&alloc_info, None).unwrap() };
            unsafe {
                device
                    .device()
                    .bind_buffer_memory(buffer, memory, 0)
                    .unwrap()
            };
            let ptr = unsafe {
                device
                    .device()
                    .map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty())
                    .unwrap()
            };
            buffers.push(buffer);
            memories.push(memory);
            mapped.push(ptr);
        }
        Self {
            buffers: buffers,
            memories: memories,
            mapped: mapped,
        }
    }
    //
    pub fn buffer(&self, index: usize) -> vk::Buffer {
        self.buffers[index]
    }

    pub fn mapped_ptr(&self, index: usize) -> *mut std::ffi::c_void {
        self.mapped[index]
    }
}
