use egui::Rgba;
use egui_wgpu::ScreenDescriptor;
use std::sync::Arc;
use wgpu::SurfaceTexture;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use crate::{error::Result as CasterResult, ContentType};

/// egui-based display window for casting content
pub struct CastWindow {
    window: Option<Arc<Window>>,
    egui_ctx: Option<egui::Context>,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,
    wgpu_state: Option<WgpuState>,

    // Playback state
    content_type: Option<ContentType>,
    playback_state: PlaybackState,
    volume: f32,
    seek_position: f32,

    // Content rendering
    content_texture: Option<wgpu::Texture>,
    content_data: Vec<u8>,
}

/// WGPU rendering state for the window.
/// 
/// The `Surface` has a `'static` lifetime because it is created from an `Arc<Window>` 
/// which is also stored with the same lifetime. This coupling is implicit but safe
/// since both the window and surface are owned by the same `CastWindow` instance.
struct WgpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Loading,
}

impl CastWindow {
    pub fn new() -> Self {
        Self {
            window: None,
            egui_ctx: None,
            egui_state: None,
            egui_renderer: None,
            wgpu_state: None,
            content_type: None,
            playback_state: PlaybackState::Stopped,
            volume: 0.8,
            seek_position: 0.0,
            content_texture: None,
            content_data: Vec::new(),
        }
    }

    pub fn set_content(&mut self, content_type: ContentType, data: Vec<u8>) {
        self.content_type = Some(content_type);
        self.content_data = data;
        self.playback_state = PlaybackState::Loading;
    }

    pub fn play(&mut self) {
        if self.playback_state != PlaybackState::Playing {
            self.playback_state = PlaybackState::Playing;
        }
    }

    pub fn pause(&mut self) {
        if self.playback_state == PlaybackState::Playing {
            self.playback_state = PlaybackState::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.playback_state = PlaybackState::Stopped;
        self.seek_position = 0.0;
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn seek(&mut self, position: f32) {
        self.seek_position = position.clamp(0.0, 1.0);
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        // Top panel with controls
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Q8 Caster");

                ui.separator();

                // Playback controls
                if ui.button("⏮").clicked() {
                    self.seek_position = 0.0;
                }

                match self.playback_state {
                    PlaybackState::Playing => {
                        if ui.button("⏸").clicked() {
                            self.pause();
                        }
                    }
                    PlaybackState::Paused | PlaybackState::Stopped => {
                        if ui.button("▶").clicked() {
                            self.play();
                        }
                    }
                    PlaybackState::Loading => {
                        ui.spinner();
                    }
                }

                if ui.button("⏹").clicked() {
                    self.stop();
                }

                if ui.button("⏭").clicked() {
                    self.seek_position = 1.0;
                }

                ui.separator();

                // Volume control
                ui.label("🔊");
                ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).show_value(false));

                ui.separator();

                // Status
                ui.label(format!("Status: {:?}", self.playback_state));

                if let Some(ref content_type) = self.content_type {
                    ui.label(format!("Type: {:?}", content_type));
                }
            });
        });

        // Bottom panel with seek bar
        egui::TopBottomPanel::bottom("seek").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("0:00");
                if ui.add(egui::Slider::new(&mut self.seek_position, 0.0..=1.0).show_value(false)).changed() {
                    // Seek position changed
                }
                ui.label("0:00");
            });
            ui.add_space(4.0);
        });

        // Central panel for content display
        egui::CentralPanel::default().show(ctx, |ui| {
            // Display content based on type
            match &self.content_type {
                Some(ContentType::Markdown { .. }) => {
                    self.render_markdown(ui);
                }
                Some(ContentType::Image { .. }) => {
                    self.render_image(ui);
                }
                Some(ContentType::Video { .. }) => {
                    self.render_video(ui);
                }
                Some(ContentType::Audio { .. }) => {
                    self.render_audio(ui);
                }
                Some(ContentType::Pdf { page }) => {
                    self.render_pdf(ui, *page);
                }
                Some(ContentType::WebAssembly { .. }) => {
                    self.render_wasm(ui);
                }
                Some(ContentType::ScreenMirror { .. }) => {
                    self.render_screen_mirror(ui);
                }
                Some(_) => {
                    ui.centered_and_justified(|ui| {
                        ui.label("Rendering not implemented for this content type");
                    });
                }
                None => {
                    ui.centered_and_justified(|ui| {
                        ui.label("No content loaded");
                        ui.label("Ready to cast...");
                    });
                }
            }
        });
    }

    fn render_markdown(&self, ui: &mut egui::Ui) {
        if let Ok(text) = std::str::from_utf8(&self.content_data) {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(egui::Label::new(text).wrap());
            });
        }
    }

    fn render_image(&self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.label("Image rendering");
            // TODO: Load image with image crate and display
        });
    }

    fn render_video(&self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.label("Video playback");
            // TODO: Integrate with GStreamer for video playback
        });
    }

    fn render_audio(&self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.heading("🎵");
            ui.label("Audio playback");

            // Audio visualizer placeholder
            ui.add_space(20.0);
            let painter = ui.painter();
            let rect = ui.available_rect_before_wrap();
            let center = rect.center();

            // Simple waveform visualization
            for i in 0..50 {
                let x = rect.left() + (i as f32 / 50.0) * rect.width();
                let height = (i as f32 * 0.1).sin() * 50.0 * self.volume;
                let y1 = center.y - height;
                let y2 = center.y + height;

                painter.line_segment(
                    [egui::pos2(x, y1), egui::pos2(x, y2)],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)),
                );
            }
        });
    }

    fn render_pdf(&self, ui: &mut egui::Ui, page: Option<u32>) {
        ui.centered_and_justified(|ui| {
            ui.label(format!("PDF Document - Page: {}", page.unwrap_or(1)));
            // TODO: Render PDF using pdfium-render
        });
    }

    fn render_wasm(&self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.label("WebAssembly Module");
            ui.label("Running WASM...");
            // TODO: Execute WASM module with wasmer
        });
    }

    fn render_screen_mirror(&self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.label("Screen Mirroring Active");
            // TODO: Display captured screen content
        });
    }
}

