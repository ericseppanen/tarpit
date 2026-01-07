use clap::Parser;
use fuser::MountOption;

use std::path::PathBuf;
use std::time::Duration;

use crate::fs::TarpitFs;

mod fs;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(long)]
    auto_unmount: bool,
    #[arg(long)]
    allow_root: bool,
    mount_point: PathBuf,
    /// Number of directories
    #[arg(long, default_value_t = 1000)]
    dirs: u64,
    /// Number of files per directory
    #[arg(long, default_value_t = 1000)]
    files_per_dir: u64,
    /// How much to slow down filesystem operations
    #[arg(long, default_value_t = 0)]
    slowdown_ms: u64,
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

    let fs = TarpitFs::builder()
        .dirs(args.dirs)
        .files(args.files_per_dir)
        .slowdown(Duration::from_millis(args.slowdown_ms))
        .build();

    fuser::mount2(fs, &args.mount_point, &options).unwrap();
}
