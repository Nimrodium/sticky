use clap::Parser;
use clap::Subcommand;
use colorize::AnsiColor;
use std::path::PathBuf;
use std::str::FromStr;

mod formats;
#[macro_use]
mod utils;
use crate::formats::{ArchiveFormat, Format, Options};
use crate::utils::get_stem_ext;
use utils::{infer, Mode};
const NAME: &str = "sticky";
static mut VERBOSE: bool = false;
/// An easy archive handler, supporting zip, tar.gz, and zstd
#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    /// extract source to target dir
    extract: bool,
    #[arg(short, long, default_value_t = false)]
    /// compress source to target dir
    compress: bool,
    #[arg(required = true,num_args = 1..)]
    /// source files
    sources: Vec<String>,
    #[arg(short, long)]
    /// target directory/archive
    target: Option<String>,
    #[arg(short, long)]
    format: Option<Format>,
    #[arg(short, long)]
    /// compression level
    level: Option<i64>,
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}
//test
#[derive(Debug, Parser)]
struct ArgsBetter {
    #[command(subcommand)]
    commands: Commands,
    #[arg(short, long, global = true)]
    /// compression level
    level: Option<i64>,
    #[arg(short, long, global = true)]
    format: Option<Format>,
    #[arg(short, long, global = true)]
    target: Option<PathBuf>,
    #[arg(short, long, default_value_t = false, global = true)]
    dry_run: bool,
    #[arg(short, long, default_value_t = false, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Extract { archive: PathBuf },
    Compress { sources: Vec<PathBuf> },
}
fn main() -> Result<(), String> {
    let args = ArgsBetter::parse();
    let options = Options::new(args.level);
    match args.commands {
        Commands::Extract { archive } => {
            let parts = get_stem_ext(&archive);
            let target = if let Some(target) = args.target {
                target
            } else {
                if let Some((stem, _)) = parts.as_ref() {
                    PathBuf::from(stem)
                } else {
                    return Err("could not infer output directory".to_string());
                }
            };
            let format = if let Some(format) = args.format {
                format
            } else {
                if let Some((_, ext)) = parts.as_ref() {
                    Format::from_str(ext)?
                } else {
                    let default = Format::default();
                    verbose!(
                        "could not infer archive format, using default ({})",
                        default.to_string()
                    );
                    default
                }
            };
            // verbose!("extracting archive {archive:?} -> {target:?}");
            if !args.dry_run {
                info!("extracting {archive:?} -> {target:?}");
                format.extract(&archive, &target)?
            };
            success!("successfully extracted archive {archive:?} -> {target:?}");
            Ok(())
        }
        Commands::Compress { sources } => {
            let format = if let Some(format) = args.format {
                format
            } else {
                let default = Format::default();
                verbose!(
                    "could not infer archive format, using default ({})",
                    default.to_string()
                );
                default
            };
            let target = if let Some(target) = args.target {
                target
            } else {
                if sources.len() == 1 {
                    sources[0].with_added_extension(format.to_string())
                } else {
                    return Err(
                        "with more than one source the --target flag is required".to_string()
                    );
                }
            };
            verbose!(
                "compressing sources {sources:?} -> {target:?} with {} archive format",
                format.to_string()
            );
            if !args.dry_run {
                format.compress(&sources, &target, options)?
            };
            success!("successfully compressed {sources:?} -> {target:?}");
            Ok(())
        }
    }
}
fn old_main() -> Result<(), String> {
    let args = Args::parse();
    let options = Options::new(args.level);
    let sources = args
        .sources
        .iter()
        .map(|s| PathBuf::from(s))
        .collect::<Vec<PathBuf>>();
    unsafe { VERBOSE = args.verbose };
    let target = args.target.map(|t| PathBuf::from(t));
    let (mode, format) = {
        let (op_mode, op_format) = infer(&sources, &target);
        let mode = if args.compress && args.extract {
            return Err("cannot use compress and extract at the same time!".to_string());
        } else if args.compress {
            Mode::Compress
        } else if args.extract {
            Mode::Extract
        } else {
            if let Some(m) = op_mode {
                m
            } else {
                return Err(
                    "cannot infer extract/compress, please use --extract, --compress flags"
                        .to_string(),
                );
            }
        };
        let format = if let Some(f) = args.format {
            f
        } else {
            if let Some(f) = op_format {
                f // vec is stupid actually
            } else {
                Format::default()
                // return Err("cannot infer archive format, please use -f flag".to_string());
            }
        };
        (mode, format)
    };
    match mode {
        Mode::Compress => {
            let target = if let Some(t) = target {
                t
            } else {
                sources[0].with_added_extension(format.get_extension())
            };
            verbose!(
                "compressing sources {sources:?} -> {target:?} with {} archive format",
                format.to_string()
            );
            format.compress(&sources, &target, options)?;
            success!("successfully compressed {sources:?} -> {target:?}");
        }
        Mode::Extract => {
            let archive = sources[0].clone();
            let target = if let Some(t) = target {
                t
            } else {
                PathBuf::from(
                    archive
                        .to_string_lossy()
                        .strip_suffix(&(".".to_string() + &format.get_extension().as_str()))
                        .unwrap_or(&archive.to_string_lossy())
                        .to_string(),
                )
            };
            verbose!("extracting archive {archive:?} -> {target:?}");
            format.extract(&archive, &target)?;
            success!("successfully extracted archive {archive:?} -> {target:?}");
        }
    }
    Ok(())
}
