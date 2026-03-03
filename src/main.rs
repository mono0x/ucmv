mod cli;
mod norm;
mod rename;

use clap::Parser;
use cli::Args;
use norm::Form;
use rename::{check_op, collect_ops, execute_op};

fn run(
    paths: &[std::path::PathBuf],
    form: Form,
    notest: bool,
    recursive: bool,
) -> anyhow::Result<()> {
    let ops = collect_ops(paths, &form, recursive);

    if ops.is_empty() {
        println!("No files to rename.");
        return Ok(());
    }

    for op in &ops {
        println!(
            "{} -> {}",
            op.dir.join(&op.from).display(),
            op.dir.join(&op.to).display()
        );
        if let Err(e) = check_op(op) {
            eprintln!("Error: {e}");
        }
    }

    if notest {
        for op in &ops {
            if let Err(e) = execute_op(op) {
                eprintln!("Error: {e}");
            }
        }
    } else {
        println!("No files renamed. Use --notest to rename files.");
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let form = if args.nfc { Form::Nfc } else { Form::Nfd };
    run(&args.paths, form, args.notest, args.recursive)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use unicode_normalization::IsNormalized;

    // NFD: U+304B + U+3099 (decomposed form of が)
    const GA_NFD: &str = "\u{304B}\u{3099}";
    // NFC: U+304C (precomposed form of が)
    const GA_NFC: &str = "\u{304C}";

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn ls(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    // dry-run: files must not be renamed
    #[test]
    fn dry_run_does_not_rename() {
        let dir = tempdir();
        fs::write(dir.path().join(format!("{GA_NFD}.txt")), "").unwrap();

        run(&[dir.path().to_path_buf()], Form::Nfc, false, false).unwrap();

        let names = ls(dir.path());
        assert_eq!(names.len(), 1);
        assert!(names[0].contains(GA_NFD) || !names[0].contains(GA_NFC));
    }

    // dry-run: must not panic even when a conflict exists
    #[test]
    fn dry_run_reports_conflict() {
        let dir = tempdir();
        fs::write(dir.path().join(format!("{GA_NFD}.txt")), "nfd").unwrap();
        // On macOS this overwrites the same file, but still verifies no panic occurs
        fs::write(dir.path().join(format!("{GA_NFC}.txt")), "nfc").unwrap();

        let result = run(&[dir.path().to_path_buf()], Form::Nfc, false, false);
        assert!(result.is_ok());
    }

    // When an NFC file with a different inode already exists, execute_op must return an error
    // and leave both files intact.
    #[test]
    fn execute_does_not_overwrite_existing_file() {
        let dir = tempdir();
        let nfd_path = dir.path().join(format!("{GA_NFD}.txt"));
        let nfc_path = dir.path().join(format!("{GA_NFC}.txt"));

        fs::write(&nfd_path, "nfd content").unwrap();
        fs::write(&nfc_path, "nfc content").unwrap();

        // On macOS, NFC and NFD resolve to the same inode, so there is no conflict to test.
        let op = rename::RenameOp {
            dir: dir.path().to_path_buf(),
            from: format!("{GA_NFD}.txt").into(),
            to: format!("{GA_NFC}.txt").into(),
        };
        #[cfg(not(target_os = "macos"))]
        {
            let result = execute_op(&op);
            assert!(result.is_err(), "expected error on conflict");
            assert_eq!(fs::read_to_string(&nfc_path).unwrap(), "nfc content");
            assert_eq!(fs::read_to_string(&nfd_path).unwrap(), "nfd content");
        }
        #[cfg(target_os = "macos")]
        {
            // On macOS, NFC and NFD share the same inode, so the rename succeeds without conflict.
            let result = execute_op(&op);
            assert!(result.is_ok());
        }
    }

    // --notest: NFD filename must be renamed to NFC
    #[test]
    fn execute_renames_nfd_to_nfc() {
        let dir = tempdir();
        fs::write(dir.path().join(format!("{GA_NFD}.txt")), "").unwrap();

        run(&[dir.path().to_path_buf()], Form::Nfc, true, false).unwrap();

        let names = ls(dir.path());
        assert_eq!(names.len(), 1);
        let name = &names[0];
        assert!(
            unicode_normalization::is_nfc_quick(name.chars()) != IsNormalized::No,
            "expected NFC filename, got: {name:?}"
        );
    }

    // --notest: NFC filename must be renamed to NFD
    #[test]
    fn execute_renames_nfc_to_nfd() {
        let dir = tempdir();
        fs::write(dir.path().join(format!("{GA_NFC}.txt")), "").unwrap();

        run(&[dir.path().to_path_buf()], Form::Nfd, true, false).unwrap();

        let names = ls(dir.path());
        assert_eq!(names.len(), 1);
        let name = &names[0];
        assert!(
            unicode_normalization::is_nfd_quick(name.chars()) != IsNormalized::No,
            "expected NFD filename, got: {name:?}"
        );
    }

    // --notest: already-normalized files must be left unchanged
    #[test]
    fn execute_skips_already_normalized() {
        let dir = tempdir();
        fs::write(dir.path().join("ascii.txt"), "").unwrap();

        run(&[dir.path().to_path_buf()], Form::Nfc, true, false).unwrap();

        assert_eq!(ls(dir.path()), vec!["ascii.txt"]);
    }

    // -r: files in subdirectories must also be renamed
    #[test]
    fn recursive_renames_subdirectory() {
        let dir = tempdir();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join(format!("{GA_NFD}.txt")), "").unwrap();

        run(&[dir.path().to_path_buf()], Form::Nfc, true, true).unwrap();

        let names = ls(&sub);
        assert_eq!(names.len(), 1);
        let name = &names[0];
        assert!(
            unicode_normalization::is_nfc_quick(name.chars()) != IsNormalized::No,
            "expected NFC filename, got: {name:?}"
        );
    }

    // without -r: files in subdirectories must not be renamed
    #[test]
    fn non_recursive_skips_subdirectory() {
        let dir = tempdir();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join(format!("{GA_NFD}.txt")), "").unwrap();

        run(&[dir.path().to_path_buf()], Form::Nfc, true, false).unwrap();

        let names = ls(&sub);
        assert_eq!(names.len(), 1);
        assert!(names[0].contains(GA_NFD) || !names[0].contains(GA_NFC));
    }

    // -r: NFD subdirectory name must be renamed to NFC
    #[test]
    fn recursive_renames_nfd_subdirectory() {
        let dir = tempdir();
        let sub = dir.path().join(format!("{GA_NFD}dir"));
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("file.txt"), "").unwrap();

        run(&[dir.path().to_path_buf()], Form::Nfc, true, true).unwrap();

        let top_names = ls(dir.path());
        assert_eq!(top_names.len(), 1);
        let dir_name = &top_names[0];
        assert!(
            unicode_normalization::is_nfc_quick(dir_name.chars()) != IsNormalized::No,
            "expected NFC directory name, got: {dir_name:?}"
        );
        // The file inside must still exist.
        let new_sub = dir.path().join(dir_name);
        let file_names = ls(&new_sub);
        assert_eq!(file_names, vec!["file.txt"]);
    }

    // -r: NFD subdirectory and NFD file inside must both be renamed to NFC
    #[test]
    fn recursive_renames_nfd_subdirectory_and_file() {
        let dir = tempdir();
        let sub = dir.path().join(format!("{GA_NFD}dir"));
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join(format!("{GA_NFD}.txt")), "").unwrap();

        run(&[dir.path().to_path_buf()], Form::Nfc, true, true).unwrap();

        let top_names = ls(dir.path());
        assert_eq!(top_names.len(), 1);
        let dir_name = &top_names[0];
        assert!(
            unicode_normalization::is_nfc_quick(dir_name.chars()) != IsNormalized::No,
            "expected NFC directory name, got: {dir_name:?}"
        );
        let new_sub = dir.path().join(dir_name);
        let file_names = ls(&new_sub);
        assert_eq!(file_names.len(), 1);
        let file_name = &file_names[0];
        assert!(
            unicode_normalization::is_nfc_quick(file_name.chars()) != IsNormalized::No,
            "expected NFC filename, got: {file_name:?}"
        );
    }
}
