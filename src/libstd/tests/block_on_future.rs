//! Tests for the block_on_future function

#![feature(block_on_future)]

use std::future::Future;
use std::task::{Context, Poll, Waker};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::thread::{block_on_future, JoinHandle, spawn};

struct WakeFromRemoteThreadFuture {
    was_polled: bool,
    wake_by_ref: bool,
    join_handle: Option<JoinHandle<()>>,
}

impl WakeFromRemoteThreadFuture {
    fn new(wake_by_ref: bool) -> Self {
        WakeFromRemoteThreadFuture {
            was_polled: false,
            wake_by_ref,
            join_handle: None,
        }
    }
}

impl Future for WakeFromRemoteThreadFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if !self.was_polled {
            self.was_polled = true;
            let waker = cx.waker().clone();
            let wake_by_ref = self.wake_by_ref;
            self.join_handle = Some(spawn(move || {
                if wake_by_ref {
                    waker.wake();
                } else {
                    waker.wake_by_ref();
                }
            }));
            Poll::Pending
        } else {
            if let Some(handle) = self.join_handle.take() {
                handle.join().unwrap();
            }
            Poll::Ready(())
        }
    }
}

struct Yield {
    iterations: usize,
}

impl Yield {
    fn new(iterations: usize) -> Self {
        Yield {
            iterations
        }
    }
}

impl Future for Yield {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.iterations == 0 {
            Poll::Ready(())
        } else {
            self.iterations -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

struct NeverReady {
}

impl NeverReady {
    fn new() -> Self {
        NeverReady {}
    }
}

impl Future for NeverReady {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Pending
    }
}

struct WakerStore {
    waker: Option<Waker>,
}

struct StoreWakerFuture {
    store: Arc<Mutex<WakerStore>>,
}

impl StoreWakerFuture {
    fn new(store: Arc<Mutex<WakerStore>>) -> Self {
        StoreWakerFuture {
            store
        }
    }
}

impl Future for StoreWakerFuture {
    type Output = ();

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        (*self.store.lock().unwrap()).waker = Some(cx.waker().clone());
        Poll::Ready(())
    }
}

struct WakeFromPreviouslyStoredWakerFuture {
    store: Arc<Mutex<WakerStore>>,
    was_polled: bool,
    join_handle: Option<JoinHandle<()>>,
}

impl WakeFromPreviouslyStoredWakerFuture {
    fn new(store: Arc<Mutex<WakerStore>>) -> Self {
        WakeFromPreviouslyStoredWakerFuture {
            store,
            was_polled: false,
            join_handle: None,
        }
    }
}

impl Future for WakeFromPreviouslyStoredWakerFuture {
    type Output = ();

    fn poll(mut self: core::pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if !self.was_polled {
            self.was_polled = true;
            // Don't take the waker from Context but from the side channel
            let waker = self.store.lock().unwrap().waker.clone().take().unwrap();
            self.join_handle = Some(spawn(move || {
                waker.wake();
            }));
            Poll::Pending
        } else {
            if let Some(handle) = self.join_handle.take() {
                handle.join().unwrap();
            }
            Poll::Ready(())
        }
    }
}

#[test]
fn wake_from_local_thread() {
    block_on_future(async {
        Yield::new(10).await;
    });
}

#[test]
fn wake_from_foreign_thread() {
    block_on_future(async {
        WakeFromRemoteThreadFuture::new(false).await;
    });
}

#[test]
fn wake_by_ref_from_foreign_thread() {
    block_on_future(async {
        WakeFromRemoteThreadFuture::new(true).await;
    });
}

#[test]
fn wake_from_multiple_threads() {
    block_on_future(async {
        WakeFromRemoteThreadFuture::new(false).await;
        WakeFromRemoteThreadFuture::new(true).await;
    });
}

#[test]
fn wake_local_remote_local() {
    block_on_future(async {
        Yield::new(10).await;
        WakeFromRemoteThreadFuture::new(false).await;
        Yield::new(20).await;
        WakeFromRemoteThreadFuture::new(true).await;
    });
}

#[test]
fn returns_result_from_task() {
    let result = block_on_future(async {
        let x = 42i32;
        Yield::new(10).await;
        x
    });
    assert_eq!(42, result);
}

#[test]
#[should_panic]
fn panics_if_waker_was_not_cloned_and_task_is_not_ready() {
    block_on_future(async {
        NeverReady::new().await;
    });
}

#[test]
fn does_not_panic_if_waker_is_cloned_and_used_a_lot_later() {
    let store = Arc::new(Mutex::new(WakerStore {
        waker: None,
    }));

    block_on_future(async {
        StoreWakerFuture::new(store.clone()).await;
        Yield::new(10).await;
        // Multiple wakes from an outdated waker - because it can
        // have been cloned multiple times.
        WakeFromPreviouslyStoredWakerFuture::new(store.clone()).await;
        WakeFromPreviouslyStoredWakerFuture::new(store.clone()).await;
        WakeFromPreviouslyStoredWakerFuture::new(store).await;
    });
}
