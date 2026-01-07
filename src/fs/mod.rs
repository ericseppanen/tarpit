use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::{EISDIR, ENOENT, ENOTDIR};
use std::ffi::OsStr;
use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

mod inode;
use inode::{DirInode, FileInode, Inode};

use crate::fs::inode::{MAX_DIRS, MAX_FILES};

const TTL: Duration = Duration::from_secs(1); // 1 second

static EPOCH: LazyLock<SystemTime> = LazyLock::new(|| UNIX_EPOCH + Duration::from_secs(1751364000));

fn dir_attr(inode: DirInode) -> FileAttr {
    FileAttr {
        ino: inode.into(),
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

fn file_attr(inode: FileInode) -> FileAttr {
    FileAttr {
        ino: inode.into(),
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

fn dir_name(num: u64) -> String {
    format!("pit{num:03}")
}

pub struct TarpitBuilder {
    num_dirs: u64,
    num_files: u64,
}

impl Default for TarpitBuilder {
    fn default() -> Self {
        Self {
            num_dirs: 10,
            num_files: 10,
        }
    }
}

impl TarpitBuilder {
    /// Set the number of directories.
    pub fn dirs(mut self, num_dirs: u64) -> Self {
        if num_dirs > MAX_DIRS {
            panic!("number of directories is too large");
        }
        self.num_dirs = num_dirs;
        self
    }

    /// Set the number of files per directory.
    pub fn files(mut self, num_files: u64) -> Self {
        if num_files > MAX_FILES {
            panic!("number of files is too large");
        }
        self.num_files = num_files;
        self
    }

    pub fn build(self) -> TarpitFs {
        TarpitFs {
            num_dirs: self.num_dirs,
            num_files: self.num_files,
        }
    }
}

pub struct TarpitFs {
    num_dirs: u64,
    num_files: u64,
}

impl TarpitFs {
    pub fn builder() -> TarpitBuilder {
        TarpitBuilder::default()
    }

    fn dir_name_to_inode(&self, name: &str) -> Option<DirInode> {
        let num: u64 = name.strip_prefix("pit")?.parse().ok()?;
        self.dir_num_to_inode(num)
    }

    fn dir_num_to_inode(&self, num: u64) -> Option<DirInode> {
        if num <= self.num_dirs {
            // inode 1 is used by the mount point.
            DirInode::from_number(num + 1)
        } else {
            None
        }
    }

    /// returns (inode, type, name)
    fn dir_num_to_dirent(&self, num: u64) -> (DirInode, FileType, String) {
        let ino = self.dir_num_to_inode(num).unwrap();
        (ino, FileType::Directory, dir_name(num))
    }

    fn inode_to_dir(&self, ino: u64) -> Option<DirInode> {
        match Inode::from_ino_u64(ino) {
            Inode::File(_) => None,
            Inode::Dir(dir_inode) => {
                if dir_inode.num() > self.num_dirs + 1 {
                    None
                } else {
                    Some(dir_inode)
                }
            }
        }
    }

    fn inode_attr(&self, inode: Inode) -> Option<FileAttr> {
        match inode {
            Inode::Dir(dir_inode) => {
                (dir_inode.num() <= self.num_dirs).then_some(dir_attr(dir_inode))
            }
            Inode::File(file_inode) => {
                (file_inode.num() <= self.num_files).then_some(file_attr(file_inode))
            }
        }
    }
}

impl Filesystem for TarpitFs {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        #![allow(
            clippy::collapsible_if,
            reason = "right style for adding more functionality later"
        )]

        let Some(name) = name.to_str() else {
            return reply.error(ENOENT);
        };

        log::info!("lookup {parent:0x} {name:?}");

        // Looking at a directory entry from the top dir.
        if parent == 1 {
            match self.dir_name_to_inode(name) {
                Some(inode) => {
                    let attr = dir_attr(inode);
                    return reply.entry(&TTL, &attr, 0);
                }
                None => {
                    log::error!("no inode found in top dir");
                    return reply.error(ENOENT);
                }
            }
        }

        // Looking at a file from a directory.
        let parent_inode = Inode::from_ino_u64(parent);
        let Inode::Dir(parent_inode) = parent_inode else {
            log::error!("parent inode {parent:0x} not a directory");
            return reply.error(ENOENT);
        };
        if parent_inode.num() > self.num_dirs + 1 {
            log::error!("parent directory num out of range");
            return reply.error(ENOENT);
        }
        if name == "hello.txt" {
            let file = FileInode::from_number(parent_inode, 2).unwrap();
            return reply.entry(&TTL, &file_attr(file), 0);
        }

        reply.error(ENOENT);
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        let inode = Inode::from_ino_u64(ino);
        match self.inode_attr(inode) {
            Some(attr) => {
                return reply.attr(&TTL, &attr);
            }
            None => {
                return reply.error(ENOENT);
            }
        }
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
        match Inode::from_ino_u64(ino) {
            Inode::Dir(_) => {
                reply.error(EISDIR);
            }
            Inode::File(file_inode) => {
                if file_inode.num() == 2 {
                    reply.data(&HELLO_TXT_CONTENT.as_bytes()[offset as usize..]);
                } else {
                    reply.error(ENOENT);
                }
            }
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
        let Inode::Dir(dir_inode) = Inode::from_ino_u64(ino) else {
            return reply.error(ENOTDIR);
        };
        let dir_num = dir_inode.num();

        let mut entries = Vec::new();

        if dir_num == 1 {
            entries.reserve(2 + self.num_dirs as usize);
            entries.extend([
                (1, FileType::Directory, ".".to_string()),
                (1, FileType::Directory, "..".to_string()),
            ]);
            let subdirs = (1..self.num_dirs + 1).map(|dir_num| {
                let (dir, ty, name) = self.dir_num_to_dirent(dir_num);
                (dir.into(), ty, name)
            });
            entries.extend(subdirs);
        } else if dir_num <= self.num_dirs + 1 {
            let file_ino: u64 = FileInode::from_number(dir_inode, 2).unwrap().into();
            entries.extend([
                (ino, FileType::Directory, ".".to_string()),
                (1, FileType::Directory, "..".to_string()),
                (file_ino, FileType::RegularFile, "hello.txt".to_string()),
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
