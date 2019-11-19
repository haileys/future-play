use std::ptr;
use std::task::{Waker, RawWaker, RawWakerVTable};

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
