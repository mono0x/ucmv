use std::path::PathBuf;
use walkdir::WalkDir;

use crate::norm::{convert, Form};

pub struct RenameOp {
    pub from: PathBuf,
    pub to: PathBuf,
}

pub fn collect_ops(paths: &[PathBuf], form: &Form, recursive: bool) -> Vec<RenameOp> {
    let max_depth = if recursive { usize::MAX } else { 1 };
    let mut ops = Vec::new();

    for path in paths {
        for entry in WalkDir::new(path).min_depth(1).max_depth(max_depth) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let file_name = match entry.file_name().to_str() {
                Some(s) => s,
                None => continue,
            };
            let converted = convert(file_name, form);
            if converted == file_name {
                continue;
            }
            let to = entry
                .path()
                .parent()
                .unwrap_or(entry.path())
                .join(&converted);
            ops.push(RenameOp {
                from: entry.into_path(),
                to,
            });
        }
    }

    ops
}

fn same_inode(a: &std::path::Path, b: &std::path::Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (a.metadata(), b.metadata()) {
        (Ok(ma), Ok(mb)) => ma.ino() == mb.ino(),
        _ => false,
    }
}

pub fn check_op(op: &RenameOp) -> anyhow::Result<()> {
    if op.to.exists() && !same_inode(&op.from, &op.to) {
        anyhow::bail!("destination already exists: {}", op.to.display());
    }
    Ok(())
}

pub fn execute_op(op: &RenameOp) -> anyhow::Result<()> {
    check_op(op)?;

    // On APFS, NFC and NFD names resolve to the same inode, so rename(nfd, nfc) is a no-op.
    // Use a temporary file as an intermediate step, following the same approach as convmv.
    if same_inode(&op.from, &op.to) {
        let dir = op.from.parent().unwrap_or(std::path::Path::new("."));
        let tmp = (1u32..)
            .map(|i| dir.join(format!("ucmvtmp{i}")))
            .find(|p| !p.exists())
            .unwrap();
        std::fs::rename(&op.from, &tmp)?;
        std::fs::rename(&tmp, &op.to)?;
    } else {
        std::fs::rename(&op.from, &op.to)?;
    }

    Ok(())
}
