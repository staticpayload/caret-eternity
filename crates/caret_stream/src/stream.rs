// Caret Stream - Core stream trait
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Future;
use pin_project_lite::pin_project;

/// An asynchronous stream of values
///
/// This trait is similar to futures::Stream but designed specifically
/// for Caret's stream processing needs.
pub trait Stream {
    /// The type of items yielded by the stream
    type Item;

    /// Attempt to pull the next value from this stream
    ///
    /// Returns:
    /// - Poll::Pending if the next value is not yet available
    /// - Poll::Ready(Some(item)) if a value is available
    /// - Poll::Ready(None) if the stream is exhausted
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;

    /// Get the size hint for this stream
    ///
    /// Returns (lower_bound, upper_bound) where upper_bound may be None
    /// if the size is unknown.
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    /// Returns the remaining length of this stream
    fn len(&self) -> usize {
        match self.size_hint() {
            (_, None) => usize::MAX,
            (lower, Some(upper)) if lower == upper => upper,
            _ => usize::MAX,
        }
    }

    /// Returns true if the stream is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert this stream into a future that resolves to the next item
    fn next(&mut self) -> Next<'_, Self>
    where
        Self: Unpin + Sized,
    {
        Next { stream: self }
    }
}

/// Future for the next value of a stream
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Next<'a, St: ?Sized + Stream> {
    stream: &'a mut St,
}

impl<St: ?Sized + Stream + Unpin> Future for Next<'_, St> {
    type Output = Option<St::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.stream).poll_next(cx)
    }
}

/// A stream that produces Result items
pub trait TryStream {
    /// The type of successful items
    type Ok;

    /// The type of errors produced
    type Error;

    /// Attempt to pull the next value from this stream
    fn poll_try_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Ok, Self::Error>>>;

    /// Returns the remaining length of this stream
    fn try_len(&self) -> usize {
        usize::MAX
    }

    /// Returns true if the stream is empty
    fn is_empty(&self) -> bool {
        self.try_len() == 0
    }
}

/// Item type for streams
pub trait StreamItem {
    /// The item type
    type Item;
}

impl<T> StreamItem for &mut T {
    type Item = T;
}

/// An iterator that yields items from a stream
pub struct StreamIterator<St: Stream + Unpin> {
    stream: St,
}

impl<St: Stream + Unpin> StreamIterator<St> {
    /// Create a new stream iterator
    pub fn new(stream: St) -> Self {
        Self { stream }
    }
}

impl<St: Stream + Unpin> Iterator for StreamIterator<St> {
    type Item = St::Item;

    fn next(&mut self) -> Option<Self::Item> {
        // This is a blocking iterator for streams
        // In practice, you'd use the async interface
        use std::task::Waker;
        use std::task::RawWaker;
        use std::task::RawWakerVTable;
        use std::task::Waker as StdWaker;

        // Create a no-op waker
        static VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_: *const ()| unsafe { std::mem::transmute::<(), RawWaker>(()) },
            |_: *const ()| {},
            |_: *const ()| {},
            |_: *const ()| {},
        );
        let raw = RawWaker::new(std::ptr::null(), &VTABLE);
        let waker = unsafe { StdWaker::from_raw(raw) };
        let mut cx = Context::from_waker(&waker);

        match Pin::new(&mut self.stream).poll_next(&mut cx) {
            Poll::Ready(item) => item,
            Poll::Pending => {
                // In a real implementation, we'd block here
                // For now, return None
                None
            }
        }
    }
}

impl<S: Stream + Unpin + Sized> Stream for &mut S {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        S::poll_next(Pin::new(&mut **self.get_mut()), cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

impl<S: Stream + Unpin + Sized> Stream for Box<S> {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut *self).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

pin_project! {
    /// A stream that yields items from an iterator
    #[derive(Debug, Clone)]
    pub struct Iter<I> {
        #[pin]
        iter: I,
    }
}

impl<I> Iter<I>
where
    I: Iterator,
{
    /// Create a new stream from an iterator
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I> Stream for Iter<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.project().iter.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let iter = unsafe { self.map_unchecked_mut(|s| &mut s.iter) };
        iter.size_hint()
    }
}

pin_project! {
    /// A stream that yields items from a slice
    #[derive(Debug, Clone)]
    pub struct Slice<'a, T> {
        slice: &'a [T],
        index: usize,
    }
}

impl<'a, T> Slice<'a, T> {
    /// Create a new stream from a slice
    pub fn new(slice: &'a [T]) -> Self {
        Self { slice, index: 0 }
    }
}

