use clap::Parser;
use fuser::{
    FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry,
    Request,
};
use libc::ENOENT;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TTL: Duration = Duration::from_secs(1); // 1 second

const EPOCH: LazyLock<SystemTime> = LazyLock::new(|| UNIX_EPOCH + Duration::from_secs(1751364000));

fn dir_attr(ino: u64) -> FileAttr {
    FileAttr {
        ino,
        size: 0,
        blocks: 0,
        atime: *EPOCH,
        mtime: *EPOCH,
        ctime: *EPOCH,
        crtime: *EPOCH,
        kind: FileType::Directory,
        perm: 0o755,
        nlink: 2,
        uid: 501,
        gid: 20,
        rdev: 0,
        flags: 0,
        blksize: 512,
    }
}

const HELLO_TXT_CONTENT: &str = "Hello World!\n";

fn file_attr(ino: u64) -> FileAttr {
    FileAttr {
        ino,
        size: 13,
        blocks: 1,
        atime: *EPOCH,
        mtime: *EPOCH,
        ctime: *EPOCH,
        crtime: *EPOCH,
        kind: FileType::RegularFile,
        perm: 0o644,
        nlink: 1,
        uid: 501,
        gid: 20,
        rdev: 0,
        flags: 0,
        blksize: 512,
    }
}

const NUM_DIRS: u64 = 1000;

fn dir_name(num: u64) -> String {
    format!("pit{num:03}")
}

/// Directories are offset by a constant so they are kept separate from files
const DIR_INODE_OFFSET: u64 = 0x10000;

fn dir_name_to_inode(name: &str) -> Option<u64> {
    let num: u64 = name.strip_prefix("pit")?.parse().ok()?;
    dir_num_to_inode(num)
}

fn dir_num_to_inode(num: u64) -> Option<u64> {
    if num < NUM_DIRS as u64 {
        Some(num + DIR_INODE_OFFSET)
    } else {
        None
    }
}

/// returns (inode, type, name)
fn dir_num_to_dirent(num: u64) -> (u64, FileType, String) {
    let ino = dir_num_to_inode(num).unwrap();
    (ino, FileType::Directory, dir_name(num))
}

fn inode_to_dir_num(ino: u64) -> Option<u64> {
    let num = ino.checked_sub(DIR_INODE_OFFSET)?;
    if num < NUM_DIRS { Some(num) } else { None }
}

fn _inode_to_dir_name(ino: u64) -> Option<String> {
    let num = inode_to_dir_num(ino)?;
    Some(dir_name(num))
}

fn inode_to_dir_attr(ino: u64) -> Option<FileAttr> {
    let _num = inode_to_dir_num(ino)?;
    Some(dir_attr(ino))
}

struct TarpitFs;

impl Filesystem for TarpitFs {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let Some(name) = name.to_str() else {
            return reply.error(ENOENT);
        };

        // Looking at a directory entry from the top dir.
        if parent == 1
            && let Some(ino) = dir_name_to_inode(name)
        {
            let attr = inode_to_dir_attr(ino).unwrap();
            return reply.entry(&TTL, &attr, 0);
        }

        // Looking at a file from a directory.
        if let Some(_dir_num) = inode_to_dir_num(parent) {
            if name == "hello.txt" {
                return reply.entry(&TTL, &file_attr(2), 0);
            }
        }

        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        if ino == 1 {
            return reply.attr(&TTL, &dir_attr(ino));
        }

        // A single file that's linked from every subdirectory
        if ino == 2 {
            return reply.attr(&TTL, &file_attr(ino));
        }

        if let Some(attr) = inode_to_dir_attr(ino) {
            return reply.attr(&TTL, &attr);
        }

        reply.error(ENOENT)
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        if ino == 2 {
            reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let mut entries = Vec::with_capacity(NUM_DIRS as usize + 2);
        if ino == 1 {
            entries.extend([
                (1, FileType::Directory, ".".to_string()),
                (1, FileType::Directory, "..".to_string()),
            ]);
            let subdirs = (1..NUM_DIRS).into_iter().map(dir_num_to_dirent);
            entries.extend(subdirs);
        } else if let Some(_dir_num) = inode_to_dir_num(ino) {
            entries.extend([
                (ino, FileType::Directory, ".".to_string()),
                (1, FileType::Directory, "..".to_string()),
                (2, FileType::RegularFile, "hello.txt".to_string()),
            ]);
        } else {
            return reply.error(ENOENT);
        };

        // Deliberate slowdown
        std::thread::sleep(Duration::from_millis(50));

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                break;
            }
        }
        reply.ok();
    }
}

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
