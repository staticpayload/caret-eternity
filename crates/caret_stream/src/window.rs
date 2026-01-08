// Caret Stream - Windowing operations
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::time::{Duration, Instant};
use crate::stream::Stream;

/// Window trait for defining windowing strategies
pub trait Window {
    /// The type of items in the window
    type Item;

    /// Check if a window should close based on the item
    fn should_close(&mut self, item: &Self::Item) -> bool;

    /// Reset the window state
    fn reset(&mut self);

    /// Get the current window size
    fn size(&self) -> usize;
}

/// A tumbling (fixed-size, non-overlapping) window
#[derive(Debug, Clone)]
pub struct TumblingWindow<T> {
    size: usize,
    count: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> TumblingWindow<T> {
    /// Create a new tumbling window with the given size
    pub fn new(size: usize) -> Self {
        Self {
            size,
            count: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the window size
    pub fn with_size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }
}

impl<T> Window for TumblingWindow<T> {
    type Item = T;

    fn should_close(&mut self, _item: &Self::Item) -> bool {
        self.count += 1;
        if self.count >= self.size {
            self.count = 0;
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn size(&self) -> usize {
        self.size
    }
}

/// A sliding (overlapping) window
#[derive(Debug, Clone)]
pub struct SlidingWindow<T> {
    window_size: usize,
    slide_size: usize,
    count: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> SlidingWindow<T> {
    /// Create a new sliding window
    ///
    /// `window_size`: The total size of the window
    /// `slide_size`: How many items to slide before creating a new window
    pub fn new(window_size: usize, slide_size: usize) -> Self {
        Self {
            window_size,
            slide_size,
            count: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a new sliding window with hop size
    pub fn with_hop(window_size: usize, hop: usize) -> Self {
        Self::new(window_size, hop)
    }
}

impl<T> Window for SlidingWindow<T> {
    type Item = T;

    fn should_close(&mut self, _item: &Self::Item) -> bool {
        self.count += 1;
        // Close after we've collected enough for a slide
        if self.count >= self.slide_size {
            self.count = 0;
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn size(&self) -> usize {
        self.window_size
    }
}

/// A count-based window that closes after N items
#[derive(Debug, Clone)]
pub struct CountWindow {
    count: usize,
    max_count: usize,
}

impl CountWindow {
    /// Create a new count window
    pub fn new(count: usize) -> Self {
        Self {
            count: 0,
            max_count: count,
        }
    }
}

impl Window for CountWindow {
    type Item = (); // Doesn't use the item type

    fn should_close(&mut self, _item: &Self::Item) -> bool {
        self.count += 1;
        self.count >= self.max_count
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn size(&self) -> usize {
        self.max_count
    }
}

/// A time-based window
#[derive(Debug, Clone)]
pub struct TimeWindow {
    pub duration: Duration,
    start: Option<Instant>,
}

impl TimeWindow {
    /// Create a new time window
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            start: None,
        }
    }

    /// Create a new time window from seconds
    pub fn from_secs(secs: u64) -> Self {
        Self::new(Duration::from_secs(secs))
    }

    /// Create a new time window from milliseconds
    pub fn from_millis(millis: u64) -> Self {
        Self::new(Duration::from_millis(millis))
    }

    /// Check if the window has expired based on time
    pub fn is_expired(&self) -> bool {
        if let Some(start) = self.start {
            start.elapsed() >= self.duration
        } else {
            false
        }
    }

    /// Start the window timer
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Get the remaining time in the window
    pub fn remaining(&self) -> Option<Duration> {
        self.start.map(|start| {
            self.duration.saturating_sub(start.elapsed())
        })
    }

    /// Get the elapsed time since window start
    pub fn elapsed(&self) -> Option<Duration> {
        self.start.map(|start| start.elapsed())
    }
}

impl Default for TimeWindow {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

impl Window for TimeWindow {
    type Item = (); // Time-based windows don't use items

    fn should_close(&mut self, _item: &Self::Item) -> bool {
        if self.start.is_none() {
            self.start = Some(Instant::now());
            false
        } else {
            self.is_expired()
        }
    }

    fn reset(&mut self) {
        self.start = None;
    }

    fn size(&self) -> usize {
        usize::MAX // Time windows don't have a fixed size
    }
}

/// Extension trait for window operations on streams
pub trait WindowExt: Stream {
    /// Create tumbling windows of the given size
    fn tumbling(self, size: usize) -> TumblingWindow<Self::Item>
    where
        Self: Sized,
    {
        TumblingWindow::new(size)
    }

    /// Create sliding windows
    fn sliding(self, window_size: usize, slide_size: usize) -> SlidingWindow<Self::Item>
    where
        Self: Sized,
    {
        SlidingWindow::new(window_size, slide_size)
    }

    /// Create time-based windows (duration-based)
    fn time_window(self, duration: Duration) -> TimeWindow
    where
        Self: Sized,
    {
        TimeWindow::new(duration)
    }
}

impl<St: Stream> WindowExt for St {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tumbling_window() {
        let mut window = TumblingWindow::<i32>::new(3);
        assert_eq!(window.size(), 3);
        assert!(!window.should_close(&1));
        assert!(!window.should_close(&2));
        assert!(window.should_close(&3));
    }

    #[test]
    fn test_sliding_window() {
        let mut window = SlidingWindow::<i32>::new(5, 2);
        assert_eq!(window.size(), 5);
        assert!(!window.should_close(&1));
        assert!(window.should_close(&2));
    }

    #[test]
    fn test_count_window() {
        let mut window = CountWindow::new(5);
        assert_eq!(window.size(), 5);
        for _i in 1..5 {
            assert!(!window.should_close(&()));
        }
        assert!(window.should_close(&()));
    }

    #[test]
    fn test_time_window() {
        let window = TimeWindow::from_millis(100);
        assert!(window.remaining().is_none());
        assert!(window.elapsed().is_none());
        assert!(!window.is_expired());
    }

    #[test]
    fn test_time_window_default() {
        let window = TimeWindow::default();
        assert_eq!(window.duration, Duration::from_secs(1));
    }

    #[test]
    fn test_window_from_secs() {
        let window = TimeWindow::from_secs(5);
        assert_eq!(window.duration, Duration::from_secs(5));
    }

    #[test]
    fn test_window_from_millis() {
        let window = TimeWindow::from_millis(500);
        assert_eq!(window.duration, Duration::from_millis(500));
    }
}
