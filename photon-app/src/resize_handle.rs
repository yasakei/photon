use std::rc::Rc;

use floem::{
    View, ViewId,
    context::EventCx,
    event::{Event, EventPropagation},
    peniko::kurbo::Point,
};

/// A thin drag handle that correctly tracks pointer drags in window coordinates.
///
/// Why this exists:
/// - `pointer_event.pos` delivered to `on_event` handlers is local to the view.
///   For a 4px wide handle that moves with the content being resized, a local
///   delta (`pos - drag_start`) stays near zero, so resizing stalls.
/// - Re-reading the current size on every `PointerMove` while keeping the
///   original `drag_start` double-counts the delta (size already grew + full
///   delta from start applied again), causing overshoot / lag.
/// - Declarative `on_event(PointerMove)` handlers are gated by
///   `rect.contains(pos)`. Once the cursor leaves the 4px handle, moves stop
///   firing. `request_active()` alone does not fix this for `on_event`; only
///   `event_before_children` in a custom `View` keeps receiving moves outside
///   the view while active (same pattern as Floem's `Slider` / `ResizableStack`).
///
/// This view:
/// - handles events in `event_before_children`, so it keeps receiving moves
///   while active, even when the cursor leaves the handle,
/// - converts local positions to window positions via
///   `layout_rect().origin() + local`, so the delta is stable even though the
///   handle itself moves during the drag,
/// - reports window positions to the caller, which should snapshot the initial
///   size on down and compute `new = initial + (current_window - start_window)`.
pub struct ResizeHandle {
    id: ViewId,
    on_down: Option<Rc<dyn Fn(Point)>>,
    on_move: Option<Rc<dyn Fn(Point)>>,
    on_up: Option<Rc<dyn Fn()>>,
    dragging: bool,
}

pub fn resize_handle() -> ResizeHandle {
    ResizeHandle {
        id: ViewId::new(),
        on_down: None,
        on_move: None,
        on_up: None,
        dragging: false,
    }
}

impl ResizeHandle {
    pub fn on_down(mut self, f: impl Fn(Point) + 'static) -> Self {
        self.on_down = Some(Rc::new(f));
        self
    }

    pub fn on_move(mut self, f: impl Fn(Point) + 'static) -> Self {
        self.on_move = Some(Rc::new(f));
        self
    }

    pub fn on_up(mut self, f: impl Fn() + 'static) -> Self {
        self.on_up = Some(Rc::new(f));
        self
    }

    fn window_pos(&self, local: Point) -> Point {
        let origin = self.id.layout_rect().origin();
        Point::new(origin.x + local.x, origin.y + local.y)
    }
}

impl View for ResizeHandle {
    fn id(&self) -> ViewId {
        self.id
    }

    fn event_before_children(
        &mut self,
        cx: &mut EventCx,
        event: &Event,
    ) -> EventPropagation {
        match event {
            Event::PointerDown(e) => {
                if !e.button.is_primary() {
                    return EventPropagation::Continue;
                }
                // Only start a drag when the press begins inside the handle.
                if let Some(size) = self.id.get_size() {
                    if !size.to_rect().contains(e.pos) {
                        return EventPropagation::Continue;
                    }
                }
                cx.update_active(self.id);
                self.dragging = true;
                let window = self.window_pos(e.pos);
                if let Some(f) = &self.on_down {
                    f(window);
                }
                EventPropagation::Stop
            }
            Event::PointerMove(e) => {
                if self.dragging {
                    if !cx.is_active(self.id) {
                        // Lost active without an up (e.g. another view stole it).
                        self.dragging = false;
                        if let Some(f) = &self.on_up {
                            f();
                        }
                        return EventPropagation::Continue;
                    }
                    let window = self.window_pos(e.pos);
                    if let Some(f) = &self.on_move {
                        f(window);
                    }
                    return EventPropagation::Stop;
                }
                EventPropagation::Continue
            }
            Event::PointerUp(_) => {
                if self.dragging {
                    self.dragging = false;
                    if let Some(f) = &self.on_up {
                        f();
                    }
                    return EventPropagation::Stop;
                }
                EventPropagation::Continue
            }
            Event::FocusLost | Event::WindowLostFocus => {
                if self.dragging {
                    self.dragging = false;
                    if let Some(f) = &self.on_up {
                        f();
                    }
                }
                EventPropagation::Continue
            }
            _ => EventPropagation::Continue,
        }
    }
}
