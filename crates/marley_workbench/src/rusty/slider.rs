//! A value along a track, for the Graph tab's Display and Forces (#657). Drag the thumb, press
//! the track, or with the focus on the thumb use the arrow keys, Page Up and Page Down (ten
//! steps) and Home and End. It is Ely GPUI Components' `Slider` (`src/forms/slider.rs` at
//! `2f8b2f6`): one thumb, horizontal, ported onto the fork's gpui and Zed's theme, in `f32`; its
//! notice is below. Zed's `ui` has no slider. A range or a step that makes no sense draws a slider
//! that does not move, where Ely asserts.

// The slider is ported from Ely GPUI Components, under this notice:
//
// MIT License
//
// Copyright (c) 2026 Ely GPUI Component contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::rc::Rc;

use gpui::{
    App, Bounds, Div, DragMoveEvent, ElementId, EmptyView, EntityId, FocusHandle, MouseButton,
    Orientation, Pixels, Point, Role, SharedString, Stateful, Window, canvas, px, relative,
};
use ui::prelude::*;

/// The rail's thickness and the thumb's size (Ely's tokens).
const RAIL: Pixels = px(4.);
const THUMB: Pixels = px(16.);

/// `value` as a fraction of `min..=max`.
fn fraction(value: f32, min: f32, max: f32) -> f32 {
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

/// The value at `fraction` of `min..=max`, on the `step` grid.
fn value_at(fraction: f32, min: f32, max: f32, step: f32) -> f32 {
    let raw = fraction.clamp(0.0, 1.0).mul_add(max - min, min);
    ((raw - min) / step)
        .round()
        .mul_add(step, min)
        .clamp(min, max)
}

/// Where a key moves `value`: arrows step, Page keys jump ten, Home and End go to the ends.
fn keyed(key: &str, value: f32, min: f32, max: f32, step: f32) -> Option<f32> {
    let next = match key {
        "left" | "down" => value - step,
        "right" | "up" => value + step,
        "pagedown" => step.mul_add(-10.0, value),
        "pageup" => step.mul_add(10.0, value),
        "home" => min,
        "end" => max,
        _ => return None,
    };
    Some(next.clamp(min, max))
}

/// How far along a track `at` is, from its left end; 0 for a track not yet laid out.
fn along(bounds: Bounds<Pixels>, at: Point<Pixels>) -> f32 {
    if bounds.size.width <= px(0.) {
        return 0.0;
    }
    (at.x - bounds.left()) / bounds.size.width
}

/// A drag of the thumb, owned by one slider so another's track ignores it.
struct Thumb {
    owner: EntityId,
}

type OnChange = Rc<dyn Fn(f32, &mut Window, &mut App)>;

/// A value along a track.
#[derive(IntoElement)]
pub(super) struct Slider {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    label: SharedString,
    on_change: Option<OnChange>,
}

impl Slider {
    /// A slider at `value`, from 0 to 100 a whole step at a time until told otherwise.
    pub(super) fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            label: SharedString::default(),
            on_change: None,
        }
    }

    pub(super) const fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub(super) const fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// What a screen reader calls it.
    pub(super) fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Called with a value on the step's grid that differs from the one shown by half a step or
    /// more.
    pub(super) fn on_change(
        mut self,
        handler: impl Fn(f32, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn movable(&self) -> bool {
        [self.value, self.min, self.max, self.step]
            .iter()
            .all(|number| number.is_finite())
            && self.min < self.max
            && self.step > 0.0
    }
}

/// A slider's numbers, which every part of it draws from.
#[derive(Clone, Copy)]
struct Track {
    value: f32,
    min: f32,
    max: f32,
    step: f32,
}

impl Track {
    /// The value on a grid point `fraction` of the way along.
    fn at(self, fraction: f32) -> f32 {
        value_at(fraction, self.min, self.max, self.step)
    }

    /// How far along the value is.
    fn fraction(self) -> f32 {
        fraction(self.value, self.min, self.max)
    }
}

/// The rail and, over it, the fill up to `at` of its length.
fn rail_and_fill(at: f32, cx: &App) -> [Div; 2] {
    let colors = cx.theme().colors();
    let line = |bar: Div| {
        bar.absolute()
            .top(relative(0.5))
            .mt(RAIL * -0.5)
            .h(RAIL)
            .rounded_full()
    };
    [
        line(div()).left_0().right_0().bg(colors.border),
        line(div()).left_0().w(relative(at)).bg(colors.text_accent),
    ]
}

impl Slider {
    /// How far along the thumb and the fill are drawn: at the start for a slider that cannot move.
    fn shown_at(&self, track: Track) -> f32 {
        if self.movable() {
            track.fraction()
        } else {
            0.0
        }
    }

    /// The thumb: the slider's accessible node, its focus, its drag and its keys.
    fn thumb(
        &self,
        track: Track,
        owner: EntityId,
        focus: &FocusHandle,
        commit: &OnChange,
        window: &Window,
        cx: &App,
    ) -> Stateful<Div> {
        let colors = cx.theme().colors();
        let commit = Rc::clone(commit);
        div()
            .id((self.id.clone(), "thumb"))
            .role(Role::Slider)
            .aria_label(self.label.clone())
            .aria_numeric_value(f64::from(track.value))
            .aria_min_numeric_value(f64::from(track.min))
            .aria_max_numeric_value(f64::from(track.max))
            .aria_numeric_value_step(f64::from(track.step))
            .aria_orientation(Orientation::Horizontal)
            .track_focus(focus)
            .absolute()
            .top_0()
            .left(relative(self.shown_at(track)))
            .ml(THUMB * -0.5)
            .size(THUMB)
            .rounded_full()
            .bg(colors.elevated_surface_background)
            .border_1()
            .border_color(if focus.is_focused(window) {
                colors.border_focused
            } else {
                colors.border
            })
            .when(self.movable(), |thumb| {
                thumb
                    .cursor_pointer()
                    .on_drag(Thumb { owner }, |_, _, _, cx| cx.new(|_| EmptyView))
                    .on_key_down(move |event, window, cx| {
                        let Track {
                            value,
                            min,
                            max,
                            step,
                        } = track;
                        if let Some(next) = keyed(&event.keystroke.key, value, min, max, step) {
                            cx.stop_propagation();
                            commit(next, window, cx);
                        }
                    })
            })
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let movable = self.movable();
        let track = Track {
            value: self.value,
            min: self.min,
            max: self.max,
            step: self.step,
        };
        let owner = window
            .use_keyed_state((self.id.clone(), "drag"), cx, |_, _| ())
            .entity_id();
        let focus = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let measured = window.use_keyed_state((self.id.clone(), "bounds"), cx, |_, _| {
            Bounds::<Pixels>::default()
        });
        let at = self.shown_at(track);
        let on_change = self.on_change.clone();
        let commit: OnChange = Rc::new(move |next, window, cx| {
            // A change under half a step is none, so no two floats are compared for equality.
            if (next - track.value).abs() < track.step * 0.5 {
                return;
            }
            if let Some(on_change) = &on_change {
                on_change(next, window, cx);
            }
        });
        let thumb = self.thumb(track, owner, &focus, &commit, window, cx);
        let bounds_state = measured.clone();
        let inner = div()
            .id((self.id.clone(), "inner"))
            .relative()
            .size_full()
            .children(rail_and_fill(at, cx))
            .child(thumb)
            .child(
                canvas(
                    move |bounds, _, cx| {
                        if *bounds_state.read(cx) != bounds {
                            bounds_state.update(cx, |state, _| *state = bounds);
                        }
                    },
                    |_, (), _, _| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
            .when(movable, |inner| {
                let drag = Rc::clone(&commit);
                inner
                    .on_drag(Thumb { owner }, |_, _, _, cx| cx.new(|_| EmptyView))
                    .on_drag_move(move |event: &DragMoveEvent<Thumb>, window, cx| {
                        if event.drag(cx).owner != owner {
                            return;
                        }
                        drag(
                            track.at(along(event.bounds, event.event.position)),
                            window,
                            cx,
                        );
                    })
                    .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        let bounds = *measured.read(cx);
                        cx.stop_propagation();
                        window.focus(&focus, cx);
                        commit(track.at(along(bounds, event.position)), window, cx);
                    })
            });
        div()
            .id(self.id)
            .flex_1()
            .min_w_0()
            .h(THUMB)
            .px(THUMB * 0.5)
            .when(!movable, |outer| outer.opacity(0.5))
            .child(inner)
    }
}
