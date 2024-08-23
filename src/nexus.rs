use anyhow::{anyhow, Result};
use futures::{stream, StreamExt};
use reqwest_middleware::ClientWithMiddleware;
use serde_json::Value;
use url::Url;

use crate::client::get_nexus_client;
use crate::gem::Gemspec;

const CONCURRENT_REQUESTS: usize = 20;

type ResultCollection = Vec<Result<NexusResult>>;

pub(crate) async fn check_packages(
    packages: &Vec<Gemspec>,
    nexus_url: &str,
    verbose: bool,
) -> Result<Vec<NexusResult>> {
    let nexus = get_nexus(nexus_url)?;
    let client = get_nexus_client()?;

    let nexus_results = stream::iter(packages)
        .map(|package| async { nexus.check_package(&client, package).await })
        .buffer_unordered(CONCURRENT_REQUESTS)
        .collect::<ResultCollection>()
        .await;

    let (oks, errors): (ResultCollection, ResultCollection) =
        nexus_results.into_iter().partition(Result::is_ok);
    let oks: Vec<NexusResult> = oks.into_iter().map(Result::unwrap).collect();

    if verbose {
        errors
            .into_iter()
            .map(Result::unwrap_err)
            .for_each(|error| println!("{}", error));
    }

    Ok(oks)
}

pub(crate) fn get_nexus(repo_url: &str) -> Result<Nexus> {
    let url = Url::parse(repo_url)?;

    Ok(Nexus::new(url))
}

pub(crate) struct Nexus {
    // Nexus repository url
    repo_url: Url,

    // Which type of artefact should to request
    format_artefact: String,
}

#[derive(Debug)]
pub(crate) struct NexusResult {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) purl: String,
    pub(crate) is_exist: bool,
}

impl Nexus {
    ///
    /// Initializes new Nexus instance
    ///
    pub(crate) fn new(repo_url: Url) -> Self {
        Nexus {
            format_artefact: String::from("rubygems"),
            repo_url,
        }
    }

    ///
    /// Check package existance in Nexus repository
    ///
    pub(crate) async fn check_package(
        &self,
        client: &ClientWithMiddleware,
        package: &Gemspec,
    ) -> Result<NexusResult> {
        let name = &package.name;
        let version = &package.version;
        let response = self.send_request(client, name, version).await;

        let check_result = self.check_response(response, name, version);

        check_result.map(|is_exist| NexusResult {
            name: name.to_string(),
            version: version.to_string(),
            purl: package.purl.to_string(),
            is_exist,
        })
    }

    //
    // Parses respose json and try to check whether package exists in
    // Nexus repository
    //
    fn check_response(&self, response: Result<String>, name: &str, version: &str) -> Result<bool> {
        if let Ok(result) = response {
            let json: serde_json::Result<Value> = serde_json::from_str(&result);

            if let Ok(nexus_response) = json {
                return Ok(!nexus_response["items"].as_array().unwrap().is_empty());
            }

            return Err(anyhow!("An error occurred when we try to parse Nexus json response. Package: {}, version: {}, platform: {}", name, version, self.format_artefact));
        }

        Err(anyhow!(
                "An error occurred when we try to fetch data from Nexus. Package: {}, version: {}, platform: {}",
                name,
                version,
                self.format_artefact)
            )
    }

    //
    // Sends request to Nesus and try to receive response
    //
    async fn send_request(
        &self,
        client: &ClientWithMiddleware,
        name: &str,
        version: &str,
    ) -> Result<String> {
        let url = self.get_search_url(name, version);
        let response = client.get(url).send().await?;
        let content = response.text().await?;

        Ok(content)
    }

    //
    // Builds query url for package existing checking
    //
    fn get_search_url(&self, name: &str, version: &str) -> String {
        let mut base_url = self.repo_url.clone();

        base_url.set_path("service/rest/v1/search/assets");
        base_url
            .query_pairs_mut()
            .append_pair("name", name)
            .append_pair("version", version)
            .append_pair("format", &self.format_artefact);

        base_url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_search_url() {
        let nexus = Nexus::new(Url::parse("https://mynexus.com").unwrap());
        let name = "rails";
        let version = "7.1.1";

        let url = nexus.get_search_url(name, version);

        assert_eq!(url, String::from("https://mynexus.com/service/rest/v1/search/assets?name=rails&version=7.1.1&format=rubygems"))
    }

    #[test]
    fn test_when_request_fail() {
        let nexus = Nexus::new(Url::parse("https://mynexus.com").unwrap());
        let name = "rails";
        let version = "7.1.1";
        let respose: Result<String> = Err(anyhow!("Network error"));

        let result = nexus.check_response(respose, name, version);

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err().to_string().as_str(), "An error occurred when we try to fetch data from Nexus. Package: rails, version: 7.1.1, platform: rubygems");
    }

