// Caret Stream - Sink trait and implementations
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Future;
use pin_project_lite::pin_project;

/// Error type for sink operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkError {
    /// The sink has been closed
    Closed,
    /// An error occurred during the sink operation
    Error,
}

impl std::fmt::Display for SinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SinkError::Closed => write!(f, "Sink closed"),
            SinkError::Error => write!(f, "Sink error"),
        }
    }
}

impl std::error::Error for SinkError {}

/// A sink that can receive values
///
/// Similar to futures::Sink but designed for Caret's needs.
pub trait Sink {
    /// The type of values that can be sent to this sink
    type Item;

    /// Attempt to send a value to this sink
    ///
    /// Returns:
    /// - Poll::Ready(Ok(())) if the value was sent
    /// - Poll::Pending if the sink is not ready to accept values
    /// - Poll::Ready(Err(SinkError)) if an error occurred
    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>>;

    /// Begin flushing this sink
    ///
    /// Returns:
    /// - Poll::Ready(Ok(())) if the sink is flushed
    /// - Poll::Pending if flushing is in progress
    /// - Poll::Ready(Err(SinkError)) if an error occurred
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>>;

    /// Close this sink
    ///
    /// Returns:
    /// - Poll::Ready(Ok(())) if the sink is closed
    /// - Poll::Pending if closing is in progress
    /// - Poll::Ready(Err(SinkError)) if an error occurred
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>>;

    /// Send a value to this sink
    fn start_send(
        self: Pin<&mut Self>,
        item: Self::Item,
    ) -> Result<(), Self::Item>;

    /// Flush this sink, completing all pending sends
    fn flush(&mut self) -> Flush<'_, Self>
    where
        Self: Unpin + Sized,
    {
        Flush { sink: self }
    }

    /// Close this sink
    fn close(&mut self) -> Close<'_, Self>
    where
        Self: Unpin + Sized,
    {
        Close { sink: self }
    }

    /// A helper that returns `true` if the sink is ready to accept values
    fn is_ready(&self) -> bool
    where
        Self: Unpin,
    {
        // Default implementation assumes ready
        true
    }
}

/// Future for flushing a sink
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Flush<'a, Si: ?Sized + Unpin> {
    sink: &'a mut Si,
}

impl<Si: ?Sized + Sink + Unpin> Future for Flush<'_, Si> {
    type Output = Result<(), SinkError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.sink).poll_flush(cx)
    }
}

/// Future for closing a sink
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Close<'a, Si: ?Sized + Unpin> {
    sink: &'a mut Si,
}

impl<Si: ?Sized + Sink + Unpin> Future for Close<'_, Si> {
    type Output = Result<(), SinkError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut *self.sink).poll_close(cx)
    }
}

pin_project! {
    /// A sink that drains all items (does nothing with them)
    #[derive(Debug, Clone)]
    pub struct Drain<T> {
        _phantom: std::marker::PhantomData<T>,
        closed: bool,
    }
}

impl<T> Drain<T> {
    /// Create a new drain sink
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            closed: false,
        }
    }
}

impl<T> Default for Drain<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Sink for Drain<T> {
    type Item = T;

    fn poll_ready(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        if self.closed {
            Poll::Ready(Err(SinkError::Closed))
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        if self.closed {
            Poll::Ready(Err(SinkError::Closed))
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        self.closed = true;
        Poll::Ready(Ok(()))
    }

    fn start_send(
        self: Pin<&mut Self>,
        _item: Self::Item,
    ) -> Result<(), Self::Item> {
        if self.closed {
            Err(_item)
        } else {
            Ok(())
        }
    }
}

pin_project! {
    /// A sink that collects items into a vector
    #[derive(Debug)]
    pub struct VecSink<T> {
        vec: Vec<T>,
        capacity: usize,
        closed: bool,
    }
}

impl<T> VecSink<T> {
    /// Create a new vector sink
    pub fn new() -> Self {
        Self {
            vec: Vec::new(),
            capacity: 0,
            closed: false,
        }
    }

    /// Create a new vector sink with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
            capacity,
            closed: false,
        }
    }

    /// Extract the collected vector
    pub fn into_vec(self) -> Vec<T> {
        self.vec
    }

    /// Get a reference to the collected items
    pub fn get_ref(&self) -> &[T] {
        &self.vec
    }
}

impl<T> Default for VecSink<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Sink for VecSink<T> {
    type Item = T;

    fn poll_ready(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        if self.closed {
            Poll::Ready(Err(SinkError::Closed))
        } else if self.capacity > 0 && self.vec.len() >= self.capacity {
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        if self.closed {
            Poll::Ready(Err(SinkError::Closed))
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        self.closed = true;
        Poll::Ready(Ok(()))
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: Self::Item,
    ) -> Result<(), Self::Item> {
        if self.closed {
            Err(item)
        } else {
            // Safety: we have mutable access to self
            let mut_ref = unsafe { self.map_unchecked_mut(|s| s) };
            mut_ref.vec.push(item);
            Ok(())
        }
    }
}

pin_project! {
    /// A sink that applies a function to each item
    #[derive(Debug, Clone)]
    pub struct FnSink<T, F> {
        func: F,
        _phantom: std::marker::PhantomData<T>,
        closed: bool,
    }
}

impl<T, F> FnSink<T, F>
where
    F: FnMut(T),
{
    /// Create a new function sink
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
            closed: false,
        }
    }
}

