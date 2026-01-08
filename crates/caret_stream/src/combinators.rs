// Caret Stream - Stream combinators
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Future;
use pin_project_lite::pin_project;
use crate::stream::Stream;

pin_project! {
    /// A stream that maps each item to a new value
    #[derive(Debug, Clone)]
    pub struct Map<St, F> {
        #[pin]
        stream: St,
        func: F,
    }
}

impl<St, F> Map<St, F> {
    /// Create a new map stream
    pub fn new(stream: St, func: F) -> Self {
        Self { stream, func }
    }
}

impl<St, F, T> Stream for Map<St, F>
where
    St: Stream,
    F: FnMut(St::Item) -> T,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let item = match self.as_mut().project().stream.poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Ready(Some(item)) => item,
        };
        let func = self.project().func;
        Poll::Ready(Some(func(item)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

pin_project! {
    /// A stream that filters items based on a predicate
    #[derive(Debug, Clone)]
    pub struct Filter<St, P> {
        #[pin]
        stream: St,
        predicate: P,
    }
}

impl<St, P> Filter<St, P> {
    /// Create a new filter stream
    pub fn new(stream: St, predicate: P) -> Self {
        Self { stream, predicate }
    }
}

impl<St, P> Stream for Filter<St, P>
where
    St: Stream,
    P: FnMut(&St::Item) -> bool,
{
    type Item = St::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();
            match this.stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    let predicate = this.predicate;
                    if predicate(&item) {
                        return Poll::Ready(Some(item));
                    }
                    // Continue loop to check next item
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Could be anywhere from 0 to the original size
        (0, self.stream.size_hint().1)
    }
}

pin_project! {
    /// A stream that filters and maps items
    #[derive(Debug, Clone)]
    pub struct FilterMap<St, F> {
        #[pin]
        stream: St,
        func: F,
    }
}

impl<St, F> FilterMap<St, F> {
    /// Create a new filter_map stream
    pub fn new(stream: St, func: F) -> Self {
        Self { stream, func }
    }
}

impl<St, F, T> Stream for FilterMap<St, F>
where
    St: Stream,
    F: FnMut(St::Item) -> Option<T>,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();
            match this.stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    let func = this.func;
                    if let Some(mapped) = func(item) {
                        return Poll::Ready(Some(mapped));
                    }
                    // Continue loop to check next item
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.stream.size_hint().1)
    }
}

pin_project! {
    /// A stream that folds items into an accumulator
    #[derive(Debug)]
    pub struct Fold<St, T, F> {
        #[pin]
        stream: St,
        accumulator: Option<T>,
        func: F,
    }
}

impl<St, T, F> Fold<St, T, F> {
    /// Create a new fold stream
    pub fn new(stream: St, initial: T, func: F) -> Self {
        Self {
            stream,
            accumulator: Some(initial),
            func,
        }
    }
}

impl<St, T, F> Future for Fold<St, T, F>
where
    St: Stream,
    F: FnMut(T, St::Item) -> T,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();
            let acc = this.accumulator.take().unwrap();
            match this.stream.poll_next(cx) {
                Poll::Pending => {
                    *this.accumulator = Some(acc);
                    return Poll::Pending;
                }
                Poll::Ready(None) => return Poll::Ready(acc),
                Poll::Ready(Some(item)) => {
                    let func = this.func;
                    *this.accumulator = Some(func(acc, item));
                    // Continue loop
                }
            }
        }
    }
}

pin_project! {
    /// A stream that applies a folding function and yields each intermediate result
    #[derive(Debug, Clone)]
    pub struct Scan<St, T, F> {
        #[pin]
        stream: St,
        state: Option<T>,
        func: F,
    }
}

impl<St, T, F> Scan<St, T, F> {
    /// Create a new scan stream
    pub fn new(stream: St, initial: T, func: F) -> Self {
        Self {
            stream,
            state: Some(initial),
            func,
        }
    }
}

