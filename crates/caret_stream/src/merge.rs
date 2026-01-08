// Caret Stream - Stream merging and combining operations
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::pin::Pin;
use std::task::{Context, Poll};
use pin_project_lite::pin_project;
use crate::stream::Stream;

pin_project! {
    /// A stream that merges multiple streams into one
    ///
    /// This polls streams in order, moving to the next when one is pending.
    #[derive(Debug)]
    pub struct Merge<A, B> {
        #[pin]
        a: A,
        #[pin]
        b: B,
        state: MergeState,
    }
}

#[derive(Debug, Clone, Copy)]
enum MergeState {
    A,
    B,
}

impl<A, B> Merge<A, B>
where
    A: Stream,
    B: Stream<Item = A::Item>,
{
    /// Create a new merge of two streams
    pub fn new(a: A, b: B) -> Self {
        Self {
            a,
            b,
            state: MergeState::A,
        }
    }
}

impl<A, B> Stream for Merge<A, B>
where
    A: Stream,
    B: Stream<Item = A::Item>,
{
    type Item = A::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.state {
            MergeState::A => {
                match this.a.poll_next(cx) {
                    Poll::Ready(item) => {
                        *this.state = MergeState::B;
                        Poll::Ready(item)
                    }
                    Poll::Pending => {
                        *this.state = MergeState::B;
                        // Try stream B
                        Pin::get_mut(self).poll_next(cx)
                    }
                }
            }
            MergeState::B => {
                match this.b.poll_next(cx) {
                    Poll::Ready(item) => {
                        *this.state = MergeState::A;
                        Poll::Ready(item)
                    }
                    Poll::Pending => {
                        *this.state = MergeState::A;
                        // Try stream A
                        Pin::get_mut(self).poll_next(cx)
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();

        let lower = a_lower.saturating_add(b_lower);
        let upper = match (a_upper, b_upper) {
            (Some(a), Some(b)) => a.checked_add(b),
            _ => None,
        };

        (lower, upper)
    }
}

pin_project! {
    /// A stream that selects from the first available stream
    ///
    /// Unlike `Merge`, this immediately polls all streams and returns
    /// the first ready item, creating more fair scheduling.
    #[derive(Debug)]
    pub struct Select<A, B> {
        #[pin]
        a: A,
        #[pin]
        b: B,
    }
}

impl<A, B> Select<A, B>
where
    A: Stream,
    B: Stream<Item = A::Item>,
{
    /// Create a new select of two streams
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A, B> Stream for Select<A, B>
where
    A: Stream,
    B: Stream<Item = A::Item>,
{
    type Item = A::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        // Poll both streams and return whichever is ready first
        let mut a_pending = false;
        let mut b_pending = false;

        // Try stream A
        match this.a.poll_next(cx) {
            Poll::Ready(item) => return Poll::Ready(item),
            Poll::Pending => a_pending = true,
        }

        // Try stream B
        match this.b.poll_next(cx) {
            Poll::Ready(item) => return Poll::Ready(item),
            Poll::Pending => b_pending = true,
        }

        // Both pending
        if a_pending && b_pending {
            Poll::Pending
        } else {
            // One stream is exhausted, check the other
            Poll::Pending
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (a_lower, a_upper) = self.a.size_hint();
        let (b_lower, b_upper) = self.b.size_hint();

        let lower = a_lower.saturating_add(b_lower);
        let upper = match (a_upper, b_upper) {
            (Some(a), Some(b)) => a.checked_add(b),
            _ => None,
        };

        (lower, upper)
    }
}

/// Extension trait for merging streams
pub trait MergeExt: Stream {
    /// Merge this stream with another
    fn merge<B>(self, other: B) -> Merge<Self, B>
    where
        Self: Sized,
        B: Stream<Item = Self::Item>,
    {
        Merge::new(self, other)
    }

    /// Select from the first available stream
    fn select<B>(self, other: B) -> Select<Self, B>
    where
        Self: Sized,
        B: Stream<Item = Self::Item>,
    {
        Select::new(self, other)
    }
}

impl<St: Stream> MergeExt for St {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::FromIter;

    #[test]
    fn test_merge_size_hint() {
        let a = FromIter::new(vec![1, 2, 3]);
        let b = FromIter::new(vec![4, 5]);
        let merged = Merge::new(a, b);
        assert_eq!(merged.size_hint(), (5, Some(5)));
    }

    #[test]
    fn test_select_size_hint() {
        let a = FromIter::new(vec![1, 2, 3]);
        let b = FromIter::new(vec![4, 5]);
        let select = Select::new(a, b);
        assert_eq!(select.size_hint(), (5, Some(5)));
    }
}
