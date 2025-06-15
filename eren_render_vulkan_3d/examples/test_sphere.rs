use std::{collections::HashSet, sync::Arc};

use ash::vk;
use eren_render_vulkan_3d::render::{
    render_item::{Material, Mesh, RenderItem},
    renderer_3d::Renderer3D,
};
use eren_render_vulkan_core::{
    context::GraphicsContext,
    vulkan::memory::{MemoryError, create_buffer_with_memory},
};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize};
use winit::window::Window;

use native_dialog::{DialogBuilder, MessageLevel};

const PLANE_VERTS: [[f32; 8]; 4] = [
    [-1.0, 0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0],
    [1.0, 0.0, -1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
    [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0],
    [-1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0],
];
const PLANE_IDXS: [u32; 6] = [0, 1, 2, 2, 3, 0];

pub fn show_error_popup_and_panic<E: std::fmt::Display>(error: E, context: &str) -> ! {
    DialogBuilder::message()
        .set_level(MessageLevel::Error)
        .set_title(context)
        .set_text(error.to_string())
        .alert()
        .show()
        .unwrap();
    panic!("{}: {}", context, error);
}

fn create_mesh_from_data(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    vertices: &[[f32; 8]],
    indices: &[u32],
) -> Result<Arc<Mesh>, MemoryError> {
    let vertex_buffer_size = (vertices.len() * size_of::<[f32; 8]>()) as vk::DeviceSize;
    let index_buffer_size = (indices.len() * size_of::<u32>()) as vk::DeviceSize;

    // Vertex buffer
    let (vertex_buffer, vertex_memory) = create_buffer_with_memory(
        instance,
        physical_device,
        device,
        vertex_buffer_size,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    // Map vertex buffer memory and copy data
    unsafe {
        let data_ptr = device
            .map_memory(
                vertex_memory,
                0,
                vertex_buffer_size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to map vertex memory") as *mut [f32; 8];

        std::ptr::copy_nonoverlapping(vertices.as_ptr(), data_ptr, vertices.len());

        device.unmap_memory(vertex_memory);
    }

    // Index buffer
    let (index_buffer, index_memory) = create_buffer_with_memory(
        instance,
        physical_device,
        device,
        index_buffer_size,
        vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    // Map index buffer memory and copy data
    unsafe {
        let data_ptr = device
            .map_memory(
                index_memory,
                0,
                index_buffer_size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to map index memory") as *mut u32;

        std::ptr::copy_nonoverlapping(indices.as_ptr(), data_ptr, indices.len());

        device.unmap_memory(index_memory);
    }

    Ok(Arc::new(Mesh {
        vertex_buffer,
        vertex_memory,
        index_buffer,
        index_memory,
        index_count: indices.len() as u32,
    }))
}

// Dummy sphere generator returning ([f32; 8] position+normal+uv, u32 indices).
fn generate_uv_sphere(radius: f32, lon: u32, lat: u32) -> (Vec<[f32; 8]>, Vec<u32>) {
    let mut verts = Vec::new();
    let mut idxs = Vec::new();
    for y in 0..=lat {
        let v = y as f32 / lat as f32;
        let theta = v * std::f32::consts::PI;
        for x in 0..=lon {
            let u = x as f32 / lon as f32;
            let phi = u * std::f32::consts::TAU;
            let pos = [
                radius * phi.sin() * theta.sin(),
                radius * theta.cos(),
                radius * phi.cos() * theta.sin(),
            ];
            let nx = pos[0] / radius;
            let ny = pos[1] / radius;
            let nz = pos[2] / radius;
            verts.push([pos[0], pos[1], pos[2], nx, ny, nz, u, v]);
        }
    }
    // indices
    for y in 0..lat {
        for x in 0..lon {
            let i0 = y * (lon + 1) + x;
            let i1 = i0 + lon + 1;
            idxs.extend_from_slice(&[i0, i1, i0 + 1, i0 + 1, i1, i1 + 1]);
        }
    }
    (verts, idxs.iter().map(|&i| i as u32).collect())
}

fn create_dummy_material(device: &ash::Device) -> Result<Arc<Material>, vk::Result> {
    let layout_info = vk::DescriptorSetLayoutCreateInfo::default();
    let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };
    let descriptor_set_layouts = vec![descriptor_set_layout];

    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(1)
        .pool_sizes(&[]);

    let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&descriptor_set_layouts);

    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

    Ok(Arc::new(Material {
        descriptor_set_layout,
        descriptor_pool,
        descriptor_set: descriptor_sets[0],
    }))
}

struct TestWindowEventHandler {
    graphics_context: GraphicsContext,
    renderer: Option<Renderer3D>,
    render_items: Vec<RenderItem>,
}

impl TestWindowEventHandler {
    fn recreate_renderer(&mut self) {
        let instance_manager = self.graphics_context.instance_manager.as_ref().unwrap();
        let physical_device_manager = self
            .graphics_context
            .physical_device_manager
            .as_ref()
            .unwrap();
        let device_manager = self.graphics_context.device_manager.as_ref().unwrap();
        let swapchain_manager = self.graphics_context.swapchain_manager.as_ref().unwrap();

        let renderer = match Renderer3D::new(
            &instance_manager.instance,
            physical_device_manager.physical_device,
            device_manager.device.clone(),
            &self.graphics_context.swapchain_image_views,
            swapchain_manager.preferred_surface_format,
            swapchain_manager.image_extent,
        ) {
            Ok(renderer) => renderer,
            Err(e) => show_error_popup_and_panic(e, "Failed to create renderer"),
        };

        self.renderer = Some(renderer);

        let plane_mesh = match create_mesh_from_data(
            &instance_manager.instance,
            physical_device_manager.physical_device,
            &device_manager.device,
            &PLANE_VERTS,
            &PLANE_IDXS,
        ) {
            Ok(mesh) => mesh,
            Err(e) => show_error_popup_and_panic(e, "Failed to create plane mesh"),
        };

        let (sphere_verts, sphere_idxs) = generate_uv_sphere(1.0, 32, 16);

        let sphere_mesh = match create_mesh_from_data(
            &instance_manager.instance,
            physical_device_manager.physical_device,
            &device_manager.device,
            &sphere_verts,
            &sphere_idxs,
        ) {
            Ok(mesh) => mesh,
            Err(e) => show_error_popup_and_panic(e, "Failed to create sphere mesh"),
        };

        let material = match create_dummy_material(&device_manager.device) {
            Ok(material) => material,
            Err(e) => show_error_popup_and_panic(e, "Failed to create dummy material"),
        };

        self.render_items.push(RenderItem {
            mesh: plane_mesh,
            material: material.clone(),
            transform: glam::Mat4::IDENTITY,
        });

        self.render_items.push(RenderItem {
            mesh: sphere_mesh,
            material,
            transform: glam::Mat4::IDENTITY,
        });
    }

    fn clear(&mut self) {
        self.renderer = None;

        let device_manager = self.graphics_context.device_manager.as_ref().unwrap();
        let device = &device_manager.device;

        let mut unique_meshes = HashSet::new();
        let mut unique_materials = HashSet::new();

        for render_item in &self.render_items {
            unique_meshes.insert(Arc::as_ptr(&render_item.mesh));
            unique_materials.insert(Arc::as_ptr(&render_item.material));
        }

        for mesh_ptr in &unique_meshes {
            let mesh = unsafe { Arc::from_raw(*mesh_ptr) };
            unsafe {
                device.destroy_buffer(mesh.vertex_buffer, None);
                device.free_memory(mesh.vertex_memory, None);
                device.destroy_buffer(mesh.index_buffer, None);
                device.free_memory(mesh.index_memory, None);
            }
            std::mem::forget(mesh);
        }

        for material_ptr in &unique_materials {
            let material = unsafe { Arc::from_raw(*material_ptr) };
            unsafe {
                device.destroy_descriptor_pool(material.descriptor_pool, None);
                device.destroy_descriptor_set_layout(material.descriptor_set_layout, None);
            }
            std::mem::forget(material);
        }

        self.render_items.clear();

        self.graphics_context.destroy();
    }
}

impl WindowEventHandler for TestWindowEventHandler {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        println!(
            "Window ready: {}x{}",
            window.inner_size().width,
            window.inner_size().height
        );

        match self.graphics_context.init(window) {
            Ok(_) => {}
            Err(e) => show_error_popup_and_panic(e, "Failed to initialize graphics context"),
        };

        self.recreate_renderer();
    }

    fn on_window_lost(&mut self) {
        println!("Window lost");

        self.clear();
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        println!("Window resized: {:?}", size);

        self.graphics_context.resize(size);
    }

    fn redraw(&mut self) {
        if let Some(renderer) = &self.renderer {
            match self.graphics_context.redraw(renderer, &self.render_items) {
                Ok(renderer_needs_recreation) => {
                    if renderer_needs_recreation {
                        self.recreate_renderer();
                    }
                }
                Err(e) => show_error_popup_and_panic(e, "Failed to redraw graphics context"),
            }
        }
    }

    fn on_window_close_requested(&mut self) {
        self.clear();
    }
}

fn main() {
    match WindowLifecycleManager::new(
        WindowConfig {
            width: 800,
            height: 600,
            title: "Test Window",
            canvas_id: None,
        },
        TestWindowEventHandler {
            graphics_context: match GraphicsContext::new() {
                Ok(graphics_context) => graphics_context,
                Err(e) => show_error_popup_and_panic(e, "Failed to create graphics context"),
            },
            renderer: None,
            render_items: Vec::new(),
        },
    )
    .start_event_loop()
    {
        Ok(_) => {}
        Err(e) => show_error_popup_and_panic(e, "Failed to start event loop"),
    }
}
