//! Where the Graph tab's nodes sit, and how its view looks at them (#647).
//!
//! [`Layout`] is Ely's Fruchterman-Reingold `Force` in world units: every pair pushes apart by
//! k²/d, every edge pulls by d²/k, a pull to the origin holds the whole, and each move is capped by
//! a heat that cools a step at a time. The Graph tab runs it on the background executor a batch at
//! a time. [`Viewport`] is Ely's pan and zoom about a point, kept around a centre so it needs no
//! view size until it draws; [`nearest`] is Ely's hit test.
//!
//! Ported from Ely GPUI Components (`src/charts/network.rs` and `src/canvas/view.rs` at
//! `2f8b2f6`), in world units with Rusty's link length in place of Ely's unit square, with no
//! random nudge; Ely's notice is below.

// The portions named above are ported from Ely GPUI Components, under this notice:
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

use std::collections::HashMap;
use std::f32::consts::TAU;

/// The length an edge settles towards, in world units: a node's spacing.
pub const LINK: f32 = 80.0;

/// The pull to the origin, per world unit away from it. Chosen so a local graph's edges settle
/// near [`LINK`] and the whole vault stays a disc rather than a ring of orphans far out.
const GRAVITY: f32 = 4.0;

/// The heat a new layout starts at, and the heat it counts as settled under, in links.
const HEAT_START: f32 = 4.0;
const HEAT_SETTLED: f32 = 0.05;

/// The heat a change gives back: a read, a filter, a pin.
const HEAT_WARM: f32 = 1.0;

/// Each step keeps this much of the heat (Ely's).
const COOLING: f32 = 0.96;

/// The most steps a run takes, settled or not (d3-force's default run).
pub const MAX_STEPS: usize = 300;

/// The golden angle, for the spiral new nodes start on.
const GOLDEN_ANGLE: f32 = 2.399_963;

/// The nearest and the farthest the view zooms (Rusty's).
pub const ZOOMS: (f32, f32) = (0.15, 6.0);

/// The nearest a fit zooms (Rusty's), so a lone node is not drawn huge.
const FIT_NEAREST: f32 = 1.6;

/// A count or an index as a float, for the places it sizes; past `u16`'s range it is held there,
/// which the cap keeps the layout far below.
#[must_use]
pub fn float(count: usize) -> f32 {
    f32::from(u16::try_from(count).unwrap_or(u16::MAX))
}

/// The nodes' places and what moves them: a run of steps cooling towards rest.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layout {
    places: Vec<(f32, f32)>,
    edges: Vec<(usize, usize)>,
    pinned: Vec<bool>,
    heat: f32,
    steps: usize,
}

impl Layout {
    /// A layout for `count` nodes joined by `edges`: a node in `kept` stays where it was, a new
    /// one starts beside its first placed neighbour or on the golden-angle spiral, and `centre`
    /// is pinned at the origin. A layout that keeps most of its places starts warm, not hot.
    #[must_use]
    pub fn seeded(
        count: usize,
        edges: Vec<(usize, usize)>,
        kept: &[Option<(f32, f32)>],
        centre: Option<usize>,
    ) -> Self {
        let mut places: Vec<Option<(f32, f32)>> = (0..count)
            .map(|at| kept.get(at).copied().flatten())
            .collect();
        let held = places.iter().filter(|place| place.is_some()).count();
        let mut neighbours: Vec<Vec<usize>> = vec![Vec::new(); count];
        for (from, to) in &edges {
            if *from < count && *to < count {
                neighbours[*from].push(*to);
                neighbours[*to].push(*from);
            }
        }
        let spread = (float(count).sqrt() * LINK * 0.3).max(LINK);
        for at in 0..count {
            if places[at].is_some() {
                continue;
            }
            let beside = neighbours[at]
                .iter()
                .find_map(|other| places.get(*other).copied().flatten());
            let angle = float(at) * GOLDEN_ANGLE;
            places[at] = Some(beside.map_or_else(
                || {
                    let radius = spread * ((float(at) + 1.0) / float(count.max(1))).sqrt();
                    (angle.cos() * radius, angle.sin() * radius)
                },
                |(x, y)| {
                    (
                        angle.cos().mul_add(LINK * 0.5, x),
                        angle.sin().mul_add(LINK * 0.5, y),
                    )
                },
            ));
        }
        let mut pinned = vec![false; count];
        // The centre is held: at the origin when new, else where it was, so a read again or a
        // centre dragged and dropped does not move the whole picture.
        if let Some(centre) = centre.filter(|centre| *centre < count) {
            places[centre] = Some(kept.get(centre).copied().flatten().unwrap_or_default());
            pinned[centre] = true;
        }
        let heat = if held * 2 > count {
            HEAT_WARM
        } else {
            HEAT_START
        } * LINK;
        Self {
            places: places.into_iter().map(Option::unwrap_or_default).collect(),
            edges,
            pinned,
            heat,
            steps: 0,
        }
    }

