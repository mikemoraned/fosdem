use chrono::NaiveDate;
use gray_matter::{engine::YAML, Matter};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid filename format: {0}")]
    InvalidFilename(String),
    #[error("Invalid date in filename: {0}")]
    InvalidDate(String),
    #[error("Missing required frontmatter field: {0}")]
    MissingFrontmatter(String),
    #[error("Invalid frontmatter: {0}")]
    InvalidFrontmatter(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostFrontmatter {
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Post {
    pub date: NaiveDate,
    pub title: String,
    pub tags: Vec<String>,
    pub content_markdown: String,
    pub content_html: String,
}

impl Post {
    pub fn slug(&self) -> String {
        self.date.format("%Y-%m-%d").to_string()
    }

    pub fn url_path(&self) -> String {
        format!("/blog/{}/", self.slug())
    }
}

/// Parse a markdown file into a Post
pub fn parse_post(filename: &str, content: &str) -> Result<Post, BlogError> {
    // Extract date from filename (e.g., "2026-02-02.md")
    let date = parse_date_from_filename(filename)?;

    // Parse frontmatter
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(content);

    let frontmatter: PostFrontmatter = parsed
        .data
        .ok_or_else(|| BlogError::MissingFrontmatter("title".to_string()))?
        .deserialize()
        .map_err(|e| BlogError::InvalidFrontmatter(e.to_string()))?;

    // Convert markdown to HTML
    let content_markdown = parsed.content.clone();
    let content_html = markdown_to_html(&content_markdown);

    Ok(Post {
        date,
        title: frontmatter.title,
        tags: frontmatter.tags,
        content_markdown,
        content_html,
    })
}

fn parse_date_from_filename(filename: &str) -> Result<NaiveDate, BlogError> {
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| BlogError::InvalidFilename(filename.to_string()))?;

    NaiveDate::parse_from_str(stem, "%Y-%m-%d")
        .map_err(|_| BlogError::InvalidDate(filename.to_string()))
}

fn markdown_to_html(markdown: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_SMART_PUNCTUATION;

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// A collection of blog posts
#[derive(Debug, Clone)]
pub struct BlogIndex {
    posts: Vec<Post>,
}

impl BlogIndex {
    /// Load all posts from a directory
    pub fn load_from_dir(dir: &Path) -> Result<Self, BlogError> {
        let mut posts = Vec::new();

        if !dir.exists() {
            return Ok(Self { posts });
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "md") {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| BlogError::InvalidFilename(path.display().to_string()))?;

                let content = std::fs::read_to_string(&path)?;
                let post = parse_post(filename, &content)?;
                posts.push(post);
            }
        }

        // Sort by date, newest first
        posts.sort_by(|a, b| b.date.cmp(&a.date));

        Ok(Self { posts })
    }

    /// Get all posts, newest first
    pub fn all_posts(&self) -> &[Post] {
        &self.posts
    }

    /// Get the most recent N posts
    pub fn recent_posts(&self, count: usize) -> Vec<&Post> {
        self.posts.iter().take(count).collect()
    }

    /// Find a post by its date slug (e.g., "2026-02-02")
    pub fn find_by_slug(&self, slug: &str) -> Option<&Post> {
        self.posts.iter().find(|p| p.slug() == slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_post() {
        let content = r#"---
title: My First Post
tags:
  - rust
  - web
---

This is the content of my post.

## A heading

Some more content.
"#;

        let post = parse_post("2026-02-02.md", content).unwrap();

        assert_eq!(post.date, NaiveDate::from_ymd_opt(2026, 2, 2).unwrap());
        assert_eq!(post.title, "My First Post");
        assert_eq!(post.tags, vec!["rust", "web"]);
        assert!(post.content_html.contains("<h2>"));
        assert!(post.content_html.contains("A heading"));
        assert_eq!(post.slug(), "2026-02-02");
        assert_eq!(post.url_path(), "/blog/2026-02-02/");
    }

    #[test]
    fn test_parse_post_no_tags() {
        let content = r#"---
title: Simple Post
---

Just some content.
"#;

        let post = parse_post("2025-01-15.md", content).unwrap();

        assert_eq!(post.title, "Simple Post");
        assert!(post.tags.is_empty());
    }

    #[test]
    fn test_invalid_filename() {
        let content = "---\ntitle: Test\n---\nContent";
        let result = parse_post("not-a-date.md", content);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_title() {
        let content = "---\ntags: [foo]\n---\nContent";
        let result = parse_post("2026-02-02.md", content);
        assert!(result.is_err());
    }
}
