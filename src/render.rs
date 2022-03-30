use std::{iter::once, mem::size_of, rc::Rc};

use bytemuck::{cast_slice, Pod, Zeroable};
use cgmath::Vector3;
use futures_lite::future;
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer,
    BufferBinding, BufferBindingType, BufferUsages, Color, DepthBiasState, DepthStencilState,
    Device, DeviceDescriptor, Extent3d, Face, Features, FragmentState, FrontFace, IndexFormat,
    Instance, Limits, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor, PolygonMode,
    PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderStages, StencilState, Surface,
    SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::window::Window;

use crate::camera::{Camera, Matrices};

pub struct State {
    surface: Surface,
    format: TextureFormat,
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    camera_buffer: Buffer,
    camera_group: BindGroup,
    depth_buffer: TextureView,
}

pub fn init(window: &Window) -> State {
    let instance = Instance::new(Backends::all());
    let surface = unsafe { instance.create_surface(window) };
    let init_device = future::block_on(init_device(&instance, &surface));
    let (format, device, queue) = (init_device.format, init_device.device, init_device.queue);

    let shader = device.create_shader_module(&include_wgsl!("shader.wgsl"));

    let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: &[0; size_of::<Matrices>()],
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });

    let camera_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    let camera_group = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &camera_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::Buffer(BufferBinding {
                buffer: &camera_buffer,
                offset: 0,
                size: None,
            }),
        }],
    });

    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: None,
        layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&camera_group_layout],
            push_constant_ranges: &[],
        })),
        vertex: VertexState {
            module: &shader,
            entry_point: "vertex",
            buffers: &[VertexBufferLayout {
                array_stride: size_of::<Vertex>() as u64,
                step_mode: VertexStepMode::Vertex,
                attributes: &vertex_attr_array![
                    0 => Float32x3,
                    1 => Float32x3,
                    2 => Float32x3,
                ],
            }],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: Some(Face::Back),
            unclipped_depth: false,
            polygon_mode: PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: "fragment",
            targets: &[format.into()],
        }),
        multiview: None,
    });

    let depth_buffer = create_depth_buffer(&device, 1, 1);

    State {
        surface,
        format,
        device,
        queue,
        pipeline,
        camera_buffer,
        camera_group,
        depth_buffer,
    }
}

async fn init_device(instance: &Instance, surface: &Surface) -> InitDevice {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(surface),
            ..Default::default()
        })
        .await
        .unwrap();

    let format = surface.get_preferred_format(&adapter).unwrap();

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(),
                limits: Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    InitDevice {
        device,
        queue,
        format,
    }
}

struct InitDevice {
    device: Device,
    queue: Queue,
    format: TextureFormat,
}

impl State {
    pub fn configure(&mut self, width: u32, height: u32) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.format,
                width,
                height,
                present_mode: PresentMode::Fifo,
            },
        );

        self.depth_buffer = create_depth_buffer(&self.device, width, height);
    }

    pub fn render(&self, camera: &Camera, meshes: &[Rc<Mesh>]) {
        self.queue
            .write_buffer(&self.camera_buffer, 0, cast_slice(&[camera.matrices()]));

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.camera_group, &[]);
            for mesh in meshes {
                pass.set_vertex_buffer(0, mesh.vertices.slice(..));
                pass.set_index_buffer(mesh.triangles.slice(..), IndexFormat::Uint16);
                pass.draw_indexed(0..mesh.triangles_len * 3, 0, 0..1);
            }
        }

        self.queue.submit(once(encoder.finish()));
        frame.present();
    }

    pub fn create_mesh(&self, vertices: &[Vertex], triangles: &[[u16; 3]]) -> Mesh {
        Mesh {
            vertices: self.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: cast_slice(vertices),
                usage: BufferUsages::VERTEX,
            }),
            triangles: self.device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: cast_slice(triangles),
                usage: BufferUsages::INDEX,
            }),
            triangles_len: triangles.len() as u32,
        }
    }
}

pub struct Mesh {
    vertices: Buffer,
    triangles: Buffer,
    triangles_len: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub color: Vector3<f32>,
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

fn create_depth_buffer(device: &Device, width: u32, height: u32) -> TextureView {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT,
    });

    texture.create_view(&Default::default())
}
