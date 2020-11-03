use std::io::{Error, Result};
use std::vec::IntoIter;

use crate::display::DisplayKind;
use crate::ffi::macos::{
  CGDisplayBounds, CGDisplayIsMain, CGDisplayPixelsHigh, CGDisplayPixelsWide, CGError,
  CGGetOnlineDisplayList, CGMainDisplayID,
};

#[derive(Copy, Clone, Debug)]
pub struct Display(u32);

impl Display {
  pub fn handle(&self) -> u32 {
    self.0
  }

  pub fn x(&self) -> i32 {
    unsafe { CGDisplayBounds(self.0) }.origin.x as i32
  }

  pub fn y(&self) -> i32 {
    unsafe { CGDisplayBounds(self.0) }.origin.y as i32
  }

  pub fn width(&self) -> u32 {
    unsafe { CGDisplayPixelsWide(self.0) as u32 }
  }

  pub fn height(&self) -> u32 {
    unsafe { CGDisplayPixelsHigh(self.0) as u32 }
  }

  pub fn kind(&self) -> DisplayKind {
    let is_main = unsafe { CGDisplayIsMain(self.0) };
    if is_main == 1 {
      DisplayKind::Primary
    } else {
      DisplayKind::Standard
    }
  }
}

pub struct Displays(IntoIter<u32>);

pub fn get_primary() -> Result<Display> {
  Ok(Display(unsafe { CGMainDisplayID() }))
}

pub fn get_displays() -> Result<Displays> {
  let mut length = 0;
  let mut displays = Vec::with_capacity(16);
  let error = unsafe {
    CGGetOnlineDisplayList(
      displays.capacity() as u32,
      displays.as_mut_ptr(),
      &mut length,
    )
  };

  if error != CGError::Success {
    return Err(Error::from_raw_os_error(error as i32));
  }

  unsafe {
    displays.set_len(length as usize);
  }

  Ok(Displays(displays.into_iter()))
}

impl Iterator for Displays {
  type Item = Display;

  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().map(Display)
  }
}

#[cfg(test)]
mod tests {
  use super::DisplayKind;
  use super::{get_displays, get_primary, Display};

  #[test]
  fn test_get_primary() {
    let display = get_primary().unwrap();

    assert_eq!(display.x(), 0);
    assert_eq!(display.y(), 0);
    assert!(display.width() > 0);
    assert!(display.height() > 0);
    assert_eq!(display.kind(), DisplayKind::Primary);
  }

  #[test]
  fn test_get_displays() {
    let displays: Vec<Display> = get_displays().unwrap().collect();
    assert!(!displays.is_empty());
    for display in displays {
      if display.kind() == DisplayKind::Primary {
        assert_eq!(display.x(), 0);
        assert_eq!(display.y(), 0);
        assert!(display.width() > 0);
        assert!(display.height() > 0);
      } else {
        assert!(display.width() > 0);
        assert!(display.height() > 0);
      }
    }
  }
}
