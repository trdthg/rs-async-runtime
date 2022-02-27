use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

// 为 Delay 类型实现 Future 特征
impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            Poll::Ready("aaa")
        } else {
            // 下面的wake其实是没有必要的, 只要还是pending, executor就会将task添加到队尾, execotor的操作其实就扮演了waker的身份

            // 只要没有ready就重新加入队列，这种做法执行失败就立即重会占用大量cpu资源
            // 应该等到ready是在重新唤醒(加入队列)
            // cx.waker().wake_by_ref();

            // 下面的就是稍微优化过的wake策略, 但是目前runtime还不能接受wake通知
            // let waker = cx.waker().clone();
            // let when = self.when;

            // // 生成一个计时器线程
            // thread::spawn(move || {
            //     let now = Instant::now();
            //     if now < when {
            //         thread::sleep(when - now);
            //     }
            //     waker.wake();
            // });

            Poll::Pending
        }
    }
}
pub struct Delay {
    when: Instant,
}

impl Delay {
    pub fn new(when: Instant) -> Self {
        Self { when }
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
