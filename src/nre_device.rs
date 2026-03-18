use ash::fuchsia::external_memory;
use ash::vk;
use ash::Entry;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
    }
}

pub struct NreDevice {
    entry: Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    surface: vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
}

impl NreDevice {
    pub fn new(window: &winit::window::Window) -> Self {
        let entry = unsafe { Entry::load().unwrap() };
        let instance = Self::create_instance(&entry, window);
        let (surface, surface_loader) = Self::create_surface(&entry, &instance, window);
        let physical_device = Self::choose_physical_device(&instance);
        let (device, graphics_queue) = Self::create_logical_device(&instance, &physical_device);

        Self {
            entry,
            instance,
            physical_device,
            device,
            graphics_queue,
            surface,
            surface_loader,
        }
    }

    fn create_instance(entry: &Entry, window: &winit::window::Window) -> ash::Instance {
        let extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap();

        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            ..Default::default()
        };

        unsafe { entry.create_instance(&create_info, None).unwrap() }
    }

    fn choose_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {
        let devices = unsafe { instance.enumerate_physical_devices().unwrap() };

        devices
            .into_iter()
            .find(|device| Self::is_device_suitable(instance, device))
            .expect("no suitable GPU found")
    }

    fn is_device_suitable(instance: &ash::Instance, device: &vk::PhysicalDevice) -> bool {
        let indices = Self::find_queue_families(instance, device);
        indices.is_complete()
    }

    fn find_queue_families(
        instance: &ash::Instance,
        device: &vk::PhysicalDevice,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*device) };

        let mut indices = QueueFamilyIndices {
            graphics_family: None,
        };

        for (i, family) in queue_families.iter().enumerate() {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
            }

            if indices.is_complete() {
                break;
            }
        }
        indices
    }

    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> (ash::Device, vk::Queue) {
        let indices = Self::find_queue_families(instance, physical_device);

        let queue_priority = 1.0f32;
        let queue_create_info = vk::DeviceQueueCreateInfo {
            queue_family_index: indices.graphics_family.unwrap(),
            queue_count: 1,
            p_queue_priorities: &queue_priority,
            ..Default::default()
        };

        let device_features = vk::PhysicalDeviceFeatures::default();

        let create_info = vk::DeviceCreateInfo {
            p_queue_create_infos: &queue_create_info,
            queue_create_info_count: 1,
            p_enabled_features: &device_features,
            ..Default::default()
        };

        let device = unsafe {
            instance
                .create_device(*physical_device, &create_info, None)
                .unwrap()
        };

        let graphics_queue =
            unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };

        (device, graphics_queue)
    }

    fn create_surface(
        entry: &Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> (vk::SurfaceKHR, ash::khr::surface::Instance) {
        let surface = unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .unwrap()
        };
        let surface_loader = ash::khr::surface::Instance::new(entry, instance);
        (surface, surface_loader)
    }
    //
}
