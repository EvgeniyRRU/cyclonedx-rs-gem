use anyhow::{anyhow, Result};
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};

pub(crate) mod licenses;

use licenses::{get_license, KnownLicense, License, UnknownLicense};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GemspecResponse {
    authors: String,
    number: String,
    platform: String,
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

type GemfileItem<'a> = (&'a str, &'a str, Option<&'a str>);

///
/// Make request to rubygems.org and try to find gem information
/// If all ok, this function returns Gemspec struct, which serializable
/// to bom.json format
///
pub(crate) async fn get_gem(
    client: &ClientWithMiddleware,
    gem_source: GemfileItem<'_>,
) -> Result<Gemspec> {
    let (name, version, _) = gem_source;
    let url = format!("https://rubygems.org/api/v1/versions/{name}.json");

    let response = client.get(url).send().await?;
    let status = response.status();

    let result: Result<Gemspec> = match status.as_u16() {
        200 => {
            let json = response.text().await?;

            match find_version_gem(&json, gem_source) {
                Some(gem) => Ok(Gemspec::new(&gem_source, gem)),
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
fn find_version_gem(gem_data: &str, gem_source: GemfileItem) -> Option<GemspecResponse> {
    let (_, version, platform) = gem_source;
    if let Ok(gem_list) = serde_json::from_str::<Vec<GemspecResponse>>(gem_data) {
        return gem_list
            .iter()
            .find(|item| match platform {
                Some(platform) => (item.number == version) && (item.platform == platform),
                None => item.number == version,
            })
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
    fn new<'a>(gem_source: &'a GemfileItem<'a>, spec: GemspecResponse) -> Self {
        let (name, version, platform) = gem_source;
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

        let purl = match platform {
            Some(platform) => format!("pkg:gem/{name}@{version}?platform={platform}"),
            None => format!("pkg:gem/{name}@{version}"),
        };

        Gemspec {
            name: name.to_string(),
            version: version.to_string(),
            purl,
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
        let gem_source = ("rails", "7.1.1", None);
        let (name, version, _) = gem_source;
        let spec = GemspecResponse {
            authors: String::from("David Heinemeier Hansson"),
            number: String::from("7.1.1"),
            platform: String::from("ruby"),
            summary: String::from("Object-relational mapper framework (part of Rails)."),
            sha: String::from("f8dd03c0f3a462d616781dba3637a281ec86aaf6e643b56bea308e451ee96325"),
            licenses: Some(vec![String::from("MIT")]),
        };
        let result = Gemspec::new(&gem_source, spec);

        assert_eq!(result.name, name);
        assert_eq!(result.version, version);
        assert_eq!(result.purl.as_str(), "pkg:gem/rails@7.1.1");
        assert_eq!(result.author.as_str(), "David Heinemeier Hansson");
        assert_eq!(
            result.description.as_str(),
            "Object-relational mapper framework (part of Rails)."
        );
    }

    #[test]
    fn test_gemspec_new_known_license_platform() {
        let gem_source = ("nokogiri", "1.16.5", Some("x86_64-linux"));
        let (name, version, _) = gem_source;
        let spec = GemspecResponse {
            authors: String::from("Mike Dalessio, Aaron Patterson, Yoko Harada, Akinori MUSHA, John Shahid, Karol Bucek, Sam Ruby, Craig Barnes, Stephen Checkoway, Lars Kanis, Sergio Arbeo, Timothy Elliott, Nobuyoshi Nakada"),
            number: String::from("1.16.5"),
            platform: String::from("x86_64-linux"),
            summary: String::from("Nokogiri (鋸) makes it easy and painless to work with XML and HTML from Ruby. It provides a sensible, easy-to-understand API for reading, writing, modifying, and querying documents. It is fast and standards-compliant by relying on native parsers like libxml2, libgumbo, or xerces."),
            sha: String::from("0ca238da870066bed2f7837af6f35791bb9b76c4c5638999c46aac44818a6a97"),
            licenses: Some(vec![String::from("MIT")]),
        };
        let result = Gemspec::new(&gem_source, spec);

        assert_eq!(result.name, name);
        assert_eq!(result.version, version);
        assert_eq!(
            result.purl.as_str(),
            "pkg:gem/nokogiri@1.16.5?platform=x86_64-linux"
        );
        assert_eq!(result.author.as_str(), "Mike Dalessio, Aaron Patterson, Yoko Harada, Akinori MUSHA, John Shahid, Karol Bucek, Sam Ruby, Craig Barnes, Stephen Checkoway, Lars Kanis, Sergio Arbeo, Timothy Elliott, Nobuyoshi Nakada");
        assert_eq!(
            result.description.as_str(),
            "Nokogiri (鋸) makes it easy and painless to work with XML and HTML from Ruby. It provides a sensible, easy-to-understand API for reading, writing, modifying, and querying documents. It is fast and standards-compliant by relying on native parsers like libxml2, libgumbo, or xerces."
        );
    }
}