impl<T, F> Sink for FnSink<T, F>
where
    F: FnMut(T),
{
    type Item = T;

    fn poll_ready(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        if self.closed {
            Poll::Ready(Err(SinkError::Closed))
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        if self.closed {
            Poll::Ready(Err(SinkError::Closed))
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        self.closed = true;
        Poll::Ready(Ok(()))
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: Self::Item,
    ) -> Result<(), Self::Item> {
        if self.closed {
            Err(item)
        } else {
            // Safety: we have mutable access to self
            let mut_ref = unsafe { self.map_unchecked_mut(|s| &mut s.func) };
            (mut_ref)(item);
            Ok(())
        }
    }
}

pin_project! {
    /// A sink that forwards to another sink with a transformation
    #[derive(Debug)]
    pub struct With<Si, F, T> {
        #[pin]
        sink: Si,
        func: F,
        _phantom: std::marker::PhantomData<T>,
    }
}

impl<Si, F, T> With<Si, F, T>
where
    Si: Sink,
    F: FnMut(T) -> Si::Item,
{
    /// Create a new `With` sink
    pub fn new(sink: Si, func: F) -> Self {
        Self {
            sink,
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Si, F, T> Sink for With<Si, F, T>
where
    Si: Sink + Unpin,
    F: FnMut(T) -> Si::Item,
{
    type Item = T;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        let this = self.project();
        this.sink.poll_ready(cx)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        let this = self.project();
        this.sink.poll_flush(cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        let this = self.project();
        this.sink.poll_close(cx)
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: Self::Item,
    ) -> Result<(), Self::Item> {
        let this = self.project();
        let transformed = (this.func)(item);
        this.sink.start_send(transformed)
            .map_err(|_| SinkError::Error)
    }
}

pin_project! {
    /// A sink that applies a fallible function to each item
    #[derive(Debug)]
    pub struct SinkFlatMap<Si, F, T> {
        #[pin]
        sink: Si,
        func: F,
        _phantom: std::marker::PhantomData<T>,
    }
}

impl<Si, F, T> SinkFlatMap<Si, F, T>
where
    Si: Sink,
    F: FnMut(T) -> Result<Si::Item, SinkError>,
{
    /// Create a new `SinkFlatMap` sink
    pub fn new(sink: Si, func: F) -> Self {
        Self {
            sink,
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<Si, F, T> Sink for SinkFlatMap<Si, F, T>
where
    Si: Sink + Unpin,
    F: FnMut(T) -> Result<Si::Item, SinkError>,
{
    type Item = T;

    fn poll_ready(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        let this = self.project();
        this.sink.poll_ready(cx)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        let this = self.project();
        this.sink.poll_flush(cx)
    }

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SinkError>> {
        let this = self.project();
        this.sink.poll_close(cx)
    }

    fn start_send(
        self: Pin<&mut Self>,
        item: Self::Item,
    ) -> Result<(), Self::Item> {
        let this = self.project();
        let transformed = (this.func)(item).map_err(|_| SinkError::Error)?;
        this.sink.start_send(transformed)
            .map_err(|_| SinkError::Error)?;
        Ok(())
    }
}

/// Create a drain sink
pub fn drain<T>() -> Drain<T> {
    Drain::new()
}

/// Create a vector sink
pub fn vec_sink<T>() -> VecSink<T> {
    VecSink::new()
}

/// Create a vector sink with capacity
pub fn vec_sink_with<T>(capacity: usize) -> VecSink<T> {
    VecSink::with_capacity(capacity)
}

/// Create a function sink
pub fn fn_sink<T, F>(func: F) -> FnSink<T, F>
where
    F: FnMut(T),
{
    FnSink::new(func)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drain_sink() {
        let mut sink = Drain::new();
        assert!(sink.is_ready());
        sink.start_send(42).unwrap();
        assert!(!sink.closed);
    }

    #[test]
    fn test_vec_sink() {
        let sink = VecSink::new();
        assert!(!sink.closed);
    }

    #[test]
    fn test_vec_sink_with_capacity() {
        let sink = VecSink::<i32>::with_capacity(10);
        assert_eq!(sink.vec.capacity(), 10);
    }

    #[test]
    fn test_fn_sink() {
        let mut count = 0;
        let mut sink = FnSink::new(|x: i32| {
            count += x;
        });
        sink.start_send(5).unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_sink_error_display() {
        assert_eq!(format!("{}", SinkError::Closed), "Sink closed");
        assert_eq!(format!("{}", SinkError::Error), "Sink error");
    }
}
