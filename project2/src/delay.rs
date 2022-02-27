use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

pub struct Delay {
    when: Instant,
}

impl Delay {
    pub fn new(when: Instant) -> Self {
        Self { when }
    }
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            Poll::Ready("aaa")
        } else {
            println!("a");
            // cx.waker().wake_by_ref();
            let waker = cx.waker().clone();
            let when = self.when;
            thread::spawn(move || {
                let now = Instant::now();
                if now < when {
                    thread::sleep(when - now);
                }
                waker.wake();
            });
            Poll::Pending
        }
    }
}

#[tokio::test]
async fn test_delay() {
    let delay = Delay {
        when: Instant::now() + Duration::from_secs(2),
    };

    let out = delay.await;
    assert_eq!(out, "aaa");
}