    /// The nodes' places, in world units.
    #[must_use]
    pub fn places(&self) -> &[(f32, f32)] {
        &self.places
    }

    /// How many steps the run has taken.
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }

    /// Whether the run is over: cooled to rest, or out of steps.
    #[must_use]
    pub fn settled(&self) -> bool {
        self.heat < HEAT_SETTLED * LINK || self.steps >= MAX_STEPS
    }

    /// Starts the run again from a little heat, as a change asks.
    pub fn warm(&mut self) {
        self.heat = self.heat.max(HEAT_WARM * LINK);
        self.steps = 0;
    }

    /// Holds node `at` at `place` and warms the rest to settle around it (Ely's `pin`).
    pub fn pin(&mut self, at: usize, place: (f32, f32)) {
        if let (Some(spot), Some(pinned)) = (self.places.get_mut(at), self.pinned.get_mut(at)) {
            *spot = place;
            *pinned = true;
            self.warm();
        }
    }

    /// Lets node `at` move again.
    pub fn release(&mut self, at: usize) {
        if let Some(pinned) = self.pinned.get_mut(at) {
            *pinned = false;
        }
    }

    /// Steps until about `pairs` pair checks are spent, the run settles or it is out of steps;
    /// at least one step when it is not settled. Returns the steps taken.
    pub fn run(&mut self, pairs: usize) -> usize {
        let count = self.places.len();
        let per_step = (count * count.saturating_sub(1) / 2 + self.edges.len()).max(1);
        let mut taken = 0;
        while !self.settled() && (taken == 0 || (taken + 1) * per_step <= pairs) {
            self.step();
            taken += 1;
        }
        taken
    }

    /// One step (Ely's `Force::step`): every pair pushes apart, every edge pulls together, the
    /// origin pulls everything, and each free node moves no farther than the heat.
    fn step(&mut self) {
        let count = self.places.len();
        let mut moves = vec![(0.0_f32, 0.0_f32); count];
        // The push k²/d along the unit vector is the offset times k²/d², so the pairs need no
        // square root. Plain products, not `mul_add` or `hypot`: without the FMA target feature,
        // which these builds do not enable, both are library calls, and this loop runs about two
        // million times a batch.
        for a in 0..count {
            let (ax, ay) = self.places[a];
            for b in a + 1..count {
                let (bx, by) = self.places[b];
                let (dx, dy) = (ax - bx, ay - by);
                let across = dx * dx;
                let down = dy * dy;
                let squared = across + down;
                let push = if squared < NEAR * NEAR {
                    let (x, y, distance) = apart((ax, ay), (bx, by), a, b);
                    let scale = LINK * LINK / distance;
                    (x * scale, y * scale)
                } else {
                    let scale = LINK * LINK / squared;
                    (dx * scale, dy * scale)
                };
                moves[a].0 += push.0;
                moves[a].1 += push.1;
                moves[b].0 -= push.0;
                moves[b].1 -= push.1;
            }
        }
        for (a, b) in &self.edges {
            let (Some(from), Some(to)) = (self.places.get(*a), self.places.get(*b)) else {
                continue;
            };
            let (x, y, distance) = apart(*from, *to, *a, *b);
            let pull = distance * distance / LINK;
            moves[*a] = (
                (-x).mul_add(pull, moves[*a].0),
                (-y).mul_add(pull, moves[*a].1),
            );
            moves[*b] = (x.mul_add(pull, moves[*b].0), y.mul_add(pull, moves[*b].1));
        }
        for (at, place) in self.places.iter_mut().enumerate() {
            if self.pinned[at] {
                continue;
            }
            let (x, y) = (
                (-place.0).mul_add(GRAVITY, moves[at].0),
                (-place.1).mul_add(GRAVITY, moves[at].1),
            );
            let length = x.hypot(y).max(1e-6);
            let reach = length.min(self.heat) / length;
            *place = (x.mul_add(reach, place.0), y.mul_add(reach, place.1));
        }
        self.heat *= COOLING;
        self.steps += 1;
    }
}

/// Two places nearer than this count as one spot.
const NEAR: f32 = 0.01;

