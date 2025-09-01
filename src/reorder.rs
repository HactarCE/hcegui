//! Flexible API for drag-and-drop and reordering.
//!
//! - Any UI widget or layout can be made draggable
//! - Any UI widget or layout can be given a handle for dragging
//! - Any UI widget or layout can be made a target for dragging
//! - Any UI widget or layout can be made a target for reordering
//! - Multiple separate drag-and-drop environments can coexist and even overlap
//!   in the same UI
//!
//! # Examples
//!
//! ```
//! # egui::__run_test_ui(|ui| {
//! use hcegui::*;
//!
//! let mut elements = vec!["point", "line", "plane", "space"];
//! let mut dnd = reorder::Dnd::new(ui.ctx(), ui.next_auto_id());
//! for (i, &elem) in elements.iter().enumerate() {
//!     dnd.reorderable_with_handle(ui, i, |ui, _| ui.label(elem));
//! }
//! if let Some(r) = dnd.finish(ui).if_done_dragging() {
//!     r.reorder_vec(&mut elements);
//! }
//! # });
//! ```
//!
//! For more advanced examples, see
//! [`bin/demo/reorder.rs`](https://github.com/HactarCE/hcegui/blob/main/src/bin/demo/reorder.rs).

use std::hash::Hash;

/// Whether the payload should be placed before or after the target.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum BeforeOrAfter {
    Before,
    After,
}

/// Styling for [`Dnd`].
#[derive(Debug, Copy, Clone)]
pub struct DndStyle {
    /// Rounding of hole left behind by the payload.
    pub payload_hole_rounding: f32,
    /// Opacity of background in the hole left behind by the payload.
    pub payload_hole_opacity: f32,
    /// Opacity of dragged payload.
    pub payload_opacity: f32,
    /// Width of non-reorder drop zone stroke.
    pub drop_zone_stroke_width: f32,
    /// Rounding of non-reorder drop zones.
    pub drop_zone_rounding: f32,
    /// Width of reorder drop zone line stroke.
    pub reorder_stroke_width: f32,
}
impl Default for DndStyle {
    fn default() -> Self {
        Self {
            payload_hole_rounding: 3.0,
            payload_hole_opacity: 0.25,
            payload_opacity: 1.0,
            drop_zone_stroke_width: 2.0,
            drop_zone_rounding: 3.0,
            reorder_stroke_width: 2.0,
        }
    }
}

/// Drag-and-drop environment.
///
/// - `Payload` is a type that identifies the things being dragged.
/// - `Target` is a type that indentifies the drop zones.
///
/// For reordering a list with `usize` indices, use [`ReorderDnd`].
///
/// Note that you **must** call either [`Dnd::finish()`] or
/// [`Dnd::allow_unfinished()`] before the `Dnd` goes out of scope.
#[derive(Debug)]
pub struct Dnd<Payload, Target> {
    ctx: egui::Context,

    /// ID used to store state.
    id: egui::Id,
    /// Styling
    pub style: DndStyle,
    /// State persisted between frames.
    current_drag: Option<DndDragState>,
    /// Payload value being dragged.
    payload: Option<Payload>,
    /// Target where the payload is being hovered.
    target: Option<Target>,
    /// Locations where the payload can be dropped for reordering.
    reorder_drop_zones: Vec<ReorderTarget<Target>>,
}
impl<Payload, Target> Dnd<Payload, Target> {
    /// Constructs a new drag-and-drop context.
    #[track_caller]
    pub fn new(ctx: &egui::Context, id: impl Into<egui::Id>) -> Self {
        let id = id.into();

        let (last_frame_was_unfinished, state) = ctx.data_mut(|data| {
            let last_frame_was_unfinished = data.remove_temp::<()>(id).is_some();
            data.insert_temp(id, ()); // marker that `finish()` has not been called yet
            let state = data.remove_temp::<DndDragState>(id);
            (last_frame_was_unfinished, state)
        });
        assert!(
            !last_frame_was_unfinished,
            "Dnd dropped without calling `finish()`. Call `allow_unfinished()` if this is intentional.",
        );

        let mut this = Self {
            ctx: ctx.clone(),

            id,
            style: DndStyle::default(),
            current_drag: state,
            payload: None,
            target: None,
            reorder_drop_zones: vec![],
        };

        ctx.input(|input| {
            if !(input.pointer.any_down() || input.pointer.any_released()) {
                // Done dragging -> delete payload
                this.current_drag = None;
            }
        });

        this
    }

