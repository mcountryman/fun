#[cfg(target_os = "windows")]
mod windows;

mod imp {
  #[cfg(target_os = "windows")]
  pub use super::windows::*;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DisplayKind {
  Primary,
  Secondary,
}

pub struct Display(imp::Display);

impl Display {
  pub fn primary() -> Option<Display> {
    imp::Display::all()
      .into_iter()
      .map(Display)
      .find(|display| display.kind() == DisplayKind::Primary)
  }

  pub fn all() -> Vec<Display> {
    imp::Display::all().into_iter().map(Display).collect()
  }

  pub fn x(&self) -> i32 {
    self.0.x
  }

  pub fn y(&self) -> i32 {
    self.0.y
  }

  pub fn width(&self) -> u32 {
    self.0.width
  }

  pub fn height(&self) -> u32 {
    self.0.height
  }

  pub fn kind(&self) -> DisplayKind {
    self.0.kind
  }
}
