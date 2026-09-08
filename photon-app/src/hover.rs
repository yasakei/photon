use floem::{
    Renderer, View, ViewId,
    context::{ComputeLayoutCx, EventCx, LayoutCx, PaintCx},
    event::{Event, EventPropagation},
    peniko::kurbo::{Point, Rect},
    reactive::{RwSignal, Scope, SignalGet, SignalUpdate, create_rw_signal},
    style::{CursorStyle, Style},
    taffy::prelude::NodeId,
    text::TextLayout,
    views::Decorators,
    views::editor::id::EditorId,
};

use crate::{
    command::InternalCommand, listener::Listener, markdown::LinkSpan,
};

#[derive(Clone)]
pub struct HoverData {
    pub active: RwSignal<bool>,
    pub offset: RwSignal<usize>,
    pub editor_id: RwSignal<EditorId>,
    pub content: RwSignal<Vec<MarkdownContent>>,
    pub layout_rect: RwSignal<Rect>,
}

impl HoverData {
    pub fn new(cx: Scope) -> Self {
        Self {
            active: cx.create_rw_signal(false),
            offset: cx.create_rw_signal(0),
            content: cx.create_rw_signal(Vec::new()),
            editor_id: cx.create_rw_signal(EditorId::next()),
            layout_rect: cx.create_rw_signal(Rect::ZERO),
        }
    }
}

use crate::markdown::MarkdownContent;

/// Rich text that opens markdown links on click.
///
/// Mirrors floem's `RichText` wrapping logic (clone + re-wrap when wider
/// than available) and hit-tests the *painted* layout, so clicks map to the
/// same lines the user sees. Assumes no padding on the view itself.
pub struct ClickableHoverText {
    id: ViewId,
    base: TextLayout,
    links: Vec<LinkSpan>,
    internal_command: Listener<InternalCommand>,
    text_node: Option<NodeId>,
    available: Option<TextLayout>,
    available_width: Option<f32>,
    over_link: RwSignal<bool>,
}

pub fn clickable_hover_text(
    layout: TextLayout,
    links: Vec<LinkSpan>,
    internal_command: Listener<InternalCommand>,
) -> impl View {
    let id = ViewId::new();
    let over_link = create_rw_signal(false);
    ClickableHoverText {
        id,
        base: layout,
        links,
        internal_command,
        text_node: None,
        available: None,
        available_width: None,
        over_link,
    }
    .style(move |s| {
        s.max_width(600.0).cursor(if over_link.get() {
            CursorStyle::Pointer
        } else {
            CursorStyle::Default
        })
    })
}

impl ClickableHoverText {
    fn painted_layout(&self) -> &TextLayout {
        self.available.as_ref().unwrap_or(&self.base)
    }

    fn text_origin(&self) -> Point {
        self.text_node
            .and_then(|node| self.id.taffy_layout(node))
            .map(|layout| {
                Point::new(
                    layout.location.x as f64,
                    layout.location.y as f64,
                )
            })
            .unwrap_or_default()
    }

    /// URL under `point` (view-local coordinates), if any.
    ///
    /// Glyph offsets are relative to their logical line ("index of cluster
    /// in original line"), while link spans are document-global, so each
    /// glyph is translated via its line's base offset. Works across wrapped
    /// lines. Assumes '\n' line endings (what LSP hovers use).
    fn link_at(&self, point: Point) -> Option<String> {
        let layout = self.painted_layout();
        let size = layout.size();
        if point.x < 0.0
            || point.y < 0.0
            || point.x > size.width
            || point.y > size.height
        {
            return None;
        }
        let mut line_starts = Vec::with_capacity(layout.lines().len());
        let mut offset = 0usize;
        for line in layout.lines() {
            line_starts.push(offset);
            offset += line.text().len() + 1;
        }
        for run in layout.layout_runs() {
            if point.y < run.line_top as f64
                || point.y >= run.line_top as f64 + run.line_height as f64
            {
                continue;
            }
            let Some(base) = line_starts.get(run.line_i).copied() else {
                continue;
            };
            for glyph in run.glyphs {
                if point.x >= glyph.x as f64
                    && point.x <= glyph.x as f64 + glyph.w as f64
                {
                    let index = base + glyph.start;
                    return self
                        .links
                        .iter()
                        .find(|link| link.range.contains(&index))
                        .map(|link| link.url.clone());
                }
            }
        }
        None
    }
}

impl View for ClickableHoverText {
    fn id(&self) -> ViewId {
        self.id
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "Clickable Hover Text".into()
    }

    fn layout(&mut self, cx: &mut LayoutCx) -> NodeId {
        cx.layout_node(self.id(), true, |_cx| {
            let size = self.base.size();
            let width = size.width as f32;
            let mut height = size.height as f32;

            if let Some(layout) = self.available.as_ref() {
                height = height.max(layout.size().height as f32);
            }

            if self.text_node.is_none() {
                self.text_node = Some(self.id.new_taffy_node());
            }
            let text_node = self.text_node.unwrap();

            let style =
                Style::new().width(width).height(height).to_taffy_style();
            self.id.set_taffy_style(text_node, style);
            vec![text_node]
        })
    }

