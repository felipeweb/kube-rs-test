#![allow(unused_variables)]
use crate::k8sserver;
use seahorse::{Command, Context, Flag, FlagType};

pub fn server_command() -> Command {
    Command::new("server")
        .action(server_action)
        .flag(Flag::new("addr", FlagType::String).alias("a"))
}

fn server_action(c: &Context) {
    let addr = c
        .string_flag("addr")
        .ok()
        .or(Option::from("0.0.0.0:8080".to_owned()));
    let _ = k8sserver::start_server(addr.unwrap());
}
