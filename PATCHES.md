# Local patches

Handler's fork of `gpui-component` 0.5.1 (crates.io) with the maintained changes below,
licensed under Apache-2.0; see `LICENSE-APACHE`. The first commit is the unmodified
published package; the second applies these patches.

## Markdown list item width

This is the minimal backport of upstream
[PR #2158](https://github.com/longbridge/gpui-component/pull/2158), commit
[`5ce0ea4f`](https://github.com/longbridge/gpui-component/commit/5ce0ea4f).
`src/text/node.rs` adds `w_full().min_w_0()` to the outer list item and its
row, and `flex_1().min_w_0()` to the two `overflow_hidden` paragraph wrappers
in `Node::render_list_item`. Without those constraints, GPUI allocates width
for the marker but collapses its sibling paragraph to zero, leaving visible
bullets and no list text. The repair is layout-only and preserves the upstream
Markdown AST, including nested, ordered, task, and fenced-code content.

## Markdown hard breaks

`src/text/format/markdown.rs` preserves Markdown hard breaks by appending a
newline for `mdast::Break` nodes while parsing inline content. The focused test
keeps the break between adjacent bold, code, and link spans.

## Markdown table layout

`src/text/node.rs` assigns each table a bounded, content-based flex basis per
column, then lets the columns shrink or grow to fit the table's available width.
Cell text wraps within those columns, so narrow artifact panes keep all table
content visible while prose outside the table retains its normal wrapping.

## Input caret lifecycle

`BlinkCursor` runs only between an explicit start and stop. Programmatic edits
to an unfocused field don't start a timer, repeated focus doesn't create another
loop, and stopping cancels both blinking and a pending typing pause. Text and
OTP fields stop the caret on window deactivation and restart only when focused.
Timers use GPUI's executor so lifecycle regressions can advance a test clock.

## Confidential inputs and presentation

`src/input/state.rs` adds `confidential()` to disable clipboard exports and undo
retention for private drafts and withholds surrounding text from platform input queries. `src/input/input.rs`, `state.rs` and `element.rs`
carry an optional placeholder color. Input content gets a full-width vertical
flex parent so its text does not collapse in constrained layouts.

## Enter submission

`src/input/state.rs` adds `secondary_enter_submits()` so a multiline composer can
submit with secondary Enter without inserting a newline. Single-line inputs emit
the Enter event consistently. The focused Enter tests cover both input modes.

## Resizable panel retention

`src/resizable/mod.rs` and `panel.rs` add `preserve_size_on_container_resize()`:
panels retain their requested sizes across container changes, including initial
layout and panel replacement, rather than redistributing every pane. The local
size-calculation tests cover retained and default sizing.

## Markdown inspection

`src/text/node.rs` labels paragraph elements with their source span through
`debug_selector`, allowing native UI inspection to identify rendered paragraphs.
The remaining baseline differences are formatting and tests for the changes above.