    fn compute_layout(&mut self, _cx: &mut ComputeLayoutCx) -> Option<Rect> {
        let layout = self.id.get_layout().unwrap_or_default();
        let width = self.base.size().width as f32;
        let available_width = layout.size.width;
        if width > available_width {
            if self.available_width != Some(available_width) {
                let mut wrapped = self.base.clone();
                wrapped.set_size(available_width, f32::MAX);
                self.available = Some(wrapped);
                self.available_width = Some(available_width);
                self.id.request_layout();
            }
        } else if self.available.is_some() {
            self.available = None;
            self.available_width = None;
            self.id.request_layout();
        }

        None
    }

    fn paint(&mut self, cx: &mut PaintCx) {
        let origin = self.text_origin();
        if let Some(layout) = self.available.as_ref() {
            cx.draw_text(layout, origin);
        } else {
            cx.draw_text(&self.base, origin);
        }
    }

    fn event_before_children(
        &mut self,
        _cx: &mut EventCx,
        event: &Event,
    ) -> EventPropagation {
        match event {
            Event::PointerMove(event) => {
                let origin = self.text_origin();
                let point = Point::new(
                    event.pos.x - origin.x,
                    event.pos.y - origin.y,
                );
                self.over_link.set(self.link_at(point).is_some());
                EventPropagation::Continue
            }
            Event::PointerDown(event) if event.button.is_primary() => {
                let origin = self.text_origin();
                let point = Point::new(
                    event.pos.x - origin.x,
                    event.pos.y - origin.y,
                );
                if let Some(url) = self.link_at(point) {
                    self.internal_command.send(InternalCommand::OpenWebUri {
                        uri: url,
                    });
                    return EventPropagation::Stop;
                }
                EventPropagation::Continue
            }
            _ => EventPropagation::Continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use floem::{
        reactive::Scope,
        text::{Attrs, AttrsList},
    };

    use super::*;
    use crate::markdown::LinkSpan;

    fn test_view(text: &str, links: Vec<LinkSpan>) -> ClickableHoverText {
        let cx = Scope::new();
        let mut layout = TextLayout::new();
        layout.set_text(text, AttrsList::new(Attrs::new()), None);
        ClickableHoverText {
            id: ViewId::new(),
            base: layout,
            links,
            internal_command: Listener::new(cx, |_: InternalCommand| {}),
            text_node: None,
            available: None,
            available_width: None,
            over_link: cx.create_rw_signal(false),
        }
    }

    #[test]
    fn hover_link_hit_finds_link_by_point() {
        let text = "see https://pkg.go.dev/sync for docs";
        let url_start = text.find("https://").unwrap();
        let url_end = url_start + "https://pkg.go.dev/sync".len();
        let view = test_view(
            text,
            vec![LinkSpan {
                range: url_start..url_end,
                url: "https://pkg.go.dev/sync".to_string(),
            }],
        );

        // A point inside the link's glyphs must resolve to its URL.
        let hit = view.base.hit_point(Point::new(200.0, 8.0));
        println!("hit index {0} for x=200 (link {url_start}..{url_end})", hit.index);
        assert!(
            view.link_at(Point::new(200.0, 8.0))
                == Some("https://pkg.go.dev/sync".to_string()),
            "link not found at its own position",
        );
        // Plain text resolves to nothing.
        assert_eq!(view.link_at(Point::new(4.0, 8.0)), None);
    }

    #[test]
    fn hover_link_hit_wrapped_layout() {
        let text = "Unlock unlocks m and a locked https://pkg.go.dev/sync#Mutex.Unlock \
                    is not associated with a particular goroutine at all here yes";
        let url_start = text.find("https://").unwrap();
        let url = "https://pkg.go.dev/sync#Mutex.Unlock";
        let mut view = test_view(
            text,
            vec![LinkSpan {
                range: url_start..url_start + url.len(),
                url: url.to_string(),
            }],
        );
        // Force wrapping like the hover does at small widths (size()
        // performs the shaping pass, mirroring the paint path).
        let mut wrapped = view.base.clone();
        wrapped.set_size(200.0, f32::MAX);
        let _ = wrapped.size();
        // Ground truth from run geometry alone (not from hit logic):
        // line-relative glyph offsets translated via logical line starts.
        let mut line_starts = Vec::new();
        let mut offset = 0usize;
        for line in wrapped.lines() {
            line_starts.push(offset);
            offset += line.text().len() + 1;
        }
        let mut click = None;
        for run in wrapped.layout_runs() {
            let base = line_starts[run.line_i];
            for glyph in run.glyphs {
                let (start, end) = (base + glyph.start, base + glyph.end);
                if start >= url_start + 2 && end <= url_start + url.len() {
                    click = Some(Point::new(
                        glyph.x as f64 + glyph.w as f64 / 2.0,
                        run.line_top as f64 + run.line_height as f64 / 2.0,
                    ));
                    break;
                }
            }
            if click.is_some() {
                break;
            }
        }
        let click = click.expect("link has no glyph");
        view.available = Some(wrapped);
        assert_eq!(
            view.link_at(click),
            Some(url.to_string()),
            "wrapped link not hit at {click:?}"
        );
        // Clicking the first line's plain text still misses.
        assert_eq!(view.link_at(Point::new(4.0, 4.0)), None);
    }
}
