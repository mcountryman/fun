use std::time::Instant;

use ::image::RgbaImage;
use glutin_window::GlutinWindow;
use graphics::*;
use opengl_graphics::{GlGraphics, GlyphCache, OpenGL, Texture};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateEvent};
use piston::window::WindowSettings;
use piston::Window;
use texture::TextureSettings;

use fun_capture::capture::quartz::QuartzCapture;
use fun_capture::capture::{Capture, CaptureOpts, Frame};
use fun_capture::display::get_primary;

fn main() {
  let mut window: GlutinWindow = WindowSettings::new("quartz", [1400, 900])
    .graphics_api(OpenGL::V4_1)
    .resizable(true)
    .exit_on_esc(true)
    .build()
    .unwrap();

  let display = get_primary().unwrap();
  let opts = CaptureOpts::new(display);
  let mut capture = QuartzCapture::new(opts).unwrap();

  let mut gl = GlGraphics::new(OpenGL::V4_1);
  let mut events = Events::new(EventSettings::new());
  let mut font =
    GlyphCache::new("assets/JetBrainsMono-Bold.ttf", (), TextureSettings::new()).unwrap();

  let display = get_primary().unwrap();
  let mut texture = Texture::new(69, display.width(), display.height());
  let mut last = Instant::now();

  while let Some(event) = events.next(&mut window) {
    if event.update_args().is_some() {
      if let Frame::Ready(frame) = capture.frame() {
        let frame =
          &RgbaImage::from_raw(display.width(), display.height(), frame.to_vec())
            .unwrap();

        texture = Texture::from_image(&frame, &TextureSettings::new());
      }
    }

    if let Some(args) = event.render_args() {
      gl.draw(args.viewport(), |ctx, gl| {
        clear([0.2, 0.2, 0.2, 1.0], gl);

        let window_width = window.size().width;
        let window_height = window.size().height;
        let display_width = display.width() as f64;
        let display_height = display.height() as f64;
        let ratio = (window_width / display_width).min(window_height / display_height);
        let width = (display_width * ratio) / display_width;
        let height = (display_height * ratio) / display_height;

        image(&texture, ctx.transform.scale(width, height), gl);

        text(
          [1.0, 0.0, 0.0, 1.0],
          14,
          &format!("fps: {}", 1.0 / last.elapsed().as_secs_f32()),
          &mut font,
          ctx.transform.trans(300.0, 20.0),
          gl,
        )
        .unwrap();

        last = Instant::now();
      });
    }
  }
}
