//! Implement future scoping for tokio 0.2
use futures::Future;
use tokio::runtime::Handle;

pub fn execute_fut<T, F>(f: F, handle: &Handle) -> T
where
    T: Send,
    F: Future<Output = T> + Send,
{
    debug!("haha gonna block onto a future... freezing everything in 3, 2, 1...");
    //handle.block_on(f)
    futures::executor::block_on(f)
}