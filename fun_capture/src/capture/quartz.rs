use std::borrow::Cow;
use std::ffi::c_void;
use std::io::{Error, Result};
use std::ops::Deref;
use std::ptr::null_mut;
use std::slice::from_raw_parts;

use block::ConcreteBlock;

use crate::capture::Frame;
use crate::capture::{Capture, CaptureOpts};
use crate::ffi::macos::CFDictionaryRef;
use crate::ffi::macos::{
  cfbool, dispatch_queue_create, dispatch_release, kCFTypeDictionaryKeyCallBacks,
  kCFTypeDictionaryValueCallBacks, kCGDisplayStreamMinimumFrameTime,
  kCGDisplayStreamPreserveAspectRatio, kCGDisplayStreamQueueDepth,
  kCGDisplayStreamShowCursor, CFDictionaryCreate, CFNumberCreate, CFNumberType, CFRetain,
  CGDisplayStreamFrameStatus, CGDisplayStreamStart, CGError, DispatchQueue,
  IOSurfaceDecrementUseCount, IOSurfaceGetAllocSize, IOSurfaceGetBaseAddress,
  IOSurfaceIncrementUseCount, IOSurfaceLock, IOSurfaceRef, IOSurfaceUnlock,
  SURFACE_LOCK_READ_ONLY,
};
use crate::ffi::macos::{
  CFRelease, CGDisplayStreamCreateWithDispatchQueue, CGDisplayStreamRef,
  CGDisplayStreamStop, PixelFormat,
};
use crossbeam_channel::{bounded, Receiver};

pub struct QuartzCapture {
  rx: Receiver<IOSurfaceRef>,
  queue: DispatchQueue,
  stream: CGDisplayStreamRef,
}

impl QuartzCapture {
  pub fn new(opts: CaptureOpts) -> Result<Self> {
    let (tx, rx) = bounded::<IOSurfaceRef>(5);

    // Create dispatch queue
    let queue = unsafe {
      dispatch_queue_create(
        b"fun::fun_capture::QuartzCapture\0".as_ptr() as *const _,
        null_mut(),
      )
    };

    // Create ObjC callback `block`
    let handler = ConcreteBlock::new(move |status, _, s, _| {
      if status == CGDisplayStreamFrameStatus::FrameComplete {
        tx.send(s).unwrap();
      }
    })
    .copy();

    // Create config dictionary
    let config = Self::build_config(&opts);
    let stream = unsafe {
      let display = opts.display.handle();
      let output_width = opts.display.width() as usize;
      let output_height = opts.display.height() as usize;

      // Create display stream
      CGDisplayStreamCreateWithDispatchQueue(
        display,
        output_width,
        output_height,
        PixelFormat::Argb8888,
        config,
        queue,
        &handler,
      )
    };

    unsafe { CFRelease(config) };

    if stream.is_null() {
      return Err(Error::last_os_error());
    }

    let status = unsafe { CGDisplayStreamStart(stream) };
    if status != CGError::Success {
      return Err(Error::from_raw_os_error(status as i32));
    }

    Ok(Self { rx, queue, stream })
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

      let values = [
        cfbool(opts.cursor), //
        cfbool(false),
        throttle,
        queue_length,
      ];

      let config = CFDictionaryCreate(
        null_mut(),
        keys.as_ptr(),
        values.as_ptr(),
        2,
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
    match self.rx.try_recv() {
      Ok(surface) => Frame::Ready(QuartzFrame::new(surface)),
      Err(_) => Frame::Blocking,
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
  use crate::capture::{Capture, CaptureOpts, Frame};
  use crate::display::get_primary;
  use std::time::Instant;

  #[test]
  fn test_capture() {
    let display = get_primary().unwrap();
    let opts = CaptureOpts::new(display);
    let mut capture = QuartzCapture::new(opts).unwrap();
    let instant = Instant::now();
    let mut size = 0;

    for _ in 0..1000 {
      match capture.frame() {
        Frame::Ready(frame) => size += frame.len(),
        Frame::Blocking => (),
      }
    }

    let mbit = size as f32 / 1000000.0;
    let seconds = instant.elapsed().as_secs_f32();

    println!("Mbit   = {}", mbit);
    println!("Mbit/s = {}", mbit / seconds);
  }
}
