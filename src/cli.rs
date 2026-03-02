use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Rename files by converting Unicode normalization form"
)]
#[command(group(ArgGroup::new("form").required(true).args(["nfc", "nfd"])))]
pub struct Args {
    #[arg(long)]
    pub nfc: bool,
    #[arg(long)]
    pub nfd: bool,
    #[arg(long)]
    pub notest: bool,
    #[arg(short, long)]
    pub recursive: bool,
    pub paths: Vec<PathBuf>,
}
