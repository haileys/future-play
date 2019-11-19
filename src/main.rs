mod coro;

use coro::{Coro, Chan};

async fn coro(mut chan: Chan<i32>) -> f64 {
    let a = chan.recv().await as f64;
    let b = chan.recv().await as f64;
    a + b
}

fn main() {
    let coro = Coro::run(coro);

    let mut pin_coro = Box::pin(coro);

    println!("{:?}", pin_coro.as_mut().advance(123));
    println!("{:?}", pin_coro.as_mut().advance(456));
}
