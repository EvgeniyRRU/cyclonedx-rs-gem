use regex::Regex;

#[derive(Debug)]
pub(crate) struct Source {
    name: String,
    version: String,
    platform: Option<String>,
}

pub(crate) struct Gemfile {
    pub(crate) gems: Vec<Source>,
}

///
/// Main method, parses Gemfile.lock content and returns vector,
/// which contains names and versions all installed gems from gems
/// repository (rubygems.org)
///
pub(crate) fn parse_gemfile(gemfile_content: String, verbose: bool) -> Gemfile {
    let mut gems: Vec<Source> = Vec::new();
    let mut gems_section = false;

    let gem_section_regexp = Regex::new(r"^GEM$").unwrap();
    let other_section_regexp = Regex::new(r"^[A-Z]+$").unwrap();
    let spec_regexp = Regex::new(r"^\s{4}(\S+?)\s+?\((\S+?)\)$").unwrap();

    let lines = gemfile_content.lines();

    for line in lines {
        if other_section_regexp.is_match(line) {
            if gem_section_regexp.is_match(line) {
                gems_section = true;
            } else if gems_section {
                break;
            }

            continue;
        }

        if gems_section && spec_regexp.is_match(line) {
            let captures = spec_regexp.captures(line).unwrap();
            let version_info = parse_gem_version(captures.get(2).unwrap().as_str());

            gems.push(Source {
                name: String::from(captures.get(1).unwrap().as_str()),
                version: version_info.0,
                platform: version_info.1,
            });
        }
    }

    let result = Gemfile { gems };

    if verbose {
        result.show_info();
    }

    result
}

//
// Try to find platform specific version of gem
//
fn parse_gem_version(version_str: &str) -> (String, Option<String>) {
    match version_str.split_once('-') {
        Some((version, platform)) => (version.to_string(), Some(platform.to_string())),
        None => (version_str.to_string(), None),
    }
}

impl Gemfile {
    fn show_info(&self) {
        let length = self.gems.len();

        println!("\nGemfile.lock file total contains {length} gems\n");
    }
}

impl Source {
    //
    // Returns Gemfile.lock item by tuple contains (name, version, platform)
    //
    pub(crate) fn get_source(&self) -> (&str, &str, Option<&str>) {
        match &self.platform {
            Some(platform) => (&self.name, &self.version, Some(platform)),
            None => (&self.name, &self.version, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let result = parse_gemfile(String::from(""), false);

        assert_eq!(result.gems.len(), 0);
    }

    #[test]
    fn test_when_not_contains_gem_section() {
        let gemfile = r#"
GIT
  remote: https://localgit.org/ruby/github_civisanalytics_ruby_audit.git
  revision: add080c46d56bf8b1b7a229e1207cfbd43f5a282
  branch: main
  specs:
    ruby_audit (2.3.0)
      bundler-audit (~> 0.9.0)

GIT
  remote: https://localgit.org/ruby/github_ctran_annotate_models.git
  revision: 984c33e567c17fe6101ba367c880c7494c614561
  specs:
    annotate (3.2.0)
      activerecord (>= 3.2, < 8.0)
      rake (>= 10.4, < 14.0)

GIT
  remote: https://localgit.org/ruby/github_jondot_sneakers.git
  revision: dbe56bc84fc8b50088c71a800e33b1320a33bb3e
  specs:
    sneakers (2.13.0.pre)
      bunny (~> 2.19)
      concurrent-ruby (~> 1.0)
      rake (>= 12.3, < 14.0)
      serverengine (~> 2.1)
      thor

PLATFORMS
  arm64-darwin-23
  x86_64-darwin-23
  x86_64-linux-musl

DEPENDENCIES
  active_model_otp (~> 2.3)
  active_record_union (~> 1.3)
  activeadmin (~> 3.2)
  activeadmin_addons!

RUBY VERSION
   ruby 3.3.0p0

BUNDLED WITH
   2.5.9"#;

        let result = parse_gemfile(String::from(gemfile), false);

        assert_eq!(result.gems.len(), 0);
    }

    #[test]
    fn test_when_contains_empty_gem_section() {
        let gemfile = r#"
GIT
  remote: https://localgit.org/ruby/github_civisanalytics_ruby_audit.git
  revision: add080c46d56bf8b1b7a229e1207cfbd43f5a282
  branch: main
  specs:
    ruby_audit (2.3.0)
      bundler-audit (~> 0.9.0)

GEM
  remote: https://rubygems.org/
  specs:

PLATFORMS
  arm64-darwin-23
  x86_64-darwin-23
  x86_64-linux-musl

DEPENDENCIES
  active_model_otp (~> 2.3)
  active_record_union (~> 1.3)
  activeadmin (~> 3.2)
  activeadmin_addons!

RUBY VERSION
   ruby 3.3.0p0

BUNDLED WITH
   2.5.9"#;

        let result = parse_gemfile(String::from(gemfile), false);

        assert_eq!(result.gems.len(), 0);
    }

    #[test]
    fn test_when_contains_some_gems() {
        let gemfile = r#"
GIT
  remote: https://localgit.org/ruby/github_civisanalytics_ruby_audit.git
  revision: add080c46d56bf8b1b7a229e1207cfbd43f5a282
  branch: main
  specs:
    ruby_audit (2.3.0)
      bundler-audit (~> 0.9.0)

GEM
  remote: https://rubygems.org/
  specs:
    actioncable (7.0.8.4)
      actionpack (= 7.0.8.4)
      activesupport (= 7.0.8.4)
      nio4r (~> 2.0)
      websocket-driver (>= 0.6.1)
    choice (0.2.0)
    clavius (1.0.4)
    coderay (1.1.3)
    nokogiri (1.16.5-arm64-darwin)
    opentelemetry-instrumentation-net_http (0.20.0)

PLATFORMS
  arm64-darwin-23
  x86_64-darwin-23
  x86_64-linux-musl

DEPENDENCIES
  active_model_otp (~> 2.3)
  active_record_union (~> 1.3)
  activeadmin (~> 3.2)
  activeadmin_addons!

RUBY VERSION
   ruby 3.3.0p0

BUNDLED WITH
   2.5.9"#;

        let result = parse_gemfile(String::from(gemfile), false);
        let gems = result.gems;

        assert_eq!(gems.len(), 6);
        assert_eq!(
            gems.first().unwrap().get_source(),
            ("actioncable", "7.0.8.4", None)
        );
        assert_eq!(gems.get(1).unwrap().get_source(), ("choice", "0.2.0", None));
        assert_eq!(
            gems.get(2).unwrap().get_source(),
            ("clavius", "1.0.4", None)
        );
        assert_eq!(
            gems.get(3).unwrap().get_source(),
            ("coderay", "1.1.3", None)
        );
        assert_eq!(
            gems.get(4).unwrap().get_source(),
            ("nokogiri", "1.16.5", Some("arm64-darwin"))
        );
        assert_eq!(
            gems.get(5).unwrap().get_source(),
            ("opentelemetry-instrumentation-net_http", "0.20.0", None)
        );
    }
}