    /// Overrides the style.
    #[must_use]
    pub fn with_style(mut self, style: DndStyle) -> Self {
        self.style = style;
        self
    }

    /// Returns whether there is an active drag in this context.
    pub fn is_dragging(&self) -> bool {
        self.current_drag.is_some()
    }
    /// Returns the ID of the payload being dragged, if there is one.
    pub fn payload_id(&self) -> Option<egui::Id> {
        self.current_drag.as_ref().map(|state| state.payload_id)
    }

    /// Allows the `Dnd` to be dropped without calling `finish()`.
    ///
    /// By default in debug mode, the thread will panic if a `Dnd` is dropped
    /// without calling `finish()`. (Actually the panic happens on the next
    /// frame when the `Dnd` is created again, since panicking in a destructor
    /// is rude.)
    #[must_use]
    pub fn allow_unfinished(self) -> Self {
        self.ctx.data_mut(|data| data.remove_temp::<()>(self.id)); // safe to call multiple times
        self
    }

    /// Adds a new draggable object with a custom ID. See [`Dnd::draggable()`].
    pub fn draggable_with_id<R>(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        payload: Payload,
        add_contents: impl FnOnce(&mut egui::Ui) -> (egui::Response, R),
    ) -> egui::InnerResponse<R> {
        let state = self
            .current_drag
            .as_mut()
            .filter(|state| state.payload_id == id);

        if ui.is_sizing_pass() {
            ui.scope(|ui| add_contents(ui).1)
        } else if let Some(state) = state {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
            self.payload = Some(payload);

            // Paint the widget to a different layer so that we can move it
            // around independently. Highlight the widget so that it looks like
            // it's still being hovered.
            let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
            let r = ui.scope_builder(egui::UiBuilder::new().layer_id(layer_id), |ui| {
                ui.set_opacity(self.style.payload_opacity);
                // `push_id()` is a workaround for https://github.com/emilk/egui/issues/2253
                ui.push_id(id, |ui| add_contents(ui)).inner
            });
            let (_, return_value) = r.inner;

            ui.painter().rect_filled(
                r.response.rect,
                self.style.payload_hole_rounding,
                (ui.visuals().widgets.hovered.bg_fill)
                    .gamma_multiply(self.style.payload_hole_opacity),
            );

            if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
                let delta = pointer_pos + state.cursor_offset - r.response.rect.left_top();
                ui.ctx().transform_layer_shapes(
                    layer_id,
                    egui::emath::TSTransform::from_translation(delta),
                );
                state.drop_pos = r.response.rect.center() + delta;
            }

            egui::InnerResponse::new(return_value, r.response)
        } else {
            // We must use `.scope()` *and* `.push_id()` so that the IDs are all
            // the same as the other case.
            let r = ui.scope(|ui| ui.push_id(id, |ui| add_contents(ui)).inner);
            let (drag_handle_response, return_value) = r.inner;

            // Check that the drag handle detects drags
            let drag_handle_response = drag_handle_response.interact(egui::Sense::drag());

            if !drag_handle_response.sense.senses_click() && drag_handle_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }

            if drag_handle_response.drag_started()
                && let Some(interact_pos) = drag_handle_response.interact_pointer_pos()
            {
                let cursor_offset = r.response.rect.left_top() - interact_pos;
                self.current_drag = Some(DndDragState {
                    payload_id: id,
                    cursor_offset,
                    drop_pos: r.response.rect.center(),
                });
                self.payload = Some(payload);
            }

