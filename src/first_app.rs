use crate::nre_camera::Camera;
use crate::nre_descriptor::{NreDescriptorPool, NreDescriptorSetLayout, NreUniformBuffer};
use crate::nre_device::NreDevice;
use crate::nre_file::AppCommand;
use crate::nre_game_object::NreGameObject;
use crate::nre_model::MoleculeObject;
use crate::nre_model::NreModel;
use crate::nre_pipeline::NrePipeline;
use crate::nre_renderer::NreRenderer;
use crate::nre_window::NreWindow;
use ash::vk;
use glam;
use winit::event::{Event, WindowEvent};

pub struct PushConstantData {
    transform: glam::Mat4,
    transition_t: f32,
}

pub struct FirstApp {
    nre_window: NreWindow,
    nre_device: NreDevice,
    nre_renderer: NreRenderer,
    game_objects: Vec<NreGameObject>,
    start_time: std::time::Instant,
    descriptor_set_layout: NreDescriptorSetLayout,
    descriptor_pool: NreDescriptorPool,
    uniform_buffers: NreUniformBuffer,
    descriptor_sets: Vec<vk::DescriptorSet>,
    camera: crate::nre_camera::PerspectiveCamera,
    controller: crate::nre_controller::Controller,
    keys: std::collections::HashSet<winit::keyboard::KeyCode>,
    molecule: Option<MoleculeObject>,
    molecule_pipeline: Option<NrePipeline>,
    bond_pipeline: Option<NrePipeline>,
    cmd_rx: std::sync::mpsc::Receiver<AppCommand>,
    cmd_tx: std::sync::mpsc::Sender<AppCommand>,
    modifiers: winit::event::Modifiers,
}

// !impl
impl FirstApp {
    // !func () -> app instance
    pub fn new() -> Self {
        let nre_window = NreWindow::new(800, 600, "Nemesis Rendering Engine");
        let nre_device = NreDevice::new(&nre_window.window);

        let extent = vk::Extent2D {
            width: 800,
            height: 600,
        };

        let descriptor_set_layout = NreDescriptorSetLayout::new(&nre_device);
        let descriptor_pool = NreDescriptorPool::new(&nre_device);
        let uniform_buffers = NreUniformBuffer::new(&nre_device);
        let descriptor_sets =
            descriptor_pool.allocate_descriptor_sets(&nre_device, descriptor_set_layout.layout());

        let nre_renderer = NreRenderer::new(&nre_device, extent, descriptor_set_layout.layout());

        let model = NreModel::from_obj(&nre_device, "models/character.obj");
        let mut obj1 = NreGameObject::new(model);
        obj1.translation = glam::Vec3::new(0.0, 0.0, 2.0);
        obj1.scale = glam::Vec3::splat(0.5);
        let game_objects = vec![obj1];

        let molecule_data = NreModel::from_pdb("models/molecule.pdb");
        let molecule = MoleculeObject::new(&nre_device, molecule_data);

        let bond_pipeline = NrePipeline::new_bond(
            &nre_device,
            nre_renderer.render_pass(),
            descriptor_set_layout.layout(),
        );

        let molecule_pipeline = NrePipeline::new_molecular(
            &nre_device,
            nre_renderer.render_pass(),
            descriptor_set_layout.layout(),
        );

        let start_time = std::time::Instant::now();

        for i in 0..2 {
            let buffer_info = vk::DescriptorBufferInfo {
                buffer: uniform_buffers.buffer(i),
                offset: 0,
                range: std::mem::size_of::<glam::Mat4>() as u64,
            };
            let write = vk::WriteDescriptorSet {
                dst_set: descriptor_sets[i],
                dst_binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_buffer_info: &buffer_info,
                ..Default::default()
            };
            unsafe { nre_device.device().update_descriptor_sets(&[write], &[]) };
        }

        // !block - channel creation
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<AppCommand>();
        let modifiers = winit::event::Modifiers::default();

        let mut camera =
            crate::nre_camera::PerspectiveCamera::new(800.0 / 600.0, f32::to_radians(45.0));
        let center = glam::Vec3::from(molecule.data.center_of_mass) * 0.05;
        camera.world_position = glam::Vec3::new(center.x - 0.5, center.y + 0.3, center.z - 5.0);
        camera.yaw = std::f32::consts::FRAC_2_PI + 0.9;
        camera.pitch = 0.0;
        let controller = crate::nre_controller::Controller::new();
        let keys = std::collections::HashSet::new();

        // !return
        Self {
            nre_window,
            nre_device,
            nre_renderer,
            game_objects,
            start_time,
            descriptor_set_layout,
            descriptor_pool,
            uniform_buffers,
            descriptor_sets,
            camera,
            controller,
            keys,
            molecule: Some(molecule),
            molecule_pipeline: Some(molecule_pipeline),
            bond_pipeline: Some(bond_pipeline),
            cmd_rx,
            cmd_tx,
            modifiers,
        }
    }

