use std::env::current_dir;
use std::fmt;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Path to directory containing Gemfile.lock file. If path not set, it
    // try to find Gemfile.lock in current directory
    #[arg(short, long)]
    path: Option<String>,

    // Path to output the bom file to
    #[arg(short, long)]
    output: Option<String>,

    // Output bom file format (json or xml)
    #[arg(short, long, value_enum, default_value_t=Format::Json)]
    format_file: Format,

    // Run verbosely
    #[arg(short, long)]
    verbose: bool,

    // Remote nexus repository url
    #[arg(short, long)]
    nexus_url: Option<String>,
}

#[derive(Debug)]
pub(crate) struct Params {
    pub(crate) format: Format,
    pub(crate) input_file_name: PathBuf,
    pub(crate) output_file_name: PathBuf,
    pub(crate) verbose: bool,
    pub(crate) nexus_url: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub(crate) enum Format {
    // xml output, creates bom.xml
    Xml,

    // json output, creates bom.json
    Json,
}

///
/// Parses env args and setup default values for application
///
pub(super) fn prepare_env() -> Params {
    let args = Args::parse();
    let cwd = current_dir().unwrap();

    let params = parse_params(args, cwd);

    if params.verbose {
        print_params(&params);
    }

    params
}

//
// Implements logic for arguments parsing
//
fn parse_params(args: Args, cwd: PathBuf) -> Params {
    let lock_file_name = "Gemfile.lock";
    let bom_file_name = format!("bom.{}", &args.format_file);

    let mut input_path = match args.path {
        Some(path_str) => PathBuf::from(path_str),
        None => cwd,
    };

    let mut output_path = match args.output {
        Some(path_str) => PathBuf::from(path_str),
        None => input_path.clone(),
    };

    input_path.push(lock_file_name);
    output_path.push(bom_file_name);

    Params {
        input_file_name: input_path,
        output_file_name: output_path,
        format: args.format_file,
        verbose: args.verbose,
        nexus_url: args.nexus_url,
    }
}

fn print_params(params: &Params) {
    println!("Output file format: {}", params.format);
    println!(
        "Input directory (should contains Gemfile.lock file): {}",
        params.input_file_name.to_str().unwrap()
    );
    println!(
        "Output directory (will place bom file): {}",
        params.output_file_name.to_str().unwrap()
    );
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match *self {
            Format::Json => "json",
            Format::Xml => "xml",
        };

        write!(f, "{}", str)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_params, Args, Format};
    use std::path::PathBuf;

    #[test]
    fn test_when_all_paths_none() {
        let cwd = PathBuf::from("/Users/me/work/rust/cyclonedx-rs-gem/");
        let args = Args {
            path: None,
            output: None,
            format_file: Format::Json,
            verbose: false,
            nexus_url: None,
        };

        let result = parse_params(args, cwd);

        assert!(!result.verbose);
        assert_eq!(result.format, Format::Json);
        assert_eq!(
            result.input_file_name.to_str().unwrap(),
            "/Users/me/work/rust/cyclonedx-rs-gem/Gemfile.lock"
        );
        assert_eq!(
            result.output_file_name.to_str().unwrap(),
            "/Users/me/work/rust/cyclonedx-rs-gem/bom.json"
        );
        assert_eq!(result.nexus_url, None);
    }

    #[test]
    fn test_when_input_path_exists_output_none() {
        let cwd = PathBuf::from("/Users/me/work/rust/cyclonedx-rs-gem/");
        let args = Args {
            path: Some(String::from("/Users/me/work/ruby/superrailsapp/")),
            output: None,
            format_file: Format::Xml,
            verbose: true,
            nexus_url: None,
        };

        let result = parse_params(args, cwd);

        assert!(result.verbose);
        assert_eq!(result.format, Format::Xml);
        assert_eq!(
            result.input_file_name.to_str().unwrap(),
            "/Users/me/work/ruby/superrailsapp/Gemfile.lock"
        );
        assert_eq!(
            result.output_file_name.to_str().unwrap(),
            "/Users/me/work/ruby/superrailsapp/bom.xml"
        );
        assert_eq!(result.nexus_url, None);
    }

    #[test]
    fn test_when_output_exists_intup_none() {
        let cwd = PathBuf::from("/Users/me/work/rust/cyclonedx-rs-gem/");
        let args = Args {
            path: None,
            output: Some(String::from("/Users/me/work/ruby/railsapp/")),
            format_file: Format::Xml,
            verbose: true,
            nexus_url: None,
        };

        let result = parse_params(args, cwd);

        assert!(result.verbose);
        assert_eq!(result.format, Format::Xml);
        assert_eq!(
            result.input_file_name.to_str().unwrap(),
            "/Users/me/work/rust/cyclonedx-rs-gem/Gemfile.lock"
        );
        assert_eq!(
            result.output_file_name.to_str().unwrap(),
            "/Users/me/work/ruby/railsapp/bom.xml"
        );
        assert_eq!(result.nexus_url, None);
    }

    #[test]
    fn test_when_all_paths_exists() {
        let cwd = PathBuf::from("/Users/me/work/rust/cyclonedx-rs-gem/");
        let args = Args {
            path: Some(String::from("/Users/me/work/ruby/superrailsapp/")),
            output: Some(String::from("/Users/me/work/ruby/railsapp/")),
            format_file: Format::Json,
            verbose: false,
            nexus_url: Some(String::from("https://somenexus.com/")),
        };

        let result = parse_params(args, cwd);

        assert!(!result.verbose);
        assert_eq!(result.format, Format::Json);
        assert_eq!(
            result.input_file_name.to_str().unwrap(),
            "/Users/me/work/ruby/superrailsapp/Gemfile.lock"
        );
        assert_eq!(
            result.output_file_name.to_str().unwrap(),
            "/Users/me/work/ruby/railsapp/bom.json"
        );
        assert_eq!(result.nexus_url.unwrap().as_str(), "https://somenexus.com/");
    }
}
