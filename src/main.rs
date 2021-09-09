use platform;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = platform::commands::new_app();
    app.run(args);
}
