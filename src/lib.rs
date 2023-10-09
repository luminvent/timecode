mod fraction;
mod timecode_frames;

pub use fraction::Fraction;
pub use frame_rate::FrameRate;
pub use timecode_frames::TimecodeFrames;

use num_traits::cast::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::{string::ToString, time::Duration};

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct Timecode {
  hours: u8,
  minutes: u8,
  seconds: u8,
  fraction: Fraction,
}

impl Timecode {
  pub fn new(hours: u8, minutes: u8, seconds: u8, fraction: Fraction) -> Self {
    Self {
      hours,
      minutes,
      seconds,
      fraction,
    }
  }
}

impl From<(u32, FrameRate)> for Timecode {
  fn from((number_of_frames, frame_rate): (u32, FrameRate)) -> Self {
    let fps: f32 = {
      let frame_rate: frame_rate::Ratio<u32> = frame_rate.into();
      frame_rate.to_f32().unwrap_or_default()
    };

    let hours = number_of_frames / (60 * 60 * fps as u32);
    let minutes = number_of_frames / (60 * fps as u32) - hours * 60;
    let seconds = (number_of_frames / fps as u32) - minutes * 60 - hours * 60 * 60;
    let number_of_frames = number_of_frames
      - seconds * (fps as u32)
      - minutes * 60 * (fps as u32)
      - hours * 60 * 60 * (fps as u32);
    let color_frame = false;
    let drop_frame = false;

    let fraction = Fraction::Frames(TimecodeFrames::new(
      frame_rate,
      number_of_frames as u8,
      color_frame,
      drop_frame,
    ));

    Timecode {
      hours: hours as u8,
      minutes: minutes as u8,
      seconds: seconds as u8,
      fraction,
    }
  }
}

impl Timecode {
  pub fn parse_smpte_331m(data: &[u8], frame_rate: FrameRate) -> Option<Self> {
    if data.len() != 17 {
      return None;
    }
    match data[0] {
      0x81 => Timecode::parse_smpte_12m(&data[1..], frame_rate),
      _ => None,
    }
  }

  pub fn parse_smpte_12m(data: &[u8], frame_rate: FrameRate) -> Option<Self> {
    if data.len() < 4 {
      return None;
    }

    let mask_tens_2 = 0b0011_0000;
    let mask_tens_3 = 0b0111_0000;
    let color_frame = (data[0] & 0b1000_0000) != 0;
    let drop_frame = (data[0] & 0b0100_0000) != 0;

    let fraction = Fraction::Frames(TimecodeFrames::new(
      frame_rate,
      Self::get_number(data[0], mask_tens_2),
      drop_frame,
      color_frame,
    ));
    let seconds = Self::get_number(data[1], mask_tens_3);
    let minutes = Self::get_number(data[2], mask_tens_3);
    let hours = Self::get_number(data[3], mask_tens_2);

    Some(Self {
      hours,
      minutes,
      seconds,
      fraction,
    })
  }

  // used in STL format (EBU Tech 3264)
  pub fn from_ebu_smpte_time_and_control(data: &[u8; 4], frame_rate: FrameRate) -> Self {
    let fraction = Fraction::Frames(TimecodeFrames::new(frame_rate, data[3], false, false));

    Self {
      hours: data[0],
      minutes: data[1],
      seconds: data[2],
      fraction,
    }
  }

  fn get_number(data: u8, mask_tens: u8) -> u8 {
    let mask_unit = 0x0F;

    let tens = (data & mask_tens) >> 4;
    let unit = data & mask_unit;

    (10 * tens) + unit
  }
}

impl From<Duration> for Timecode {
  fn from(duration: Duration) -> Self {
    let remaining = duration.as_secs();
    let hours = (remaining / 3600) as u8;
    let remaining = remaining % 3600;
    let minutes = (remaining / 60) as u8;
    let seconds = (remaining % 60) as u8;
    let fraction = Fraction::MilliSeconds(duration.subsec_millis() as u16);

    Timecode {
      hours,
      minutes,
      seconds,
      fraction,
    }
  }
}

