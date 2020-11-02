use std::borrow::Cow;

#[cfg(target_os = "windows")]
pub mod windows_dc;

pub enum Frame<'a> {
  Ready(Cow<'a, [u8]>),
  Blocking,
}

pub trait Capture {
  fn frame(&mut self) -> Frame;
}
