use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ptr::null_mut;

use winapi::shared::windef::{HBITMAP__, HDC__};
use winapi::um::wingdi::{
  BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject,
  BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, SRCCOPY,
};
use winapi::um::winuser::{GetDC, ReleaseDC};

use crate::capture::{Capture, CaptureOpts, Frame};
use crate::display::Display;
use std::io::Error;
use winapi::_core::ptr::null;

pub struct DisplayContextCapture {
  x: i32,
  y: i32,
  width: u32,
  height: u32,

  hdc: *mut HDC__,
  bmp: *mut HBITMAP__,
  bmp_old: *mut c_void,
}

impl DisplayContextCapture {
  pub fn new(opts: CaptureOpts) -> Self {
    unsafe {
      let hdc = CreateCompatibleDC(null_mut());
      let bmp = Self::create_bitmap(&opts.display, hdc);
      let bmp_old = SelectObject(hdc, bmp as *mut c_void);

      Self {
        x: opts.display.x(),
        y: opts.display.y(),
        width: opts.display.width(),
        height: opts.display.height(),

        hdc,
        bmp,
        bmp_old,
      }
    }
  }

  unsafe fn create_bitmap(display: &Display, hdc: *mut HDC__) -> *mut HBITMAP__ {
    let mut info = BITMAPINFO::default();
    let mut info_header = &mut info.bmiHeader;

    info_header.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    info_header.biBitCount = 32;
    info_header.biWidth = display.width() as i32;
    info_header.biHeight = display.height() as i32;
    info_header.biPlanes = 1;

    let bitmap = CreateDIBSection(
      hdc,
      &info,
      DIB_RGB_COLORS,
      // holy unsafe batman
      null_mut(),
      null_mut(),
      0,
    );

    if bitmap.is_null() {
      panic!("Failed to create bitmap: {:?}", Error::last_os_error());
    }

    bitmap
  }
}

impl Capture for DisplayContextCapture {
  fn frame(&mut self) -> Frame {
    unsafe {
      let hdc = self.hdc;
      let hdc_target = GetDC(null_mut());

      BitBlt(
        hdc,
        0,
        0,
        self.width as i32,
        self.height as i32,
        hdc_target,
        self.x,
        self.y,
        SRCCOPY,
      );

      ReleaseDC(null_mut(), hdc_target);
    }

    Frame::Blocking
  }
}

impl Drop for DisplayContextCapture {
  fn drop(&mut self) {
    unsafe {
      SelectObject(self.hdc, self.bmp_old);
      DeleteDC(self.hdc);
      DeleteObject(self.bmp as *mut c_void);
    }
  }
}
