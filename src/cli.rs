use std::env;
use std::error::Error;
use std::fmt::{self, Display};
use std::path::PathBuf;

/// Custom error type for command line errors.
///
/// # Fields
///
/// * 'MissingArgument' - Contains a static str for immediately returning
///                 what argument is missing.
/// * 'InvalidArgument' - Contains a String for returning the InvalidArgument. A String
///                 is used instead of a &'static str because it allows you to handle
///                 the value once the string that cause the error goes out of scope.
///
/// # Examples
///
/// '''
/// // Pull input arguments and skip the operating path.
/// let mut args = env::args().skip(1).enumerate();
///
/// // Define a tuple to populate with values from args.
/// let mut path_flags: (PathBuf, String) = (PathBuf::new(), Vec::new());
///
/// // Iterate through each argument: arg, with their position: i.
/// while let Some((i, arg)) = args.next() {
///     match (i, arg.trim()) {
///         (_, "-path") => {
///             let (_, next_arg) = args.next().ok_or(CliError::MissingArgument(
///                 "Error: Missing path after -path flag.",
///             )).unwrap();
///             let path_flags.0 = PathBuf::from(next_arg);
///         }
///         (_, _) => {
///             return Err(CliError::InvalidArgument(arg.to_string())),
///         }
///     }
/// }
/// '''
#[derive(Debug)]
pub enum CliError {
    MissingArgument(&'static str),
    InvalidArgument(String),
}

// Define how CliErrors are displayed.
impl Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::MissingArgument(flag) => {
                write!(f, "Missing Argument '{flag}'")
            }
            CliError::InvalidArgument(arg) => {
                write!(f, "Invalid argument '{arg}'")
            }
        }
    }
}

// Implement the Error interface for CliError.
impl Error for CliError {}

/// A struct for holding the input arguments given.
///
/// # Attributes
///
/// * 'path' - A PathBuf containing the path to the image.
/// * 'flags' - A vector containing the given flags for manipulating
///                 the image.
///
/// # Examples
///
/// '''
/// let args = match InputArguments::build() {
///     Ok(arguments) => arguments,
///     Err(e) => {
///         eprintln!("Error reading args!");
///         process::exit(1);
///     }
/// };
/// '''
pub struct InputArguments {
    pub input_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub flags: Vec<Flags>,
}

/// A enum containing possible flags for operating on
/// images. Not yet implemented, but eventually the goal
/// will be to support dithering to pallete, and more
/// as the scope of this project inevitably increases.
#[derive(Debug)]
pub enum Flags {
    NotYetImplemented,
}

impl InputArguments {
    /// Constructor function for InputArguments, gets arguments,
    /// and stores them in an InputArguments struct.
    ///
    /// # Returns
    ///
    /// An InputArguments instance popuated with the arguments
    /// provided: A path and a vector containing the flags given.
    pub fn build() -> Result<InputArguments, CliError> {
        let mut path_flags = InputArguments {
            input_path: PathBuf::new(),
            output_path: None,
            flags: Vec::new(),
        };

        let mut args = env::args().skip(1).enumerate();

        if args.len() < 1 {
            return Err(CliError::MissingArgument(
                "Error: No arguments given, refer to usage for more information.",
            ));
        }

        while let Some((i, arg)) = args.next() {
            match (i, arg.trim()) {
                (_, "-i") | (_, "-input") => {
                    let (_, input_path) = args.next().ok_or(CliError::MissingArgument(
                        "Error: Missing input path. Is -input/-i followed by a valid path?",
                    ))?;

                    let file_path = PathBuf::from(&input_path);
                    match file_path.exists() {
                        true => {
                            path_flags.input_path = file_path;
                        }
                        false => {
                            eprintln!("Invalid input path given. Does the path exist?");
                            return Err(CliError::InvalidArgument(input_path));
                        }
                    };
                }
                (_, "-o") | (_, "-out") | (_, "-output") => {
                    let (_, output_path) = args.next().ok_or(CliError::MissingArgument(
                        "Error: Missing output path, -output called without path following.",
                    ))?;

                    let output_file_path = PathBuf::from(&output_path);
                    path_flags.output_path = Some(output_file_path);
                }
                (_, "-h") | (_, "-help") | (_, "--help") => help(),
                (_, "-u") | (_, "-usage") | (_, "--usage") => usage(),
                (0, "help") => help(),
                (0, "usage") => usage(),
                // Here flags will be implemented as added match arms appending Flags enum members
                // to path_flags.flags.
                (_, arg) => {
                    return Err(CliError::InvalidArgument(arg.to_string()));
                }
            }
        }
        if path_flags.output_path.is_none() && path_flags.input_path.exists() {
            let output_path = match path_flags.input_path.clone().parent() {
                Some(p) => p.join("output"),
                None => PathBuf::from("output"),
            };

            eprintln!("Warning: No output path given, defaulted to the same directory and extension as the input file but with name 'output'.");
            path_flags.output_path = Some(output_path);
        }

        Ok(path_flags)
    }
}

pub fn usage() {
    println!("todo!");
}

pub fn help() {
    println!("todo!");
}
