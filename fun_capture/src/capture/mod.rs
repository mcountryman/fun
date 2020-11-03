use crate::display::Display;
use std::fmt::Debug;

#[cfg(target_os = "macos")]
pub mod quartz;
#[cfg(target_os = "windows")]
pub mod windows_dc;

#[derive(Debug)]
pub enum Frame<T: Debug> {
  Ready(T),
  Blocking,
}

pub trait Capture<T: Debug> {
  fn frame(&mut self) -> Frame<T>;
}

pub struct CaptureOpts {
  pub(crate) cursor: bool,
  pub(crate) display: Display,
  pub(crate) frame_rate: u32,
  pub(crate) frame_queue: u8,
}

impl CaptureOpts {
  pub fn new(display: Display) -> Self {
    Self {
      cursor: true,
      display,
      frame_rate: 0,
      frame_queue: 3,
    }
  }

  pub fn cursor(&mut self, cursor: bool) -> &mut Self {
    self.cursor = cursor;
    self
  }

  pub fn frame_rate(&mut self, frame_rate: u32) -> &mut Self {
    self.frame_rate = frame_rate;
    self
  }

  pub fn frame_queue(&mut self, frame_queue: u8) -> &mut Self {
    self.frame_queue = frame_queue;
    self
  }
}
