pub use frame_rate::FrameRate;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct TimecodeFrames {
  frame_rate: FrameRate,
  number_of_frames: u8,
  drop_frame: bool,
  color_frame: bool,
}

impl TimecodeFrames {
  pub fn new(
    frame_rate: FrameRate,
    number_of_frames: u8,
    drop_frame: bool,
    color_frame: bool,
  ) -> Self {
    Self {
      frame_rate,
      number_of_frames,
      drop_frame,
      color_frame,
    }
  }

  pub fn frame_rate(&self) -> FrameRate {
    self.frame_rate
  }

  pub fn number_of_frames(&self) -> u8 {
    self.number_of_frames
  }

  pub fn drop_frame(&self) -> bool {
    self.drop_frame
  }

  pub fn color_frame(&self) -> bool {
    self.color_frame
  }
}

impl ToString for TimecodeFrames {
  fn to_string(&self) -> String {
    let separator = if self.drop_frame() { ';' } else { ':' };

    format!("{}{:02}", separator, self.number_of_frames())
  }
}
