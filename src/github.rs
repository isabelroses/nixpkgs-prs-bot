use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct PullRequest {
    title: String,
    html_url: String,
    number: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Markdown,
    PlainText,
}

pub struct FetchArgs<'a> {
    pub client: &'a Client,
    pub date: &'a str,
    pub output_format: OutputFormat,
    pub no_links: bool,
}

/// Fetch merged PRs from the nixpkgs repository
///
/// # Arguments
/// * `args` - The fetch arguments
///
/// # Returns
/// The formatted PRs
///
/// # Errors
/// If the request fails
pub async fn fetch_prs(args: FetchArgs<'_>) -> Result<String, crate::Error> {
    let response = args
        .client
        .get(format!(
            "https://api.github.com/search/issues?q=repo:nixos/nixpkgs+is:pr+is:merged+merged:{}",
            args.date
        ))
        .send()
        .await?;

    let json: Value = response.json().await?;
    let prs: Vec<PullRequest> = serde_json::from_value(json["items"].clone()).unwrap_or_default();

    let mut modules = Vec::new();
    let mut lib = Vec::new();
    let mut packages = Vec::new();

    let lib_regex = Regex::new(r"^lib(:|/[^:]+:)")?;

    for pr in prs.iter().filter(|pr| !pr.title.contains("Backport")) {
        let formatted = match (args.no_links, args.output_format) {
            (true, OutputFormat::Markdown) => format!("- #{} {}", pr.number, pr.title),
            (true, OutputFormat::PlainText) => format!("#{} {}", pr.number, pr.title),
            (false, OutputFormat::Markdown) => {
                format!("- [#{}]({}) {}", pr.number, pr.html_url, pr.title)
            }
            (false, OutputFormat::PlainText) => format!("#{}: {}", pr.title, pr.html_url),
        };

        if pr.title.starts_with("nixos") {
            modules.push((pr.title.contains("init"), formatted));
        } else if lib_regex.is_match(&pr.title) {
            lib.push((pr.title.contains("init"), formatted));
        } else {
            packages.push((pr.title.contains("init"), formatted));
        }
    }

    let mut output = vec![];
    let heading = match args.output_format {
        OutputFormat::Markdown => format!("# Merged PRs for {}", args.date),
        OutputFormat::PlainText => format!("Merged PRs for {}", args.date),
    };
    output.push(heading);

    if let Some(category) = format_category("Modules", args.output_format, &mut modules) {
        output.push(category);
    }
    if let Some(category) = format_category("Lib", args.output_format, &mut lib) {
        output.push(category);
    }
    if let Some(category) = format_category("Packages", args.output_format, &mut packages) {
        output.push(category);
    }

    Ok(output.join("\n\n"))
}

fn format_category(
    name: &str,
    out_format: OutputFormat,
    prs: &mut [(bool, String)],
) -> Option<String> {
    if prs.is_empty() {
        return None;
    }

    prs.sort_by(|a, b| b.0.cmp(&a.0));

    let header = match out_format {
        OutputFormat::Markdown => format!("## {name}"),
        OutputFormat::PlainText => name.to_string(),
    };

    let output = format!(
        "{}\n{}",
        header,
        prs.iter()
            .map(|(_, pr)| pr.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );

    Some(output)
}
