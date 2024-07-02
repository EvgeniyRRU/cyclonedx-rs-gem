use regex::Regex;

#[derive(Debug)]
pub(crate) struct Source {
    pub(crate) name: String,
    pub(crate) version: String,
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

            gems.push(Source {
                name: String::from(captures.get(1).unwrap().as_str()),
                version: String::from(captures.get(2).unwrap().as_str()),
            });
        }
    }

    let result = Gemfile { gems };

    if verbose {
        result.show_info();
    }

    result
}

impl Gemfile {
    fn show_info(&self) {
        let length = self.gems.len();

        println!("\nGemfile.lock file total contains {} gems\n", length);
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

        assert_eq!(gems.len(), 4);
        assert_eq!(gems.get(0).unwrap().name.as_str(), "actioncable");
        assert_eq!(gems.get(0).unwrap().version.as_str(), "7.0.8.4");

        assert_eq!(gems.get(1).unwrap().name.as_str(), "choice");
        assert_eq!(gems.get(1).unwrap().version.as_str(), "0.2.0");

        assert_eq!(gems.get(2).unwrap().name.as_str(), "clavius");
        assert_eq!(gems.get(2).unwrap().version.as_str(), "1.0.4");

        assert_eq!(gems.get(3).unwrap().name.as_str(), "coderay");
        assert_eq!(gems.get(3).unwrap().version.as_str(), "1.1.3");
    }
}
