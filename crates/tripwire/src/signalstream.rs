//! Provides [SignalStream] (turns signals into a [Stream])

use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

#[cfg(unix)]
use tokio::signal::unix::Signal;

#[cfg(windows)]
use tokio::signal::windows::CtrlC;

/// A wrapper around platform-specific signals that implements [Stream].
#[derive(Debug)]
pub struct SignalStream {
    #[cfg(unix)]
    inner: Signal,
    #[cfg(windows)]
    inner: CtrlC,
}

impl SignalStream {
    /// Create a new `SignalStream` from a Unix signal.
    #[cfg(unix)]
    pub fn new(signal: Signal) -> Self {
        Self { inner: signal }
    }

    /// Create a new `SignalStream` from a Windows CtrlC handler.
    #[cfg(windows)]
    pub fn new(signal: CtrlC) -> Self {
        Self { inner: signal }
    }

    /// Get back the inner signal.
    #[cfg(unix)]
    pub fn into_inner(self) -> Signal {
        self.inner
    }

    /// Get back the inner CtrlC handler.
    #[cfg(windows)]
    pub fn into_inner(self) -> CtrlC {
        self.inner
    }
}

impl Stream for SignalStream {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<()>> {
        #[cfg(unix)]
        {
            self.inner.poll_recv(cx)
        }
        #[cfg(windows)]
        {
            match Pin::new(&mut self.inner).poll(cx) {
                Poll::Ready(Ok(())) => Poll::Ready(Some(())),
                Poll::Ready(Err(_)) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

#[cfg(unix)]
impl AsRef<Signal> for SignalStream {
    fn as_ref(&self) -> &Signal {
        &self.inner
    }
}

#[cfg(unix)]
impl AsMut<Signal> for SignalStream {
    fn as_mut(&mut self) -> &mut Signal {
        &mut self.inner
    }
}

#[cfg(windows)]
impl AsRef<CtrlC> for SignalStream {
    fn as_ref(&self) -> &CtrlC {
        &self.inner
    }
}

#[cfg(windows)]
impl AsMut<CtrlC> for SignalStream {
    fn as_mut(&mut self) -> &mut CtrlC {
        &mut self.inner
    }
}