impl<'a, T> Stream for Slice<'a, T> {
    type Item = &'a T;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.index >= self.slice.len() {
            Poll::Ready(None)
        } else {
            let item = Some(&self.slice[self.index]);
            self.index += 1;
            Poll::Ready(item)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.slice.len().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

pin_project! {
    /// A stream that yields a single value
    #[derive(Debug, Clone)]
    pub struct Once<T> {
        #[pin]
        value: Option<T>,
    }
}

impl<T> Once<T> {
    /// Create a new stream that yields a single value
    pub fn new(value: T) -> Self {
        Self { value: Some(value) }
    }

    /// Create an empty once stream
    pub fn empty() -> Self {
        Self { value: None }
    }
}

impl<T> Stream for Once<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.project().value.take())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let has_value = unsafe { self.map_unchecked(|s| &s.value).is_some() };
        if has_value {
            (1, Some(1))
        } else {
            (0, Some(0))
        }
    }
}

pin_project! {
    /// A stream that yields values from a vector
    #[derive(Debug)]
    pub struct FromIter<T> {
        #[pin]
        iter: std::vec::IntoIter<T>,
    }
}

impl<T> FromIter<T> {
    /// Create a new stream from a vector
    pub fn new(vec: Vec<T>) -> Self {
        Self {
            iter: vec.into_iter(),
        }
    }
}

impl<T> Stream for FromIter<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.project().iter.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let iter = unsafe { self.map_unchecked_mut(|s| &mut s.iter) };
        iter.size_hint()
    }
}

pin_project! {
    /// A stream that repeats a value indefinitely
    #[derive(Debug, Clone)]
    pub struct Repeat<T> {
        value: T,
    }
}

impl<T> Repeat<T> {
    /// Create a new stream that repeats a value
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Clone> Stream for Repeat<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(self.value.clone()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

pin_project! {
    /// A stream that yields items by calling a function repeatedly
    #[derive(Debug, Clone)]
    pub struct RepeatWith<F> {
        func: F,
    }
}

impl<F> RepeatWith<F> {
    /// Create a new stream that repeats by calling a function
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<T, F> Stream for RepeatWith<F>
where
    F: FnMut() -> T,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(self.project().func()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

/// Create a stream from an iterator
pub fn from_iter<I>(iter: I) -> Iter<I::IntoIter>
where
    I: IntoIterator,
{
    Iter {
        iter: iter.into_iter(),
    }
}

/// Create a stream that yields a single value
pub fn once<T>(value: T) -> Once<T> {
    Once::new(value)
}

/// Create a stream that repeats a value
pub fn repeat<T>(value: T) -> Repeat<T> {
    Repeat::new(value)
}

/// Create a stream that repeats by calling a function
pub fn repeat_with<F>(func: F) -> RepeatWith<F> {
    RepeatWith::new(func)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_stream() {
        let stream = Iter::new(vec![1, 2, 3].into_iter());
        assert_eq!(stream.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_slice_stream() {
        let slice = vec![1, 2, 3];
        let stream = Slice::new(&slice);
        assert_eq!(stream.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_once_stream() {
        let stream = Once::new(42);
        assert_eq!(stream.size_hint(), (1, Some(1)));
    }

    #[test]
    fn test_once_empty() {
        let stream = Once::<()>::empty();
        assert_eq!(stream.size_hint(), (0, Some(0)));
    }

    #[test]
    fn test_from_iter() {
        let stream = from_iter(vec![1, 2, 3]);
        assert_eq!(stream.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_repeat() {
        let stream = Repeat::new(42);
        assert_eq!(stream.size_hint(), (usize::MAX, None));
    }

    #[test]
    fn test_repeat_with() {
        let mut count = 0;
        let stream = RepeatWith::new(|| {
            count += 1;
            count
        });
        assert_eq!(stream.size_hint(), (usize::MAX, None));
    }

    #[test]
    fn test_stream_is_empty() {
        let slice: Vec<i32> = vec![];
        let stream = Slice::new(&slice);
        assert!(stream.is_empty());
    }

    #[test]
    fn test_stream_len() {
        let slice = vec![1, 2, 3];
        let stream = Slice::new(&slice);
        assert_eq!(stream.len(), 3);
    }

    #[test]
    fn test_size_hint_unknown() {
        let stream = Repeat::new(42);
        assert_eq!(stream.size_hint(), (usize::MAX, None));
    }
}
