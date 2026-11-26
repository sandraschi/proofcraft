use crate::{ProofError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteratureRef {
    pub id: String,
    pub arxiv_id: String,
    pub title: String,
    pub summary: String,
    pub url: String,
}

pub fn normalize_arxiv_id(input: &str) -> String {
    let trimmed = input.trim();
    let stripped = trimmed
        .strip_prefix("https://arxiv.org/abs/")
        .or_else(|| trimmed.strip_prefix("http://arxiv.org/abs/"))
        .or_else(|| trimmed.strip_prefix("https://arxiv.org/pdf/"))
        .or_else(|| trimmed.strip_prefix("arxiv:"))
        .unwrap_or(trimmed);
    stripped.trim_end_matches(".pdf").to_string()
}

pub fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let rest = &xml[start..];
    let end = rest.find(&close)?;
    Some(rest[..end].split_whitespace().collect::<Vec<_>>().join(" "))
}

pub fn parse_arxiv_atom(xml: &str, arxiv_id: &str) -> Result<LiteratureRef> {
    let entry_start = xml
        .find("<entry>")
        .ok_or_else(|| ProofError::NotFound(format!("arXiv entry not found for {arxiv_id}")))?;
    let entry = &xml[entry_start..];
    let title = extract_xml_tag(entry, "title").ok_or_else(|| {
        ProofError::Provider("missing <title> in arXiv Atom response".to_string())
    })?;
    let summary = extract_xml_tag(entry, "summary").unwrap_or_default();
    Ok(LiteratureRef {
        id: format!("lit-{}", arxiv_id.replace(['/', '.'], "-")),
        arxiv_id: arxiv_id.to_string(),
        title,
        summary,
        url: format!("https://arxiv.org/abs/{arxiv_id}"),
    })
}

pub fn fetch_arxiv_metadata(id_or_url: &str) -> Result<LiteratureRef> {
    let arxiv_id = normalize_arxiv_id(id_or_url);
    if arxiv_id.is_empty() {
        return Err(ProofError::InvalidInput("empty arXiv ID".to_string()));
    }
    let url = format!("https://export.arxiv.org/api/query?id_list={arxiv_id}");
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(15))
        .build();
    let resp = agent
        .get(&url)
        .set("User-Agent", "proofcraft/0.2.0 (research-harness)")
        .call()
        .map_err(|e| ProofError::Provider(format!("arXiv fetch failed: {e}")))?;
    let body = resp
        .into_string()
        .map_err(|e| ProofError::Provider(format!("arXiv body read failed: {e}")))?;
    parse_arxiv_atom(&body, &arxiv_id)
}
