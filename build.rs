extern crate subprocess;
use subprocess::Exec;
use std::env;

// Try running "cargo build -vv" to see this output.
// For development, just run npm commands in "web" directly.

fn main() {
    env::set_current_dir("web").unwrap();
    Exec::shell("npm run build").join().unwrap();
}
