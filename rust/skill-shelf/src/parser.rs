use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::model::{ManagedGroupRecord, ParsedSkill};

const DEFAULT_SPECIALIZED_DOMAIN_GROUP: &str = "specialized-domain";
const DEFAULT_SPECIALIZED_DOMAIN_DESCRIPTION: &str = "Specialized domain skills";

pub fn parse_skill_file(
    skill_file_path: impl AsRef<Path>,
    managed_groups: &[ManagedGroupRecord],
    max_keywords: usize,
    group_from_path: Option<&str>,
) -> Result<ParsedSkill> {
    let skill_file_path = skill_file_path.as_ref();
    let raw = fs::read_to_string(skill_file_path)
        .with_context(|| format!("failed to read {}", skill_file_path.display()))?;
    let frontmatter = parse_frontmatter(&raw)?;
    let metadata = fs::metadata(skill_file_path)
        .with_context(|| format!("failed to stat {}", skill_file_path.display()))?;

    let skill_name = frontmatter.name.trim().to_string();
    if skill_name.is_empty() {
        bail!("missing required frontmatter field: name");
    }

    let description = normalize_description(frontmatter.description.trim());
    if description.is_empty() {
        bail!("missing required frontmatter field: description");
    }

    let keywords = derive_keywords(&description, max_keywords);

    let (group, group_description) = if let Some(group) = group_from_path {
        let managed = managed_groups.iter().find(|entry| entry.group == group);
        (
            group.to_string(),
            managed
                .map(|entry| entry.group_description.clone())
                .unwrap_or_else(|| group.to_string()),
        )
    } else {
        match_managed_group(&skill_name, &description, &keywords, managed_groups)?
    };

    Ok(ParsedSkill {
        skill_name,
        description,
        group,
        group_description,
        keywords,
        skill_path: skill_file_path.to_string_lossy().into_owned(),
        updated_at_ms: metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs_f64() * 1000.0)
            .unwrap_or(0.0),
    })
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    name: String,
    description: String,
}

fn parse_frontmatter(raw: &str) -> Result<Frontmatter> {
    let mut lines = raw.lines();
    if lines.next() != Some("---") {
        bail!("missing frontmatter start");
    }

    let mut frontmatter = String::new();
    let mut closed = false;
    for line in lines {
        if line == "---" {
            closed = true;
            break;
        }
        frontmatter.push_str(line);
        frontmatter.push('\n');
    }

    if !closed {
        bail!("missing frontmatter end");
    }

    serde_yaml::from_str(&frontmatter).with_context(|| "failed to parse YAML frontmatter")
}

fn normalize_description(description: &str) -> String {
    description.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn derive_keywords(description: &str, max_keywords: usize) -> Vec<String> {
    let mut keywords = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for part in description
        .to_lowercase()
        .split(|c: char| c.is_whitespace() || c == ',')
    {
        let token = trim_keyword_token(part);
        if token.chars().count() < 3 || !seen.insert(token.to_string()) {
            continue;
        }
        keywords.push(token.to_string());
        if keywords.len() >= max_keywords.max(1) {
            break;
        }
    }
    keywords
}

fn match_managed_group(
    skill_name: &str,
    description: &str,
    keywords: &[String],
    groups: &[ManagedGroupRecord],
) -> Result<(String, String)> {
    let mut best: Option<(&ManagedGroupRecord, usize)> = None;
    let haystack = format!(
        "{} {} {}",
        skill_name.to_lowercase(),
        description.to_lowercase(),
        keywords.join(" ")
    );

    for group in groups {
        let mut score = 0usize;
        for keyword in &group.keywords {
            if haystack.contains(&keyword.to_lowercase()) {
                score += 4;
            }
        }
        for alias in &group.aliases {
            if haystack.contains(&alias.to_lowercase()) {
                score += 2;
            }
        }
        if haystack.contains(&group.group.to_lowercase()) {
            score += 1;
        }

        if best
            .map(|(_, best_score)| score > best_score)
            .unwrap_or(score > 0)
        {
            best = Some((group, score));
        }
    }

    if let Some((group, _)) = best {
        return Ok((group.group.clone(), group.group_description.clone()));
    }

    if let Some(group) = groups
        .iter()
        .find(|group| group.group == DEFAULT_SPECIALIZED_DOMAIN_GROUP)
    {
        return Ok((group.group.clone(), group.group_description.clone()));
    }

    Ok((
        DEFAULT_SPECIALIZED_DOMAIN_GROUP.to_string(),
        DEFAULT_SPECIALIZED_DOMAIN_DESCRIPTION.to_string(),
    ))
}

fn is_cjk(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
}

fn trim_keyword_token(part: &str) -> &str {
    let start = part
        .char_indices()
        .find(|(_, ch)| is_keyword_char(*ch))
        .map(|(index, _)| index);
    let Some(start) = start else {
        return "";
    };

    let end = part
        .char_indices()
        .rev()
        .find(|(_, ch)| is_keyword_char(*ch))
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(start);

    &part[start..end]
}

fn is_keyword_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || is_cjk(c)
}
