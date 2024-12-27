use crate::{
    display_map::ToDisplayPoint, AnchorRangeExt, Autoscroll, DiffRowHighlight, DisplayPoint,
    Editor, MultiBuffer, RowExt,
};
use collections::BTreeMap;
use futures::Future;
use git::diff::DiffHunkStatus;
use gpui::{
    AnyWindowHandle, AppContext, Keystroke, ModelContext, Pixels, Point, View, ViewContext,
    VisualTestContext, WindowHandle,
};
use itertools::Itertools;
use language::{Buffer, BufferSnapshot, LanguageRegistry};
use multi_buffer::{ExcerptRange, ToPoint};
use parking_lot::RwLock;
use project::{FakeFs, Project};
use std::{
    any::TypeId,
    ops::{Deref, DerefMut, Range},
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use ui::Context;
use util::{
    assert_set_eq,
    test::{generate_marked_text, marked_text_ranges},
};

use super::{build_editor, build_editor_with_project};

pub struct EditorTestContext {
    pub cx: gpui::VisualTestContext,
    pub window: AnyWindowHandle,
    pub editor: View<Editor>,
    pub assertion_cx: AssertionContextManager,
}

impl EditorTestContext {
    pub async fn new(cx: &mut gpui::TestAppContext) -> EditorTestContext {
        let fs = FakeFs::new(cx.executor());
        let root = Self::root_path();
        fs.insert_tree(
            root,
            serde_json::json!({
                ".git": {},
                "file": "",
            }),
        )
        .await;
        let project = Project::test(fs.clone(), [root], cx).await;
        let buffer = project
            .update(cx, |project, cx| {
                project.open_local_buffer(root.join("file"), cx)
            })
            .await
            .unwrap();
        let editor = cx.add_window(|cx| {
            let editor =
                build_editor_with_project(project, MultiBuffer::build_from_buffer(buffer, cx), cx);
            editor.focus(cx);
            editor
        });
        let editor_view = editor.root_view(cx).unwrap();

        cx.run_until_parked();
        Self {
            cx: VisualTestContext::from_window(*editor.deref(), cx),
            window: editor.into(),
            editor: editor_view,
            assertion_cx: AssertionContextManager::new(),
        }
    }

    #[cfg(target_os = "windows")]
    fn root_path() -> &'static Path {
        Path::new("C:\\root")
    }

    #[cfg(not(target_os = "windows"))]
    fn root_path() -> &'static Path {
        Path::new("/root")
    }

    pub async fn for_editor(editor: WindowHandle<Editor>, cx: &mut gpui::TestAppContext) -> Self {
        let editor_view = editor.root_view(cx).unwrap();
        Self {
            cx: VisualTestContext::from_window(*editor.deref(), cx),
            window: editor.into(),
            editor: editor_view,
            assertion_cx: AssertionContextManager::new(),
        }
    }

    pub fn new_multibuffer<const COUNT: usize>(
        cx: &mut gpui::TestAppContext,
        excerpts: [&str; COUNT],
    ) -> EditorTestContext {
        let mut multibuffer = MultiBuffer::new(language::Capability::ReadWrite);
        let buffer = cx.new_model(|cx| {
            for excerpt in excerpts.into_iter() {
                let (text, ranges) = marked_text_ranges(excerpt, false);
                let buffer = cx.new_model(|cx| Buffer::local(text, cx));
                multibuffer.push_excerpts(
                    buffer,
                    ranges.into_iter().map(|range| ExcerptRange {
                        context: range,
                        primary: None,
                    }),
                    cx,
                );
            }
            multibuffer
        });

        let editor = cx.add_window(|cx| {
            let editor = build_editor(buffer, cx);
            editor.focus(cx);
            editor
        });

        let editor_view = editor.root_view(cx).unwrap();
        Self {
            cx: VisualTestContext::from_window(*editor.deref(), cx),
            window: editor.into(),
            editor: editor_view,
            assertion_cx: AssertionContextManager::new(),
        }
    }

    pub fn condition(
        &self,
        predicate: impl FnMut(&Editor, &AppContext) -> bool,
    ) -> impl Future<Output = ()> {
        self.editor
            .condition::<crate::EditorEvent>(&self.cx, predicate)
    }

    #[track_caller]
    pub fn editor<F, T>(&mut self, read: F) -> T
    where
        F: FnOnce(&Editor, &ViewContext<Editor>) -> T,
    {
        self.editor.update(&mut self.cx, |this, cx| read(this, cx))
    }

    #[track_caller]
    pub fn update_editor<F, T>(&mut self, update: F) -> T
    where
        F: FnOnce(&mut Editor, &mut ViewContext<Editor>) -> T,
    {
        self.editor.update(&mut self.cx, update)
    }

    pub fn multibuffer<F, T>(&mut self, read: F) -> T
    where
        F: FnOnce(&MultiBuffer, &AppContext) -> T,
    {
        self.editor(|editor, cx| read(editor.buffer().read(cx), cx))
    }

    pub fn update_multibuffer<F, T>(&mut self, update: F) -> T
    where
        F: FnOnce(&mut MultiBuffer, &mut ModelContext<MultiBuffer>) -> T,
    {
        self.update_editor(|editor, cx| editor.buffer().update(cx, update))
    }

    pub fn buffer_text(&mut self) -> String {
        self.multibuffer(|buffer, cx| buffer.snapshot(cx).text())
    }

    pub fn display_text(&mut self) -> String {
        self.update_editor(|editor, cx| editor.display_text(cx))
    }

    pub fn buffer<F, T>(&mut self, read: F) -> T
    where
        F: FnOnce(&Buffer, &AppContext) -> T,
    {
        self.multibuffer(|multibuffer, cx| {
            let buffer = multibuffer.as_singleton().unwrap().read(cx);
            read(buffer, cx)
        })
    }

    pub fn language_registry(&mut self) -> Arc<LanguageRegistry> {
        self.editor(|editor, cx| {
            editor
                .project
                .as_ref()
                .unwrap()
                .read(cx)
                .languages()
                .clone()
        })
    }

    pub fn update_buffer<F, T>(&mut self, update: F) -> T
    where
        F: FnOnce(&mut Buffer, &mut ModelContext<Buffer>) -> T,
    {
        self.update_multibuffer(|multibuffer, cx| {
            let buffer = multibuffer.as_singleton().unwrap();
            buffer.update(cx, update)
        })
    }

    pub fn buffer_snapshot(&mut self) -> BufferSnapshot {
        self.buffer(|buffer, _| buffer.snapshot())
    }

    pub fn add_assertion_context(&self, context: String) -> ContextHandle {
        self.assertion_cx.add_context(context)
    }

    pub fn assertion_context(&self) -> String {
        self.assertion_cx.context()
    }

    // unlike cx.simulate_keystrokes(), this does not run_until_parked
    // so you can use it to test detailed timing
    pub fn simulate_keystroke(&mut self, keystroke_text: &str) {
        let keystroke = Keystroke::parse(keystroke_text).unwrap();
        self.cx.dispatch_keystroke(self.window, keystroke);
    }

    pub fn run_until_parked(&mut self) {
        self.cx.background_executor.run_until_parked();
    }

    pub fn ranges(&mut self, marked_text: &str) -> Vec<Range<usize>> {
        let (unmarked_text, ranges) = marked_text_ranges(marked_text, false);
        assert_eq!(self.buffer_text(), unmarked_text);
        ranges
    }

    pub fn display_point(&mut self, marked_text: &str) -> DisplayPoint {
        let ranges = self.ranges(marked_text);
        let snapshot = self
            .editor
            .update(&mut self.cx, |editor, cx| editor.snapshot(cx));
        ranges[0].start.to_display_point(&snapshot)
    }

    pub fn pixel_position(&mut self, marked_text: &str) -> Point<Pixels> {
        let display_point = self.display_point(marked_text);
        self.pixel_position_for(display_point)
    }

    pub fn pixel_position_for(&mut self, display_point: DisplayPoint) -> Point<Pixels> {
        self.update_editor(|editor, cx| {
            let newest_point = editor.selections.newest_display(cx).head();
            let pixel_position = editor.pixel_position_of_newest_cursor.unwrap();
            let line_height = editor
                .style()
                .unwrap()
                .text
                .line_height_in_pixels(cx.rem_size());
            let snapshot = editor.snapshot(cx);
            let details = editor.text_layout_details(cx);

            let y = pixel_position.y
                + line_height * (display_point.row().as_f32() - newest_point.row().as_f32());
            let x = pixel_position.x + snapshot.x_for_display_point(display_point, &details)
                - snapshot.x_for_display_point(newest_point, &details);
            Point::new(x, y)
        })
    }

    // Returns anchors for the current buffer using `«` and `»`
    pub fn text_anchor_range(&mut self, marked_text: &str) -> Range<language::Anchor> {
        let ranges = self.ranges(marked_text);
        let snapshot = self.buffer_snapshot();
        snapshot.anchor_before(ranges[0].start)..snapshot.anchor_after(ranges[0].end)
    }

    pub fn set_diff_base(&mut self, diff_base: &str) {
        self.cx.run_until_parked();
        let fs = self
            .update_editor(|editor, cx| editor.project.as_ref().unwrap().read(cx).fs().as_fake());
        let path = self.update_buffer(|buffer, _| buffer.file().unwrap().path().clone());
        fs.set_index_for_repo(
            &Self::root_path().join(".git"),
            &[(path.as_ref(), diff_base.to_string())],
        );
        self.cx.run_until_parked();
    }

    /// Change the editor's text and selections using a string containing
    /// embedded range markers that represent the ranges and directions of
    /// each selection.
    ///
    /// Returns a context handle so that assertion failures can print what
    /// editor state was needed to cause the failure.
    ///
    /// See the `util::test::marked_text_ranges` function for more information.
    pub fn set_state(&mut self, marked_text: &str) -> ContextHandle {
        let state_context = self.add_assertion_context(format!(
            "Initial Editor State: \"{}\"",
            marked_text.escape_debug()
        ));
        let (unmarked_text, selection_ranges) = marked_text_ranges(marked_text, true);
        self.editor.update(&mut self.cx, |editor, cx| {
            editor.set_text(unmarked_text, cx);
            editor.change_selections(Some(Autoscroll::fit()), cx, |s| {
                s.select_ranges(selection_ranges)
            })
        });
        state_context
    }

    /// Only change the editor's selections
    pub fn set_selections_state(&mut self, marked_text: &str) -> ContextHandle {
        let state_context = self.add_assertion_context(format!(
            "Initial Editor State: \"{}\"",
            marked_text.escape_debug()
        ));
        let (unmarked_text, selection_ranges) = marked_text_ranges(marked_text, true);
        self.editor.update(&mut self.cx, |editor, cx| {
            assert_eq!(editor.text(cx), unmarked_text);
            editor.change_selections(Some(Autoscroll::fit()), cx, |s| {
                s.select_ranges(selection_ranges)
            })
        });
        state_context
    }

    /// Assert about the text of the editor, the selections, and the expanded
    /// diff hunks.
    ///
    /// Diff hunks are indicated by lines starting with `+` and `-`.
    #[track_caller]
    pub fn assert_state_with_diff(&mut self, expected_diff: String) {
        let has_diff_markers = expected_diff
            .lines()
            .any(|line| line.starts_with("+") || line.starts_with("-"));
        let expected_diff_text = expected_diff
            .split('\n')
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    String::new()
                } else if has_diff_markers {
                    line.to_string()
                } else {
                    format!("  {line}")
                }
            })
            .join("\n");

        let actual_selections = self.editor_selections();
        let actual_marked_text =
            generate_marked_text(&self.buffer_text(), &actual_selections, true);

        // Read the actual diff from the editor's row highlights and block
        // decorations.
        let actual_diff = self.editor.update(&mut self.cx, |editor, cx| {
            let snapshot = editor.snapshot(cx);
            let insertions = editor
                .highlighted_rows::<DiffRowHighlight>()
                .map(|(range, _)| {
                    let start = range.start.to_point(&snapshot.buffer_snapshot);
                    let end = range.end.to_point(&snapshot.buffer_snapshot);
                    start.row..end.row
                })
                .collect::<Vec<_>>();
            let deletions = editor
                .diff_map
                .hunks
                .iter()
                .filter_map(|hunk| {
                    if hunk.blocks.is_empty() {
                        return None;
                    }
                    let row = hunk
                        .hunk_range
                        .start
                        .to_point(&snapshot.buffer_snapshot)
                        .row;
                    let (_, buffer, _) = editor
                        .buffer()
                        .read(cx)
                        .excerpt_containing(hunk.hunk_range.start, cx)
                        .expect("no excerpt for expanded buffer's hunk start");
                    let buffer_id = buffer.read(cx).remote_id();
                    let change_set = &editor
                        .diff_map
                        .diff_bases
                        .get(&buffer_id)
                        .expect("should have a diff base for expanded hunk")
                        .change_set;
                    let deleted_text = change_set
                        .read(cx)
                        .base_text
                        .as_ref()
                        .expect("no base text for expanded hunk")
                        .read(cx)
                        .as_rope()
                        .slice(hunk.diff_base_byte_range.clone())
                        .to_string();
                    if let DiffHunkStatus::Modified | DiffHunkStatus::Removed = hunk.status {
                        Some((row, deleted_text))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            format_diff(actual_marked_text, deletions, insertions)
        });

        pretty_assertions::assert_eq!(actual_diff, expected_diff_text, "unexpected diff state");
    }

    /// Make an assertion about the editor's text and the ranges and directions
    /// of its selections using a string containing embedded range markers.
    ///
    /// See the `util::test::marked_text_ranges` function for more information.
    #[track_caller]
    pub fn assert_editor_state(&mut self, marked_text: &str) {
        let (expected_text, expected_selections) = marked_text_ranges(marked_text, true);
        pretty_assertions::assert_eq!(self.buffer_text(), expected_text, "unexpected buffer text");
        self.assert_selections(expected_selections, marked_text.to_string())
    }

    pub fn editor_state(&mut self) -> String {
        generate_marked_text(self.buffer_text().as_str(), &self.editor_selections(), true)
    }

    #[track_caller]
    pub fn assert_editor_background_highlights<Tag: 'static>(&mut self, marked_text: &str) {
        let expected_ranges = self.ranges(marked_text);
        let actual_ranges: Vec<Range<usize>> = self.update_editor(|editor, cx| {
            let snapshot = editor.snapshot(cx);
            editor
                .background_highlights
                .get(&TypeId::of::<Tag>())
                .map(|h| h.1.clone())
                .unwrap_or_default()
                .iter()
                .map(|range| range.to_offset(&snapshot.buffer_snapshot))
                .collect()
        });
        assert_set_eq!(actual_ranges, expected_ranges);
    }

    #[track_caller]
    pub fn assert_editor_text_highlights<Tag: ?Sized + 'static>(&mut self, marked_text: &str) {
        let expected_ranges = self.ranges(marked_text);
        let snapshot = self.update_editor(|editor, cx| editor.snapshot(cx));
        let actual_ranges: Vec<Range<usize>> = snapshot
            .text_highlight_ranges::<Tag>()
            .map(|ranges| ranges.as_ref().clone().1)
            .unwrap_or_default()
            .into_iter()
            .map(|range| range.to_offset(&snapshot.buffer_snapshot))
            .collect();
        assert_set_eq!(actual_ranges, expected_ranges);
    }

    #[track_caller]
    pub fn assert_editor_selections(&mut self, expected_selections: Vec<Range<usize>>) {
        let expected_marked_text =
            generate_marked_text(&self.buffer_text(), &expected_selections, true);
        self.assert_selections(expected_selections, expected_marked_text)
    }

    #[track_caller]
    fn editor_selections(&mut self) -> Vec<Range<usize>> {
        self.editor
            .update(&mut self.cx, |editor, cx| {
                editor.selections.all::<usize>(cx)
            })
            .into_iter()
            .map(|s| {
                if s.reversed {
                    s.end..s.start
                } else {
                    s.start..s.end
                }
            })
            .collect::<Vec<_>>()
    }

    #[track_caller]
    fn assert_selections(
        &mut self,
        expected_selections: Vec<Range<usize>>,
        expected_marked_text: String,
    ) {
        let actual_selections = self.editor_selections();
        let actual_marked_text =
            generate_marked_text(&self.buffer_text(), &actual_selections, true);
        if expected_selections != actual_selections {
            pretty_assertions::assert_eq!(
                actual_marked_text,
                expected_marked_text,
                "{}Editor has unexpected selections",
                self.assertion_context(),
            );
        }
    }
}

