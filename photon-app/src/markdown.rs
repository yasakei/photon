use floem::text::{
    Attrs, AttrsList, FamilyOwned, LineHeightValue, Style, TextLayout, Weight,
};
use photon_core::{language::PhotonLanguage, syntax::Syntax};
use lapce_xi_rope::Rope;
use lsp_types::MarkedString;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag};
use smallvec::SmallVec;

use crate::config::{PhotonConfig, color::PhotonColor};

#[derive(Clone, Debug)]
pub struct LinkSpan {
    /// Byte range of the link text inside the layout.
    pub range: std::ops::Range<usize>,
    pub url: String,
}

/// Find bare `http(s)://` URLs in already-built text (LSP hovers often
/// contain URLs that are not markdown links).
fn find_bare_urls(text: &str) -> Vec<LinkSpan> {
    let mut links = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let rest = &text[i..];
        let start = if let Some(pos) = rest.find("https://") {
            (i + pos, "https://".len())
        } else if let Some(pos) = rest.find("http://") {
            (i + pos, "http://".len())
        } else {
            break;
        };
        let mut end = start.0 + start.1;
        while end < bytes.len() {
            let c = bytes[end] as char;
            if c.is_whitespace() || "<>)]\"'".contains(c) {
                break;
            }
            end += 1;
        }
        // Trim trailing punctuation that is rarely part of the URL.
        while end > start.0 + start.1
            && ".,;:!?".contains(bytes[end - 1] as char)
        {
            end -= 1;
        }
        if end > start.0 + start.1 {
            links.push(LinkSpan {
                range: start.0..end,
                url: text[start.0..end].to_string(),
            });
        }
        i = end.max(start.0 + 1);
    }
    links
}

#[derive(Clone)]
pub enum MarkdownContent {
    Text {
        layout: TextLayout,
        links: Vec<LinkSpan>,
    },
    Image { url: String, title: String },
    Separator,
}