    pub fn run(mut self) {
        let event_loop = self.nre_window.event_loop;
        event_loop
            .run(move |event, elwt| match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    elwt.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event, .. },
                    ..
                } => {
                    use winit::keyboard::PhysicalKey;
                    if let PhysicalKey::Code(code) = event.physical_key {
                        if event.state == winit::event::ElementState::Pressed {
                            self.keys.insert(code);
                        } else {
                            self.keys.remove(&code);
                        }
                    }

                    // /!cmd : Cmd + O / Ctrl + O
                    if event.state == winit::event::ElementState::Pressed {
                        use winit::keyboard::KeyCode;
                        if let PhysicalKey::Code(KeyCode::KeyO) = event.physical_key {
                            #[cfg(target_os = "macos")]
                            let modifier = self.modifiers.state().super_key();
                            #[cfg(not(target_os = "macos"))]
                            let modifier = self.modifiers.state().control_key();

                            if modifier {
                                let tx = self.cmd_tx.clone();
                                std::thread::spawn(move || {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Protein Data Bank (PDB)", &["pdb"])
                                        .pick_file()
                                    {
                                        tx.send(AppCommand::LoadMolecule(path)).ok();
                                    }
                                });
                            }
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    if self.nre_renderer.needs_resize() {
                        let size = self.nre_window.window.inner_size();
                        let new_extent = vk::Extent2D {
                            width: size.width,
                            height: size.height,
                        };
                        self.nre_renderer.recreate_swapchain(
                            &self.nre_device,
                            new_extent,
                            self.descriptor_set_layout.layout(),
                        );
                    }

                    if let Some(cmd) = self.nre_renderer.begin_frame(&self.nre_device) {
                        self.nre_renderer.begin_render_pass(cmd, &self.nre_device);
                        let time = self.start_time.elapsed().as_secs_f32();

                        let dt = 1.0 / 60.0;
                        self.controller.update(dt, &self.keys, &mut self.camera);

                        // drain command queue
                        while let Ok(cmd) = self.cmd_rx.try_recv() {
                            match cmd {
                                AppCommand::LoadMolecule(path) => {
                                    let path_str = path.to_str().unwrap();
                                    let molecule_data = NreModel::from_pdb(path_str);
                                    let new_molecule =
                                        MoleculeObject::new(&self.nre_device, molecule_data);
                                    self.molecule = Some(new_molecule);
                                    self.controller.transition_t = 0.0;
                                    self.controller.transition_dir = 0.0;
                                }
                            }
                        }

                        let view = self.camera.view_matrix();
                        let proj = self.camera.projection_matrix();
                        let frame = self.nre_renderer.current_frame_index();

                        unsafe {
                            self.nre_device.device().cmd_bind_pipeline(
                                cmd,
                                ash::vk::PipelineBindPoint::GRAPHICS,
                                self.nre_renderer.pipeline(),
                            );
                            self.nre_device.device().cmd_bind_descriptor_sets(
                                cmd,
                                vk::PipelineBindPoint::GRAPHICS,
                                self.nre_renderer.pipeline_layout(),
                                0,
                                &[self.descriptor_sets[frame]],
                                &[],
                            );

                            let vp = proj * view;
                            let ptr = self.uniform_buffers.mapped_ptr(frame) as *mut glam::Mat4;
                            ptr.write(vp);

                            for obj in &self.game_objects {
                                let model_mat = obj.transform();
                                let push_data = PushConstantData {
                                    transform: model_mat,
                                    transition_t: 0.0,
                                };
                                let push_bytes = std::slice::from_raw_parts(
                                    &push_data as *const PushConstantData as *const u8,
                                    std::mem::size_of::<PushConstantData>(),
                                );
                                self.nre_device.device().cmd_push_constants(
                                    cmd,
                                    self.nre_renderer.pipeline_layout(),
                                    vk::ShaderStageFlags::VERTEX,
                                    0,
                                    push_bytes,
                                );
                                self.nre_device.device().cmd_bind_vertex_buffers(
                                    cmd,
                                    0,
                                    &[obj.model.vertex_buffer()],
                                    &[0],
                                );
                                self.nre_device.device().cmd_draw(
                                    cmd,
                                    obj.model.vertex_count(),
                                    1,
                                    0,
                                    0,
                                );
                            }

                            let push_data = PushConstantData {
                                transform: glam::Mat4::IDENTITY,
                                transition_t: self.controller.transition_t,
                            };

                            let push_bytes = std::slice::from_raw_parts(
                                &push_data as *const PushConstantData as *const u8,
                                std::mem::size_of::<PushConstantData>(),
                            );

                            self.nre_device.device().cmd_push_constants(
                                cmd,
                                self.nre_renderer.pipeline_layout(),
                                vk::ShaderStageFlags::VERTEX,
                                0,
                                push_bytes,
                            );

                            // bonds first
                            if let (Some(mol_obj), Some(b_pipeline)) =
                                (&self.molecule, &self.bond_pipeline)
                            {
                                let mol_model = &mol_obj.views[mol_obj.current_view].model;
                                if let (Some(bvb), Some(bib), Some(biib)) = (
                                    mol_model.bond_vertex_buffer,
                                    mol_model.bond_index_buffer,
                                    mol_model.bond_instance_buffer,
                                ) {
                                    self.nre_device.device().cmd_bind_descriptor_sets(
                                        cmd,
                                        vk::PipelineBindPoint::GRAPHICS,
                                        b_pipeline.pipeline_layout(),
                                        0,
                                        &[self.descriptor_sets[frame]],
                                        &[],
                                    );
                                    self.nre_device.device().cmd_bind_pipeline(
                                        cmd,
                                        vk::PipelineBindPoint::GRAPHICS,
                                        b_pipeline.pipeline(),
                                    );
                                    self.nre_device.device().cmd_bind_vertex_buffers(
                                        cmd,
                                        0,
                                        &[bvb],
                                        &[0],
                                    );
                                    self.nre_device.device().cmd_bind_vertex_buffers(
                                        cmd,
                                        2,
                                        &[biib],
                                        &[0],
                                    );
                                    self.nre_device.device().cmd_bind_index_buffer(
                                        cmd,
                                        bib,
                                        0,
                                        vk::IndexType::UINT32,
                                    );
                                    self.nre_device.device().cmd_draw_indexed(
                                        cmd,
                                        mol_model.bond_index_count,
                                        mol_model.bond_instance_count,
                                        0,
                                        0,
                                        0,
                                    );
                                }
                            }

                            // atoms on top
                            if let (Some(mol_obj), Some(mol_pipeline)) =
                                (&self.molecule, &self.molecule_pipeline)
                            {
                                let mol_model = &mol_obj.views[mol_obj.current_view].model;
                                self.nre_device.device().cmd_bind_pipeline(
                                    cmd,
                                    vk::PipelineBindPoint::GRAPHICS,
                                    mol_pipeline.pipeline(),
                                );
                                self.nre_device.device().cmd_bind_vertex_buffers(
                                    cmd,
                                    0,
                                    &[mol_model.vertex_buffer()],
                                    &[0],
                                );
                                self.nre_device.device().cmd_bind_vertex_buffers(
                                    cmd,
                                    1,
                                    &[mol_model.instance_buffer().unwrap()],
                                    &[0],
                                );
                                self.nre_device.device().cmd_bind_index_buffer(
                                    cmd,
                                    mol_model.index_buffer().unwrap(),
                                    0,
                                    vk::IndexType::UINT32,
                                );
                                self.nre_device.device().cmd_draw_indexed(
                                    cmd,
                                    mol_model.index_count(),
                                    mol_model.instance_count(),
                                    0,
                                    0,
                                    0,
                                );
                            }
                        } // closes unsafe
                        self.nre_renderer.end_render_pass(cmd, &self.nre_device);
                        self.nre_renderer.end_frame(&self.nre_device);
                    } // closes if let Some(cmd)
                } // closes RedrawRequested
                Event::AboutToWait => {
                    self.nre_window.window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    let new_extent = vk::Extent2D {
                        width: size.width,
                        height: size.height,
                    };
                    self.nre_renderer.recreate_swapchain(
                        &self.nre_device,
                        new_extent,
                        self.descriptor_set_layout.layout(),
                    );
                    self.camera.aspect_ratio = size.width as f32 / size.height as f32;
                }
                // modifiers changed event arm
                Event::WindowEvent {
                    event: WindowEvent::ModifiersChanged(mods),
                    ..
                } => {
                    self.modifiers = mods;
                }
                _ => {}
            })
            .unwrap();
    }
}
