use crate::error::AppError;

#[derive(Debug, Default)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub argument_hint: Option<String>,
}

/// Parses YAML frontmatter from a skill.md file's content.
pub fn parse_frontmatter(content: &str) -> Result<SkillFrontmatter, AppError> {
    let matter = gray_matter::Matter::<gray_matter::engine::YAML>::new();
    let result = matter.parse(content);

    match result.data {
        Some(gray_matter::Pod::Hash(map)) => {
            let name = map.get(&"name".to_string()).and_then(|v| match v {
                gray_matter::Pod::String(s) => Some(s.clone()),
                _ => None,
            });
            let description = map.get(&"description".to_string()).and_then(|v| match v {
                gray_matter::Pod::String(s) => Some(s.clone()),
                _ => None,
            });
            let argument_hint = map.get(&"argument-hint".to_string()).and_then(|v| match v {
                gray_matter::Pod::String(s) => Some(s.clone()),
                _ => None,
            });
            Ok(SkillFrontmatter {
                name,
                description,
                argument_hint,
            })
        }
        Some(_) => Err(AppError::Frontmatter(
            "unexpected frontmatter format".into(),
        )),
        None => Ok(SkillFrontmatter::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_frontmatter() {
        let content = "---\nname: analyze-api\ndescription: Analyze API requirements\n---\n\nBody content here.";
        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm.name.as_deref(), Some("analyze-api"));
        assert_eq!(fm.description.as_deref(), Some("Analyze API requirements"));
    }

    #[test]
    fn test_parse_empty_frontmatter() {
        let content = "No frontmatter here.";
        let fm = parse_frontmatter(content).unwrap();
        assert!(fm.name.is_none());
    }

    #[test]
    fn test_parse_with_argument_hint() {
        let content = "---\nname: test\ndescription: desc\nargument-hint: <file-path>\n---\n";
        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm.argument_hint.as_deref(), Some("<file-path>"));
    }
}
