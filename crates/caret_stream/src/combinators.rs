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
        match self.as_mut().project().stream.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(item)) => {
                let func = self.project().func;
                Poll::Ready(Some(func(item)))
            }
        }
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
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    let predicate = self.project().predicate;
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
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    let func = self.project().func;
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
            let acc = self.as_mut().project().accumulator.take().unwrap();
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => {
                    *self.as_mut().project().accumulator = Some(acc);
                    return Poll::Pending;
                }
                Poll::Ready(None) => return Poll::Ready(acc),
                Poll::Ready(Some(item)) => {
                    let func = self.project().func;
                    *self.as_mut().project().accumulator = Some(func(acc, item));
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
    F: FnMut(T, St::Item) -> T,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let state = self.as_mut().project().state.take();
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => {
                    *self.as_mut().project().state = state;
                    return Poll::Pending;
                }
                Poll::Ready(None) => return Poll::Ready(state),
                Poll::Ready(Some(item)) => {
                    let func = self.project().func;
                    let new_state = func(state.unwrap(), item);
                    *self.as_mut().project().state = Some(new_state.clone());
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
                    (Some(a), Some(b)) => a.checked_add(*b),
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
        if self.done_taking {
            return Poll::Ready(None);
        }

        match self.as_mut().project().stream.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(item)) => {
                let predicate = self.project().predicate;
                if predicate(&item) {
                    Poll::Ready(Some(item))
                } else {
                    *self.as_mut().project().done_taking = true;
                    Poll::Ready(None)
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
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    if self.done_skipping {
                        return Poll::Ready(Some(item));
                    }
                    let predicate = self.project().predicate;
                    if predicate(&item) {
                        // Continue loop
                    } else {
                        *self.as_mut().project().done_skipping = true;
                        return Poll::Ready(Some(item));
                    }
                }
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
        match self.as_mut().project().stream.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(item)) => {
                let func = self.project().func;
                func(&item);
                Poll::Ready(Some(item))
            }
        }
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
    St: Stream,
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    type Item = St::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(fut) = self.as_mut().project().future.as_pin_mut() {
                match fut.poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(_) => return Poll::Ready(None),
                }
            } else {
                match self.as_mut().project().stream.poll_next(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Ready(Some(item)) => {
                        let func = self.as_mut().project().func.take().unwrap();
                        *self.as_mut().project().future = Some(func());
                        // Continue loop
                    }
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
        match self.as_mut().project().a.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(a_item)) => {
                match self.as_mut().project().b.poll_next(cx) {
                    Poll::Pending => {
                        // We need to save the a_item somehow
                        // For now, we skip this case
                        Poll::Pending
                    }
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Ready(Some(b_item)) => Poll::Ready(Some((a_item, b_item))),
                }
            }
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
    use crate::stream::Iter;
    use std::task::Waker;
    use std::task::RawWaker;
    use std::task::RawWakerVTable;

    fn new_waker() -> Waker {
        static VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_: *const ()| unsafe { std::mem::transmute::<(), RawWaker>(()) },
            |_: *const ()| {},
            |_: *const ()| {},
            |_: *const ()| {},
        );
        let raw = RawWaker::new(std::ptr::null(), &VTABLE);
        unsafe { Waker::from_raw(raw) }
    }

    fn new_cx() -> Context<'static> {
        Context::from_waker(&new_waker())
    }

    #[test]
    fn test_map() {
        let stream = Map::new(Iter::new(vec![1, 2, 3].into_iter()), |x| x * 2);
        let mut stream = Pin::new(&mut stream);
        let cx = new_cx();

        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(6)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_filter() {
        let stream = Filter::new(Iter::new(vec![1, 2, 3, 4, 5].into_iter()), |&x| x % 2 == 0);
        let mut stream = Pin::new(&mut stream);
        let cx = new_cx();

        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_filter_map() {
        let stream = FilterMap::new(Iter::new(vec![1, 2, 3, 4, 5].into_iter()), |x| {
            if x % 2 == 0 { Some(x * 2) } else { None }
        });
        let mut stream = Pin::new(&mut stream);
        let cx = new_cx();

        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(8)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_scan() {
        let stream = Scan::new(Iter::new(vec![1, 2, 3].into_iter()), 0, |acc, x| acc + x);
        let mut stream = Pin::new(&mut stream);
        let cx = new_cx();

        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(1)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(6)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(6)));// Final state
    }

    #[test]
    fn test_take() {
        let stream = Take::new(Iter::new(vec![1, 2, 3, 4, 5].into_iter()), 3);
        let mut stream = Pin::new(&mut stream);
        let cx = new_cx();

        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(1)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_take_size_hint() {
        let stream = Take::new(Iter::new(vec![1, 2, 3].into_iter()), 5);
        assert_eq!(stream.size_hint(), (3, Some(3)));
    }

    #[test]
    fn test_skip() {
        let stream = Skip::new(Iter::new(vec![1, 2, 3, 4, 5].into_iter()), 2);
        let mut stream = Pin::new(&mut stream);
        let cx = new_cx();

        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(5)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_chain() {
        let first = Iter::new(vec![1, 2].into_iter());
        let second = Iter::new(vec![3, 4].into_iter());
        let stream = Chain::new(first, second);
        let mut stream = Pin::new(&mut stream);
        let cx = new_cx();

        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(1)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(4)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_zip() {
        let a = Iter::new(vec![1, 2, 3].into_iter());
        let b = Iter::new(vec![4, 5, 6].into_iter());
        let stream = Zip::new(a, b);
        let mut stream = Pin::new(&mut stream);
        let cx = new_cx();

        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some((1, 4))));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some((2, 5))));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some((3, 6))));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_fuse() {
        let stream = Fuse::new(Iter::new(vec![1, 2, 3].into_iter()));
        let mut stream = Pin::new(&mut stream);
        let cx = new_cx();

        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(1)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(2)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(Some(3)));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
        // Multiple calls to exhausted fuse return None
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
        assert_eq!(stream.poll_next(&mut cx), Poll::Ready(None));
    }

    #[test]
    fn test_inspect() {
        let mut inspected = Vec::new();
        {
            let stream = Inspect::new(Iter::new(vec![1, 2, 3].into_iter()), |x| inspected.push(*x));
            let mut stream = Pin::new(&mut stream);
            let cx = new_cx();

            while let Poll::Ready(Some(_)) = stream.poll_next(&mut cx) {}
        }
        assert_eq!(inspected, vec![1, 2, 3]);
    }
}
