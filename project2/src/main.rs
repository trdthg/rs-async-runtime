mod delay;
use delay::Delay;

use crossbeam::channel::{Receiver, Sender};
use futures::task::ArcWake;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
/// 是一个execotor
struct MiniTokio {
    sender: Sender<Arc<Task>>,
    receiver: Receiver<Arc<Task>>,
}

impl MiniTokio {
    pub fn new() -> Self {
        let (cx, rx) = crossbeam::channel::unbounded();
        Self {
            sender: cx,
            receiver: rx,
        }
    }

    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Task {
            future: Mutex::new(Box::pin(future)),
            sender: self.sender.clone(),
        };
        self.sender
            .send(Arc::new(task))
            .expect("spawner send new task failed");
    }

    fn run(&self) {
        while let Ok(task) = self.receiver.recv() {
            let waker = futures::task::waker(task.clone());
            let mut cx = Context::from_waker(&waker);

            let mut future = task.future.lock().expect("加锁失败");
            let res = future.as_mut().poll(&mut cx);
            match res {
                Poll::Ready(value) => {
                    println!("future is ready: {value:?}");
                }
                Poll::Pending => {
                    println!("future is pending");
                }
            }
        }
        println!("run done!");
    }
}

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    sender: Sender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self
            .sender
            .send(arc_self.clone())
            .expect("send会queue失败了");
    }
}

fn main() {
    let mut runtime = MiniTokio::new();
    runtime.spawn(async {
        let when = Instant::now() + Duration::from_secs(2);
        let future = Delay::new(when);

        let out = future.await;
        assert_eq!(out, "aaa");

        println!("{out}");
        std::process::exit(0);
    });
    runtime.run();
}