            egui::InnerResponse::new(return_value, r.response)
        }
    }

    /// Adds a new draggable object, using `payload` for the ID.
    ///
    /// The first value returned by `add_contents` is used as the response for
    /// the drag handle, which may be any widget or region that does not use
    /// drags for other interaction.
    pub fn draggable<R>(
        &mut self,
        ui: &mut egui::Ui,
        payload: Payload,
        add_contents: impl FnOnce(&mut egui::Ui, egui::Id) -> (egui::Response, R),
    ) -> egui::InnerResponse<R>
    where
        Payload: Hash,
    {
        let id = self.id.with(&payload);
        self.draggable_with_id(ui, id, payload, |ui| add_contents(ui, id))
    }

    /// Add a drop zone onto an existing widget.
    ///
    /// `target` is a value representing this drop zone.
    pub fn drop_zone(&mut self, ui: &mut egui::Ui, r: &egui::Response, target: Target) {
        if ui.is_sizing_pass() {
            return;
        }

        if !self.is_dragging() {
            return;
        }

        let color = ui.visuals().widgets.active.bg_stroke.color;
        let width = self.style.drop_zone_stroke_width;
        let active_stroke = egui::Stroke { width, color };

        let color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let inactive_stroke = egui::Stroke { width, color };

        let is_active = self
            .current_drag
            .as_ref()
            .is_some_and(|s| r.interact_rect.contains(s.drop_pos));

        if is_active {
            self.target = Some(target);
        }

        let stroke = if is_active {
            active_stroke
        } else {
            inactive_stroke
        };

        ui.painter().rect_stroke(
            r.rect,
            self.style.drop_zone_rounding,
            stroke,
            egui::StrokeKind::Outside,
        );
    }

    /// Ends the drag-and-drop context and returns a response.
    pub fn finish(mut self, ui: &egui::Ui) -> DndResponse<Payload, Target> {
        self = self.allow_unfinished();

        // If nothing is being dragged, do nothing
        let Some(state) = self.current_drag.take() else {
            return DndResponse::Inactive;
        };
        let Some(payload) = self.payload.take() else {
            return DndResponse::Inactive;
        };

        // Compute reorder drop target and draw line
        let reorder_drop_target = (|| {
            let cursor_pos = ui.input(|input| input.pointer.interact_pos())?;
            let drop_pos = state.drop_pos;

            let clip_rect = &ui.clip_rect();
            if !clip_rect.contains(egui::pos2(drop_pos.x, cursor_pos.y))
                && !clip_rect.contains(egui::pos2(cursor_pos.x, drop_pos.y))
            {
                return None; // cursor position is outside the current UI
            }

            let closest = std::mem::take(&mut self.reorder_drop_zones)
                .into_iter()
                .filter_map(|drop_zone| {
                    let [a, b] = drop_zone.line_endpoints;
                    let distance_to_cursor = if drop_zone.direction.is_horizontal() {
                        (a.y..=b.y)
                            .contains(&drop_pos.y)
                            .then(|| (a.x - cursor_pos.x).abs())
                    } else {
                        (a.x..=b.x)
                            .contains(&drop_pos.x)
                            .then(|| (a.y - cursor_pos.y).abs())
                    };
                    Some((drop_zone, distance_to_cursor?))
                })
                .min_by(|(_, distance1), (_, distance2)| f32::total_cmp(distance1, distance2));

            closest.map(|(drop_zone, _distance)| {
                let color = ui.visuals().widgets.active.bg_stroke.color;
                let stroke = egui::Stroke::new(self.style.reorder_stroke_width, color);
                ui.painter()
                    .with_clip_rect(drop_zone.clip_rect.expand(self.style.reorder_stroke_width))
                    .line_segment(drop_zone.line_endpoints, stroke);
                drop_zone.target
            })
        })();
        if self.target.is_none() {
            // IIFE to mimic try_block
            self.target = reorder_drop_target;
        }

        // Compute response and store state
        if self.ctx.input(|input| input.pointer.any_released()) {
            if let Some(target) = self.target.take() {
                // done dragging
                DndResponse::DoneDragging(DndMove { payload, target })
            } else {
                // done dragging but not hovering any endpoint
                DndResponse::Inactive
            }
        } else {
            // still dragging
            self.ctx
                .data_mut(|data| data.insert_temp::<DndDragState>(self.id, state));
            let target = self.target.take();
            DndResponse::MidDrag(DndMove { payload, target })
        }
    }

    /// Adds a new reorder drop zone at `ui.cursor()`.
    pub fn reorder_drop_zone(&mut self, ui: &mut egui::Ui, target: Target) {
        let dir = ui.layout().main_dir;
        let rect = ui.cursor();
        self.reorder_drop_zones.push(ReorderTarget {
            line_endpoints: match dir {
                egui::Direction::LeftToRight => [rect.left_top(), rect.left_bottom()],
                egui::Direction::RightToLeft => [rect.right_top(), rect.right_bottom()],
                egui::Direction::TopDown => [rect.left_top(), rect.right_top()],
                egui::Direction::BottomUp => [rect.left_bottom(), rect.right_bottom()],
            },
            clip_rect: ui.clip_rect(),
            direction: dir,
            target,
        });
    }
}

