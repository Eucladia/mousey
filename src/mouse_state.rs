use std::fmt::Write;
use std::fs::File;
use std::io::Write as IoWrite;
use std::time::Instant;

// The maximum number of points per segment
const MAX_MOUSE_POINTS: usize = 125;
// The minimum number of points per segment
const MIN_MOUSE_POINTS: usize = 50;
// Time difference in milliseconds to force create a new segment
const SEGMENT_TIME: usize = 5000;

pub struct State {
  pub file: File,
  pub tracking: bool,
  pub coordinates: Vec<Coordinate>,
}

pub struct Coordinate {
  pub point: (u32, u32),
  pub moved_at: Instant,
}

impl State {
  pub fn new(file: File) -> Self {
    State {
      file,
      tracking: true,
      coordinates: Vec::with_capacity(MAX_MOUSE_POINTS),
    }
  }

  pub fn add_point(&mut self, point: (i32, i32), time: Instant) {
    let last_coord = self.coordinates.last();
    let last_point = last_coord.map(|x| x.point);
    let (x, y) = (point.0 as u32, point.1 as u32);

    // We received a spurious message - https://devblogs.microsoft.com/oldnewthing/20031001-00/?p=42343
    if Some((x, y)) == last_point {
      return;
    }

    let num_coordinates = self.coordinates.iter().len();

    let has_min = num_coordinates >= MIN_MOUSE_POINTS;
    let capped_points = num_coordinates >= MAX_MOUSE_POINTS;
    let new_segment = last_coord.map_or(false, |x| {
      time.duration_since(x.moved_at).as_millis() as usize >= SEGMENT_TIME
    });

    if capped_points || (new_segment && has_min) {
      let mut points = self.coordinates.iter().enumerate().fold(
        // Assume each coordinate is going to be 4 digits
        String::with_capacity(num_coordinates * 2 * 4 + num_coordinates - 1),
        |mut acc, (idx, x)| {
          let _ = write!(acc, "{}_{}", x.point.0, x.point.1);

          if idx + 1 != num_coordinates {
            acc.push(',');
          }

          acc
        },
      );

      self.coordinates.clear();

      let _ = writeln!(points,);
      let _ = self.file.write(points.as_bytes());
      let _ = self.file.flush();
    }

    self.coordinates.push(Coordinate::new(x, y, time));
  }
}

impl Coordinate {
  pub const fn new(x: u32, y: u32, time: Instant) -> Self {
    Coordinate {
      point: (x, y),
      moved_at: time,
    }
  }
}