pub fn parse_markdown(
    text: &str,
    line_height: f64,
    config: &PhotonConfig,
) -> Vec<MarkdownContent> {
    let mut res = Vec::new();

    let mut current_text = String::new();
    let code_font_family: Vec<FamilyOwned> =
        FamilyOwned::parse_list(&config.editor.font_family).collect();

    let default_attrs = Attrs::new()
        .color(config.color(PhotonColor::EDITOR_FOREGROUND))
        .font_size(config.ui.font_size() as f32)
        .line_height(LineHeightValue::Normal(line_height as f32));
    let mut attr_list = AttrsList::new(default_attrs.clone());

    let mut builder_dirty = false;

    let mut pos = 0;

    let mut tag_stack: SmallVec<[(usize, Tag); 4]> = SmallVec::new();

    // Link spans for the text currently being built; attached to the layout
    // when it is flushed so clicks can open them.
    let mut pending_links: Vec<LinkSpan> = Vec::new();

    let parser = Parser::new_ext(
        text,
        Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_HEADING_ATTRIBUTES,
    );
    let mut last_text = CowStr::from("");
    // Whether we should add a newline on the next entry
    // This is used so that we don't emit newlines at the very end of the generation
    let mut add_newline = false;
    for event in parser {
        // Add the newline since we're going to be outputting more
        if add_newline {
            current_text.push('\n');
            builder_dirty = true;
            pos += 1;
            add_newline = false;
        }

        match event {
            Event::Start(tag) => {
                tag_stack.push((pos, tag));
            }
            Event::End(end_tag) => {
                if let Some((start_offset, tag)) = tag_stack.pop() {
                    if end_tag != tag.to_end() {
                        tracing::warn!("Mismatched markdown tag");
                        continue;
                    }

                    if let Some(attrs) = attribute_for_tag(
                        default_attrs.clone(),
                        &tag,
                        &code_font_family,
                        config,
                    ) {
                        attr_list
                            .add_span(start_offset..pos.max(start_offset), attrs);
                    }

                    if should_add_newline_after_tag(&tag) {
                        add_newline = true;
                    }

                    if let Tag::Link { dest_url, .. } = &tag {
                        // A link whose text crossed an image flush has stale
                        // offsets; drop it rather than misattribute clicks.
                        if start_offset <= pos {
                            pending_links.push(LinkSpan {
                                range: start_offset..pos,
                                url: dest_url.to_string(),
                            });
                        }
                    }

                    match &tag {
                        Tag::CodeBlock(kind) => {
                            let language =
                                if let CodeBlockKind::Fenced(language) = kind {
                                    md_language_to_photon_language(language)
                                } else {
                                    None
                                };

                            highlight_as_code(
                                &mut attr_list,
                                default_attrs.clone().family(&code_font_family),
                                language,
                                &last_text,
                                start_offset,
                                config,
                            );
                            builder_dirty = true;
                        }
                        Tag::Image {
                            link_type: _,
                            dest_url: dest,
                            title,
                            id: _,
                        } => {
                            // TODO: Are there any link types that would change how the
                            // image is rendered?

                            if builder_dirty {
                                pending_links
                                    .extend(find_bare_urls(&current_text));
                                let mut text_layout = TextLayout::new();
                                text_layout.set_text(&current_text, attr_list, None);
                                res.push(MarkdownContent::Text {
                                    layout: text_layout,
                                    links: std::mem::take(&mut pending_links),
                                });
                                attr_list = AttrsList::new(default_attrs.clone());
                                current_text.clear();
                                pos = 0;
                                builder_dirty = false;
                            }

                            res.push(MarkdownContent::Image {
                                url: dest.to_string(),
                                title: title.to_string(),
                            });
                        }
                        _ => {
                            // Presumably?
                            builder_dirty = true;
                        }
                    }
                } else {
                    tracing::warn!("Unbalanced markdown tag")
                }
            }
            Event::Text(text) => {
                if let Some((_, tag)) = tag_stack.last() {
                    if should_skip_text_in_tag(tag) {
                        continue;
                    }
                }
                current_text.push_str(&text);
                pos += text.len();
                last_text = text;
                builder_dirty = true;
            }
            Event::Code(text) => {
                attr_list.add_span(
                    pos..pos + text.len(),
                    default_attrs.clone().family(&code_font_family),
                );
                current_text.push_str(&text);
                pos += text.len();
                builder_dirty = true;
            }
            // TODO: Some minimal 'parsing' of html could be useful here, since some things use
            // basic html like `<code>text</code>`.
            Event::Html(text) => {
                attr_list.add_span(
                    pos..pos + text.len(),
                    default_attrs
                        .clone()
                        .family(&code_font_family)
                        .color(config.color(PhotonColor::MARKDOWN_BLOCKQUOTE)),
                );
                current_text.push_str(&text);
                pos += text.len();
                builder_dirty = true;
            }
            Event::HardBreak => {
                current_text.push('\n');
                pos += 1;
                builder_dirty = true;
            }
            Event::SoftBreak => {
                current_text.push(' ');
                pos += 1;
                builder_dirty = true;
            }
            Event::Rule => {}
            Event::FootnoteReference(_text) => {}
            Event::TaskListMarker(_text) => {}
            Event::InlineHtml(_) => {} // TODO(panekj): Implement
            Event::InlineMath(_) => {} // TODO(panekj): Implement
            Event::DisplayMath(_) => {} // TODO(panekj): Implement
        }
    }

    if builder_dirty {
        pending_links.extend(find_bare_urls(&current_text));
        let mut text_layout = TextLayout::new();
        text_layout.set_text(&current_text, attr_list, None);
        res.push(MarkdownContent::Text {
            layout: text_layout,
            links: pending_links,
        });
    }

    res
}

fn attribute_for_tag<'a>(
    default_attrs: Attrs<'a>,
    tag: &Tag,
    code_font_family: &'a [FamilyOwned],
    config: &PhotonConfig,
) -> Option<Attrs<'a>> {
    use pulldown_cmark::HeadingLevel;
    match tag {
        Tag::Heading {
            level,
            id: _,
            classes: _,
            attrs: _,
        } => {
            // The size calculations are based on the em values given at
            // https://drafts.csswg.org/css2/#html-stylesheet
            let font_scale = match level {
                HeadingLevel::H1 => 2.0,
                HeadingLevel::H2 => 1.5,
                HeadingLevel::H3 => 1.17,
                HeadingLevel::H4 => 1.0,
                HeadingLevel::H5 => 0.83,
                HeadingLevel::H6 => 0.75,
            };
            let font_size = font_scale * config.ui.font_size() as f64;
            Some(
                default_attrs
                    .font_size(font_size as f32)
                    .weight(Weight::BOLD),
            )
        }
        Tag::BlockQuote(_block_quote) => Some(
            default_attrs
                .style(Style::Italic)
                .color(config.color(PhotonColor::MARKDOWN_BLOCKQUOTE)),
        ),
        Tag::CodeBlock(_) => Some(default_attrs.family(code_font_family)),
        Tag::Emphasis => Some(default_attrs.style(Style::Italic)),
        Tag::Strong => Some(default_attrs.weight(Weight::BOLD)),
        // TODO: Strikethrough support
        Tag::Link {
            link_type: _,
            dest_url: _,
            title: _,
            id: _,
        } => {
            // Clicks are handled via the recorded link spans
            Some(default_attrs.color(config.color(PhotonColor::EDITOR_LINK)))
        }
        // All other tags are currently ignored
        _ => None,
    }
}

