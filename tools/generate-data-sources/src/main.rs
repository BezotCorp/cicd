mod data_sources;

use std::process::ExitCode;

fn main() -> ExitCode {
    match data_sources::generate() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("generate-data-sources: {error}");
            ExitCode::FAILURE
        }
    }
}
