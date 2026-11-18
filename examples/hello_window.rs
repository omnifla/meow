use std::sync::Arc;

use meow::base::{Event, Point, Rect, Size, Widget};
use meow::render::WgpuRenderer;
use meow::style::Color;
use meow::widgets::{Button, ButtonStyle, Flex, Panel, Slot};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct Gpu {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: WgpuRenderer,
    ui: Flex,
    pointer: Point,
}

fn build_ui() -> Flex {
    let style = ButtonStyle {
        normal: Color::rgba(0.35, 0.6, 1.0, 1.0),
        hovered: Color::rgba(0.5, 0.72, 1.0, 1.0),
        pressed: Color::rgba(0.2, 0.4, 0.8, 1.0),
    };
    let button = Button::new(Size { width: 80.0, height: 80.0 }, style)
        .on_click(|| println!("clicked"));

    Flex::column()
        .padding(16.0)
        .gap(12.0)
        .child(Panel {
            size: Size { width: 240.0, height: 60.0 },
            color: Color::rgba(1.0, 0.45, 0.7, 1.0),
        })
        .child(
            Flex::row()
                .gap(12.0)
                .child(button)
                .slot(
                    Slot::new(Panel {
                        size: Size { width: 80.0, height: 80.0 },
                        color: Color::rgba(0.4, 0.9, 0.5, 1.0),
                    })
                    .grow(1.0),
                )
                .child(Panel {
                    size: Size { width: 80.0, height: 80.0 },
                    color: Color::rgba(1.0, 0.85, 0.3, 1.0),
                }),
        )
}

impl Gpu {
    fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("meow"))
                .expect("failed to create window"),
        );
        let size = window.inner_size();

        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle_from_env(Box::new(
                event_loop.owned_display_handle(),
            )),
        );
        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("no compatible adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("failed to create device");

        let config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface not supported by adapter");
        surface.configure(&device, &config);

        let renderer = WgpuRenderer::new(&device, config.format);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            renderer,
            ui: build_ui(),
            pointer: Point::default(),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.window.request_redraw();
    }

    fn handle(&mut self, event: Event) {
        let bounds = Rect::new(0.0, 0.0, self.config.width as f32, self.config.height as f32);
        if self.ui.event(bounds, &event) {
            self.window.request_redraw();
        }
    }

    fn draw(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            _ => {
                self.surface.configure(&self.device, &self.config);
                self.window.request_redraw();
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.renderer.begin(Size {
            width: self.config.width as f32,
            height: self.config.height as f32,
        });

        self.ui.paint(
            Rect::new(0.0, 0.0, self.config.width as f32, self.config.height as f32),
            &mut self.renderer,
        );

        self.renderer.render(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            Color::rgba(0.08, 0.08, 0.1, 1.0),
        );

        self.queue.submit(Some(encoder.finish()));
        self.window.pre_present_notify();
        self.queue.present(frame);
    }
}

#[derive(Default)]
struct App {
    gpu: Option<Gpu>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_none() {
            let gpu = Gpu::new(event_loop);
            gpu.window.request_redraw();
            self.gpu = Some(gpu);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => gpu.resize(size.width, size.height),
            WindowEvent::RedrawRequested => gpu.draw(),
            WindowEvent::CursorMoved { position, .. } => {
                let pointer = Point {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                gpu.pointer = pointer;
                gpu.handle(Event::PointerMoved(pointer));
            }
            WindowEvent::CursorLeft { .. } => gpu.handle(Event::PointerLeft),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let pointer = gpu.pointer;
                gpu.handle(match state {
                    ElementState::Pressed => Event::PointerPressed(pointer),
                    ElementState::Released => Event::PointerReleased(pointer),
                });
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.run_app(&mut App::default()).expect("event loop failed");
}