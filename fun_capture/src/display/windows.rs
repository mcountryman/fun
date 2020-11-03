use std::io::Error;
use std::mem::size_of;
use std::ptr::{null, null_mut};
use std::vec::IntoIter;

use winapi::shared::minwindef::{BOOL, LPARAM, TRUE};
use winapi::shared::windef::{HDC, HMONITOR, LPRECT};
use winapi::um::winuser::{
  EnumDisplayMonitors, GetMonitorInfoW, MONITORINFO, MONITORINFOF_PRIMARY,
};

use crate::display::DisplayKind;

#[derive(Copy, Clone, Debug)]
pub struct Display {
  x: i32,
  y: i32,
  width: u32,
  height: u32,
  kind: DisplayKind,
  handle: HMONITOR,
}

impl Display {
  fn new(info: MONITORINFO, handle: HMONITOR) -> Self {
    Self {
      x: info.rcWork.left,
      y: info.rcWork.top,
      width: (info.rcWork.right - info.rcWork.left) as u32,
      height: (info.rcWork.bottom - info.rcWork.top) as u32,

      handle,
      kind: if info.dwFlags == MONITORINFOF_PRIMARY {
        DisplayKind::Primary
      } else {
        DisplayKind::Standard
      },
    }
  }

  pub fn x(&self) -> i32 {
    self.x
  }

  pub fn y(&self) -> i32 {
    self.y
  }

  pub fn width(&self) -> u32 {
    self.width
  }

  pub fn height(&self) -> u32 {
    self.height
  }

  pub fn kind(&self) -> DisplayKind {
    self.kind
  }
}

pub fn get_displays() -> IntoIter<Display> {
  let mut displays = Vec::new();
  let data = &mut displays as *mut _;
  let result = unsafe {
    EnumDisplayMonitors(null_mut(), null(), Some(display_enumerator), data as LPARAM)
  };

  if result != TRUE {
    panic!("Failed to enumerate displays: {}", Error::last_os_error());
  }

  displays.into_iter()
}

unsafe extern "system" fn display_enumerator(
  handle: HMONITOR,
  _: HDC,
  _: LPRECT,
  data: LPARAM,
) -> BOOL {
  let monitors: &mut Vec<Display> = std::mem::transmute(data);
  let mut info = MONITORINFO::default();
  let info_ptr: *mut _ = &mut info;

  info.cbSize = size_of::<MONITORINFO>() as u32;

  let result = GetMonitorInfoW(handle, info_ptr);
  if result == TRUE {
    monitors.push(Display::new(info, handle));
  }

  TRUE
}

#[cfg(test)]
mod tests {
  use super::Display;
  use crate::display::get_displays;

  #[test]
  fn test_get_all() {
    assert!(get_displays().next().is_some());
  }
}
