use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct NeedsParser;

pub fn parse_needsfile(content: &str) -> Result<Vec<String>, pest::error::Error<Rule>> {
    let pairs = NeedsParser::parse(Rule::needsfile, content)?;
    let mut binaries = Vec::new();

    for pair in pairs {
        // The top-level rule is needsfile, which contains binary rules (since line is silent)
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::binary => {
                    binaries.push(inner_pair.as_str().to_string());
                }
                _ => {}
            }
        }
    }

    Ok(binaries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content = "git\ncargo\nnode";
        let binaries = parse_needsfile(content).unwrap();
        assert_eq!(binaries, vec!["git", "cargo", "node"]);
    }

    #[test]
    fn test_parse_with_comments() {
        let content = "git\n# this is a comment\ncargo # inline comment\nnode";
        let binaries = parse_needsfile(content).unwrap();
        assert_eq!(binaries, vec!["git", "cargo", "node"]);
    }

    #[test]
    fn test_parse_empty_lines() {
        let content = "\n\ngit\n\n\ncargo\n";
        let binaries = parse_needsfile(content).unwrap();
        assert_eq!(binaries, vec!["git", "cargo"]);
    }
    
    #[test]
    fn test_parse_complex_names() {
        let content = "ripgrep\nfd-find\npython3.9\n_underscore";
        let binaries = parse_needsfile(content).unwrap();
        assert_eq!(binaries, vec!["ripgrep", "fd-find", "python3.9", "_underscore"]);
    }
}
