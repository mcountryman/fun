use std::cell::RefCell;
use std::ffi::c_void;
use std::io::{Error, Result};
use std::ptr::null_mut;

use block::{Block, ConcreteBlock};

use crate::capture::Frame;
use crate::capture::{Capture, CaptureOpts};
use crate::ffi::macos::{
  cfbool, dispatch_queue_create, dispatch_release, kCFTypeDictionaryKeyCallBacks,
  kCFTypeDictionaryValueCallBacks, kCGDisplayStreamMinimumFrameTime,
  kCGDisplayStreamPreserveAspectRatio, kCGDisplayStreamQueueDepth,
  kCGDisplayStreamShowCursor, CFDictionaryCreate, CFNumberCreate, CFNumberType, CFRetain,
  CGDisplayStreamFrameStatus, DispatchQueue, IOSurfaceDecrementUseCount,
  IOSurfaceGetAllocSize, IOSurfaceGetBaseAddress, IOSurfaceIncrementUseCount,
  IOSurfaceLock, IOSurfaceRef, IOSurfaceUnlock, SURFACE_LOCK_READ_ONLY,
};
use crate::ffi::macos::{CFDictionaryRef, FrameAvailableHandler};
use crate::ffi::macos::{
  CFRelease, CGDisplayStreamCreateWithDispatchQueue, CGDisplayStreamRef,
  CGDisplayStreamStop, PixelFormat,
};
use std::borrow::Cow;
use std::ops::Deref;
use std::slice::from_raw_parts;

pub struct QuartzCapture {
  queue: DispatchQueue,
  stream: CGDisplayStreamRef,
  surface: RefCell<Option<IOSurfaceRef>>,
}

impl QuartzCapture {
  pub fn new(opts: CaptureOpts) -> Result<Self> {
    let surface = RefCell::<Option<IOSurfaceRef>>::new(None);
    let queue = unsafe {
      dispatch_queue_create(b"quadrupleslap.scrap\0".as_ptr() as *const _, null_mut())
    };

    let handler_surface = surface.clone();
    let handler: FrameAvailableHandler = ConcreteBlock::new(move |status, _, s, _| {
      println!("frame_status: {:#?}", status);

      if status == CGDisplayStreamFrameStatus::FrameComplete {
        handler_surface.replace(Some(s));
      }
    })
    .copy();

    let config = Self::build_config(&opts);
    let stream = unsafe {
      let display = opts.display.handle();
      let output_width = opts.display.width() as usize;
      let output_height = opts.display.height() as usize;

      CGDisplayStreamCreateWithDispatchQueue(
        display,
        output_width,
        output_height,
        PixelFormat::Argb8888,
        config,
        queue,
        &*handler as *const Block<_, _> as *const c_void,
      )
    };

    unsafe { CFRelease(config) };

    if stream.is_null() {
      return Err(Error::last_os_error());
    }

    Ok(Self {
      queue,
      stream,
      surface,
    })
  }

  fn build_config(opts: &CaptureOpts) -> CFDictionaryRef {
    unsafe {
      let throttle = CFNumberCreate(
        null_mut(),
        CFNumberType::Float64,
        &opts.frame_rate as *const _ as *const c_void,
      );

      let queue_length = CFNumberCreate(
        null_mut(),
        CFNumberType::Float64,
        &opts.frame_queue as *const _ as *const c_void,
      );

      let keys = [
        kCGDisplayStreamShowCursor,
        kCGDisplayStreamPreserveAspectRatio,
        kCGDisplayStreamMinimumFrameTime,
        kCGDisplayStreamQueueDepth,
      ];

      let values = [cfbool(opts.cursor), cfbool(false), throttle, queue_length];

      let config = CFDictionaryCreate(
        null_mut(),
        keys.as_ptr(),
        values.as_ptr(),
        4,
        &kCFTypeDictionaryKeyCallBacks,
        &kCFTypeDictionaryValueCallBacks,
      );

      CFRelease(throttle);
      CFRelease(queue_length);

      config
    }
  }
}

impl<'a> Capture<QuartzFrame<'a>> for QuartzCapture {
  fn frame(&mut self) -> Frame<QuartzFrame<'a>> {
    match self.surface.replace(None) {
      Some(surface) => Frame::Ready(QuartzFrame::new(surface)),
      None => Frame::Blocking,
    }
  }
}

impl Drop for QuartzCapture {
  fn drop(&mut self) {
    if self.stream.is_null() {
      return;
    }

    unsafe {
      let _ = CGDisplayStreamStop(self.stream);
      CFRelease(self.stream);
      dispatch_release(self.queue)
    }
  }
}

#[derive(Debug)]
pub struct QuartzFrame<'a> {
  inner: Cow<'a, [u8]>,
  surface: IOSurfaceRef,
}

impl QuartzFrame<'_> {
  pub fn new(surface: IOSurfaceRef) -> Self {
    let inner = unsafe {
      CFRetain(surface);
      IOSurfaceIncrementUseCount(surface);
      IOSurfaceLock(surface, SURFACE_LOCK_READ_ONLY, null_mut());

      from_raw_parts(
        IOSurfaceGetBaseAddress(surface) as *const u8,
        IOSurfaceGetAllocSize(surface),
      )
    };

    Self {
      inner: Cow::from(inner),
      surface,
    }
  }
}

impl<'a> Deref for QuartzFrame<'a> {
  type Target = Cow<'a, [u8]>;

  fn deref<'b>(&'b self) -> &'b Cow<'a, [u8]> {
    &self.inner
  }
}

impl Drop for QuartzFrame<'_> {
  fn drop(&mut self) {
    unsafe {
      IOSurfaceUnlock(self.surface, SURFACE_LOCK_READ_ONLY, null_mut());
      IOSurfaceDecrementUseCount(self.surface);
      CFRelease(self.surface);
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::capture::quartz::QuartzCapture;
  use crate::capture::{Capture, CaptureOpts};
  use crate::display::get_primary;

  #[test]
  fn test_capture() {
    let display = get_primary().unwrap();
    let opts = CaptureOpts::new(display);
    let mut capture = QuartzCapture::new(opts).unwrap();

    for _ in 0..10 {
      let frame = capture.frame();
      println!("{:#?}", frame);
      std::thread::sleep(std::time::Duration::from_secs(1));
    }

    panic!("test");
  }
}
