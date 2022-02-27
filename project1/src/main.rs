mod delay;
use delay::Delay;

use std::collections::VecDeque;
use std::future::{self, Future};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

/// 是一个execotor
struct MiniTokio {
    tasks: VecDeque<Task>,
}

type Task = Pin<Box<dyn Future<Output = ()>>>;

impl MiniTokio {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    fn spawn<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.push_back(Box::pin(f));
    }

    fn run(&mut self) {
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        while let Some(mut task) = self.tasks.pop_front() {
            if task.as_mut().poll(&mut cx).is_pending() {
                println!("a");
                self.tasks.push_back(task);
            }
        }
    }
}

fn main() {
    let mut runtime = MiniTokio::new();
    runtime.spawn(async {
        let when = Instant::now() + Duration::from_secs(2);
        let future = Delay::new(when);

        let out = future.await;
        println!("{out}");
        assert_eq!(out, "aaa");
    });
    runtime.spawn(async {
        let when = Instant::now() + Duration::from_secs(2);
        let future = Delay::new(when);

        let out = future.await;
        println!("{out}");
        assert_eq!(out, "aaa");
    });
    runtime.run();
}
