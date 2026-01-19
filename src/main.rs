use std::{env, fs};

fn init() {
    let paths = [".minigit/objects/info", ".minigit/objects/pack"];

    paths.iter().for_each(|p| fs::create_dir_all(p).unwrap());

    println!("repo initialized");
}

fn main() {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("init") => init(),
        Some(cmd) => eprintln!("unknown command: {}", cmd),
        None => eprintln!("no command provided"),
    }
}
