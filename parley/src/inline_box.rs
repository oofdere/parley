// Copyright 2024 the Parley Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::VerticalAlign;

/// A box to be laid out inline with text
#[derive(PartialEq, Debug, Clone)]
pub struct InlineBox {
    /// User-specified identifier for the box, which can be used by the user to determine which box in
    /// parley's output corresponds to which box in its input.
    pub id: u64,
    /// Whether the box is in-flow (takes up space in the layout) or out-of-flow (e.g. absolutely positioned or floated)
    pub kind: InlineBoxKind,
    /// The byte offset into the underlying text string at which the box should be placed.
    /// This must not be within a Unicode code point.
    pub index: usize,
    /// The width of the box in pixels
    pub width: f32,
    /// The height of the box in pixels
    pub height: f32,
    /// The baseline of the box in pixels, relative to the top of the box
    ///
    /// If `None`, the baseline is the box's bottom edge.
    pub baseline: Option<f32>,
    /// Vertical alignment of the box within its line, relative to the enclosing style span.
    pub vertical_align: VerticalAlign,
}

/// An [`InlineBox`] together with the style index of the span that contains it.
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct LayoutInlineBox {
    pub(crate) inline_box: InlineBox,
    /// Style index of the span containing the box, against which the box's `vertical_align`
    /// is resolved. The box itself has no style of its own.
    pub(crate) parent_style_index: u16,
    /// Offset of the box's baseline above the baseline of the independent aligned subtree it
    /// belongs to (that of the parent style), resolved when the box is placed on a line (box
    /// sizes may change between building and line breaking). Unused for out-of-flow and
    /// `vertical-align: top | bottom` boxes.
    pub(crate) baseline_offset: f32,
}

/// Whether a box is in-flow (takes up space in the layout) or out-of-flow (e.g. absolutely positioned)
/// or custom-out-of-flow (line-breaking should yield control flow)
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum InlineBoxKind {
    /// `InFlow` boxes take up space in the layout and flow in line with text
    ///
    /// They correspond to `display: inline-block` boxes in CSS.
    InFlow,
    /// `OutOfFlow` boxes are assigned a position as if they were a zero-sized inline box, but
    /// do not take up space in the layout.
    ///
    /// They correspond to `position: absolute` boxes in CSS.
    OutOfFlow,
    /// `CustomOutOfFlow` boxes also do not take up space in the layout, but they are not assigned a position
    /// by Parley. When they are encountered, control flow is yielded back to the caller who is then responsible
    /// for laying out the box.
    ///
    /// They can be used to implement advanced layout modes such as CSS's `float`
    CustomOutOfFlow,
    /// `SpanStart` boxes take up space at the start of the style span they are pushed in, without
    /// growing the line beyond that span's own box.
    ///
    /// They are the margin, border and padding at the inline-start edge of a CSS inline box (CSS 2
    /// §10.3.1): unlike an `InFlow` box, they are no line break opportunity, and stay on the line of
    /// the content that follows them. Collapsible white space collapses across them, and a line
    /// holding one holds the span's box.
    SpanStart,
    /// `SpanEnd` boxes take up space at the end of the style span they are pushed in, without
    /// growing the line beyond that span's own box.
    ///
    /// They are the margin, border and padding at the inline-end edge of a CSS inline box: unlike
    /// an `InFlow` box, they are no line break opportunity, and stay on the line of the content
    /// that precedes them.
    SpanEnd,
}

impl InlineBoxKind {
    /// Whether this is a [`SpanStart`](Self::SpanStart) or [`SpanEnd`](Self::SpanEnd) box.
    pub fn is_span_edge(self) -> bool {
        matches!(self, Self::SpanStart | Self::SpanEnd)
    }
}
