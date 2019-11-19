mod waker;

use waker::{Coro, Chan};

async fn coro(mut chan: Chan) -> i32 {
    let a = chan.recv().await;
    let b = chan.recv().await;
    a + b
}

fn main() {
    let coro = Coro::run(coro);

    let mut pin_coro = Box::pin(coro);

    println!("{:?}", pin_coro.as_mut().advance(123));
    println!("{:?}", pin_coro.as_mut().advance(456));
}