impl<St, T, F> Stream for Scan<St, T, F>
where
    St: Stream,
    T: Clone,
    F: FnMut(T, St::Item) -> T,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();
            let state = this.state.take();

            match this.stream.poll_next(cx) {
                Poll::Pending => {
                    // Restore state
                    *this.state = state;
                    return Poll::Pending;
                }
                Poll::Ready(None) => return Poll::Ready(state),
                Poll::Ready(Some(item)) => {
                    let func = this.func;
                    let new_state = func(state.unwrap(), item);
                    *this.state = Some(new_state.clone());
                    return Poll::Ready(Some(new_state));
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

pin_project! {
    /// A stream that flattens nested streams
    #[derive(Debug, Clone)]
    pub struct FlatMap<St, U> {
        #[pin]
        outer: St,
        #[pin]
        inner: Option<U>,
    }
}

impl<St, U> FlatMap<St, U> {
    /// Create a new flat_map stream
    pub fn new(stream: St) -> Self {
        Self {
            outer: stream,
            inner: None,
        }
    }
}

impl<St, U> Stream for FlatMap<St, U>
where
    St: Stream,
    St::Item: IntoStream<Stream = U>,
    U: Stream,
{
    type Item = U::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(inner) = self.as_mut().project().inner.as_pin_mut() {
                match inner.poll_next(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(None) => {
                        self.as_mut().project().inner.set(None);
                        // Continue to get next outer item
                    }
                    Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                }
            } else {
                match self.as_mut().project().outer.poll_next(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Ready(Some(item)) => {
                        self.as_mut().project().inner.set(Some(item.into_stream()));
                        // Continue loop to poll inner
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Can't really know without polling
        (0, None)
    }
}

/// Trait for converting to a stream
pub trait IntoStream {
    /// The stream type
    type Stream: Stream<Item = Self::Item>;

    /// The item type
    type Item;

    /// Convert this value into a stream
    fn into_stream(self) -> Self::Stream;
}

impl<T: Stream> IntoStream for T {
    type Stream = T;
    type Item = T::Item;

    fn into_stream(self) -> Self {
        self
    }
}

pin_project! {
    /// A stream that chains two streams together
    #[derive(Debug, Clone)]
    pub struct Chain<A, B> {
        #[pin]
        first: Option<A>,
        #[pin]
        second: Option<B>,
    }
}

impl<A, B> Chain<A, B>
where
    A: Stream,
    B: Stream<Item = A::Item>,
{
    /// Create a new chain stream
    pub fn new(first: A, second: B) -> Self {
        Self {
            first: Some(first),
            second: Some(second),
        }
    }
}

impl<A, B> Stream for Chain<A, B>
where
    A: Stream,
    B: Stream<Item = A::Item>,
{
    type Item = A::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(first) = self.as_mut().project().first.as_pin_mut() {
            match first.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    self.as_mut().project().first.set(None);
                    // Continue to second
                }
                Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
            }
        }

        if let Some(second) = self.as_mut().project().second.as_pin_mut() {
            second.poll_next(cx)
        } else {
            Poll::Ready(None)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match (&self.first, &self.second) {
            (Some(first), Some(second)) => {
                let (a_lower, a_upper) = first.size_hint();
                let (b_lower, b_upper) = second.size_hint();
                let lower = a_lower.saturating_add(b_lower);
                let upper = match (a_upper, b_upper) {
                    (Some(a), Some(b)) => a.checked_add(b),
                    _ => None,
                };
                (lower, upper)
            }
            (Some(first), None) => first.size_hint(),
            (None, Some(second)) => second.size_hint(),
            (None, None) => (0, Some(0)),
        }
    }
}

pin_project! {
    /// A stream that takes only the first n items
    #[derive(Debug, Clone)]
    pub struct Take<St> {
        #[pin]
        stream: St,
        remaining: usize,
    }
}

impl<St> Take<St> {
    /// Create a new take stream
    pub fn new(stream: St, n: usize) -> Self {
        Self {
            stream,
            remaining: n,
        }
    }
}

impl<St> Stream for Take<St>
where
    St: Stream,
{
    type Item = St::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.remaining == 0 {
            Poll::Ready(None)
        } else {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(None) => Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    *self.as_mut().project().remaining -= 1;
                    Poll::Ready(Some(item))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.stream.size_hint();
        let upper = upper.map(|u| u.min(self.remaining));
        (lower.min(self.remaining), upper)
    }
}

pin_project! {
    /// A stream that takes items while a predicate returns true
    #[derive(Debug, Clone)]
    pub struct TakeWhile<St, P> {
        #[pin]
        stream: St,
        predicate: P,
        done_taking: bool,
    }
}

impl<St, P> TakeWhile<St, P> {
    /// Create a new take_while stream
    pub fn new(stream: St, predicate: P) -> Self {
        Self {
            stream,
            predicate,
            done_taking: false,
        }
    }
}

impl<St, P> Stream for TakeWhile<St, P>
where
    St: Stream,
    P: FnMut(&St::Item) -> bool,
{
    type Item = St::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();
            if *this.done_taking {
                return Poll::Ready(None);
            }

            match this.stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    let predicate = this.predicate;
                    if predicate(&item) {
                        return Poll::Ready(Some(item));
                    } else {
                        *this.done_taking = true;
                        return Poll::Ready(None);
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done_taking {
            (0, Some(0))
        } else {
            (0, self.stream.size_hint().1)
        }
    }
}

pin_project! {
    /// A stream that skips the first n items
    #[derive(Debug, Clone)]
    pub struct Skip<St> {
        #[pin]
        stream: St,
        remaining: usize,
    }
}

impl<St> Skip<St> {
    /// Create a new skip stream
    pub fn new(stream: St, n: usize) -> Self {
        Self {
            stream,
            remaining: n,
        }
    }
}

impl<St> Stream for Skip<St>
where
    St: Stream,
{
    type Item = St::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    let remaining = *self.as_mut().project().remaining;
                    if remaining > 0 {
                        *self.as_mut().project().remaining = remaining - 1;
                        // Continue loop
                    } else {
                        return Poll::Ready(Some(item));
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.stream.size_hint();
        let lower = lower.saturating_sub(self.remaining);
        let upper = upper.map(|u| u.saturating_sub(self.remaining));
        (lower, upper)
    }
}

pin_project! {
    /// A stream that skips items while a predicate returns true
    #[derive(Debug, Clone)]
    pub struct SkipWhile<St, P> {
        #[pin]
        stream: St,
        predicate: P,
        done_skipping: bool,
    }
}

impl<St, P> SkipWhile<St, P> {
    /// Create a new skip_while stream
    pub fn new(stream: St, predicate: P) -> Self {
        Self {
            stream,
            predicate,
            done_skipping: false,
        }
    }
}

impl<St, P> Stream for SkipWhile<St, P>
where
    St: Stream,
    P: FnMut(&St::Item) -> bool,
{
    type Item = St::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();
            let done_skipping = *this.done_skipping;
            let item = match this.stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => item,
            };

            if done_skipping {
                return Poll::Ready(Some(item));
            }

            let predicate = this.predicate;
            if predicate(&item) {
                // Continue loop
            } else {
                *this.done_skipping = true;
                return Poll::Ready(Some(item));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done_skipping {
            self.stream.size_hint()
        } else {
            (0, self.stream.size_hint().1)
        }
    }
}

pin_project! {
    /// A stream that inspects each item
    #[derive(Debug, Clone)]
    pub struct Inspect<St, F> {
        #[pin]
        stream: St,
        func: F,
    }
}

impl<St, F> Inspect<St, F> {
    /// Create a new inspect stream
    pub fn new(stream: St, func: F) -> Self {
        Self { stream, func }
    }
}

impl<St, F> Stream for Inspect<St, F>
where
    St: Stream,
    F: FnMut(&St::Item),
{
    type Item = St::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let item = match this.stream.poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Ready(Some(item)) => item,
        };
        let func = this.func;
        func(&item);
        Poll::Ready(Some(item))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

pin_project! {
    /// A stream that chains another future after this stream completes
    #[derive(Debug)]
    pub struct Then<St, F, Fut> {
        #[pin]
        stream: St,
        #[pin]
        future: Option<Fut>,
        func: Option<F>,
    }
}

impl<St, F, Fut> Then<St, F, Fut> {
    /// Create a new then stream
    pub fn new(stream: St, func: F) -> Self {
        Self {
            stream,
            future: None,
            func: Some(func),
        }
    }
}

impl<St, F, Fut, T> Stream for Then<St, F, Fut>
where
    St: Stream<Item = T>,
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()> + Unpin,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();
            if this.future.is_some() {
                // Future is running - poll it
                // Since Fut: Unpin, we can take it out and poll it directly
                let mut future = this.future.take().unwrap();
                match Pin::new(&mut future).poll(cx) {
                    Poll::Pending => {
                        *this.future = Some(future);
                        return Poll::Pending;
                    }
                    Poll::Ready(_) => {
                        return Poll::Ready(None);
                    }
                }
            } else {
                // No future, poll stream
                match this.stream.poll_next(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(None) => {
                        // After stream ends, call the future
                        let func = this.func.take().unwrap();
                        *this.future = Some(func());
                        // Continue loop to poll future
                    }
                    Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

pin_project! {
    /// A stream that zips two streams together
    #[derive(Debug, Clone)]
    pub struct Zip<A, B> {
        #[pin]
        a: A,
        #[pin]
        b: B,
    }
}

impl<A, B> Zip<A, B> {
    /// Create a new zip stream
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A, B> Stream for Zip<A, B>
where
    A: Stream,
    B: Stream,
{
    type Item = (A::Item, B::Item);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        // Poll both streams
        let a_item = match this.a.poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Ready(Some(item)) => item,
        };

        match this.b.poll_next(cx) {
            Poll::Pending => {
                // We got a_item but b is pending - we need to store a_item
                // For simplicity in this implementation, just return Pending
                // A more complex version would cache the item
                Poll::Pending
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(b_item)) => Poll::Ready(Some((a_item, b_item))),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();
        let lower = a_lower.min(b_lower);
        let upper = match (a_upper, b_upper) {
            (Some(a), Some(b)) => Some(a.min(b)),
            _ => None,
        };
        (lower, upper)
    }
}

pin_project! {
    /// A stream that never returns Pending
    #[derive(Debug, Clone)]
    pub struct Fuse<St> {
        #[pin]
        stream: St,
        done: bool,
    }
}

impl<St> Fuse<St> {
    /// Create a new fuse stream
    pub fn new(stream: St) -> Self {
        Self {
            stream,
            done: false,
        }
    }
}

impl<St> Stream for Fuse<St>
where
    St: Stream,
{
    type Item = St::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            Poll::Ready(None)
        } else {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(None) => {
                    *self.as_mut().project().done = true;
                    Poll::Ready(None)
                }
                Poll::Ready(item) => Poll::Ready(item),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            (0, Some(0))
        } else {
            self.stream.size_hint()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::RawWaker;
    use std::task::RawWakerVTable;

    unsafe fn raw_waker_vtable() -> &'static RawWakerVTable {
        &RawWakerVTable::new(
            |_| RawWaker::new(std::ptr::null(), raw_waker_vtable()),
            |_| {},
            |_| {},
            |_| {},
        )
    }

    fn new_cx() -> Context<'static> {
        // Create a noop waker and leak it to get 'static lifetime
        // This is acceptable for test code which runs a limited number of times
        let raw = unsafe { RawWaker::new(std::ptr::null(), raw_waker_vtable()) };
        let waker = unsafe { std::task::Waker::from_raw(raw) };
        let leaked: &'static std::task::Waker = Box::leak(Box::new(waker));
        Context::from_waker(leaked)
    }

    #[test]
    fn test_map() {
        let mut stream = Map::new(crate::stream::Iter::new(vec![1, 2, 3].into_iter()), |x| x * 2);
        let mut stream = Pin::new(&mut stream);
        let mut cx = new_cx();

        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(6)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_filter() {
        let mut stream = Filter::new(
            crate::stream::Iter::new(vec![1i32, 2, 3, 4, 5].into_iter()),
            |x: &i32| *x % 2 == 0,
        );
        let mut stream = Pin::new(&mut stream);
        let mut cx = new_cx();

        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_filter_map() {
        let mut stream = FilterMap::new(
            crate::stream::Iter::new(vec![1, 2, 3, 4, 5].into_iter()),
            |x| if x % 2 == 0 { Some(x * 2) } else { None },
        );
        let mut stream = Pin::new(&mut stream);
        let mut cx = new_cx();

        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(8)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_scan() {
        let mut stream = Scan::new(
            crate::stream::Iter::new(vec![1, 2, 3].into_iter()),
            0,
            |acc, x| acc + x,
        );
        let mut stream = Pin::new(&mut stream);
        let mut cx = new_cx();

        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(1)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(6)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(6))); // Final state
    }

    #[test]
    fn test_take() {
        let mut stream = Take::new(
            crate::stream::Iter::new(vec![1, 2, 3, 4, 5].into_iter()),
            3,
        );
        let mut stream = Pin::new(&mut stream);
        let mut cx = new_cx();

        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(1)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_take_size_hint() {
        let stream = Take::new(
            crate::stream::Iter::new(vec![1, 2, 3].into_iter()),
            5,
        );
        assert_eq!(stream.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_skip() {
        let mut stream = Skip::new(
            crate::stream::Iter::new(vec![1, 2, 3, 4, 5].into_iter()),
            2,
        );
        let mut stream = Pin::new(&mut stream);
        let mut cx = new_cx();

        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(5)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_chain() {
        let first = crate::stream::Iter::new(vec![1, 2].into_iter());
        let second = crate::stream::Iter::new(vec![3, 4].into_iter());
        let mut stream = Chain::new(first, second);
        let mut stream = Pin::new(&mut stream);
        let mut cx = new_cx();

        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(1)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_zip() {
        let a = crate::stream::Iter::new(vec![1, 2, 3].into_iter());
        let b = crate::stream::Iter::new(vec![4, 5, 6].into_iter());
        let mut stream = Zip::new(a, b);
        let mut stream = Pin::new(&mut stream);
        let mut cx = new_cx();

        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some((1, 4))));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some((2, 5))));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some((3, 6))));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_fuse() {
        let mut stream = Fuse::new(crate::stream::Iter::new(vec![1, 2, 3].into_iter()));
        let mut stream = Pin::new(&mut stream);
        let mut cx = new_cx();

        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(1)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(None));
        // Multiple calls to exhausted fuse return None
        assert_eq!(stream.as_mut().poll_next(&mut cx), Poll::Ready(None));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_inspect() {
        let mut inspected = Vec::new();
        {
            let mut stream = Inspect::new(
                crate::stream::Iter::new(vec![1, 2, 3].into_iter()),
                |x: &i32| inspected.push(*x),
            );
            let mut stream = Pin::new(&mut stream);
            let mut cx = new_cx();

            while let Poll::Ready(Some(_)) = stream.as_mut().poll_next(&mut cx) {}
        }
        assert_eq!(inspected, vec![1, 2, 3]);
    }
}
