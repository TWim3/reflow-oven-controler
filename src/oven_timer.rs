use embassy_time::Instant;

pub struct OvenTimer {
    instant: Option<Instant>,
}

impl OvenTimer {
    pub fn new() -> Self {
        Self { instant: None }
    }

    pub fn elapsed_secs(&mut self) -> u64 {
        match self.instant {
            Some(start_instant) => start_instant.elapsed().as_secs(),
            None => {
                self.instant = Some(Instant::now());
                self.instant.unwrap().elapsed().as_secs()
            }
        }
    }

    pub fn clear(&mut self) {
        self.instant = None;
    }
}
