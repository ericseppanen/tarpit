pub const MAX_DIRS: u64 = (1 << 32) - 1;

pub const MAX_FILES: u64 = (1 << 32) - 1;

/// A directory inode number.
///
/// When converted to a `u64`, the directory number will use the lower 32 bits.
///
/// The number `0` is reserved.
/// The number `1` is used for the mount point.
#[derive(Copy, Clone, Debug)]
pub struct DirInode(u32);

/// A file inode number.
///
/// When converted to a `u64`, the value will contain its directory inode number
/// in the lower 32 bits, and the file number in the upper 32 bits.
///
/// The number `0` is reserved for the parent directory itself.
///
#[derive(Copy, Clone, Debug)]
pub struct FileInode(DirInode, u32);

pub enum Inode {
    Dir(DirInode),
    File(FileInode),
}

impl From<DirInode> for Inode {
    fn from(dir_inode: DirInode) -> Self {
        Self::Dir(dir_inode)
    }
}

impl From<FileInode> for Inode {
    fn from(file_inode: FileInode) -> Self {
        Self::File(file_inode)
    }
}

impl From<DirInode> for u64 {
    fn from(inode: DirInode) -> Self {
        inode.0 as u64
    }
}

impl From<FileInode> for u64 {
    fn from(inode: FileInode) -> Self {
        // Lower 32 bits identify the directory; upper 32 bits identify the file.
        let dir_num = inode.0.0;
        let file_num = inode.1;
        ((file_num as u64) << 32) | (dir_num as u64)
    }
}

impl From<Inode> for u64 {
    fn from(inode: Inode) -> Self {
        match inode {
            Inode::Dir(dir_inode) => dir_inode.into(),
            Inode::File(file_inode) => file_inode.into(),
        }
    }
}

impl DirInode {
    pub fn from_number(num: u64) -> Option<Self> {
        if num > MAX_DIRS {
            None
        } else {
            Some(Self(num.try_into().unwrap()))
        }
    }

    pub fn num(&self) -> u64 {
        self.0.into()
    }
}

impl FileInode {
    pub fn from_number(parent: DirInode, num: u64) -> Option<Self> {
        if num > MAX_FILES {
            None
        } else {
            Some(Self(parent, num.try_into().unwrap()))
        }
    }

    pub fn num(&self) -> u64 {
        self.1.into()
    }
}
impl Inode {
    pub fn from_ino_u64(ino: u64) -> Self {
        let dir_number = (ino & 0xFFFF_FFFF) as u32;
        let file_number = (ino >> 32) as u32;

        if file_number == 0 {
            assert!(dir_number != 0);
            DirInode(dir_number).into()
        } else {
            FileInode(DirInode(dir_number), file_number).into()
        }
    }
}
