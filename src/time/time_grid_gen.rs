use crate::time::{Duration, GridTick, GridTickType, SampleRate, TimeCode};
use gpui::Pixels;
use log::info;

pub struct GridTickGenerator {
    generators: Vec<SpecGridTickGenerator>,
}

impl GridTickGenerator {
    pub fn new(sample_rate: SampleRate, min_primary_width: Pixels) -> Self {
        let generator = move |s: Duration, p: Duration| -> SpecGridTickGenerator {
            SpecGridTickGenerator {
                sample_rate,
                min_primary_width,
                secondary_duration: s,
                primary_duration: p,
            }
        };

        Self {
            generators: vec![
                generator(Duration::Millis(1), Duration::Millis(1)),
                generator(Duration::Millis(1), Duration::Millis(2)),
                generator(Duration::Millis(1), Duration::Millis(5)),
                //
                generator(Duration::Millis(1), Duration::Millis(10)),
                generator(Duration::Millis(2), Duration::Millis(20)),
                generator(Duration::Millis(5), Duration::Millis(50)),
                //
                generator(Duration::Millis(10), Duration::Millis(100)),
                generator(Duration::Millis(20), Duration::Millis(200)),
                generator(Duration::Millis(50), Duration::Millis(500)),
                //
                generator(Duration::Millis(100), Duration::Seconds(1)),
                generator(Duration::Millis(200), Duration::Seconds(2)),
                generator(Duration::Millis(500), Duration::Seconds(5)),
                generator(Duration::Seconds(1), Duration::Seconds(10)),
                generator(Duration::Seconds(2), Duration::Seconds(20)),
                generator(Duration::Seconds(3), Duration::Seconds(30)),
                //
                generator(Duration::Seconds(10), Duration::Minutes(1)),
                generator(Duration::Seconds(20), Duration::Minutes(2)),
                generator(Duration::Seconds(30), Duration::Minutes(3)),
                //
                generator(Duration::Minutes(1), Duration::Minutes(10)),
                generator(Duration::Minutes(2), Duration::Minutes(20)),
                generator(Duration::Minutes(3), Duration::Minutes(30)),
                //
                generator(Duration::Minutes(10), Duration::Hours(1)),
                generator(Duration::Minutes(20), Duration::Hours(2)),
                generator(Duration::Minutes(30), Duration::Hours(3)),
                //
                generator(Duration::Hours(1), Duration::Hours(10)),
                generator(Duration::Hours(2), Duration::Hours(20)),
                generator(Duration::Hours(3), Duration::Hours(30)),
            ],
        }
    }

    pub fn generate(
        &self,
        frames_per_px: f32,
        start_frame: usize,
        viewport_width: Pixels,
    ) -> Vec<GridTick> {
        for generator in &self.generators {
            if generator.supports(frames_per_px) {
                return generator.generate(frames_per_px, start_frame, viewport_width);
            }
        }
        Vec::new()
    }
}

struct SpecGridTickGenerator {
    sample_rate: SampleRate,
    primary_duration: Duration,
    secondary_duration: Duration,
    min_primary_width: Pixels,
}

impl SpecGridTickGenerator {
    fn supports(&self, frames_per_px: f32) -> bool {
        let sample_rate: f32 = self.sample_rate.into();
        let seconds_per_px = frames_per_px / sample_rate;
        let primary_width = self.to_pixels(seconds_per_px, self.primary_duration);
        primary_width >= self.min_primary_width
    }

    fn generate(
        &self,
        frames_per_px: f32,
        start_frame: usize,
        viewport_width: Pixels,
    ) -> Vec<GridTick> {
        info!(
            "generate {:?} {:?}",
            self.primary_duration, self.secondary_duration
        );

        let mut ticks: Vec<GridTick> = Vec::new();

        let sample_rate: f32 = self.sample_rate.into();
        let seconds_per_px = frames_per_px / sample_rate;

        let start_time = self.frames_to_time(start_frame);
        let mut primary = start_time.truncate(self.primary_duration);
        loop {
            if primary >= start_time {
                let offset_x = self.to_pixels(seconds_per_px, primary - start_time);
                if offset_x >= viewport_width {
                    break;
                }
                ticks.push(GridTick {
                    tick_type: GridTickType::PRIMARY,
                    time: primary,
                    offset_x,
                    label: Some(primary.to_string().into()),
                })
            }

            let next_primary = primary + self.primary_duration;
            let mut secondary = primary + self.secondary_duration;
            while secondary < next_primary {
                if secondary >= start_time {
                    let offset_x = self.to_pixels(seconds_per_px, secondary - start_time);
                    if offset_x >= viewport_width {
                        break;
                    }
                    ticks.push(GridTick {
                        tick_type: GridTickType::SECONDARY,
                        time: secondary,
                        offset_x,
                        label: None,
                    })
                }
                secondary += self.secondary_duration;
            }
            primary = next_primary;
        }

        ticks
    }

    fn frames_to_time(&self, frame: usize) -> TimeCode {
        let sample_rate: u64 = self.sample_rate.into();
        let millis = (frame as u64 * 1000) / sample_rate;
        TimeCode::from_millis(millis)
    }

    fn to_pixels(&self, seconds_per_px: f32, duration: Duration) -> Pixels {
        let seconds: f32 = duration.to_millis() as f32 / 1000f32;
        Pixels::from(seconds / seconds_per_px)
    }
}