/// The unit vector from `b` to `a` and their distance; two nodes on one spot part along an angle
/// their indices give, where Rusty draws a random one, so every run is the same.
fn apart(a: (f32, f32), b: (f32, f32), first: usize, second: usize) -> (f32, f32, f32) {
    let (dx, dy) = (a.0 - b.0, a.1 - b.1);
    let distance = dx.hypot(dy);
    if distance < NEAR {
        let angle = float((first * 7 + second * 13) % 360) / 360.0 * TAU;
        return (angle.cos(), angle.sin(), NEAR);
    }
    (dx / distance, dy / distance, distance)
}

/// Where the view looks: the world point at its middle, and its zoom (Ely's `Viewport`, kept
/// about its middle rather than its corner).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// The world point drawn at the view's middle.
    pub centre: (f32, f32),
    /// View pixels per world unit.
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            centre: (0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl Viewport {
    /// A world point in view pixels from the top left of a view of `size`.
    #[must_use]
    pub fn to_view(self, (x, y): (f32, f32), size: (f32, f32)) -> (f32, f32) {
        (
            (x - self.centre.0).mul_add(self.zoom, size.0 / 2.0),
            (y - self.centre.1).mul_add(self.zoom, size.1 / 2.0),
        )
    }

    /// A point of a view of `size` as a world point.
    #[must_use]
    pub fn to_world(self, (x, y): (f32, f32), size: (f32, f32)) -> (f32, f32) {
        (
            (x - size.0 / 2.0) / self.zoom + self.centre.0,
            (y - size.1 / 2.0) / self.zoom + self.centre.1,
        )
    }

    /// Zoomed by `factor` about a view point, which keeps its world point; held inside [`ZOOMS`].
    #[must_use]
    pub fn zoomed(self, factor: f32, about: (f32, f32), size: (f32, f32)) -> Self {
        let held = self.to_world(about, size);
        let zoom = (self.zoom * factor).clamp(ZOOMS.0, ZOOMS.1);
        let moved = Self {
            centre: self.centre,
            zoom,
        };
        let drifted = moved.to_world(about, size);
        Self {
            centre: (
                moved.centre.0 + held.0 - drifted.0,
                moved.centre.1 + held.1 - drifted.1,
            ),
            zoom,
        }
    }

    /// Moved so the content slides by a view distance.
    #[must_use]
    pub fn panned(self, (dx, dy): (f32, f32)) -> Self {
        Self {
            centre: (
                self.centre.0 - dx / self.zoom,
                self.centre.1 - dy / self.zoom,
            ),
            zoom: self.zoom,
        }
    }

    /// Shows every point in a view of `size` with `margin` view pixels clear each side, no nearer
    /// than Rusty's fit; about `centre` when given, so it stays in the middle, else about the
    /// points' box.
    #[must_use]
    pub fn fitting(
        points: &[(f32, f32)],
        size: (f32, f32),
        margin: f32,
        centre: Option<(f32, f32)>,
    ) -> Self {
        let Some(first) = points.first() else {
            return Self::default();
        };
        let (mut low, mut high) = (*first, *first);
        for (x, y) in points {
            low = (low.0.min(*x), low.1.min(*y));
            high = (high.0.max(*x), high.1.max(*y));
        }
        let middle =
            centre.unwrap_or_else(|| (f32::midpoint(low.0, high.0), f32::midpoint(low.1, high.1)));
        let half = (
            (high.0 - middle.0).max(middle.0 - low.0).max(1.0),
            (high.1 - middle.1).max(middle.1 - low.1).max(1.0),
        );
        let room = (
            (size.0 / 2.0 - margin).max(1.0),
            (size.1 / 2.0 - margin).max(1.0),
        );
        Self {
            centre: middle,
            zoom: (room.0 / half.0)
                .min(room.1 / half.1)
                .clamp(ZOOMS.0, FIT_NEAREST),
        }
    }
}

/// The node nearest `point` within its radius and `slack`, in view pixels (Ely's hit test).
#[must_use]
pub fn nearest(
    places: &[(f32, f32)],
    radii: &[f32],
    point: (f32, f32),
    slack: f32,
) -> Option<usize> {
    places
        .iter()
        .zip(radii)
        .enumerate()
        .map(|(at, (place, radius))| (at, (place.0 - point.0).hypot(place.1 - point.1), *radius))
        .filter(|(_, distance, radius)| *distance <= radius + slack)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(at, _, _)| at)
}

/// The places of the nodes named in `ids`, by id, to seed the next layout with.
#[must_use]
pub fn places_by_id<'a>(
    ids: impl IntoIterator<Item = &'a str>,
    places: &[(f32, f32)],
) -> HashMap<String, (f32, f32)> {
    ids.into_iter()
        .zip(places)
        .map(|(id, place)| (id.to_string(), *place))
        .collect()
}
