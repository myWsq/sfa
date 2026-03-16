use std::process::ExitCode;

fn main() -> ExitCode {
    match sfa_cli::run(std::env::args_os()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(err.exit_code())
        }
    }
}