fn format_diff(
    text: String,
    actual_deletions: Vec<(u32, String)>,
    actual_insertions: Vec<Range<u32>>,
) -> String {
    let mut diff = String::new();
    for (row, line) in text.split('\n').enumerate() {
        let row = row as u32;
        if row > 0 {
            diff.push('\n');
        }
        if let Some(text) = actual_deletions
            .iter()
            .find_map(|(deletion_row, deleted_text)| {
                if *deletion_row == row {
                    Some(deleted_text)
                } else {
                    None
                }
            })
        {
            for line in text.lines() {
                diff.push('-');
                if !line.is_empty() {
                    diff.push(' ');
                    diff.push_str(line);
                }
                diff.push('\n');
            }
        }
        let marker = if actual_insertions.iter().any(|range| range.contains(&row)) {
            "+ "
        } else {
            "  "
        };
        diff.push_str(format!("{marker}{line}").trim_end());
    }
    diff
}

impl Deref for EditorTestContext {
    type Target = gpui::VisualTestContext;

    fn deref(&self) -> &Self::Target {
        &self.cx
    }
}

impl DerefMut for EditorTestContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cx
    }
}

/// Tracks string context to be printed when assertions fail.
/// Often this is done by storing a context string in the manager and returning the handle.
#[derive(Clone)]
pub struct AssertionContextManager {
    id: Arc<AtomicUsize>,
    contexts: Arc<RwLock<BTreeMap<usize, String>>>,
}

impl Default for AssertionContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AssertionContextManager {
    pub fn new() -> Self {
        Self {
            id: Arc::new(AtomicUsize::new(0)),
            contexts: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn add_context(&self, context: String) -> ContextHandle {
        let id = self.id.fetch_add(1, Ordering::Relaxed);
        let mut contexts = self.contexts.write();
        contexts.insert(id, context);
        ContextHandle {
            id,
            manager: self.clone(),
        }
    }

    pub fn context(&self) -> String {
        let contexts = self.contexts.read();
        format!("\n{}\n", contexts.values().join("\n"))
    }
}

/// Used to track the lifetime of a piece of context so that it can be provided when an assertion fails.
/// For example, in the EditorTestContext, `set_state` returns a context handle so that if an assertion fails,
/// the state that was set initially for the failure can be printed in the error message
pub struct ContextHandle {
    id: usize,
    manager: AssertionContextManager,
}

impl Drop for ContextHandle {
    fn drop(&mut self) {
        let mut contexts = self.manager.contexts.write();
        contexts.remove(&self.id);
    }
}