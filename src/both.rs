use core::future::Future;
use core::mem;
use core::pin::Pin;
use core::ptr;
use core::task::{Context, Waker, RawWaker, RawWakerVTable, Poll};

enum Fut<F: Future> {
    Pending(F),
    Ready(F::Output),
    Taken,
}

impl<F: Future> Fut<F> {
    pub fn new(fut: F) -> Self {
        Fut::Pending(fut)
    }

    pub fn take_ready(&mut self) -> F::Output {
        match self {
            Fut::Pending(_) => panic!("take_ready on pending Fut"),
            Fut::Ready(_) => {
                let ready = mem::replace(self, Fut::Taken);

                if let Fut::Ready(value) = ready {
                    value
                } else {
                    unreachable!()
                }
            }
            Fut::Taken => panic!("take_ready on taken Fut"),
        }
    }

    pub fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> bool {
        let self_ref = unsafe { Pin::get_unchecked_mut(self) };

        let fut = match self_ref {
            Fut::Pending(f) => unsafe { Pin::new_unchecked(f) },
            Fut::Ready(_) => return true,
            Fut::Taken => panic!("poll on taken Fut"),
        };

        let result = F::poll(fut, cx);

        match result {
            Poll::Pending => {
                false
            }
            Poll::Ready(value) => {
                let self_ptr = self_ref as *mut _;
                unsafe {
                    ptr::drop_in_place(self_ptr);
                    ptr::write(self_ptr, Fut::Ready(value));
                }
                true
            }
        }
    }
}

pub struct Both<F: Future, G: Future> {
    f: Fut<F>,
    g: Fut<G>,
}

impl<F: Future, G: Future> Future for Both<F, G> {
    type Output = (F::Output, G::Output);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let self_ref = unsafe { Pin::get_unchecked_mut(self) };

        let f_ready = unsafe { Pin::new_unchecked(&mut self_ref.f) }.poll(cx);
        let g_ready = unsafe { Pin::new_unchecked(&mut self_ref.g) }.poll(cx);

        if f_ready && g_ready {
            let f_value = self_ref.f.take_ready();
            let g_value = self_ref.g.take_ready();
            return Poll::Ready((f_value, g_value));
        } else {
            return Poll::Pending;
        }
    }
}

pub fn both<F, G>(
    f: impl Future<Output = F>,
    g: impl Future<Output = G>,
) -> impl Future<Output = (F, G)> {
    Both { f: Fut::Pending(f), g: Fut::Pending(g) }
}
