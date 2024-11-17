use afl::*;
use rwf::http::Request;
use tokio::runtime::*;

fn main() {
    let runtime = Builder::new_current_thread().enable_all().build().unwrap();
    fuzz!(|data: &[u8]| {
        runtime.block_on(async move {
            let addr = "127.0.0.1:8000".parse().unwrap();
            Request::read(addr, data).await.unwrap()
        });
    });
}
