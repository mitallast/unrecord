use gpui::{AppContext, Context, Entity, Subscription};
use gpui_component::slider::{SliderEvent, SliderScale, SliderState};

pub struct TimelineState {
    pub(super) height_slider: Entity<SliderState>,
    pub(super) timescale_slider: Entity<SliderState>,
    pub(super) current_height: f32,
    pub(super) current_timescale: f32,
    _subscriptions: Vec<Subscription>,
}

impl TimelineState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let current_height = 44.0;
        let current_timescale = 100.0;

        let height_slider = cx.new(|_| {
            SliderState::new()
                .min(44.0)
                .max(800.0)
                .default_value(current_height)
                .step(1.0)
                .scale(SliderScale::Linear)
        });

        let timescale_slider = cx.new(|_| {
            SliderState::new()
                .min(1.0)
                .max(100.0)
                .default_value(current_timescale)
                .step(1.0)
                .scale(SliderScale::Linear)
        });

        let sub_h = cx.subscribe(&height_slider, |this, _, event, cx| match event {
            SliderEvent::Change(_) => {
                let h = this.height_slider.read(cx).value().start();
                this.current_height = h;
                cx.notify();
            }
        });
        let sub_ts = cx.subscribe(&timescale_slider, |this, _, event, cx| match event {
            SliderEvent::Change(_) => {
                let h = this.timescale_slider.read(cx).value().start();
                this.current_timescale = h;
                cx.notify();
            }
        });

        let _subscriptions = vec![sub_h, sub_ts];

        Self {
            height_slider,
            timescale_slider,
            current_height,
            current_timescale,
            _subscriptions,
        }
    }
}
