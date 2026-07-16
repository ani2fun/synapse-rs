//! The transport bar (oracle: `TransportBar`): first/prev/play-pause/next/last, a scrubber,
//! and the step label — over the ONE `Playback` stepper. The single interval timer lives
//! here: started on play, cleared on pause/unmount; `tick` self-stops at the end.

use leptos::prelude::*;
use synapse_shared::viz::playback::State;

const STEP_DELAY_MS: u32 = 900;

#[component]
pub fn TransportBar(state: RwSignal<State>) -> impl IntoView {
    let timer: StoredValue<Option<gloo_timers::callback::Interval>, LocalStorage> =
        StoredValue::new_local(None);

    // One timer, driven by `playing`: an always-on tick is harmless (paused = no-op), but
    // clearing on pause keeps the browser quiet.
    Effect::new(move |_| {
        let playing = state.get().playing;
        if playing && timer.read_value().is_none() {
            timer.set_value(Some(gloo_timers::callback::Interval::new(
                STEP_DELAY_MS,
                move || {
                    state.update(|s| *s = s.tick());
                },
            )));
        } else if !playing {
            timer.set_value(None); // dropping the Interval cancels it
        }
    });
    on_cleanup(move || timer.set_value(None));

    view! {
        <div class="transport">
            <button class="transport__btn" title="First step"
                    on:click=move |_| state.update(|s| *s = s.jump_to(0))>"⏮"</button>
            <button class="transport__btn" title="Previous step"
                    on:click=move |_| state.update(|s| *s = s.previous())>"‹"</button>
            <button class="transport__btn transport__btn--play"
                    title="Play / pause"
                    on:click=move |_| state.update(|s| *s = s.toggle_play())>
                {move || if state.get().playing { "⏸" } else { "▶" }}
            </button>
            <button class="transport__btn" title="Next step"
                    on:click=move |_| state.update(|s| *s = s.next())>"›"</button>
            <button class="transport__btn" title="Last step"
                    on:click=move |_| state.update(|s| *s = s.jump_to(i64::MAX))>"⏭"</button>
            <input
                class="transport__scrubber"
                type="range"
                min="0"
                max=move || (state.get().count.saturating_sub(1)).to_string()
                prop:value=move || state.get().index.to_string()
                on:input=move |event| {
                    if let Ok(i) = event_target_value(&event).parse::<i64>() {
                        state.update(|s| *s = s.jump_to(i));
                    }
                }
            />
            <span class="transport__label">
                {move || format!("{} / {}", state.get().index + 1, state.get().count)}
            </span>
        </div>
    }
}
