use criterion::{criterion_group, criterion_main, Criterion};

use fun_capture::capture::{Capture, CaptureOpts, Frame};
use fun_capture::display::get_primary;
use std::sync::atomic::{AtomicU32, Ordering};

pub fn benchmark(c: &mut Criterion) {
  #[cfg(target_os = "macos")]
  c.bench_function("quartz", |b| {
    use fun_capture::capture::quartz::QuartzCapture;

    let display = get_primary().unwrap();
    let mut capture = QuartzCapture::new(CaptureOpts::new(display)).unwrap();
    let bad_frames = AtomicU32::new(0);

    b.iter(move || {
      match capture.frame() {
        Frame::Ready(_) => 0,
        Frame::Blocking => bad_frames.fetch_add(1, Ordering::SeqCst),
      };

      if bad_frames.load(Ordering::SeqCst) > 10 {
        panic!("Too many empty frames");
      }
    });
  });

  #[cfg(target_os = "windows")]
  c.bench_function("windows_dc", |b| {
    use fun_capture::capture::windows_dc::DisplayContextCapture;

    let display = get_primary().unwrap();
    let mut capture = DisplayContextCapture::new(CaptureOpts::new(display));

    b.iter(move || {
      capture.frame();
    });
  });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