/// Decides whether newlines should be added after a specific markdown tag
fn should_add_newline_after_tag(tag: &Tag) -> bool {
    !matches!(
        tag,
        Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. }
    )
}

/// Whether it should skip the text node after a specific tag  
/// For example, images are skipped because it emits their title as a separate text node.  
fn should_skip_text_in_tag(tag: &Tag) -> bool {
    matches!(tag, Tag::Image { .. })
}

fn md_language_to_photon_language(lang: &str) -> Option<PhotonLanguage> {
    // TODO: There are many other names commonly used that should be supported
    PhotonLanguage::from_name(lang)
}

/// Highlight the text in a richtext builder like it was a markdown codeblock
pub fn highlight_as_code(
    attr_list: &mut AttrsList,
    default_attrs: Attrs,
    language: Option<PhotonLanguage>,
    text: &str,
    start_offset: usize,
    config: &PhotonConfig,
) {
    let syntax = language.map(Syntax::from_language);

    let styles = syntax
        .map(|mut syntax| {
            syntax.parse(0, Rope::from(text), None);
            syntax.styles
        })
        .unwrap_or(None);

    if let Some(styles) = styles {
        for (range, style) in styles.iter() {
            if let Some(color) = style
                .fg_color
                .as_ref()
                .and_then(|fg| config.style_color(fg))
            {
                attr_list.add_span(
                    start_offset + range.start..start_offset + range.end,
                    default_attrs.clone().color(color),
                );
            }
        }
    }
}

pub fn from_marked_string(
    text: MarkedString,
    config: &PhotonConfig,
) -> Vec<MarkdownContent> {
    match text {
        MarkedString::String(text) => parse_markdown(&text, 1.8, config),
        // This is a short version of a code block
        MarkedString::LanguageString(code) => {
            // TODO: We could simply construct the MarkdownText directly
            // Simply construct the string as if it was written directly
            parse_markdown(
                &format!("```{}\n{}\n```", code.language, code.value),
                1.8,
                config,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MarkdownContent, find_bare_urls, parse_markdown};
    use crate::config::PhotonConfig;

    #[test]
    fn gopls_style_hover_records_link() {
        let config = PhotonConfig::default();
        let text = "Unlock unlocks m. See `Mutex`.\n\n[(sync.Mutex).Unlock on pkg.go.dev](https://pkg.go.dev/sync#Mutex.Unlock)\n";
        let contents = parse_markdown(text, 1.8, &config);
        let mut links = Vec::new();
        for content in &contents {
            if let MarkdownContent::Text { layout: _, links: item_links } = content
            {
                links.extend(item_links.iter().cloned());
            }
        }
        assert_eq!(links.len(), 1, "expected one link, got {links:?}");
        assert_eq!(links[0].url, "https://pkg.go.dev/sync#Mutex.Unlock");
    }

    #[test]
    fn bare_urls_are_found() {
        let text = "see https://pkg.go.dev/sync#Mutex.Unlock, ok?";
        let links = find_bare_urls(text);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://pkg.go.dev/sync#Mutex.Unlock");
        assert_eq!(&text[links[0].range.clone()], links[0].url);

        let text = "no links here (https://x.y)";
        let links = find_bare_urls(text);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://x.y");

        assert!(find_bare_urls("nothing here").is_empty());
    }
}

pub fn from_plaintext(
    text: &str,
    line_height: f64,
    config: &PhotonConfig,
) -> Vec<MarkdownContent> {
    let mut text_layout = TextLayout::new();
    text_layout.set_text(
        text,
        AttrsList::new(
            Attrs::new()
                .font_size(config.ui.font_size() as f32)
                .line_height(LineHeightValue::Normal(line_height as f32)),
        ),
        None,
    );
    vec![MarkdownContent::Text {
        layout: text_layout,
        links: Vec::new(),
    }]
}
