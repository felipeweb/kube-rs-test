use seahorse::App;
mod crdgen;
mod server;
pub fn new_app() -> App {
    return App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("cli [name]")
        .command(server::server_command())
        .command(crdgen::crdgen_command());
}
