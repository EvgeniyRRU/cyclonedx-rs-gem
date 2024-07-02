use anyhow::Result;
use serde::Serialize;
use uuid::Uuid;

use crate::gem::Gemspec;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Bom {
    bom_format: String,
    spec_version: String,
    serial_number: String,
    version: u8,
    components: Vec<Gemspec>,
}

///
/// Serialize gems collection to json string
///
pub(super) fn serialize(gems: Vec<Gemspec>) -> Result<String> {
    let serial_number = format!("urn:uuid:{}", Uuid::new_v4());

    build_json(gems, serial_number)
}

//
// Builds bom.json file content. A separate function need for
// testing
//
fn build_json(gems: Vec<Gemspec>, serial_number: String) -> Result<String> {
    let bom_content = Bom::new(gems, serial_number);

    Ok(serde_json::to_string_pretty(&bom_content)?)
}

impl Bom {
    fn new(components: Vec<Gemspec>, serial_number: String) -> Self {
        Bom {
            bom_format: String::from("CycloneDX"),
            spec_version: String::from("1.5"),
            serial_number,
            version: 1,
            components,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gem::licenses::{KnownLicense, License, UnknownLicense};
    use crate::gem::HashSpec;

    #[test]
    fn test_when_no_components() {
        let gems: Vec<Gemspec> = Vec::new();
        let serial = String::from("urn:uuid:b83ca3d9-6b17-4566-bd50-201af63d9c42");

        let json = build_json(gems, serial);
        let expected = r#"{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "serialNumber": "urn:uuid:b83ca3d9-6b17-4566-bd50-201af63d9c42",
  "version": 1,
  "components": []
}"#;
        assert_eq!(json.unwrap(), expected);
    }

    #[test]
    fn test_when_some_components() {
        let first_gem = Gemspec {
            name: String::from("activemodel"),
            version: String::from("7.0.8.4"),
            purl: String::from("pkg:gem/activemodel@7.0.8.4"),
            author: String::from("David Heinemeier Hansson"),
            licenses: vec![License::KnownLicense(KnownLicense::new(String::from("MIT")))],
            description: String::from("A toolkit for building modeling frameworks like Active Record. Rich support for attributes, callbacks, validations, serialization, internationalization, and testing."),
            hashes: vec!(HashSpec{
                alg: String::from("SHA-256"),
                content: String::from("ef4e092d8644121b3e756e831bed6a16878317d02b9611bec8efcfdaee6525d6")
            })

        };
        let second_gem = Gemspec {
            name: String::from("brakeman"),
            version: String::from("6.0.1"),
            purl: String::from("pkg:gem/brakeman@6.0.1"),
            author: String::from("Justin Collins"),
            licenses: vec![License::UnknownLicense(UnknownLicense::new(String::from(
                "Brakeman Public Use License",
            )))],
            description: String::from("Security vulnerability scanner for Ruby on Rails."),
            hashes: vec![HashSpec {
                alg: String::from("SHA-256"),
                content: String::from(
                    "39641c63bc247bbdf993a349de90a13e146c464c872191f2adc12555bde591be",
                ),
            }],
        };

        let gems = vec![first_gem, second_gem];
        let serial = String::from("urn:uuid:b83ca3d9-6b17-4566-bd50-201af63d9c42");

        let json = build_json(gems, serial);
        let expected = r#"{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "serialNumber": "urn:uuid:b83ca3d9-6b17-4566-bd50-201af63d9c42",
  "version": 1,
  "components": [
    {
      "name": "activemodel",
      "version": "7.0.8.4",
      "purl": "pkg:gem/activemodel@7.0.8.4",
      "licenses": [
        {
          "id": "MIT",
          "properties": []
        }
      ],
      "author": "David Heinemeier Hansson",
      "description": "A toolkit for building modeling frameworks like Active Record. Rich support for attributes, callbacks, validations, serialization, internationalization, and testing.",
      "hashes": [
        {
          "alg": "SHA-256",
          "content": "ef4e092d8644121b3e756e831bed6a16878317d02b9611bec8efcfdaee6525d6"
        }
      ]
    },
    {
      "name": "brakeman",
      "version": "6.0.1",
      "purl": "pkg:gem/brakeman@6.0.1",
      "licenses": [
        {
          "name": "Brakeman Public Use License",
          "properties": []
        }
      ],
      "author": "Justin Collins",
      "description": "Security vulnerability scanner for Ruby on Rails.",
      "hashes": [
        {
          "alg": "SHA-256",
          "content": "39641c63bc247bbdf993a349de90a13e146c464c872191f2adc12555bde591be"
        }
      ]
    }
  ]
}"#;
        assert_eq!(json.unwrap(), expected);
    }
}