    #[test]
    fn test_when_request_success_broken_json() {
        let nexus = Nexus::new(Url::parse("https://mynexus.com").unwrap());
        let name = "rails";
        let version = "7.1.1";
        let respose: Result<String> = Ok(String::from("{ \"foo\": \"bar\""));

        let result = nexus.check_response(respose, name, version);

        assert_eq!(result.is_err(), true);
        assert_eq!(result.unwrap_err().to_string().as_str(), "An error occurred when we try to parse Nexus json response. Package: rails, version: 7.1.1, platform: rubygems");
    }

    #[test]
    fn test_when_request_success_empty_items() {
        let nexus = Nexus::new(Url::parse("https://mynexus.com").unwrap());
        let name = "rails";
        let version = "7.1.1";
        let response_content = r#"{
  "continuationToken": null,
  "items": []
}"#;
        let respose: Result<String> = Ok(String::from(response_content));
        let result = nexus.check_response(respose, name, version);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_when_request_success_not_empty_items() {
        let nexus = Nexus::new(Url::parse("https://mynexus.com").unwrap());
        let name = "rails";
        let version = "7.1.1";
        let response_content = r#"{
  "continuationToken": null,
  "items": [
    {
      "blobCreated": null,
      "checksum": {
        "md5": "c2e554f1ab23f90f16ea1124bd2c6d40",
        "sha1": "4e6bda3ae6205f0cb5fc72b93007ec0912611015",
        "sha256": "66e736acc1d1ba5ca1b598fc8b6024715aee02025467bfa87682dbeb823ddc17",
        "sha512": "1334f84a0a056646f698ca231171f3ff86324d63d684d8a0c13a6fe9dc61faaa0e8ef8acadd8c322567e8f8d2812711b2646fd27da33887be3d04f82a83a9cde"
      },
      "contentType": "application/octet-stream",
      "downloadUrl": "https://mynexus.com/repository/ruby-manual-upload/gems/rails-7.1.1.gem",
      "fileSize": 7168,
      "format": "rubygems",
      "id": "cnVieS1tYW51YWwtdXBsb2FkOjAyZDQyYzQ0OTg3ZWIyYzA4MzY0MzM1N2E5ODE1YmQ4",
      "lastDownloaded": "2024-02-13T03:49:13.959+00:00",
      "lastModified": "2023-10-19T13:39:14.453+00:00",
      "path": "gems/rails-7.1.1.gem",
      "repository": "ruby-manual-upload",
      "uploader": "uploader",
      "uploaderIp": "127.0.0.1"
    },
    {
      "blobCreated": null,
      "checksum": {
        "md5": "42d4e79aa9f42162c1a9205fb3a72461",
        "sha1": "0e5dcc3a7c6eb3725c6fbd4856109b1288ec485b",
        "sha256": "1a7d28e989ece1a1d20fb359e9bbdaba37be192d9c8271d4f01b20ef56e69a80"
      },
      "contentType": "application/octet-stream",
      "downloadUrl": "https://mynexus.com/repository/ruby-manual-upload/quick/Marshal.4.8/rails-7.1.1.gemspec.rz",
      "fileSize": 782,
      "format": "rubygems",
      "id": "cnVieS1tYW51YWwtdXBsb2FkOjhkYjNlZTdhZDkwMWY0N2JjNGYzMzZmZDUwYWZmNzVh",
      "lastDownloaded": "2024-06-04T10:54:07.916+00:00",
      "lastModified": "2023-10-19T13:39:14.452+00:00",
      "path": "quick/Marshal.4.8/rails-7.1.1.gemspec.rz",
      "repository": "ruby-manual-upload",
      "uploader": "uploader",
      "uploaderIp": "127.0.0.1"
    }
  ]
}"#;
        let respose: Result<String> = Ok(String::from(response_content));
        let result = nexus.check_response(respose, name, version);

        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), true);
    }
}
