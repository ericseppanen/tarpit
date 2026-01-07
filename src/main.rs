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
    /// Number of directories
    #[arg(long, default_value_t = 1000)]
    dirs: u64,
    /// Number of files per directory
    #[arg(long, default_value_t = 1000)]
    files_per_dir: u64,
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
        .build();

    fuser::mount2(fs, &args.mount_point, &options).unwrap();
}
