use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;

use anyhow::{bail, Result};
use futures::{stream, StreamExt};
use reqwest_middleware::ClientWithMiddleware;

mod bom_se;
mod bundler;
mod config;
mod gem;

const CONCURRENT_REQUESTS: usize = 50;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let params = config::prepare_env();

    let content = read_gemfilelock(&params.input_file_name)?;
    let specs = bundler::parse_gemfile(content, params.verbose);

    let client = get_client()?;
    let gems = fetch_gems_info(&client, specs.gems, params.verbose).await;

    let bom_file = bom_se::serialize(gems, &params.format)?;

    write_bomfile(&params.output_file_name, bom_file)?;

    Ok(())
}

//
// This is a core fumction. It spawns threads and dispatches it
// to make requests and fetch all gems info from rubygems.org
//
type GemspecResultsPartition = (
    Vec<Result<gem::Gemspec, anyhow::Error>>,
    Vec<Result<gem::Gemspec, anyhow::Error>>,
);
async fn fetch_gems_info(
    client: &ClientWithMiddleware,
    specs: Vec<bundler::Source>,
    verbose: bool,
) -> Vec<gem::Gemspec> {
    let gem_specs_results = stream::iter(specs)
        .map(|source| async move {
            let source_info = source.get_source();
            gem::get_gem(&client, source_info).await
        })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<Vec<Result<gem::Gemspec, anyhow::Error>>>()
        .await;

    let (successes, errors): GemspecResultsPartition =
        gem_specs_results.into_iter().partition(Result::is_ok);

    let gem_specs: Vec<gem::Gemspec> = successes.into_iter().map(Result::unwrap).collect();
    if verbose {
        errors
            .into_iter()
            .map(Result::unwrap_err)
            .for_each(|error| println!("{}", error));
        println!(
            "\nTotal received info about {} gems from rubygems.org",
            gem_specs.len()
        );
    }
    gem_specs
}

//
// Try to find Gemfile.lock file and return it content. If it not possible
// to open file, it aborts process
//
fn read_gemfilelock(file_name: &PathBuf) -> Result<String> {
    let gemfile = fs::read_to_string(file_name);

    match gemfile {
        Ok(content) => Ok(content),
        Err(error) => match error.kind() {
            ErrorKind::NotFound => bail!("Specified path does not contains Gemfile.lock file"),
            ErrorKind::PermissionDenied => bail!("Permission denied to open Gemfile.lock file"),
            _ => bail!("Unknown error while reading file"),
        },
    }
}

//
// Try to write on disk bom.json or bom.xml file
//
fn write_bomfile(file_name: &PathBuf, content: String) -> Result<()> {
    let mut file = File::create(file_name)?;

    file.write_all(content.as_bytes())?;

    Ok(())
}

pub(crate) fn get_client() -> Result<ClientWithMiddleware> {
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let retry_policy =
        reqwest_retry::policies::ExponentialBackoff::builder().build_with_max_retries(3);
    let client = reqwest_middleware::ClientBuilder::new(http)
        .with(reqwest_retry::RetryTransientMiddleware::new_with_policy(
            retry_policy,
        ))
        .build();

    Ok(client)
}
