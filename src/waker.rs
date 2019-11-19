use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::rc::Rc;
use std::task::{Context, Waker, RawWaker, RawWakerVTable, Poll};

mod methods {
    use std::task::RawWaker;

    pub unsafe fn clone(_data: *const ()) -> RawWaker {
        panic!("waker::methods::clone")
    }

    pub unsafe fn wake(_data: *const ()) {
        panic!("waker::methods::wake")
    }

    pub unsafe fn wake_by_ref(_data: *const ()) {
        panic!("waker::methods::wake_by_ref")
    }

    pub unsafe fn drop(_data: *const ()) {
        println!("* waker::methods::drop")
    }
}

static RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    methods::clone,
    methods::wake,
    methods::wake_by_ref,
    methods::drop,
);

pub fn waker() -> Waker {
    unsafe { Waker::from_raw(RawWaker::new(ptr::null(), &RAW_WAKER_VTABLE)) }
}

pub struct Chan {
    cell: Rc<RefCell<Option<i32>>>,
}

impl Chan {
    pub fn recv(&mut self) -> ChanRecv<'_> {
        ChanRecv { chan: self }
    }
}

pub struct ChanRecv<'a> {
    chan: &'a mut Chan,
}

impl<'a> Future for ChanRecv<'a> {
    type Output = i32;

    fn poll(self: Pin<&mut Self>, _ctx: &mut Context) -> Poll<i32> {
        match self.chan.cell.borrow_mut().take() {
            Some(x) => Poll::Ready(x),
            None => Poll::Pending,
        }
    }
}

pub struct Coro<Fut> {
    cell: Rc<RefCell<Option<i32>>>,
    fut: Fut,
}

impl<Fut> Coro<Fut> where Fut: Future<Output = i32> {
    pub fn run(f: impl FnOnce(Chan) -> Fut) -> Self {
        let cell = Rc::new(RefCell::new(None));

        let fut = f(Chan { cell: cell.clone() });

        Coro { cell, fut }
    }

    pub fn advance(self: Pin<&mut Self>, value: i32) -> Poll<i32> {
        *self.cell.borrow_mut() = Some(value);
        let waker = waker();
        let mut ctx = Context::from_waker(&waker);
        let pin_fut = unsafe { self.map_unchecked_mut(|c| &mut c.fut) };
        Fut::poll(pin_fut, &mut ctx)
    }
}
