use core::future::Future;
use core::pin::Pin;
use core::ptr;
use core::task::{Context, Waker, RawWaker, RawWakerVTable, Poll};

use std::cell::RefCell;
use std::rc::Rc;

mod methods {
    use std::task::RawWaker;

    pub unsafe fn clone(_data: *const ()) -> RawWaker {
        super::null_raw_waker()
    }

    pub unsafe fn wake(_data: *const ()) {}

    pub unsafe fn wake_by_ref(_data: *const ()) {}

    pub unsafe fn drop(_data: *const ()) {}
}

static RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    methods::clone,
    methods::wake,
    methods::wake_by_ref,
    methods::drop,
);

fn null_raw_waker() -> RawWaker {
    RawWaker::new(ptr::null(), &RAW_WAKER_VTABLE)
}

fn null_waker() -> Waker {
    unsafe { Waker::from_raw(null_raw_waker()) }
}

pub struct Chan<T> {
    cell: Rc<RefCell<Option<T>>>,
}

impl<T> Chan<T> {
    pub fn recv(&self) -> ChanRecv<'_, T> {
        ChanRecv { chan: self }
    }
}

pub struct ChanRecv<'a, T> {
    chan: &'a Chan<T>,
}

impl<'a, T> Future for ChanRecv<'a, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context) -> Poll<T> {
        match self.chan.cell.borrow_mut().take() {
            Some(x) => Poll::Ready(x),
            None => Poll::Pending,
        }
    }
}

pub struct Coro<T, Fut> {
    cell: Rc<RefCell<Option<T>>>,
    fut: Fut,
}

impl<T, Fut> Coro<T, Fut> where Fut: Future {
    pub fn run(f: impl FnOnce(Chan<T>) -> Fut) -> Self {
        let cell = Rc::new(RefCell::new(None));

        let fut = f(Chan { cell: cell.clone() });

        Coro { cell, fut }
    }

    pub fn advance(mut self: Pin<&mut Self>, value: T) -> Poll<Fut::Output> {
        *self.cell.borrow_mut() = Some(value);
        let waker = null_waker();
        let mut ctx = Context::from_waker(&waker);
        let pin_fut = unsafe { self.as_mut().map_unchecked_mut(|c| &mut c.fut) };
        Fut::poll(pin_fut, &mut ctx)
    }
}
