// Caret Stream - Stream extension trait
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::pin::Pin;
use std::task::{Context, Poll};
use pin_project_lite::pin_project;
use crate::stream::Stream;
use crate::combinators::*;

/// Extension trait providing combinator methods for streams
pub trait StreamExt: Stream {
    /// Map each item to a new value
    fn map<F, T>(self, func: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> T,
    {
        Map::new(self, func)
    }

    /// Filter items based on a predicate
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        Filter::new(self, predicate)
    }

    /// Filter and map items in one step
    fn filter_map<F, T>(self, func: F) -> FilterMap<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<T>,
    {
        FilterMap::new(self, func)
    }

    /// Fold items into an accumulator
    fn fold<T, F>(self, initial: T, func: F) -> Fold<Self, T, F>
    where
        Self: Sized,
        F: FnMut(T, Self::Item) -> T,
    {
        Fold::new(self, initial, func)
    }

    /// Scan items, yielding each intermediate result
    fn scan<T, F>(self, initial: T, func: F) -> Scan<Self, T, F>
    where
        Self: Sized,
        F: FnMut(T, Self::Item) -> T,
    {
        Scan::new(self, initial, func)
    }

    /// Flatten nested streams
    fn flat_map<U>(self) -> FlatMap<Self, U>
    where
        Self: Sized,
        Self::Item: IntoStream<Stream = U>,
        U: Stream,
    {
        FlatMap::new(self)
    }

    /// Chain this stream with another
    fn chain<U>(self, other: U) -> Chain<Self, U>
    where
        Self: Sized,
        U: Stream<Item = Self::Item>,
    {
        Chain::new(self, other)
    }

    /// Take only the first n items
    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, n)
    }

    /// Take items while a predicate returns true
    fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        TakeWhile::new(self, predicate)
    }

    /// Skip the first n items
    fn skip(self, n: usize) -> Skip<Self>
    where
        Self: Sized,
    {
        Skip::new(self, n)
    }

    /// Skip items while a predicate returns true
    fn skip_while<P>(self, predicate: P) -> SkipWhile<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        SkipWhile::new(self, predicate)
    }

    /// Inspect each item
    fn inspect<F>(self, func: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item),
    {
        Inspect::new(self, func)
    }

    /// Fuse the stream so it never returns Pending after completion
    fn fuse(self) -> Fuse<Self>
    where
        Self: Sized,
    {
        Fuse::new(self)
    }

    /// Collect all items into a vector
    fn collect<Vec>(self) -> Collect<Self, Vec>
    where
        Self: Sized,
        Vec: FromIterator<Self::Item>,
    {
        Collect::new(self)
    }

    /// Count the number of items
    fn count(self) -> Count<Self>
    where
        Self: Sized,
    {
        Count::new(self)
    }

    /// Take the first item
    fn first(self) -> First<Self>
    where
        Self: Sized,
    {
        First::new(self)
    }

    /// Take the last item
    fn last(self) -> Last<Self>
    where
        Self: Sized,
    {
        Last::new(self)
    }

    /// Find the first item matching a predicate
    fn find<P>(self, predicate: P) -> Find<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        Find::new(self, predicate)
    }

    /// Find the index of the first item matching a predicate
    fn find_position<P>(self, predicate: P) -> FindPosition<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        FindPosition::new(self, predicate)
    }

    /// Check if any item matches a predicate
    fn any<P>(self, predicate: P) -> Any<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        Any::new(self, predicate)
    }

    /// Check if all items match a predicate
    fn all<P>(self, predicate: P) -> All<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        All::new(self, predicate)
    }

    /// Zip with another stream
    fn zip<U>(self, other: U) -> Zip<Self, U>
    where
        Self: Sized,
        U: Stream,
    {
        Zip::new(self, other)
    }

    /// Apply a function to each item (side effect only)
    fn for_each<F>(self, func: F) -> ForEach<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        ForEach::new(self, func)
    }

    /// Partition items into two vectors based on a predicate
    fn partition<P>(self, predicate: P) -> Partition<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        Partition::new(self, predicate)
    }
}

impl<St: Stream> StreamExt for St {}

pin_project! {
    /// Future that collects a stream into a collection
    pub struct Collect<St, C> {
        #[pin]
        stream: St,
        items: Option<C>,
    }
}

impl<St, C> Collect<St, C>
where
    St: Stream,
    C: FromIterator<St::Item>,
{
    pub fn new(stream: St) -> Self {
        Self {
            stream,
            items: None,
        }
    }
}

impl<St, C> futures::Future for Collect<St, C>
where
    St: Stream,
    C: FromIterator<St::Item>,
{
    type Output = C;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use std::iter;

        // Initialize the collector on first poll
        if self.items.is_none() {
            self.as_mut().project().items = Some(iter::empty().collect());
        }

        let mut buffer = Vec::new();
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => {
                    // Accumulate buffered items
                    if !buffer.is_empty() {
                        if let Some(items) = self.as_mut().project().items.as_mut() {
                            *items = items.iter().cloned().chain(buffer.drain(..)).collect();
                        }
                    }
                    return Poll::Pending;
                }
                Poll::Ready(None) => {
                    // Stream exhausted, return collected items
                    if let Some(items) = self.as_mut().project().items.take() {
                        return Poll::Ready(items);
                    }
                    return Poll::Ready(iter::empty().collect());
                }
                Poll::Ready(Some(item)) => {
                    buffer.push(item);
                }
            }
        }
    }
}

pin_project! {
    /// Future that counts items in a stream
    pub struct Count<St> {
        #[pin]
        stream: St,
        count: usize,
    }
}

impl<St> Count<St> {
    pub fn new(stream: St) -> Self {
        Self { stream, count: 0 }
    }
}

