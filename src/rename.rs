use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::norm::{Form, convert};

pub struct RenameOp {
    pub dir: PathBuf,
    pub from: std::ffi::OsString,
    pub to: std::ffi::OsString,
}

impl RenameOp {
    pub fn src_path(&self) -> PathBuf {
        self.dir.join(&self.from)
    }

    pub fn dst_path(&self) -> PathBuf {
        self.dir.join(&self.to)
    }
}

pub fn collect_ops(paths: &[PathBuf], form: Form, recursive: bool) -> Vec<RenameOp> {
    let max_depth = if recursive { usize::MAX } else { 1 };
    let mut ops = Vec::new();

    for path in paths {
        for entry in WalkDir::new(path)
            .min_depth(1)
            .max_depth(max_depth)
            .contents_first(true)
        {
            let Ok(entry) = entry else { continue };
            let Some(file_name) = entry.file_name().to_str() else {
                continue;
            };
            let converted = convert(file_name, form);
            if converted == file_name {
                continue;
            }
            let dir = entry
                .path()
                .parent()
                .expect("walkdir min_depth(1) guarantees a parent")
                .to_path_buf();
            ops.push(RenameOp {
                dir,
                from: entry.file_name().to_owned(),
                to: converted.into(),
            });
        }
    }

    ops
}

fn same_inode(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (a.metadata(), b.metadata()) {
        (Ok(ma), Ok(mb)) => ma.ino() == mb.ino(),
        _ => false,
    }
}

pub fn check_op(op: &RenameOp) -> anyhow::Result<()> {
    let from = op.src_path();
    let to = op.dst_path();
    if to.exists() && !same_inode(&from, &to) {
        anyhow::bail!("destination already exists: {}", to.display());
    }
    Ok(())
}

pub fn execute_op(op: &RenameOp) -> anyhow::Result<()> {
    check_op(op)?;

    let from = op.src_path();
    let to = op.dst_path();

    // On APFS, NFC and NFD names resolve to the same inode, so rename(nfd, nfc) is a no-op.
    // Use a temporary file as an intermediate step, following the same approach as convmv.
    if same_inode(&from, &to) {
        let tmp = (1u32..)
            .map(|i| op.dir.join(format!("ucmvtmp{i}")))
            .find(|p| !p.exists())
            .unwrap();
        std::fs::rename(&from, &tmp)?;
        std::fs::rename(&tmp, &to)?;
    } else {
        std::fs::rename(&from, &to)?;
    }

    Ok(())
}
