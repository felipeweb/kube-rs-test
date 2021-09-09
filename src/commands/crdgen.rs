#![allow(unused_variables)]
use crate::manager;
use kube::CustomResourceExt;
use seahorse::{Command, Context};
use std::io;
use std::io::Write;

pub fn crdgen_command() -> Command {
    Command::new("crdgen").action(crdgen_action)
}

fn crdgen_action(c: &Context) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = handle.write_all(
        serde_yaml::to_string(&manager::Foo::crd())
            .unwrap()
            .as_bytes(),
    );
}