impl Timecode {
  pub fn hours(&self) -> u8 {
    self.hours
  }

  pub fn minutes(&self) -> u8 {
    self.minutes
  }

  pub fn seconds(&self) -> u8 {
    self.seconds
  }

  pub fn fraction(&self) -> &Fraction {
    &self.fraction
  }
}

impl ToString for Timecode {
  fn to_string(&self) -> String {
    let fraction = self.fraction.to_string();

    format!(
      "{:02}:{:02}:{:02}{}",
      self.hours, self.minutes, self.seconds, fraction
    )
  }
}

impl From<&Timecode> for f64 {
  fn from(timecode: &Timecode) -> Self {
    (timecode.hours as f64) * 3600.0 + (timecode.minutes as f64) * 60.0 + (timecode.seconds as f64)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn timecode_from_frame() {
    let timecode = Timecode::from((24 * 60 * 60 * 10, FrameRate::_24_00));
    assert_eq!(timecode.hours(), 10);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 0);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_24_00, 0, false, false,))
    );

    let timecode = Timecode::from((24 * 60 * 60, FrameRate::_24_00));
    assert_eq!(timecode.hours(), 1);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 0);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_24_00, 0, false, false,))
    );

    let timecode = Timecode::from((24 * 60 * 60 - 1, FrameRate::_24_00));
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 59);
    assert_eq!(timecode.seconds(), 59);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_24_00, 23, false, false,))
    );

    let timecode = Timecode::from((24 * 60, FrameRate::_24_00));
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 1);
    assert_eq!(timecode.seconds(), 0);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_24_00, 0, false, false,))
    );

    let timecode = Timecode::from((24 * 60 - 1, FrameRate::_24_00));
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 59);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_24_00, 23, false, false,))
    );

    let timecode = Timecode::from((24, FrameRate::_24_00));
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 1);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_24_00, 0, false, false,))
    );

    let timecode = Timecode::from((23, FrameRate::_24_00));
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 0);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_24_00, 23, false, false,))
    );

    let timecode = Timecode::from((25, FrameRate::_25_00));
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 1);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_25_00, 0, false, false,))
    );

    let timecode = Timecode::from((24, FrameRate::_25_00));
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 0);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_25_00, 24, false, false,))
    );

    let timecode = Timecode::from_ebu_smpte_time_and_control(&[10, 0, 0, 0], FrameRate::_25_00);
    assert_eq!(timecode.hours(), 10);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 0);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_25_00, 0, false, false,))
    );

    let timecode = Timecode::from_ebu_smpte_time_and_control(&[0, 10, 0, 0], FrameRate::_25_00);
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 10);
    assert_eq!(timecode.seconds(), 0);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_25_00, 0, false, false,))
    );

    let timecode = Timecode::from_ebu_smpte_time_and_control(&[0, 0, 10, 0], FrameRate::_25_00);
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 10);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_25_00, 0, false, false,))
    );

    let timecode = Timecode::from_ebu_smpte_time_and_control(&[0, 0, 0, 10], FrameRate::_25_00);
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 0);
    assert_eq!(timecode.seconds(), 0);
    assert_eq!(
      timecode.fraction(),
      &Fraction::Frames(TimecodeFrames::new(FrameRate::_25_00, 10, false, false,))
    );
  }

  #[test]
  fn timecode_from_duration() {
    let timecode = Timecode::from(Duration::from_millis(2 * 60 * 1000 + 5 * 1000 + 66));
    assert_eq!(timecode.hours(), 0);
    assert_eq!(timecode.minutes(), 2);
    assert_eq!(timecode.seconds(), 5);
    assert_eq!(timecode.fraction(), &Fraction::MilliSeconds(66));
  }
}
