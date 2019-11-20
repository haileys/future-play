mod both;
mod coro;

use both::both;
use coro::{Coro, Chan};

#[inline(never)]
async fn do_recv(chan: &Chan<i32>) -> f64 {
    let val = chan.recv().await as f64;
    // panic!("whoops!");
    val
}

async fn coro(mut chan: Chan<i32>) -> f64 {
    let (a, b) = both(do_recv(&chan), do_recv(&chan)).await;
    a + b
}

fn main() {
    let coro = Coro::run(coro);

    let mut pin_coro = Box::pin(coro);

    println!("{:?}", pin_coro.as_mut().advance(123));
    println!("{:?}", pin_coro.as_mut().advance(456));
}