impl ApplicationHandler for CastWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Q8 Caster")
                .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
                .with_resizable(true);

            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

            // Initialize egui
            let egui_ctx = egui::Context::default();
            let egui_state = egui_winit::State::new(
                egui_ctx.clone(),
                egui::ViewportId::ROOT,
                &window,
                Some(window.scale_factor() as f32),
                Some(2048),
            );

            // Initialize wgpu
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });

            let surface = instance.create_surface(window.clone()).unwrap();

            let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }))
            .unwrap();

            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            ))
            .unwrap();

            let size = window.inner_size();
            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface.get_capabilities(&adapter).formats[0],
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&device, &surface_config);

            let egui_renderer = egui_wgpu::Renderer::new(&device, surface_config.format, None, 1, false);

            self.window = Some(window);
            self.egui_ctx = Some(egui_ctx);
            self.egui_state = Some(egui_state);
            self.egui_renderer = Some(egui_renderer);
            self.wgpu_state = Some(WgpuState {
                device,
                queue,
                surface,
                surface_config,
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(ref mut egui_state) = self.egui_state {
            if let Some(ref window) = self.window {
                let response = egui_state.on_window_event(window, &event);

                if response.repaint {
                    window.request_redraw();
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(ref mut wgpu_state) = self.wgpu_state {
                    wgpu_state.surface_config.width = size.width;
                    wgpu_state.surface_config.height = size.height;
                    wgpu_state.surface.configure(&wgpu_state.device, &wgpu_state.surface_config);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(ref window), Some(ref egui_ctx), Some(ref mut egui_state), Some(ref mut egui_renderer), Some(ref mut wgpu_state)) =
                    (&self.window, &self.egui_ctx, &mut self.egui_state, &mut self.egui_renderer, &mut self.wgpu_state)
                {
                    let raw_input = egui_state.take_egui_input(window);
                    let output = egui_ctx.run(raw_input, |ctx| {
                        self.render_ui(ctx);
                    });

                    egui_state.handle_platform_output(window, output.platform_output);

                    let paint_jobs = egui_ctx.tessellate(output.shapes, output.pixels_per_point);

                    let screen_descriptor = ScreenDescriptor {
                        size_in_pixels: [wgpu_state.surface_config.width, wgpu_state.surface_config.height],
                        pixels_per_point: window.scale_factor() as f32,
                    };

                    let frame = wgpu_state.surface.get_current_texture().unwrap();
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    let mut encoder = wgpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

                    // Upload egui textures
                    for (id, image_delta) in &output.textures_delta.set {
                        egui_renderer.update_texture(&wgpu_state.device, &wgpu_state.queue, *id, image_delta);
                    }

                    // Render pass
                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.1,
                                        g: 0.1,
                                        b: 0.1,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                    }

                    // Free egui textures
                    for id in &output.textures_delta.free {
                        egui_renderer.free_texture(id);
                    }

                    wgpu_state.queue.submit(std::iter::once(encoder.finish()));
                    frame.present();

                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

pub fn run_cast_window() -> CasterResult<()> {
    let event_loop = EventLoop::new().map_err(|e| crate::error::CasterError::Display(e.to_string()))?;
    let mut app = CastWindow::new();

    event_loop.run_app(&mut app).map_err(|e| crate::error::CasterError::Display(e.to_string()))?;

    Ok(())
}
