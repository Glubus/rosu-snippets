use eyre::Result;

#[derive(Clone)]
pub struct SnippetsTimingPoints {
    pub time_start: i32,
    pub time_end: i32,
    pub next_update: NextUpdate,
}

#[derive(Clone)]
pub enum NextUpdate {
    TimeStart,
    TimeEnd,
}

impl SnippetsTimingPoints {
    pub fn new() -> Self {
        Self {
            time_start: 0,
            time_end: 0,
            next_update: NextUpdate::TimeStart,
        }
    }

    pub fn set_next(&mut self, ig_time: i32) -> Result<(NextUpdate), eyre::Error> {

        match self.next_update {
            NextUpdate::TimeStart => {
                self.time_start = ig_time;
                self.next_update = NextUpdate::TimeEnd;
                Ok(NextUpdate::TimeEnd)
            }
            NextUpdate::TimeEnd => {
                self.time_end = ig_time;
                self.next_update = NextUpdate::TimeStart;
                Ok(NextUpdate::TimeStart)
            }
        }
    }
}