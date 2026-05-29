use std::collections::HashSet;

use crate::model::{SkillRecord, SkillSearchResult};

pub fn search_skills(records: &[SkillRecord], query: &str, limit: usize) -> Vec<SkillSearchResult> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();
    for record in records {
        let ranked = score_skill_text(record, &normalized_query);
        if ranked.score == 0 {
            continue;
        }
        results.push(SkillSearchResult {
            skill_id: record.skill_id.clone(),
            skill_name: record.skill_name.clone(),
            description: record.description.clone(),
            group: record.group.clone(),
            score: ranked.score,
            reasons: ranked.reasons,
        });
    }

    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.skill_id.cmp(&right.skill_id))
    });
    results.truncate(limit);
    results
}

struct RankedMatch {
    score: i64,
    reasons: Vec<String>,
}

fn score_skill_text(record: &SkillRecord, normalized_query: &str) -> RankedMatch {
    let query_tokens = tokenize(normalized_query);
    let id_norm = normalize(&record.skill_id);
    let name_norm = normalize(&record.skill_name);
    let group_norm = normalize(&record.group);
    let summary_norm = normalize(&record.description);
    let tag_norms: Vec<String> = record
        .keywords
        .iter()
        .map(|keyword| normalize(keyword))
        .collect();

    let mut score = 0i64;
    let mut reasons = Vec::new();

    if id_norm == normalized_query || name_norm == normalized_query {
        score += 120;
        reasons.push("exact-name".to_string());
    }

    if starts_or_ends_with(&id_norm, normalized_query)
        || starts_or_ends_with(&name_norm, normalized_query)
    {
        score += 65;
        reasons.push("prefix-suffix".to_string());
    }

    if id_norm.contains(normalized_query)
        || name_norm.contains(normalized_query)
        || summary_norm.contains(normalized_query)
    {
        score += 40;
        reasons.push("substring".to_string());
    }

    let field_tokens: HashSet<String> = tokenize(&id_norm)
        .into_iter()
        .chain(tokenize(&name_norm))
        .chain(tokenize(&group_norm))
        .chain(tokenize(&summary_norm))
        .chain(tag_norms.iter().flat_map(|value| tokenize(value)))
        .collect();
    let token_hits: Vec<String> = query_tokens
        .iter()
        .filter(|token| field_tokens.contains(*token))
        .cloned()
        .collect();
    if !token_hits.is_empty() {
        score += (token_hits.len() as i64) * 18;
        reasons.push(format!("token-overlap:{}", token_hits.len()));
    }

    let mut best_lev = normalized_levenshtein_score(normalized_query, &id_norm)
        .min(normalized_levenshtein_score(normalized_query, &name_norm));
    for tag in &tag_norms {
        best_lev = best_lev.min(normalized_levenshtein_score(normalized_query, tag));
    }
    if best_lev <= 0.34 {
        score += ((1.0 - best_lev) * 30.0).round() as i64;
        reasons.push(format!("levenshtein:{best_lev:.2}"));
    }

    if group_norm.contains(normalized_query)
        || query_tokens.iter().any(|token| group_norm.contains(token))
    {
        score += 10;
        reasons.push("group".to_string());
    }

    let reasons = dedupe(reasons);
    RankedMatch { score, reasons }
}

fn dedupe(reasons: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for reason in reasons {
        if seen.insert(reason.clone()) {
            deduped.push(reason);
        }
    }
    deduped
}

fn starts_or_ends_with(value: &str, query: &str) -> bool {
    value.starts_with(query) || value.ends_with(query)
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .replace(['_', '/', '\\', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for segment in value.split(|c: char| !c.is_ascii_alphanumeric() && !is_cjk(c)) {
        if segment.is_empty() {
            continue;
        }

        if segment.chars().all(|c| c.is_ascii_alphanumeric()) {
            if segment.len() >= 2 {
                tokens.push(segment.to_string());
            }
            continue;
        }

        let cjk: Vec<char> = segment.chars().filter(|c| is_cjk(*c)).collect();
        for window in cjk.windows(2) {
            tokens.push(window.iter().collect());
        }
        if cjk.len() >= 2 {
            tokens.push(cjk.iter().collect());
        }
    }
    tokens
}

fn normalized_levenshtein_score(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 1.0;
    }
    let distance = levenshtein(a, b);
    distance as f64 / a.len().max(b.len()) as f64
}

fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut dp = vec![vec![0usize; b_chars.len() + 1]; a_chars.len() + 1];

    for (index, cell) in dp.iter_mut().enumerate() {
        cell[0] = index;
    }
    for (index, cell) in dp[0].iter_mut().enumerate() {
        *cell = index;
    }

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = usize::from(a_chars[i - 1] != b_chars[j - 1]);
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[a_chars.len()][b_chars.len()]
}

fn is_cjk(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
}
