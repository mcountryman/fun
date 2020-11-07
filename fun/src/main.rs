use crate::support::{App, AppBuilder, AppEvent};

use crossbeam_channel::{bounded, unbounded};
use fun_capture::capture::quartz::QuartzCapture;
use fun_capture::capture::{Capture, CaptureOpts, Frame};
use fun_capture::display::{get_primary, Display};
use glium::texture::Texture2d;
use glium::texture::{CompressedSrgbTexture2d, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerBehavior};
use glium::GlObject;
use imgui::*;
use imgui_glium_renderer::Texture;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::thread::spawn;
use std::time::Instant;

mod support;

fn main() {
  let mut frame = 0;
  let mut frames = [0.0; 10_000];
  let mut frame_last = Instant::now();
  let mut frame_max = 0.0;

  let app = AppBuilder::new()
    .size(500.0, 400.0) //
    .title("fun")
    .build();

  let mut target_id = None;

  let display = get_primary().unwrap();
  let (tx, rx) = unbounded();

  spawn(move || loop {
    let display = get_primary().unwrap();
    let mut capture = QuartzCapture::new(CaptureOpts::new(display)).unwrap();
    let display = get_primary().unwrap();

    match capture.frame() {
      Frame::Ready(frame) => {
        let image =
          RawImage2d::from_raw_rgba(frame.to_vec(), (display.width(), display.height()));

        tx.send(image).ok();
      }
      Frame::Blocking => (),
    }
  });

  app.run(move |event| {
    match event {
      AppEvent::Update(gl, renderer) => {
        if let Ok(image) = rx.try_recv() {
          let texture = Texture2d::new(gl, image).unwrap();
          let texture = Texture {
            texture: Rc::new(texture),
            sampler: SamplerBehavior {
              minify_filter: MinifySamplerFilter::Linear,
              magnify_filter: MagnifySamplerFilter::Linear,
              ..Default::default()
            },
          };

          if let Some(target_id) = target_id {
            renderer.textures().replace(target_id, texture);
          } else {
            target_id = Some(renderer.textures().insert(texture));
          }
        }
      }
      AppEvent::Render(ui) => {
        Window::new(im_str!("dekstop"))
          .size([500.0, 400.0], Condition::Once)
          .position([0.0, 0.0], Condition::Once)
          .build(ui, || {
            //

            if let Some(target_id) = target_id {
              Image::new(target_id, [display.width() as f32, display.height() as f32])
                .build(ui)
            };
          });

        Window::new(im_str!("stats"))
          .size([300.0, 150.0], Condition::Always)
          .position([10.0, 10.0], Condition::Always)
          .resizable(false)
          .build(ui, || {
            let fps = 1.0 / frame_last.elapsed().as_secs_f32();

            PlotLines::new(ui, im_str!(" "), &frames)
              .scale_min(0.0)
              .scale_max(frame_max)
              .graph_size([280.0, 110.0])
              .overlay_text(im_str!("fps"))
              .build();

            frames[frame] = fps;
            frame_max = frame_max.max(fps);
            frame_last = Instant::now();
            frame = if frame < frames.len() - 1 {
              frame + 1
            } else {
              frame_max = 0.0;
              0
            };
          });
      }
    };

    true
  });
}
