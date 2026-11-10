// Copyright 2026 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Tests for span edge boxes (`InlineBoxKind::SpanStart` and `InlineBoxKind::SpanEnd`): the margin,
//! border and padding at the edges of a CSS inline box.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::utils::ColorBrush;
use super::utils::fonts::{FONT_FAMILY_LIST, create_font_context};
use crate::{
    FontFamily, InlineBox, InlineBoxKind, Layout, LayoutContext, LineHeight, PositionedLayoutItem,
    TextStyle, VerticalAlign, WhiteSpaceCollapse,
};

const EDGE: f32 = 30.;

fn style() -> TextStyle<'static, 'static, ColorBrush> {
    TextStyle {
        font_family: FontFamily::from(FONT_FAMILY_LIST),
        font_size: 20.,
        line_height: LineHeight::Absolute(30.),
        white_space_collapse: WhiteSpaceCollapse::Collapse,
        ..TextStyle::default()
    }
}

fn edge(kind: InlineBoxKind) -> InlineBox {
    InlineBox {
        id: kind as u64,
        kind,
        index: 0,
        width: EDGE,
        height: 0.,
        baseline: None,
        vertical_align: VerticalAlign::BASELINE,
    }
}

enum Item {
    Text(&'static str),
    /// A span with edges around its items.
    Span(Vec<Item>),
}

fn build(items: &[Item]) -> (Layout<ColorBrush>, String) {
    fn push(builder: &mut crate::TreeBuilder<'_, ColorBrush>, items: &[Item]) {
        for item in items {
            match item {
                Item::Text(text) => builder.push_text(text),
                Item::Span(items) => {
                    builder.push_style_span(style());
                    builder.push_inline_box(edge(InlineBoxKind::SpanStart));
                    push(builder, items);
                    builder.push_inline_box(edge(InlineBoxKind::SpanEnd));
                    builder.pop_style_span();
                }
            }
        }
    }
    let mut fcx = create_font_context();
    let mut lcx: LayoutContext<ColorBrush> = LayoutContext::new();
    let root = style();
    let mut builder = lcx.tree_builder(&mut fcx, 1., false, &root);
    push(&mut builder, items);
    builder.build()
}

/// The kinds of the span edge boxes on each line.
fn edges_per_line(layout: &Layout<ColorBrush>) -> Vec<Vec<InlineBoxKind>> {
    layout
        .lines()
        .map(|line| {
            line.items()
                .filter_map(|item| match item {
                    PositionedLayoutItem::InlineBox(b) => Some(b.kind),
                    PositionedLayoutItem::GlyphRun(_) => None,
                })
                .collect()
        })
        .collect()
}

fn line_widths(layout: &Layout<ColorBrush>) -> Vec<f32> {
    layout.lines().map(|line| line.metrics().advance).collect()
}

#[test]
fn white_space_collapses_across_edges() {
    use Item::*;
    let (_, text) = build(&[Text("a "), Span(vec![Text(" b ")]), Text(" c")]);
    assert_eq!(text, "a b c");
    let (_, text) = build(&[Span(vec![Text(" a ")])]);
    assert_eq!(
        text, "a",
        "white space at the start and end of the text is removed"
    );
    let (_, text) = build(&[Text("a "), Span(vec![]), Text(" b")]);
    assert_eq!(
        text, "a b",
        "an empty span's edges don't keep a second space"
    );
}

#[test]
fn edges_take_space_without_breaking_the_line() {
    use Item::*;
    let (mut layout, _) = build(&[Text("aaa"), Span(vec![Text("bbb")]), Text("ccc")]);
    layout.break_all_lines(None);
    let (mut plain, _) = build(&[Text("aaabbbccc")]);
    plain.break_all_lines(None);
    let width = line_widths(&layout)[0];
    assert!(
        (width - (line_widths(&plain)[0] + 2. * EDGE)).abs() < 0.01,
        "{width}"
    );
    // Narrower than the line: there is no break opportunity, so the line overflows.
    layout.break_all_lines(Some(width - 1.));
    assert_eq!(layout.lines().count(), 1);
}

#[test]
fn a_start_edge_that_starts_a_line_stays_with_the_content_after_it() {
    use Item::*;
    let (mut layout, _) = build(&[Text("aaa "), Span(vec![Text("bbb")])]);
    // Too narrow for the span on a line of its own: it overflows the second line.
    layout.break_all_lines(Some(EDGE + 1.));
    assert_eq!(
        edges_per_line(&layout),
        [
            vec![],
            vec![InlineBoxKind::SpanStart, InlineBoxKind::SpanEnd]
        ]
    );
}

#[test]
fn a_start_edge_wraps_with_the_content_after_it() {
    use Item::*;
    let (mut layout, _) = build(&[Text("aaa "), Span(vec![Text("bbb")])]);
    layout.break_all_lines(None);
    let width = line_widths(&layout)[0];
    layout.break_all_lines(Some(width - 1.));
    assert_eq!(
        edges_per_line(&layout),
        [
            vec![],
            vec![InlineBoxKind::SpanStart, InlineBoxKind::SpanEnd]
        ]
    );
}

#[test]
fn an_end_edge_stays_with_the_content_before_it() {
    use Item::*;
    let (mut layout, _) = build(&[Span(vec![Text("aaa")]), Text(" bbb")]);
    layout.break_all_lines(None);
    let width = line_widths(&layout)[0];
    layout.break_all_lines(Some(width - 1.));
    assert_eq!(
        edges_per_line(&layout),
        [
            vec![InlineBoxKind::SpanStart, InlineBoxKind::SpanEnd],
            vec![]
        ]
    );
}

#[test]
fn min_content_keeps_edges_with_their_content() {
    use Item::*;
    let (mut layout, _) = build(&[Text("aaa "), Span(vec![Text("aaa")])]);
    let (mut plain, _) = build(&[Text("aaa aaa")]);
    let widths = layout.calculate_content_widths();
    let plain_widths = plain.calculate_content_widths();
    assert!(
        (widths.min - (plain_widths.min + 2. * EDGE)).abs() < 0.01,
        "{widths:?}"
    );
    assert!(
        (widths.max - (plain_widths.max + 2. * EDGE)).abs() < 0.01,
        "{widths:?}"
    );
    layout.break_all_lines(Some(widths.min));
    assert_eq!(layout.lines().count(), 2);
    plain.break_all_lines(None);
}

#[test]
fn edges_dont_grow_the_line() {
    use Item::*;
    let (mut layout, _) = build(&[Span(vec![Text("aaa")])]);
    let (mut plain, _) = build(&[Text("aaa")]);
    layout.break_all_lines(None);
    plain.break_all_lines(None);
    assert_eq!(layout.height(), plain.height());
}

#[test]
fn a_line_holding_only_an_empty_span_has_its_box() {
    use Item::*;
    let (mut layout, _) = build(&[Span(vec![])]);
    layout.break_all_lines(None);
    assert_eq!(layout.lines().count(), 1);
    assert_eq!(layout.height(), 30.);
    assert_eq!(line_widths(&layout)[0], 2. * EDGE);
}

#[test]
fn span_content_area_is_the_fonts_ascent_and_descent() {
    use Item::*;
    let (mut layout, _) = build(&[Text("a"), Span(vec![Text("b")])]);
    layout.break_all_lines(None);
    let line = layout.lines().next().unwrap();
    let run = line
        .items()
        .find_map(|item| match item {
            PositionedLayoutItem::GlyphRun(run) if run.style_index() != 0 => Some(run),
            _ => None,
        })
        .unwrap();
    let metrics = run.run().font_metrics();
    let (top, bottom) = line.span_content_area(run.style_index()).unwrap();
    assert!((top - (run.baseline() - metrics.ascent)).abs() < 0.01);
    assert!((bottom - (run.baseline() + metrics.descent)).abs() < 0.01);
}
