use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{
    ElementState, KeyEvent, MouseButton, MouseScrollDelta, StartCause, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key as LogicalKey, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use crate::base::{Event, Key, Modifiers, Point, Rect, Widget};
use crate::clipboard;
use crate::reactive;
use crate::render::WgpuRenderer;
use crate::style::Color;

const BACKGROUND: Color = Color::rgba(0.08, 0.08, 0.1, 1.0);
const LINE_PIXELS: f32 = 40.0;

struct Gpu {
    window: Arc<Window>,
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    renderer: WgpuRenderer,
    ui: Box<dyn Widget>,
    pointer: Point,
    modifiers: ModifiersState,
    wake: Option<Instant>,
    board: Option<arboard::Clipboard>,
}

impl Gpu {
    fn new(event_loop: &ActiveEventLoop, title: &str, ui: Box<dyn Widget>) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title(title))
                .expect("failed to create window"),
        );
        let size = window.inner_size();

        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle_from_env(
                Box::new(event_loop.owned_display_handle()),
            ));
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
            instance,
            surface,
            device,
            queue,
            config,
            renderer,
            ui,
            pointer: Point::default(),
            modifiers: ModifiersState::default(),
            wake: None,
            board: arboard::Clipboard::new().ok(),
        }
    }

    fn bounds(&self) -> Rect {
        Rect::new(
            0.0,
            0.0,
            self.config.width as f32,
            self.config.height as f32,
        )
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        self.window.request_redraw();
    }

    fn recreate_surface(&mut self) {
        if let Ok(surface) = self.instance.create_surface(self.window.clone()) {
            self.surface = surface;
            self.surface.configure(&self.device, &self.config);
            self.window.request_redraw();
        }
    }

    fn handle(&mut self, event: Event) {
        let bounds = self.bounds();
        let changed = self.ui.event(bounds, &event);
        if let Some(text) = clipboard::take_write() {
            if let Some(board) = self.board.as_mut() {
                let _ = board.set_text(text);
            }
        }
        let dirty = reactive::take_dirty();
        if changed || dirty {
            self.window.request_redraw();
        }
    }

    fn key(&mut self, event: &KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }
        let command = (self.modifiers.control_key() && !self.modifiers.alt_key())
            || self.modifiers.super_key();
        if command {
            if let LogicalKey::Character(text) = &event.logical_key {
                match text.to_lowercase().as_str() {
                    "a" => self.handle(Event::SelectAll),
                    "c" => self.handle(Event::Copy),
                    "x" => self.handle(Event::Cut),
                    "v" => {
                        let pasted = self.board.as_mut().and_then(|board| board.get_text().ok());
                        if let Some(text) = pasted {
                            self.handle(Event::Paste(text));
                        }
                    }
                    _ => {}
                }
            }
            return;
        }
        if let Some(key) = map_key(&event.logical_key) {
            let modifiers = Modifiers {
                shift: self.modifiers.shift_key(),
                ctrl: self.modifiers.control_key(),
                alt: self.modifiers.alt_key(),
            };
            self.handle(Event::KeyPressed(key, modifiers));
        }
        if let Some(text) = &event.text {
            for ch in text.chars().filter(|ch| !ch.is_control()) {
                self.handle(Event::Char(ch));
            }
        }
    }

    fn draw(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                self.window.request_redraw();
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.recreate_surface();
                return;
            }
            _ => return,
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let _ = reactive::take_wake();
        let bounds = self.bounds();
        self.renderer.begin(bounds.size);
        self.ui.paint(bounds, &mut self.renderer);
        self.wake = reactive::take_wake().map(|after| Instant::now() + after);
        self.renderer
            .render(&self.device, &self.queue, &mut encoder, &view, BACKGROUND);

        self.queue.submit(Some(encoder.finish()));
        self.window.pre_present_notify();
        self.queue.present(frame);
    }
}

struct App {
    title: String,
    ui: Option<Box<dyn Widget>>,
    gpu: Option<Gpu>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        if let Some(ui) = self.ui.take() {
            let gpu = Gpu::new(event_loop, &self.title, ui);
            gpu.window.request_redraw();
            self.gpu = Some(gpu);
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            if let Some(gpu) = self.gpu.as_mut() {
                gpu.wake = None;
                gpu.window.request_redraw();
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let flow = match self.gpu.as_ref().and_then(|gpu| gpu.wake) {
            Some(deadline) => ControlFlow::WaitUntil(deadline),
            None => ControlFlow::Wait,
        };
        event_loop.set_control_flow(flow);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(gpu) = self.gpu.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => gpu.resize(size.width, size.height),
            WindowEvent::RedrawRequested => gpu.draw(),
            WindowEvent::Occluded(false) => gpu.window.request_redraw(),
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
            WindowEvent::ModifiersChanged(modifiers) => gpu.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput { event, .. } => gpu.key(&event),
            WindowEvent::MouseWheel { delta, .. } => {
                let pixels = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -y * LINE_PIXELS,
                    MouseScrollDelta::PixelDelta(position) => -position.y as f32,
                };
                let pointer = gpu.pointer;
                gpu.handle(Event::Scroll(pointer, pixels));
            }
            _ => {}
        }
    }
}

fn map_key(key: &LogicalKey) -> Option<Key> {
    match key {
        LogicalKey::Named(NamedKey::Backspace) => Some(Key::Backspace),
        LogicalKey::Named(NamedKey::Delete) => Some(Key::Delete),
        LogicalKey::Named(NamedKey::ArrowLeft) => Some(Key::Left),
        LogicalKey::Named(NamedKey::ArrowRight) => Some(Key::Right),
        LogicalKey::Named(NamedKey::Home) => Some(Key::Home),
        LogicalKey::Named(NamedKey::End) => Some(Key::End),
        LogicalKey::Named(NamedKey::Enter) => Some(Key::Enter),
        LogicalKey::Named(NamedKey::Escape) => Some(Key::Escape),
        _ => None,
    }
}

pub fn run(title: &str, ui: impl Widget + 'static) {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = App {
        title: title.to_string(),
        ui: Some(Box::new(ui)),
        gpu: None,
    };
    event_loop.run_app(&mut app).expect("event loop failed");
}
