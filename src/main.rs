use clap::Parser;
use fuser::MountOption;

use std::path::PathBuf;

use crate::fs::TarpitFs;

mod fs;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(long)]
    auto_unmount: bool,
    #[arg(long)]
    allow_root: bool,
    mount_point: PathBuf,
}

fn main() {
    let args = Args::parse();

    env_logger::init();
    let mut options = vec![MountOption::RO, MountOption::FSName("tarpit".into())];
    if args.auto_unmount {
        options.push(MountOption::AutoUnmount);
    }
    if args.allow_root {
        options.push(MountOption::AllowRoot);
    }
    fuser::mount2(TarpitFs, &args.mount_point, &options).unwrap();
}
