use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

mod waker;

struct Chan {
    items: RefCell<VecDeque<i32>>,
}

impl Chan {
    pub fn new() -> Self {
        Chan { items: RefCell::new(VecDeque::new()) }
    }

    pub fn send(&self, value: i32) {
        self.items.borrow_mut().push_front(value)
    }

    pub fn recv(&self) -> ChanRecv<'_> {
        ChanRecv { src: self }
    }
}

struct ChanRecv<'a> {
    src: &'a Chan,
}

impl<'a> Future for ChanRecv<'a> {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<i32> {
        match self.src.items.borrow_mut().pop_front() {
            None => Poll::Pending,
            Some(x) => Poll::Ready(x),
        }
    }
}

async fn get(src: &Chan) -> i32 {
    src.recv().await
}

async fn add(a: i32, b: i32) -> i32 {
    a + b
}

async fn foo(src: &Chan) -> i32 {
    add(get(src).await, get(src).await).await
}

fn main() {
    let chan = Chan::new();

    let fut = foo(&chan);

    let mut pf = Box::pin(fut);

    let waker = waker::waker();
    let mut ctx = Context::from_waker(&waker);

    println!("{:?}", pf.as_mut().poll(&mut ctx));

    chan.send(123);

    println!("{:?}", pf.as_mut().poll(&mut ctx));

    chan.send(456);

    println!("{:?}", pf.as_mut().poll(&mut ctx));
}
