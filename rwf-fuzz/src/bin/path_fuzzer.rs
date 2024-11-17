use afl::*;
use rwf::http::Path;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = Path::parse(s);
        }
    });
}
