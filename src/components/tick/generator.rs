use crate::components::grid::GridViewportHandle;
use crate::components::tick::{GridTick, GridTickType};
use crate::time::Duration;
use gpui::{Pixels, px};

#[derive(Clone)]
pub struct GridTickGenerator {
    generators: Vec<TimeSpecGridGenerator>,
}

impl GridTickGenerator {
    pub fn new(min_primary_width: Pixels) -> Self {
        let generator = move |s: Duration, p: Duration| -> TimeSpecGridGenerator {
            TimeSpecGridGenerator {
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

    pub fn generate<V: GridViewportHandle>(&self, viewport: &V) -> Vec<GridTick> {
        for generator in &self.generators {
            if generator.supports(viewport) {
                return generator.generate(viewport);
            }
        }
        Vec::new()
    }
}

#[derive(Clone)]
struct TimeSpecGridGenerator {
    primary_duration: Duration,
    secondary_duration: Duration,
    min_primary_width: Pixels,
}

impl TimeSpecGridGenerator {
    fn supports<V: GridViewportHandle>(&self, viewport: &V) -> bool {
        viewport.duration_to_track_offset(self.primary_duration) >= self.min_primary_width
    }

    fn generate<V: GridViewportHandle>(&self, viewport: &V) -> Vec<GridTick> {
        let mut ticks: Vec<GridTick> = Vec::new();

        // padding produces time truncation, so it's required to additional conversion
        let start_time = viewport.scroll_offset_to_time(px(0.0));
        let start_px = viewport.time_to_scroll_offset(start_time);
        let end_px = start_px + viewport.viewport_size().width;

        let mut primary = start_time.truncate(self.primary_duration);
        loop {
            let offset_x = viewport.time_to_scroll_offset(primary);
            if offset_x >= end_px {
                break;
            }
            let label = Some(primary.to_string().into());
            ticks.push(GridTick {
                tick_type: GridTickType::PRIMARY,
                offset_x,
                label,
            });

            let next_primary = primary + self.primary_duration;
            let mut secondary = primary + self.secondary_duration;
            while secondary < next_primary {
                let offset_x = viewport.time_to_scroll_offset(secondary);
                if offset_x >= end_px {
                    break;
                }
                ticks.push(GridTick {
                    tick_type: GridTickType::SECONDARY,
                    offset_x,
                    label: None,
                });
                secondary += self.secondary_duration;
            }
            primary = next_primary;
        }

        ticks
    }
}