impl<St> futures::Future for Count<St>
where
    St: Stream,
{
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(*self.count),
                Poll::Ready(Some(_)) => {
                    *self.as_mut().project().count += 1;
                }
            }
        }
    }
}

pin_project! {
    /// Future that takes the first item
    pub struct First<St> {
        #[pin]
        stream: St,
    }
}

impl<St> First<St> {
    pub fn new(stream: St) -> Self {
        Self { stream }
    }
}

impl<St> futures::Future for First<St>
where
    St: Stream,
{
    type Output = Option<St::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().stream.poll_next(cx)
    }
}

pin_project! {
    /// Future that takes the last item
    pub struct Last<St> {
        #[pin]
        stream: St,
        last: bool,
    }
}

impl<St> Last<St> {
    pub fn new(stream: St) -> Self {
        Self {
            stream,
            last: false,
        }
    }
}

impl<St> futures::Future for Last<St>
where
    St: Stream,
    St::Item: Clone,
{
    type Output = Option<St::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut last_item = None;
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => {
                    if self.last {
                        return Poll::Ready(last_item);
                    }
                    return Poll::Pending;
                }
                Poll::Ready(None) => return Poll::Ready(last_item),
                Poll::Ready(Some(item)) => {
                    self.as_mut().project().last = true;
                    last_item = Some(item);
                }
            }
        }
    }
}

pin_project! {
    /// Future that finds the first item matching a predicate
    pub struct Find<St, P> {
        #[pin]
        stream: St,
        predicate: P,
    }
}

impl<St, P> Find<St, P> {
    pub fn new(stream: St, predicate: P) -> Self {
        Self { stream, predicate }
    }
}

impl<St, P> futures::Future for Find<St, P>
where
    St: Stream,
    P: FnMut(&St::Item) -> bool,
{
    type Output = Option<St::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    let predicate = self.project().predicate;
                    if predicate(&item) {
                        return Poll::Ready(Some(item));
                    }
                }
            }
        }
    }
}

pin_project! {
    /// Future that finds the index of the first item matching a predicate
    pub struct FindPosition<St, P> {
        #[pin]
        stream: St,
        predicate: P,
        index: usize,
    }
}

impl<St, P> FindPosition<St, P> {
    pub fn new(stream: St, predicate: P) -> Self {
        Self {
            stream,
            predicate,
            index: 0,
        }
    }
}

impl<St, P> futures::Future for FindPosition<St, P>
where
    St: Stream,
    P: FnMut(&St::Item) -> bool,
{
    type Output = Option<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(item)) => {
                    let predicate = self.project().predicate;
                    if predicate(&item) {
                        return Poll::Ready(Some(*self.index));
                    }
                    *self.as_mut().project().index += 1;
                }
            }
        }
    }
}

pin_project! {
    /// Future that checks if any item matches a predicate
    pub struct Any<St, P> {
        #[pin]
        stream: St,
        predicate: P,
    }
}

impl<St, P> Any<St, P> {
    pub fn new(stream: St, predicate: P) -> Self {
        Self { stream, predicate }
    }
}

impl<St, P> futures::Future for Any<St, P>
where
    St: Stream,
    P: FnMut(&St::Item) -> bool,
{
    type Output = bool;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(false),
                Poll::Ready(Some(item)) => {
                    let predicate = self.project().predicate;
                    if predicate(&item) {
                        return Poll::Ready(true);
                    }
                }
            }
        }
    }
}

pin_project! {
    /// Future that checks if all items match a predicate
    pub struct All<St, P> {
        #[pin]
        stream: St,
        predicate: P,
    }
}

impl<St, P> All<St, P> {
    pub fn new(stream: St, predicate: P) -> Self {
        Self { stream, predicate }
    }
}

impl<St, P> futures::Future for All<St, P>
where
    St: Stream,
    P: FnMut(&St::Item) -> bool,
{
    type Output = bool;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(true),
                Poll::Ready(Some(item)) => {
                    let predicate = self.project().predicate;
                    if !predicate(&item) {
                        return Poll::Ready(false);
                    }
                }
            }
        }
    }
}

pin_project! {
    /// Future that applies a function to each item
    pub struct ForEach<St, F> {
        #[pin]
        stream: St,
        func: F,
    }
}

impl<St, F> ForEach<St, F> {
    pub fn new(stream: St, func: F) -> Self {
        Self { stream, func }
    }
}

impl<St, F> futures::Future for ForEach<St, F>
where
    St: Stream,
    F: FnMut(St::Item),
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(()),
                Poll::Ready(Some(item)) => {
                    let func = self.project().func;
                    func(item);
                }
            }
        }
    }
}

pin_project! {
    /// Future that partitions items into two vectors
    pub struct Partition<St, P> {
        #[pin]
        stream: St,
        predicate: P,
        _phantom: std::marker::PhantomData<St>,
    }
}

impl<St, P> Partition<St, P> {
    pub fn new(stream: St, predicate: P) -> Self {
        Self {
            stream,
            predicate,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<St, P> futures::Future for Partition<St, P>
where
    St: Stream,
    P: FnMut(&St::Item) -> bool,
{
    type Output = (Vec<St::Item>, Vec<St::Item>);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut left = Vec::new();
        let mut right = Vec::new();
        loop {
            match self.as_mut().project().stream.poll_next(cx) {
                Poll::Pending => {
                    if !left.is_empty() || !right.is_empty() {
                        return Poll::Ready((left, right));
                    }
                    return Poll::Pending;
                }
                Poll::Ready(None) => return Poll::Ready((left, right)),
                Poll::Ready(Some(item)) => {
                    let predicate = self.project().predicate;
                    if predicate(&item) {
                        left.push(item);
                    } else {
                        right.push(item);
                    }
                }
            }
        }
    }
}
