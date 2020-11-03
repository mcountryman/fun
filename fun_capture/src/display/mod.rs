use std::io::Result;
use std::ops::Deref;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;

mod imp {
  #[cfg(target_os = "macos")]
  pub use super::macos::*;
  #[cfg(target_os = "windows")]
  pub use super::windows::*;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DisplayKind {
  Primary,
  Standard,
}

pub struct Display(imp::Display);

pub fn get_primary() -> Result<Display> {
  imp::get_primary().map(Display)
}

pub fn get_displays() -> Result<Vec<Display>> {
  imp::get_displays()
    .map(|inner| inner.into_iter().map(Display).collect())
}

impl Deref for Display {
  type Target = imp::Display;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
