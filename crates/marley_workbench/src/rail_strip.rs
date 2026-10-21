//! The rail folded to a strip of project marks (#830): a tile per project with a count of what
//! waits on the user, the project's rows beside a tile under the pointer, and the filter as a
//! popover.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, Bounds, Context, Div, Focusable as _, FontWeight, Pixels, RenderImage, Task,
    Window, anchored, canvas, deferred, img, point, px,
};
use marley_rail::Row;
use settings::Settings as _;
use ui::{IconButton, IconName, IconSize, Label, LabelSize, Tab, Tooltip, prelude::*};
use util::ResultExt as _;
use zed_actions::agents_sidebar::FocusSidebarFilter;

use super::Rail;
use crate::{MarleySettings, RailStrip};

/// The shrunk rail's width.
pub(super) const STRIP_WIDTH: Pixels = px(60.);
/// How long the rail takes to fold or open, from the design's States board.
const EASE: Duration = Duration::from_millis(180);
/// How long a flyout waits after the pointer leaves its tile, so the pointer can cross the gap
/// into it.
const FLYOUT_GRACE: Duration = Duration::from_millis(150);
const TILE_SIZE: Pixels = px(36.);
const FLYOUT_WIDTH: Pixels = px(280.);
const SEARCH_WIDTH: Pixels = px(300.);

/// The shrunk rail's state.
#[derive(Default)]
pub(super) struct Strip {
    /// Whether the rail draws as the strip, which follows `marley.rail_shrunk`.
    pub(super) shrunk: bool,
    /// When the last fold or opening began, while its ease may still run.
    motion: Option<Instant>,
    /// The project whose rows show beside its tile.
    flyout: Option<usize>,
    hover: Hover,
    flyout_close: Option<Task<()>>,
    /// Whether the filter shows as a popover beside the strip.
    pub(super) search_open: bool,
    /// Where each tile and the search button were last painted, which the overlays open beside.
    tiles: Rc<RefCell<HashMap<usize, Bounds<Pixels>>>>,
    search: Rc<Cell<Option<Bounds<Pixels>>>>,
}

/// Where the pointer is for the flyout: on its tile, on the flyout, both while crossing, or
/// neither.
#[derive(Default)]
struct Hover {
    tile: bool,
    flyout: bool,
}

fn shrunk_in_settings(cx: &App) -> bool {
    MarleySettings::get_global(cx).rail_strip == RailStrip::Shrunk
}

impl Strip {
    pub(super) fn new(cx: &App) -> Self {
        Self {
            shrunk: shrunk_in_settings(cx),
            ..Self::default()
        }
    }

    /// Asks for the next frame while a fold eases, and forgets one that has ended.
    pub(super) fn tick(&mut self, window: &Window) {
        if self.moving() {
            window.request_animation_frame();
        } else {
            self.motion = None;
        }
    }

    /// The rail's width as drawn: the strip's or `open`, eased between them while a fold runs.
    pub(super) fn width(&self, open: Pixels) -> Pixels {
        let (from, to) = if self.shrunk {
            (open, STRIP_WIDTH)
        } else {
            (STRIP_WIDTH, open)
        };
        let Some(started) = self.motion else {
            return to;
        };
        let progress = (started.elapsed().as_secs_f32() / EASE.as_secs_f32()).min(1.);
        let eased = 1. - (1. - progress).powi(3);
        from + (to - from) * eased
    }

    fn moving(&self) -> bool {
        self.motion.is_some_and(|started| started.elapsed() < EASE)
    }
}

/// A project's first letter, for a tile with no icon.
fn initial(name: &str) -> String {
    name.chars()
        .find(|character| character.is_alphanumeric())
        .map_or_else(
            || "·".to_string(),
            |character| character.to_uppercase().collect(),
        )
}

/// A number from a project's name, so its tile keeps one tint.
fn name_hash(name: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    u32::try_from(hasher.finish() & u64::from(u32::MAX)).unwrap_or_default()
}

/// The count of a project's rows that wait on the user, over its tile's corner.
fn strip_badge(index: usize, waiting: usize, cx: &App) -> Div {
    div()
        .debug_selector(move || format!("marley-rail-strip-badge-{index}"))
        .absolute()
        .top(px(-5.))
        .right(px(-5.))
        .min_w(px(16.))
        .h(px(16.))
        .px_1()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(cx.theme().status().conflict)
        .child(
            Label::new(waiting.to_string())
                .size(LabelSize::XSmall)
                .weight(FontWeight::BOLD)
                .color(Color::Custom(cx.theme().colors().editor_background)),
        )
}