impl<Payload, Target: Clone> Dnd<Payload, (Target, BeforeOrAfter)> {
    /// Creates a new reorder drop zone before and after `r`.
    pub fn reorder_drop_zone_before_after(
        &mut self,
        ui: &mut egui::Ui,
        r: &egui::Response,
        target: Target,
    ) {
        if !self.is_dragging() {
            return;
        }

        let expansion = ui.spacing().item_spacing / 2.0;
        let rect = r.rect.expand2(expansion);
        let clip_rect = ui.clip_rect().expand2(expansion);

        let dir = ui.layout().main_dir;
        let tl = rect.left_top();
        let tr = rect.right_top();
        let dl = rect.left_bottom();
        let dr = rect.right_bottom();
        self.reorder_drop_zones.push(ReorderTarget {
            line_endpoints: [tl, if dir.is_horizontal() { dl } else { tr }],
            clip_rect,
            direction: dir,
            target: (target.clone(), BeforeOrAfter::Before),
        });
        self.reorder_drop_zones.push(ReorderTarget {
            line_endpoints: [if dir.is_horizontal() { tr } else { dl }, dr],
            clip_rect,
            direction: dir,
            target: (target, BeforeOrAfter::After),
        });
    }
}

impl<I: Clone + PartialEq + Hash> Dnd<I, (I, BeforeOrAfter)> {
    /// Adds a new draggable object, using `index` for the ID. See
    /// [`Dnd::draggable()`].
    pub fn reorderable<R>(
        &mut self,
        ui: &mut egui::Ui,
        index: I,
        add_contents: impl FnOnce(&mut egui::Ui, egui::Id) -> (egui::Response, R),
    ) -> egui::InnerResponse<R> {
        let r = self.draggable(ui, index.clone(), add_contents);
        self.reorder_drop_zone_before_after(ui, &r.response, index);
        r
    }

    /// Adds a new object with a draggable handle, using `index` for the ID. See
    /// [`Dnd::draggable()`].
    pub fn reorderable_with_handle<R>(
        &mut self,
        ui: &mut egui::Ui,
        index: I,
        add_contents: impl FnOnce(&mut egui::Ui, egui::Id) -> R,
    ) -> egui::InnerResponse<R> {
        self.reorderable(ui, index, |ui, id| {
            let main_dir = ui.layout().main_dir();
            ui.horizontal(|ui| {
                if main_dir.is_vertical() {
                    ui.set_width(ui.available_width());
                }
                (ui.add(ReorderHandle), add_contents(ui, id))
            })
            .inner
        })
    }
}

