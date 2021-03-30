use futures::FutureExt;
use futures::{join, Future};
use rand::Rng;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

pub struct TestTask;
impl Future for TestTask {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("this is task!");
        let mut random = rand::thread_rng();
        let res = random.gen_range(0..100);
        if res >= 90 {
            // Poll::Ready("task finish!".to_string());
        } else {
            // cx.waker().wake_by_ref();
        }
        self.poll_unpin(cx);
        Poll::Pending
    }
}

#[test]
pub fn async_main() {
    let res = async_std::task::block_on(TestTask);
    println!("{:?}", res);
}
