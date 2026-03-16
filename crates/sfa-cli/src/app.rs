use std::ffi::OsString;

use clap::{Parser, error::ErrorKind as ClapErrorKind};

use crate::cli::{Cli, Command};
use crate::error::CliError;
use crate::output::{render_pack_summary, render_unpack_summary};
use crate::service::{ArchiveService, PackRequest, RealArchiveService, UnpackRequest};

pub fn run<I, T>(args: I) -> Result<(), CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => return handle_clap_error(err),
    };

    let service = RealArchiveService;
    dispatch(&service, cli)
}

fn dispatch<S: ArchiveService>(service: &S, cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Command::Pack(args) => {
            let output_archive = args.output_archive;
            let stats = service.pack(PackRequest {
                input_dir: args.input_dir,
                output_archive,
                codec: args.codec,
                threads: args.threads,
                bundle_target_bytes: args.bundle_target_bytes,
                small_file_threshold: args.small_file_threshold,
                integrity: args.integrity,
                preserve_owner: args.preserve_owner,
                dry_run: args.dry_run,
            })?;
            println!("{}", render_pack_summary(&stats));
            Ok(())
        }
        Command::Unpack(args) => {
            let stats = service.unpack(UnpackRequest {
                input_archive: args.input_archive,
                output_dir: args.output_dir,
                threads: args.threads,
                overwrite: args.overwrite,
                integrity: args.integrity,
                restore_owner: args.restore_owner,
                dry_run: args.dry_run,
            })?;
            println!("{}", render_unpack_summary(&stats));
            Ok(())
        }
    }
}

fn handle_clap_error(err: clap::Error) -> Result<(), CliError> {
    let kind = err.kind();
    match kind {
        ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => {
            print!("{err}");
            Ok(())
        }
        _ => Err(CliError::usage(err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ErrorKind;

    use super::run;

    #[test]
    fn clap_usage_error_maps_to_usage_exit_code() {
        let err = run(["sfa", "pack"]).expect_err("should fail");
        assert_eq!(err.kind, ErrorKind::Usage);
        assert_eq!(err.exit_code(), 2);
    }
}