/// State persisted between frames for each [`Dnd`].
#[derive(Debug, Clone)]
struct DndDragState {
    payload_id: egui::Id,
    cursor_offset: egui::Vec2,
    drop_pos: egui::Pos2,
}
impl Default for DndDragState {
    /// This value is never actually used, but the trait impl is necessary for
    /// [`egui::Data::remove_temp()`].
    fn default() -> Self {
        Self {
            payload_id: egui::Id::NULL,
            cursor_offset: Default::default(),
            drop_pos: Default::default(),
        }
    }
}

#[derive(Debug)]
struct ReorderTarget<Target> {
    line_endpoints: [egui::Pos2; 2],
    clip_rect: egui::Rect,
    direction: egui::Direction,
    target: Target,
}

/// Response from a drag-and-drop.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DndResponse<Payload, Target> {
    /// Not dragging.
    #[default]
    Inactive,
    /// In the middle of a drag-and-drop.
    MidDrag(DndMove<Payload, Option<Target>>),
    /// Just completed a drag-and-drop.
    DoneDragging(DndMove<Payload, Target>),
}
impl<Payload, Target> DndResponse<Payload, Target> {
    /// Returns the drag-and-drop response only on the frame the payload was
    /// dropped.
    pub fn if_done_dragging(self) -> Option<DndMove<Payload, Target>> {
        match self {
            DndResponse::DoneDragging(dnd_response) => Some(dnd_response),
            _ => None,
        }
    }
}

/// Drag-and-drop for reordering a sequence.
pub type ReorderDnd<I = usize> = Dnd<I, (I, BeforeOrAfter)>;

/// Drag-and-drop move.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DndMove<Payload, Target> {
    /// Thing being moved.
    pub payload: Payload,
    /// Place the payload was moved to.
    pub target: Target,
}
impl<Payload, Target> DndMove<Payload, Target> {
    /// Constructs a drag-and-drop response.
    pub fn new(payload: Payload, target: Target) -> Self {
        Self { payload, target }
    }
}

/// Drag-and-drop move for reordering a sequence.
pub type ReorderDndMove<I = usize> = DndMove<I, (I, BeforeOrAfter)>;
impl ReorderDndMove {
    /// Returns the `i` and `j` such that the element at index `i` should shift
    /// to index `j`.
    pub fn list_reorder_indices(self) -> (usize, usize) {
        let i = self.payload;
        let (j, before_or_after) = self.target;
        // Overflow/underflow is impossible because we only add/subtract 1 when `i` and
        // `j` are
        match (j.cmp(&i), before_or_after) {
            (std::cmp::Ordering::Greater, BeforeOrAfter::Before) => (i, j - 1),
            (std::cmp::Ordering::Less, BeforeOrAfter::After) => (i, j + 1),
            _ => (i, j),
        }
    }

    /// Reorders a slice.
    pub fn reorder<T>(self, v: &mut [T]) {
        let (i, j) = self.list_reorder_indices();
        if i < j {
            v[i..=j].rotate_left(1);
        } else {
            v[j..=i].rotate_right(1);
        }
    }
}

/// Visual handle for dragging widgets.
pub struct ReorderHandle;
impl egui::Widget for ReorderHandle {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, r) = ui.allocate_exact_size(egui::vec2(12.0, 20.0), egui::Sense::drag());
        if ui.is_rect_visible(rect) {
            // Change color based on hover/focus.
            let color = if r.has_focus() || r.dragged() {
                ui.visuals().strong_text_color()
            } else if r.hovered() {
                ui.visuals().text_color()
            } else {
                ui.visuals().weak_text_color()
            };

            // Draw 6 dots.
            let r = ui.spacing().button_padding.x / 2.0;
            for dy in [-2.0, 0.0, 2.0] {
                for dx in [-1.0, 1.0] {
                    const RADIUS: f32 = 1.0;
                    let pos = rect.center() + egui::vec2(dx, dy) * r;
                    ui.painter().circle_filled(pos, RADIUS, color);
                }
            }
        }
        r
    }
}
