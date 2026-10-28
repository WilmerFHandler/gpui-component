use std::time::Duration;

use gpui::{Context, Pixels, Task, px};

static INTERVAL: Duration = Duration::from_millis(500);
static PAUSE_DELAY: Duration = Duration::from_millis(300);
pub(super) const CURSOR_WIDTH: Pixels = px(1.5);

/// To manage the Input cursor blinking.
///
/// It will start blinking with a interval of 500ms.
/// Every loop will notify the view to update the `visible`, and Input will observe this update to touch repaint.
///
/// The input painter will check if this in visible state, then it will draw the cursor.
pub(crate) struct BlinkCursor {
    active: bool,
    visible: bool,
    paused: bool,
    epoch: usize,

    _task: Task<()>,
}

impl BlinkCursor {
    pub fn new() -> Self {
        Self {
            active: false,
            visible: false,
            paused: false,
            epoch: 0,
            _task: Task::ready(()),
        }
    }

    /// Start the blinking
    pub fn start(&mut self, cx: &mut Context<Self>) {
        if self.active {
            return;
        }
        self.active = true;
        self.blink(self.epoch, cx);
    }

    pub fn stop(&mut self, cx: &mut Context<Self>) {
        if !self.active {
            return;
        }
        self.active = false;
        self.paused = false;
        self.visible = false;
        self.next_epoch();
        self._task = Task::ready(());
        cx.notify();
    }

    fn next_epoch(&mut self) -> usize {
        self.epoch = self.epoch.wrapping_add(1);
        self.epoch
    }

    fn blink(&mut self, epoch: usize, cx: &mut Context<Self>) {
        if !self.active || self.paused || epoch != self.epoch {
            return;
        }

        self.visible = !self.visible;
        cx.notify();

        // Schedule the next blink
        let epoch = self.next_epoch();
        self._task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(INTERVAL).await;
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| this.blink(epoch, cx)).ok();
            }
        });
    }

    pub fn visible(&self) -> bool {
        // Keep showing the cursor if paused
        self.paused || self.visible
    }

    /// Keep an active caret visible briefly after typing. Programmatic edits
    /// to an unfocused input must not start a blinking task.
    pub fn pause(&mut self, cx: &mut Context<Self>) {
        if !self.active {
            return;
        }
        self.paused = true;
        self.visible = true;
        cx.notify();

        // Resume only this pause; stop or a newer edit invalidates it.
        let epoch = self.next_epoch();
        self._task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(PAUSE_DELAY).await;

            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    if !this.active || this.epoch != epoch {
                        return;
                    }
                    this.paused = false;
                    this.blink(epoch, cx);
                })
                .ok();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::AppContext;

    #[gpui::test]
    fn inactive_edits_and_stopped_pause_cannot_restart_blinking(cx: &mut gpui::TestAppContext) {
        let cursor = cx.new(|_| BlinkCursor::new());
        cursor.update(cx, |cursor, cx| cursor.pause(cx));
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.run_until_parked();
        cursor.update(cx, |cursor, cx| {
            assert!(!cursor.active);
            assert!(!cursor.visible());
            assert_eq!(cursor.epoch, 0);
            cursor.start(cx);
            cursor.pause(cx);
            cursor.stop(cx);
        });
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.run_until_parked();
        cursor.update(cx, |cursor, _| {
            assert!(!cursor.active);
            assert!(!cursor.paused);
            assert!(!cursor.visible());
        });
    }

    #[gpui::test]
    fn repeated_focus_keeps_one_timer_and_restart_blinks(cx: &mut gpui::TestAppContext) {
        let cursor = cx.new(|_| BlinkCursor::new());
        cursor.update(cx, |cursor, cx| {
            cursor.start(cx);
            let epoch = cursor.epoch;
            cursor.start(cx);
            assert_eq!(cursor.epoch, epoch);
            cursor.stop(cx);
            cursor.start(cx);
            assert!(cursor.visible());
        });
        cx.run_until_parked();
        cx.executor().advance_clock(INTERVAL);
        cx.run_until_parked();
        assert!(!cursor.read_with(cx, |cursor, _| cursor.visible()));
    }
}
