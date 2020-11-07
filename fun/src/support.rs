use fun_capture::capture::quartz::QuartzCapture;
use fun_capture::capture::CaptureOpts;
use fun_capture::display::get_primary;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::glutin::ContextBuilder;
use glium::{Display, Surface};
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

pub enum AppEvent<'a> {
  Render(&'a Ui<'a>),
  Update(&'a Display, &'a mut Renderer),
}

pub struct App {
  imgui: imgui::Context,
  events: EventLoop<()>,
  pub display: Display,
  platform: WinitPlatform,
  renderer: Renderer,
  font_size: f32,
}

pub struct AppBuilder {
  size: LogicalSize<f32>,
  title: String,
}

impl AppBuilder {
  pub fn new() -> Self {
    Self {
      size: LogicalSize::new(0.0, 0.0),
      title: "".to_owned(),
    }
  }

  pub fn size(mut self, width: f32, height: f32) -> Self {
    self.size = LogicalSize::new(width, height);
    self
  }

  pub fn title(mut self, title: &str) -> Self {
    self.title = title.to_owned();
    self
  }

  pub fn build(self) -> App {
    App::new(self)
  }
}

impl App {
  pub fn new(builder: AppBuilder) -> Self {
    let events = EventLoop::new();
    let context = ContextBuilder::new().with_vsync(false);
    let window = WindowBuilder::new()
      .with_title(builder.title)
      .with_inner_size(builder.size);

    let display = Display::new(window, context, &events) //
      .expect("Failed to initialize display");

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);
    {
      let window = display.gl_window();
      let window = window.window();

      platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
    }

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
      FontSource::DefaultFontData {
        config: Some(FontConfig {
          size_pixels: font_size,
          ..FontConfig::default()
        }),
      },
      FontSource::TtfData {
        data: include_bytes!("../../assets/JetBrainsMono-Bold.ttf"),
        size_pixels: font_size,
        config: Some(FontConfig {
          rasterizer_multiply: 1.75,
          glyph_ranges: FontGlyphRanges::japanese(),
          ..FontConfig::default()
        }),
      },
    ]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let renderer =
      Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

    Self {
      events,
      display,
      imgui,
      platform,
      renderer,
      font_size,
    }
  }

  pub fn run<F: 'static>(self, mut callback: F)
  where
    F: FnMut(AppEvent<'_>) -> bool,
  {
    let Self {
      events,
      display,
      mut imgui,
      mut platform,
      mut renderer,
      ..
    } = self;

    let mut last_frame = Instant::now();

    events.run(move |event, _, flow| match event {
      Event::NewEvents(_) => {
        let now = Instant::now();
        imgui.io_mut().update_delta_time(now - last_frame);
        last_frame = now;

        if !callback(AppEvent::Update(&display, &mut renderer)) {
          *flow = ControlFlow::Exit;
        }
      }
      Event::MainEventsCleared => {
        let gl_window = display.gl_window();
        platform
          .prepare_frame(imgui.io_mut(), &gl_window.window())
          .expect("Failed to prepare frame");
        gl_window.window().request_redraw();
      }
      Event::RedrawRequested(_) => {
        let mut ui = imgui.frame();

        let gl_window = display.gl_window();
        let mut target = display.draw();
        if !callback(AppEvent::Render(&ui)) {
          *flow = ControlFlow::Exit;
        }

        target.clear_color_srgb(0.1, 0.1, 0.1, 1.0);
        platform.prepare_render(&ui, gl_window.window());
        let draw_data = ui.render();
        renderer
          .render(&mut target, draw_data)
          .expect("Rendering failed");
        target.finish().expect("Failed to swap buffers");
      }
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => *flow = ControlFlow::Exit,
      event => {
        let gl_window = display.gl_window();
        platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
      }
    })
  }
}
