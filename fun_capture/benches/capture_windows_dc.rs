// #[cfg(target_os = "windows")]
use criterion::{criterion_group, criterion_main, Criterion};

use fun_capture::capture::windows_dc::DisplayContextCapture;
use fun_capture::capture::Capture;
use fun_capture::display::Display;
use std::cell::RefCell;
use std::rc::Rc;

pub fn benchmark(c: &mut Criterion) {
  c.bench_function("windows_dc", |b| {
    let displays = Display::all();
    let display = displays.first().unwrap();
    let capture = DisplayContextCapture::new(&display);
    let mut capture = RefCell::new(capture);

    b.iter(move || {
      capture.get_mut().frame();
    });
  });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
