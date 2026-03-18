use ash::vk;
use ash::Entry;

pub struct NreDevice {
    entry: Entry,
    instance: ash::Instance,
}

impl NreDevice {
    pub fn new() -> Self {
        let entry = unsafe { Entry::load().unwrap() };
        let instance = Self::create_instance(&entry);

        Self { entry, instance }
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
}
