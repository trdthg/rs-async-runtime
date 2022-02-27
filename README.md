# 一些简单的异步运行时💙

## project1
实现了一个最小的runtime

### 框架
- executor(MiniTokio): 保存任务队列，不断尝试poll每个task，如果任务完成就移除队列，如果没有完成就加到队尾
    ```rs
    struct MiniTokio {
        tasks: VecDeque<Task>,
    }
    ```
- task: 封装了future
    ```rs
    type Task = Pin<Box<dyn Future<Output = ()>>>;
    ```
- spawner: 作为runtime的函数，将task添加到队尾
    ```rs
    fn spawn<F>(&mut self, f: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.push_back(Box::pin(f));
    }
    ```
### project2
execotor本身的push_back操作就是wake的实现

只要没有ready就重新加入队列，这种做法执行失败就立即重会占用大量cpu资源, 应该等到ready是在重新唤醒(加入队列)
```rs
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if (Instant::now() >= self.when) {
        Poll::Ready("aaa")
    } else {
        Poll::Pending
    }
}
```

## project2
使用sender和receiver传递任务
### 框架
- execotor: 只需要一个receiver, 不断尝试接受任务去poll, 结果是什么无所谓
    ```rs
    struct MiniTokio {
        sender: Sender<Arc<Task>>,  // 等会在说这个
        receiver: Receiver<Arc<Task>>,
    }
    ```
- task: 除了future还有一个sender, task实现了Waker, 当task pending时会按照策略调用wake方法, 把自己send到execotor
    ```rs
    struct Task {
        future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
        sender: Sender<Arc<Task>>,
    }

    impl ArcWake for Task {
        fn wake_by_ref(arc_self: &Arc<Self>) {
            arc_self
                .sender
                .send(arc_self.clone())
        }
    }
    ```
- spawner: 因为execotor现在同时保留着sender和receiver, 两者都不会被drop, 程序不能正常退出, 下一步需要将这两个分离
    ```rs
    fn spawn<F>(&self, future: F)
        where
            F: Future<Output = ()> + Send + 'static,
        {
            let task = Task {
                future: Mutex::new(Box::pin(future)),
                sender: self.sender.clone(),
            };
            self.sender.send(Arc::new(task))
        }
    ```

### project3
1. 分离executor(receiver) 和 spawner(sender), 当receiver运行结束后receiver就销毁
```rs
struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>,
}

struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}
```