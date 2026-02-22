use crate::processor::SearchMatch;
use serde_json::{json, Value};
use std::path::Path;

/// Output format types
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum OutputFormat {
    Text,
    Json,
    Xml,
    Html,
    Markdown,
    Csv,
    Tsv,
}

/// Default implementation for OutputFormat
impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Text
    }
}

/// Output formatter for different formats
#[allow(dead_code)]
pub struct OutputFormatter {
    format: OutputFormat,
    include_metadata: bool,
    include_context: bool,
    use_color: bool,
    ndjson: bool,
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self {
            format: OutputFormat::default(),
            include_metadata: true,
            include_context: true,
            use_color: is_terminal::is_terminal(&std::io::stdout()),
            ndjson: false,
        }
    }
}

impl OutputFormatter {
    #[allow(dead_code)]
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            include_metadata: true,
            include_context: true,
            use_color: is_terminal::is_terminal(&std::io::stdout()),
            ndjson: false,
        }
    }

    #[allow(dead_code)]
    pub fn with_ndjson(mut self, ndjson: bool) -> Self {
        self.ndjson = ndjson;
        self
    }

    #[allow(dead_code)]
    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    #[allow(dead_code)]
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    #[allow(dead_code)]
    pub fn with_context(mut self, include: bool) -> Self {
        self.include_context = include;
        self
    }

    /// Format search results
    #[allow(dead_code)]
    pub fn format_results(&self, matches: &[SearchMatch], query: &str, path: &Path) -> String {
        match self.format {
            OutputFormat::Text => self.format_text(matches, query, path),
            OutputFormat::Json => self.format_json(matches, query, path),
            OutputFormat::Xml => self.format_xml(matches, query, path),
            OutputFormat::Html => self.format_html(matches, query, path),
            OutputFormat::Markdown => self.format_markdown(matches, query, path),
            OutputFormat::Csv => self.format_csv(matches, query, path),
            OutputFormat::Tsv => self.format_tsv(matches, query, path),
        }
    }

    /// Format as JSON
    #[allow(dead_code)]
    fn format_json(&self, matches: &[SearchMatch], query: &str, path: &Path) -> String {
        if self.ndjson {
            let mut out = String::new();
            for m in matches {
                let mut match_obj = json!({
                    "query": query,
                    "path": m.path.to_string_lossy(),
                    "line_number": m.line_number,
                    "line": m.line,
                    "matched_text": m.matched_text,
                    "column_start": m.column_start,
                    "column_end": m.column_end,
                });

                if self.include_context {
                    let context_before: Vec<Value> = m
                        .context_before
                        .iter()
                        .map(|(num, line)| {
                            json!({
                                "line_number": num,
                                "content": line
                            })
                        })
                        .collect();

                    let context_after: Vec<Value> = m
                        .context_after
                        .iter()
                        .map(|(num, line)| {
                            json!({
                                "line_number": num,
                                "content": line
                            })
                        })
                        .collect();

                    match_obj["context_before"] = Value::Array(context_before);
                    match_obj["context_after"] = Value::Array(context_after);
                }

                match serde_json::to_string(&match_obj) {
                    Ok(s) => {
                        out.push_str(&s);
                        out.push('\n');
                    }
                    Err(e) => {
                        let err_obj =
                            json!({"error": "json_serialization_failed", "details": e.to_string()});
                        if let Ok(s) = serde_json::to_string(&err_obj) {
                            out.push_str(&s);
                            out.push('\n');
                        } else {
                            out.push_str("{\"error\":\"json_serialization_failed\"}\n");
                        }
                    }
                }
            }
            return out;
        }

        let mut result = json!({
            "query": query,
            "path": path.to_string_lossy(),
            "total_matches": matches.len(),
            "matches": []
        });

        let matches_array = result["matches"].as_array_mut().unwrap();

        for m in matches {
            let mut match_obj = json!({
                "path": m.path.to_string_lossy(),
                "line_number": m.line_number,
                "line": m.line,
                "matched_text": m.matched_text,
                "column_start": m.column_start,
                "column_end": m.column_end,
            });

            if self.include_context {
                let context_before: Vec<Value> = m
                    .context_before
                    .iter()
                    .map(|(num, line)| {
                        json!({
                            "line_number": num,
                            "content": line
                        })
                    })
                    .collect();

                let context_after: Vec<Value> = m
                    .context_after
                    .iter()
                    .map(|(num, line)| {
                        json!({
                            "line_number": num,
                            "content": line
                        })
                    })
                    .collect();

                match_obj["context_before"] = Value::Array(context_before);
                match_obj["context_after"] = Value::Array(context_after);
            }

            matches_array.push(match_obj);
        }

        serde_json::to_string(&result).unwrap_or_else(|e| {
            format!(r#"{{"error":"json_serialization_failed","details":"{e}"}}"#)
        })
    }

    /// Format as plain text (default)
    #[allow(dead_code)]
    fn format_text(&self, matches: &[SearchMatch], query: &str, path: &Path) -> String {
        let mut output = String::new();
        let word_len = query.len();
        // metadata header
        if self.include_metadata {
            output.push_str(&format!("Query: {query}\n"));
            output.push_str(&format!("Path: {}\n", path.display()));
            output.push_str(&format!("Total matches: {}\n\n", matches.len()));
        }

        // Changes being introduced reuse the memchr method find
        // to iterate through matched text in order to ease the highlighting process

        // [Note:] Currently there is no way to handle highlighting of multiple instances of a search pattern found
        // within a matched text.

        // default one-line-per-match: path:line:col: line-with-highlight
        for m in matches {
            let ind_match = find_match_indices(query.as_bytes(), &m.matched_text.as_bytes());
            println!("text: {}, query: {}", &m.matched_text, query);
            let match_indices = ind_match.as_slice();
            let line_len = m.line.len();
            let column_start = m.column_start;
            let column_end = m.column_end;
            println!("{:#?}", ind_match);
            let before = if column_start < line_len {
                &m.line[..column_start]
            } else {
                ""
            };
            let matched = &m.matched_text;
            println!("column_end :{} \nline_len: {}", &column_end, &line_len);
            println!("char at column_end: {}", &m.line[column_end..]);
            println!("char at column_start: {}", &m.line[column_start..]);
            let after = if column_end < line_len {
                &m.line[column_end..]
            } else {
                ""
            };

            // This ordering will not work for multiple instances of highlighting in the same line
            if self.use_color {
                // ANSI yellow highlight for match
                let highlighted = highlight(matched.as_str(), match_indices, word_len);
                // let highlighted = format!("\x1b[33m{matched}\x1b[0m");
                output.push_str(&format!(
                    // "{}:{}:{}: {before}{highlighted}{after}\n",
                    "{}:{}: {highlighted}\n",
                    m.path.display(),
                    m.line_number,
                    // column_start + 1
                ));
            } else {
                output.push_str(&format!(
                    "{}:{}:{}: {before}{matched}{after}\n",
                    m.path.display(),
                    m.line_number,
                    column_start + 1
                ));
            }

            // if context requested, append a small block
            if self.include_context && (!m.context_before.is_empty() || !m.context_after.is_empty())
            {
                output.push_str("-- context --\n");
                for (num, line) in &m.context_before {
                    output.push_str(&format!("  {num} │ {line}\n"));
                }
                output.push_str(&format!("→ {} │ {before}{matched}{after}\n", m.line_number));
                for (num, line) in &m.context_after {
                    output.push_str(&format!("  {num} │ {line}\n"));
                }
                output.push('\n');
            }
        }

        output
    }

    /// Format as XML
    #[allow(dead_code)]
    fn format_xml(&self, matches: &[SearchMatch], query: &str, path: &Path) -> String {
        let mut output = String::new();
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        output.push_str("<search-results>\n");

        if self.include_metadata {
            output.push_str("  <metadata>\n");
            output.push_str(&format!("    <query>{}</query>\n", escape_xml(query)));
            output.push_str(&format!(
                "    <path>{}</path>\n",
                escape_xml(&path.to_string_lossy())
            ));
            output.push_str(&format!(
                "    <total-matches>{}</total-matches>\n",
                matches.len()
            ));
            output.push_str("  </metadata>\n");
        }

        output.push_str("  <matches>\n");

        for (i, m) in matches.iter().enumerate() {
            output.push_str(&format!("    <match index=\"{}\">\n", i + 1));
            output.push_str(&format!(
                "      <line-number>{}</line-number>\n",
                m.line_number
            ));
            output.push_str(&format!("      <line>{}</line>\n", escape_xml(&m.line)));
            output.push_str(&format!(
                "      <matched-text>{}</matched-text>\n",
                escape_xml(&m.matched_text)
            ));
            output.push_str(&format!(
                "      <column-start>{}</column-start>\n",
                m.column_start
            ));
            output.push_str(&format!(
                "      <column-end>{}</column-end>\n",
                m.column_end
            ));
            output.push_str("    </match>\n");
        }

        output.push_str("  </matches>\n");
        output.push_str("</search-results>\n");

        output
    }

    /// Format as HTML
    #[allow(dead_code)]
    fn format_html(&self, matches: &[SearchMatch], query: &str, path: &Path) -> String {
        let mut output = String::new();
        output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        output.push_str("<meta charset=\"UTF-8\">\n");
        output.push_str("<title>rfgrep Search Results</title>\n");
        output.push_str("<style>\n");
        output.push_str("body { font-family: monospace; margin: 20px; }\n");
        output.push_str(
            ".match { margin: 10px 0; padding: 10px; border-left: 3px solid #007acc; }\n",
        );
        output.push_str(".line-number { color: #666; }\n");
        output.push_str(".matched-text { background-color: #ffff00; font-weight: bold; }\n");
        output.push_str(".context { color: #888; }\n");
        output.push_str(
            ".metadata { background-color: #f5f5f5; padding: 10px; margin-bottom: 20px; }\n",
        );
        output.push_str("</style>\n</head>\n<body>\n");

        if self.include_metadata {
            output.push_str("<div class=\"metadata\">\n");
            output.push_str("<h2>Search Results</h2>\n");
            output.push_str(&format!(
                "<p><strong>Query:</strong> {}</p>\n",
                escape_html(query)
            ));
            output.push_str(&format!(
                "<p><strong>Path:</strong> {}</p>\n",
                escape_html(&path.to_string_lossy())
            ));
            output.push_str(&format!(
                "<p><strong>Total Matches:</strong> {}</p>\n",
                matches.len()
            ));
            output.push_str("</div>\n");
        }

        for (i, m) in matches.iter().enumerate() {
            output.push_str("<div class=\"match\">\n");
            output.push_str(&format!("<h3>Match {}</h3>\n", i + 1));

            // Highlight the match
            let line_len = m.line.len();
            let column_start = m.column_start.min(line_len);
            let column_end = m.column_end.min(line_len);

            let before = if column_start < line_len {
                &m.line[..column_start]
            } else {
                ""
            };
            let matched_text = &m.matched_text;
            let after = if column_end < line_len {
                &m.line[column_end..]
            } else {
                ""
            };

            output.push_str("<div>");
            let matched_html = format!(
                "<span class=\"matched-text\">{}</span>",
                escape_html(matched_text)
            );
            output.push_str(&format!(
                "<span class=\"line-number\">→ {:>4}</span> │ {}{}{}",
                m.line_number,
                escape_html(before),
                matched_html,
                escape_html(after)
            ));
            output.push_str("</div>\n");
            output.push_str("</div>\n");
        }

        output.push_str("</body>\n</html>\n");

        output
    }

    /// Format as Markdown
    #[allow(dead_code)]
    fn format_markdown(&self, matches: &[SearchMatch], query: &str, path: &Path) -> String {
        let mut output = String::new();

        output.push_str("# rfgrep Search Results\n\n");

        if self.include_metadata {
            output.push_str(&format!("**Query:** `{query}`\n"));
            output.push_str(&format!("**Path:** `{}`\n", path.display()));
            output.push_str(&format!("**Total Matches:** {}\n\n", matches.len()));
        }

        for (i, m) in matches.iter().enumerate() {
            output.push_str(&format!("## Match {}\n\n", i + 1));

            let line_len = m.line.len();
            let column_start = m.column_start.min(line_len);
            let column_end = m.column_end.min(line_len);

            let before = if column_start < line_len {
                &m.line[..column_start]
            } else {
                ""
            };
            let matched = &m.matched_text;
            let after = if column_end < line_len {
                &m.line[column_end..]
            } else {
                ""
            };

            output.push_str("**Match:**\n");
            output.push_str("```\n");
            output.push_str(&format!(
                "→ {:>4} │ {before}{matched}{after}\n",
                m.line_number
            ));
            output.push_str("```\n\n");
        }

        output
    }

    /// Format as CSV
    #[allow(dead_code)]
    fn format_csv(&self, matches: &[SearchMatch], _query: &str, _path: &Path) -> String {
        let mut output = String::default();
        // CSV header
        output.push_str("file,line_number,column_start,column_end,matched_text,line_content\n");

        for m in matches {
            let file = escape_csv(&m.path.to_string_lossy());
            let line_content = escape_csv(&m.line);
            let matched_text = escape_csv(&m.matched_text);
            output.push_str(&format!(
                "{},{},{},{},{},{}\n",
                file, m.line_number, m.column_start, m.column_end, matched_text, line_content
            ));
        }

        output
    }

    /// Format as TSV
    #[allow(dead_code)]
    fn format_tsv(&self, matches: &[SearchMatch], _query: &str, _path: &Path) -> String {
        let mut output = String::default();
        // TSV header
        output
            .push_str("file\tline_number\tcolumn_start\tcolumn_end\tmatched_text\tline_content\n");

        for m in matches {
            let file = escape_tsv(&m.path.to_string_lossy());
            let line_content = escape_tsv(&m.line);
            let matched_text = escape_tsv(&m.matched_text);
            output.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\n",
                file, m.line_number, m.column_start, m.column_end, matched_text, line_content
            ));
        }

        output
    }
}

/// Escape CSV special characters
#[allow(dead_code)]
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace("\"", "\"\""))
    } else {
        s.to_string()
    }
}

/// Escape TSV special characters
#[allow(dead_code)]
fn escape_tsv(s: &str) -> String {
    s.replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Escape XML special characters
#[allow(dead_code)]
fn escape_xml(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

/// Escape HTML special characters
#[allow(dead_code)]
fn escape_html(s: &str) -> String {
    s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
}

fn find_match_indices(pattern: &[u8], matched_text: &[u8]) -> Vec<usize> {
    use memchr::memmem;
    let iterator: Vec<_> = memmem::find_iter(matched_text, pattern).collect();
    return iterator;
}

fn highlight(text: &str, starts: &[usize], word_len: usize) -> String {
    let mut result = String::new();
    let mut last = 0;
    println!("actual text: {text}");
    for &start in starts {
        // push text before match -- This is not bound to always run but it is important
        result.push_str(&text[last..start]);
        let highlighted = &text[start..start + word_len];
        result.push_str(format!("\x1b[33m{highlighted}\x1b[0m").as_str());
        last = start + word_len;
    }

    // push remaining tail
    result.push_str(&text[last..]);
    result
}
