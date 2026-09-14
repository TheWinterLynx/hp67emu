//! Experimental GPU-generated optical/material layer.
//!
//! The calculator remains the existing vector egui panel. WGPU renders a
//! deterministic high-resolution optical layer into an offscreen RGBA texture
//! once at startup, then alpha-composites it over the vector artwork. This keeps
//! hitboxes, legends and calculator logic completely independent of the effect.

use std::sync::atomic::{AtomicBool, Ordering};

use eframe::{
    egui,
    egui_wgpu::{self, wgpu},
};

const FX_WIDTH: u32 = 660;
const FX_HEIGHT: u32 = 1240;
const MAIN_MSAA_SAMPLES: u32 = 4;

static READY: AtomicBool = AtomicBool::new(false);

struct PhotoFxResources {
    composite_pipeline: wgpu::RenderPipeline,
    composite_bind_group: wgpu::BindGroup,
    // Keep all resources explicitly alive for the lifetime of the callback.
    _texture: wgpu::Texture,
    _view: wgpu::TextureView,
    _sampler: wgpu::Sampler,
}

struct PhotoFxCallback;

impl egui_wgpu::CallbackTrait for PhotoFxCallback {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &PhotoFxResources = resources
            .get()
            .expect("HP-67 photo FX resources were not installed");
        render_pass.set_pipeline(&resources.composite_pipeline);
        render_pass.set_bind_group(0, &resources.composite_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

pub fn install(cc: &eframe::CreationContext<'_>) {
    let Some(render_state) = cc.wgpu_render_state.as_ref() else {
        return;
    };
    let device = &render_state.device;
    let queue = &render_state.queue;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("hp67-photo-fx"),
        source: wgpu::ShaderSource::Wgsl(include_str!("photo_fx.wgsl").into()),
    });

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("hp67-photo-fx-offscreen"),
        size: wgpu::Extent3d {
            width: FX_WIDTH,
            height: FX_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // First pass: synthesize the deterministic material/optical response into
    // the offscreen texture. It is static, so doing this once avoids per-frame
    // GPU work while still using a real render-to-texture pass.
    let offscreen_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("hp67-photo-fx-offscreen-layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    let offscreen_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("hp67-photo-fx-offscreen-pipeline"),
        layout: Some(&offscreen_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fx_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("hp67-photo-fx-offscreen-encoder"),
    });
    {
        let attachments = [Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("hp67-photo-fx-offscreen-pass"),
            color_attachments: &attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&offscreen_pipeline);
        pass.draw(0..3, 0..1);
    }
    queue.submit(Some(encoder.finish()));

    // Second pass: sample the offscreen layer during egui's main render pass.
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("hp67-photo-fx-sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let composite_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("hp67-photo-fx-composite-bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let composite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("hp67-photo-fx-composite-bind-group"),
        layout: &composite_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("hp67-photo-fx-composite-layout"),
        bind_group_layouts: &[&composite_bgl],
        push_constant_ranges: &[],
    });
    let composite_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("hp67-photo-fx-composite-pipeline"),
        layout: Some(&composite_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "composite_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: render_state.target_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: MAIN_MSAA_SAMPLES,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    render_state
        .renderer
        .write()
        .callback_resources
        .insert(PhotoFxResources {
            composite_pipeline,
            composite_bind_group,
            _texture: texture,
            _view: view,
            _sampler: sampler,
        });
    READY.store(true, Ordering::Release);
}

pub fn paint(ui: &egui::Ui, rect: egui::Rect) {
    if !READY.load(Ordering::Acquire) || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    ui.painter()
        .add(egui_wgpu::Callback::new_paint_callback(rect, PhotoFxCallback));
}
