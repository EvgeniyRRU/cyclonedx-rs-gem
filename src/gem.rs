use anyhow::{anyhow, Result};
use reqwest::get;
use serde::{Deserialize, Serialize};

pub(crate) mod licenses;

use licenses::{get_license, KnownLicense, License, UnknownLicense};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GemspecResponse {
    authors: String,
    number: String,
    summary: String,
    sha: String,
    licenses: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct HashSpec {
    pub(crate) alg: String,
    pub(crate) content: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Gemspec {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) purl: String,
    pub(crate) licenses: Vec<License>,
    pub(crate) author: String,
    pub(crate) description: String,
    pub(crate) hashes: Vec<HashSpec>,
}

///
/// Make request to rubygems.org and try to find gem information
/// If all ok, this function returns Gemspec struct, which serializable
/// to bom.json format
///
pub(crate) async fn get_gem(name: &str, version: &str) -> Result<Gemspec> {
    let url = format!("https://rubygems.org/api/v1/versions/{name}.json");

    let response = get(url).await?;
    let status = response.status();

    let result: Result<Gemspec> = match status.as_u16() {
        200 => {
            let json = response.text().await?;

            match find_version_gem(&json, version) {
                Some(gem) => Ok(Gemspec::new(name, version, gem)),
                None => Err(anyhow!(
                    "Could not find version {} for gem {}",
                    version,
                    name
                )),
            }
        }
        404 => Err(anyhow!("Gem not found: {}, version {}", name, version)),
        400..=499 => Err(anyhow!(
            "Client error occurred for gem {}, version {}",
            name,
            version
        )),
        500..=599 => Err(anyhow!(
            "Server error occurred for gem {}, version {}",
            name,
            version
        )),
        _ => Err(anyhow!(
            "Unknown error occurred for gem {}, version {}",
            name,
            version
        )),
    };

    result
}

//
// Try to find current version gem information from rubygems response
//
fn find_version_gem(gem_data: &str, version: &str) -> Option<GemspecResponse> {
    if let Ok(gem_list) = serde_json::from_str::<Vec<GemspecResponse>>(gem_data) {
        return gem_list
            .iter()
            .find(|item| { item.number == version })
            .cloned();
    }

    None
}

impl HashSpec {
    fn new(content: String) -> Self {
        HashSpec {
            alg: String::from("SHA-256"),
            content,
        }
    }
}

impl Gemspec {
    fn new(name: &str, version: &str, spec: GemspecResponse) -> Self {
        let license_data = get_license(spec.licenses);

        let licenses_list: Vec<License> = match license_data {
            (Some(license), None) => {
                let known_license = KnownLicense::new(license);
                vec![License::KnownLicense(known_license)]
            }
            (None, Some(license)) => {
                let unknown_license = UnknownLicense::new(license);
                vec![License::UnknownLicense(unknown_license)]
            }
            _ => vec![],
        };

        Gemspec {
            name: name.to_owned(),
            version: version.to_owned(),
            purl: format!("pkg:gem/{name}@{version}"),
            author: spec.authors,
            description: spec.summary,
            hashes: vec![HashSpec::new(spec.sha)],
            licenses: licenses_list,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_spec_new() {
        let value = HashSpec::new(String::from(
            "865f578862103c8ece7f7e0c1fdb2bf686c52bebc85b84b15bcde3bac9aa8192",
        ));

        assert_eq!(value.alg.as_str(), "SHA-256");
        assert_eq!(
            value.content.as_str(),
            "865f578862103c8ece7f7e0c1fdb2bf686c52bebc85b84b15bcde3bac9aa8192"
        );
    }

    #[test]
    fn test_gemspec_new_known_license() {
        let name = "rails";
        let version = "7.1.1";
        let spec = GemspecResponse {
            authors: String::from("David Heinemeier Hansson"),
            number: String::from("7.1.1"),
            summary: String::from("Object-relational mapper framework (part of Rails)."),
            sha: String::from("f8dd03c0f3a462d616781dba3637a281ec86aaf6e643b56bea308e451ee96325"),
            licenses: Some(vec![String::from("MIT")]),
        };
        let result = Gemspec::new(name, version, spec);

        assert_eq!(result.name, name);
        assert_eq!(result.version, version);
        assert_eq!(result.purl.as_str(), "pkg:gem/rails@7.1.1");
        assert_eq!(result.author.as_str(), "David Heinemeier Hansson");
        assert_eq!(
            result.description.as_str(),
            "Object-relational mapper framework (part of Rails)."
        );
    }
}