impl Rail {
    /// Follows `marley.rail_shrunk`: folds or opens the rail, eased unless motion is reduced.
    pub(super) fn follow_strip_setting(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let shrunk = shrunk_in_settings(cx);
        if shrunk == self.strip.shrunk {
            return;
        }
        self.strip.shrunk = shrunk;
        self.strip.motion = (!cx.reduce_motion()).then(Instant::now);
        self.strip.flyout = None;
        self.strip.hover = Hover::default();
        self.strip.flyout_close = None;
        self.strip.search_open = false;
        // The strip lists projects; the Threads view has no shrunk form.
        if shrunk {
            self.show_view(false, window, cx);
        }
        cx.notify();
    }

    /// Opens the filter as a popover beside the strip, with the keys in it.
    pub(super) fn open_strip_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.strip.search_open = true;
        self.strip.flyout = None;
        window.focus(&self.filter_editor.focus_handle(cx), cx);
        cx.notify();
    }

    pub(super) fn close_strip_search(&mut self, cx: &mut Context<Self>) {
        if self.strip.search_open {
            self.strip.search_open = false;
            cx.notify();
        }
    }

    /// Closes the flyout a moment after the pointer left both its tile and itself.
    fn leave_flyout(&mut self, cx: &Context<Self>) {
        self.strip.flyout_close = Some(cx.spawn(async move |rail, cx| {
            cx.background_executor().timer(FLYOUT_GRACE).await;
            rail.update(cx, |rail, cx| {
                if !rail.strip.hover.tile && !rail.strip.hover.flyout {
                    rail.strip.flyout = None;
                    cx.notify();
                }
            })
            .log_err();
        }));
    }

    pub(super) fn render_strip(&self, window: &Window, cx: &Context<Self>) -> impl IntoElement {
        let border = cx.theme().colors().border;
        let search_bounds = Rc::clone(&self.strip.search);
        let top = h_flex()
            .h(ui::utils::platform_title_bar_height(window) + px(1.))
            .w_full()
            .flex_none()
            .justify_center()
            .border_b_1()
            .border_color(border)
            .child(
                IconButton::new("marley-rail-open", IconName::ThreadsSidebarLeftClosed)
                    .icon_size(IconSize::Small)
                    .tooltip(Tooltip::text("Open the Rail"))
                    .on_click(|_, _, cx| crate::toggle_rail_strip(cx)),
            );
        let search = h_flex()
            .relative()
            .h(Tab::container_height(cx))
            .w_full()
            .flex_none()
            .justify_center()
            .border_b_1()
            .border_color(border)
            .child(
                canvas(
                    move |bounds, _, _| search_bounds.set(Some(bounds)),
                    |_, (), _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(
                IconButton::new("marley-rail-strip-search", IconName::MagnifyingGlass)
                    .icon_size(IconSize::Small)
                    .toggle_state(self.strip.search_open)
                    .tooltip(|_, cx| {
                        Tooltip::for_action("Filter the Rail", &FocusSidebarFilter, cx)
                    })
                    .on_click(cx.listener(|rail, _, window, cx| {
                        rail.open_strip_search(window, cx);
                    })),
            );
        let tiles = self.render_strip_tiles(cx);
        v_flex()
            .size_full()
            .child(top)
            .child(search)
            .child(
                v_flex()
                    .id("marley-rail-strip-tiles")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .items_center()
                    .py_2()
                    .gap_2()
                    .children(tiles),
            )
            .children(self.render_strip_flyout(cx))
            .children(self.render_strip_search(cx))
    }

    fn render_strip_tiles(&self, cx: &Context<Self>) -> Vec<AnyElement> {
        let rail = &self.snapshot.rail;
        let shown = self
            .multi_workspace
            .upgrade()
            .map(|multi_workspace| multi_workspace.read(cx).workspace().downgrade());
        let mut tiles = Vec::new();
        for row in marley_rail::rail_rows(rail) {
            let Row::Project(row) = row else {
                continue;
            };
            let Some(group) = self.snapshot.groups.get(row.index) else {
                continue;
            };
            let waiting = rail
                .projects
                .get(row.index)
                .map_or(0, marley_rail::needs_you_count);
            let icon = group
                .key
                .path_list()
                .paths()
                .first()
                .and_then(|root| self.project_icons.get(root))
                .and_then(|icon| icon.image.clone());
            let active = !group.closed && shown.as_ref() == Some(&group.workspace);
            tiles.push(self.render_strip_tile(row.index, &row.name, icon, active, waiting, cx));
        }
        tiles
    }

    fn render_strip_tile(
        &self,
        index: usize,
        name: &str,
        icon: Option<Arc<RenderImage>>,
        active: bool,
        waiting: usize,
        cx: &Context<Self>,
    ) -> AnyElement {
        let colors = cx.theme().colors();
        let tint = cx
            .theme()
            .players()
            .color_for_participant(name_hash(name))
            .cursor;
        let has_icon = icon.is_some();
        let face = icon.map_or_else(
            || {
                Label::new(initial(name))
                    .size(LabelSize::Large)
                    .weight(FontWeight::SEMIBOLD)
                    .color(Color::Custom(tint))
                    .into_any_element()
            },
            |icon| img(icon).size_6().rounded_sm().into_any_element(),
        );
        let tiles = Rc::clone(&self.strip.tiles);
        let group = self.snapshot.groups.get(index);
        let closed = group.is_some_and(|group| group.closed);
        let workspace = group.map(|group| group.workspace.clone());
        let open_key = group.map(|group| group.key.clone());
        div()
            .id(("marley-rail-strip-tile", index))
            .debug_selector(move || format!("marley-rail-strip-tile-{index}"))
            .relative()
            .size(TILE_SIZE)
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_lg()
            .border_2()
            .border_color(if active {
                colors.text_accent
            } else {
                gpui::transparent_black()
            })
            .bg(if has_icon {
                colors.element_background
            } else {
                tint.opacity(0.18)
            })
            .when(closed, |tile| tile.opacity(0.5))
            .hover(|style| style.bg(colors.element_hover))
            .cursor_pointer()
            .child(
                canvas(
                    move |bounds, _, _| {
                        tiles.borrow_mut().insert(index, bounds);
                    },
                    |_, (), _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(face)
            .when(waiting > 0, |tile| {
                tile.child(strip_badge(index, waiting, cx))
            })
            .on_hover(cx.listener(move |rail, hovered: &bool, _, cx| {
                rail.strip.hover.tile = *hovered;
                if *hovered {
                    rail.strip.flyout = Some(index);
                    rail.strip.flyout_close = None;
                    cx.notify();
                } else {
                    rail.leave_flyout(cx);
                }
            }))
            .on_click(cx.listener(move |rail, _, window, cx| {
                rail.strip.flyout = None;
                if closed {
                    if let Some(key) = &open_key {
                        rail.open_closed_project(key, window, cx);
                    }
                } else if let Some(workspace) = &workspace {
                    rail.activate_workspace(workspace, window, cx).log_err();
                }
            }))
            .into_any_element()
    }

    /// The hovered project's header and rows as the open rail draws them, beside its tile.
    fn render_strip_flyout(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let index = self.strip.flyout?;
        let bounds = *self.strip.tiles.borrow().get(&index)?;
        let block = self.render_blocks(Some(index), cx).into_iter().next()?;
        let colors = cx.theme().colors();
        Some(
            deferred(
                anchored()
                    .position(point(bounds.right() + px(8.), bounds.top()))
                    .snap_to_window_with_margin(px(8.))
                    .child(
                        v_flex()
                            .id("marley-rail-strip-flyout")
                            .debug_selector(|| "marley-rail-strip-flyout".into())
                            .occlude()
                            .w(FLYOUT_WIDTH)
                            .p_1p5()
                            .bg(colors.elevated_surface_background)
                            .border_1()
                            .border_color(colors.border)
                            .rounded_lg()
                            .shadow_lg()
                            .on_hover(cx.listener(|rail, hovered: &bool, _, cx| {
                                rail.strip.hover.flyout = *hovered;
                                if *hovered {
                                    rail.strip.flyout_close = None;
                                } else {
                                    rail.leave_flyout(cx);
                                }
                            }))
                            .child(block),
                    ),
            )
            .with_priority(1)
            .into_any_element(),
        )
    }

    /// The filter and the rows it matches, beside the strip's search button.
    fn render_strip_search(&self, cx: &Context<Self>) -> Option<AnyElement> {
        if !self.strip.search_open {
            return None;
        }
        let bounds = self.strip.search.get()?;
        let colors = cx.theme().colors();
        Some(
            deferred(
                anchored()
                    .position(point(bounds.right() + px(4.), bounds.top()))
                    .snap_to_window_with_margin(px(8.))
                    .child(
                        v_flex()
                            .id("marley-rail-strip-search-popover")
                            .debug_selector(|| "marley-rail-strip-search-popover".into())
                            .occlude()
                            .w(SEARCH_WIDTH)
                            .max_h(px(480.))
                            .bg(colors.elevated_surface_background)
                            .border_1()
                            .border_color(colors.border)
                            .rounded_lg()
                            .shadow_lg()
                            .overflow_hidden()
                            .on_mouse_down_out(cx.listener(|rail, _, _, cx| {
                                rail.close_strip_search(cx);
                            }))
                            .child(self.render_filter(cx))
                            .child(self.render_rows(self.render_blocks(None, cx), cx)),
                    ),
            )
            .with_priority(1)
            .into_any_element(),
        )
    }
}
