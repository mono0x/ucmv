use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::norm::{Form, convert};

pub struct RenameOp {
    pub from: PathBuf,
    pub to: PathBuf,
}

pub fn collect_ops(paths: &[PathBuf], form: &Form, recursive: bool) -> Vec<RenameOp> {
    let max_depth = if recursive { usize::MAX } else { 1 };
    let mut ops = Vec::new();
    // Maps original parent path -> renamed parent path.
    let mut prefix_map: HashMap<PathBuf, PathBuf> = HashMap::new();

    for path in paths {
        for entry in WalkDir::new(path)
            .min_depth(1)
            .max_depth(max_depth)
            .sort_by_file_name()
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let original = entry.into_path();

            // Remap parent if it was renamed.
            let from = match original.parent().and_then(|p| prefix_map.get(p)) {
                Some(new_parent) => new_parent.join(original.file_name().unwrap()),
                None => original.clone(),
            };

            let file_name = match from.file_name().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => continue,
            };
            let converted = convert(file_name, form);
            if converted == file_name {
                continue;
            }
            let to = from.parent().unwrap_or(&from).join(&converted);
            prefix_map.insert(original, to.clone());
            ops.push(RenameOp { from, to });
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
