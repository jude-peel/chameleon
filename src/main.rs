use std::error::Error;

use chameleon::cli;
use chameleon::formats;

fn main() -> Result<(), Box<dyn Error>> {
    // Get the command line arguments.
    let args = match cli::InputArguments::build() {
        Ok(args) => args,
        Err(e) => {
            cli::usage();
            return Err(Box::new(e));
        }
    };

    let extension = args.input_path.extension().ok_or_else(|| {
        cli::CliError::InvalidArgument(format!("Invalid path: {:?}", args.input_path))
    })?;

    let picture = match extension.to_str() {
        Some(ex) => match ex {
            "png" => formats::png::Png::from_path(args.input_path)?,
            _ => todo!(),
        },
        None => {
            eprint!("Failed to convert extension from OsStr, to &str.");
            return Err(Box::new(cli::CliError::InvalidArgument(format!(
                "Invalid path: {:?}",
                args.input_path
            ))));
        }
    };

    Ok(())
}
