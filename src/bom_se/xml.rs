use anyhow::{Error, Result};
use quick_xml::events::{BytesPI, BytesText, Event};
use quick_xml::writer::Writer;
use uuid::Uuid;

use crate::gem::licenses::License;
use crate::gem::Gemspec;

///
/// Serialize gems collection to xml string
///
pub(super) fn serialize(gems: &Vec<Gemspec>) -> Result<String> {
    let random_uuid = Uuid::new_v4();
    let serial_number = format!("urn:uuid:{}", random_uuid);

    build_xml(gems, &serial_number)
}

//
// Builds bom.xml content
//
fn build_xml(gems: &Vec<Gemspec>, serial_number: &str) -> Result<String> {
    let mut buffer = Vec::new();
    let mut writer = Writer::new_with_indent(&mut buffer, b' ', 2);

    let pi = r#"xml version="1.0" encoding="utf-8""#;

    writer.write_event(Event::PI(BytesPI::new(pi)))?;

    writer
        .create_element("bom")
        .with_attributes(
            vec![
                ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
                ("xmlns:xsd", "http://www.w3.org/2001/XMLSchema"),
                ("serialNumber", serial_number),
                ("version", "1"),
                ("xmlns", "http://cyclonedx.org/schema/bom/1.5"),
            ]
            .into_iter(),
        )
        // .write_pi_content(r#"xml version="1.0" encoding="utf-8"#)?
        .write_inner_content::<_, Error>(|writer| build_components(writer, &gems))?;

    let xml_bytes = writer.into_inner();

    Ok(String::from_utf8(xml_bytes.to_vec())?)
}

//
// Builds xml repersentatiom of "components" tag. It represents all dependencies
//
fn build_components(writer: &mut Writer<&mut Vec<u8>>, gems: &Vec<Gemspec>) -> Result<()> {
    writer
        .create_element("components")
        .write_inner_content::<_, Error>(|writer| {
            for gem in gems {
                build_component(writer, gem)?;
            }

            Ok(())
        })?;
    Ok(())
}

//
// Builds xml repersentatiom of "component" tag. It is main part of bom.xml,
// becase it represents one dependency
//
fn build_component(writer: &mut Writer<&mut Vec<u8>>, gem: &Gemspec) -> Result<(), Error> {
    writer
        .create_element("component")
        .with_attribute(("type", "library"))
        .write_inner_content::<_, Error>(|writer| {
            writer
                .create_element("name")
                .write_text_content(BytesText::new(&gem.name))?;

            writer
                .create_element("version")
                .write_text_content(BytesText::new(&gem.version))?;

            writer
                .create_element("description")
                .write_text_content(BytesText::new(&gem.description))?;

            writer
                .create_element("hashes")
                .write_inner_content::<_, Error>(|writer| build_hashes(writer, gem))?;

            writer
                .create_element("licenses")
                .write_inner_content::<_, Error>(|writer| build_licanses(writer, gem))?;

            writer
                .create_element("purl")
                .write_text_content(BytesText::new(&gem.purl))?;

            Ok(())
        })?;

    Ok(())
}

//
// Builds xml repersentatiom of "hash" child tag for "component" tag
//
fn build_hashes(writer: &mut Writer<&mut Vec<u8>>, gem: &Gemspec) -> Result<()> {
    for hash in &gem.hashes {
        writer
            .create_element("hash")
            .with_attribute(("alg", "SHA-256"))
            .write_text_content(BytesText::new(&hash.content))?;
    }

    Ok(())
}

//
// Builds xml repersentatiom of "licenses" child tag for "component" tag
//
fn build_licanses(writer: &mut Writer<&mut Vec<u8>>, gem: &Gemspec) -> Result<()> {
    writer
        .create_element("license")
        .write_inner_content::<_, Error>(|writer| {
            for license_type in &gem.licenses {
                match license_type {
                    License::KnownLicense(license) => {
                        writer
                            .create_element("id")
                            .write_text_content(BytesText::new(&license.id))?;
                    }
                    License::UnknownLicense(license) => {
                        writer
                            .create_element("name")
                            .write_text_content(BytesText::new(&license.name))?;
                    }
                }
            }

            Ok(())
        })?;

    Ok(())
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

        let xml = build_xml(&gems, &serial).unwrap();
        let expected = r#"<?xml version="1.0" encoding="utf-8"?>
<bom xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" serialNumber="urn:uuid:b83ca3d9-6b17-4566-bd50-201af63d9c42" version="1" xmlns="http://cyclonedx.org/schema/bom/1.5">
  <components>
  </components>
</bom>"#;

        assert_eq!(xml, expected);
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

        let xml = build_xml(&gems, &serial).unwrap();
        let expected = r#"<?xml version="1.0" encoding="utf-8"?>
<bom xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" serialNumber="urn:uuid:b83ca3d9-6b17-4566-bd50-201af63d9c42" version="1" xmlns="http://cyclonedx.org/schema/bom/1.5">
  <components>
    <component type="library">
      <name>activemodel</name>
      <version>7.0.8.4</version>
      <description>A toolkit for building modeling frameworks like Active Record. Rich support for attributes, callbacks, validations, serialization, internationalization, and testing.</description>
      <hashes>
        <hash alg="SHA-256">ef4e092d8644121b3e756e831bed6a16878317d02b9611bec8efcfdaee6525d6</hash>
      </hashes>
      <licenses>
        <license>
          <id>MIT</id>
        </license>
      </licenses>
      <purl>pkg:gem/activemodel@7.0.8.4</purl>
    </component>
    <component type="library">
      <name>brakeman</name>
      <version>6.0.1</version>
      <description>Security vulnerability scanner for Ruby on Rails.</description>
      <hashes>
        <hash alg="SHA-256">39641c63bc247bbdf993a349de90a13e146c464c872191f2adc12555bde591be</hash>
      </hashes>
      <licenses>
        <license>
          <name>Brakeman Public Use License</name>
        </license>
      </licenses>
      <purl>pkg:gem/brakeman@6.0.1</purl>
    </component>
  </components>
</bom>"#;

        assert_eq!(xml, expected);
    }
}
