use anyhow::Result;

use crate::config::Format;
use crate::gem::Gemspec;

mod json;
mod xml;

pub(super) fn serialize(gems: &Vec<Gemspec>, format: &Format) -> Result<String> {
    match format {
        Format::Xml => xml::serialize(gems),
        Format::Json => json::serialize(gems),
    }
}
