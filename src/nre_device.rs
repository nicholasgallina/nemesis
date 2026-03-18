use ash::vk;
use ash::Entry;

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
}

impl NreDevice {
    pub fn new() -> Self {
        let entry = unsafe { Entry::load().unwrap() };
        let instance = Self::create_instance(&entry);
        let physical_device = Self::choose_physical_device(&instance);

        Self {
            entry,
            instance,
            physical_device,
        }
    }

    fn create_instance(entry: &Entry) -> ash::Instance {
        let app_info = vk::ApplicationInfo {
            api_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
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
    //
}
