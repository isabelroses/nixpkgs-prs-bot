use chrono::Utc;
use fetch_prs::{fetch_prs, FetchArgs, OutputFormat};
use std::process::Command;

pub struct FediClient {
    pub instance: String,
    pub email: String,
    pub password: String,
}

type E = Box<dyn std::error::Error>;

impl FediClient {
    pub fn new(instance: String, email: String, password: String) -> Self {
        Self {
            instance,
            email,
            password,
        }
    }

    pub async fn post_to_fedi(&self, client: reqwest::Client) -> Result<String, E> {
        let auth_status = Command::new("toot").arg("auth").output();
        match auth_status {
            Ok(output) => {
                let stdout = String::from_utf8(output.stdout);
                match stdout {
                    Ok(content) => {
                        if content.trim() == "You are not logged in to any accounts" {
                            Command::new("toot")
                                .args([
                                    "login_cli",
                                    "-i",
                                    &self.instance,
                                    "-e",
                                    &self.email,
                                    "-p",
                                    &self.password,
                                ])
                                .output()?;
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Err(e) => return Err(e.into()),
        }

        let date = Utc::now()
            .date_naive()
            .pred_opt()
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();

        let fetch_args = FetchArgs {
            client: &client,
            date: &date,
            output_format: OutputFormat::Markdown,
            no_links: false,
        };

        let output = fetch_prs(fetch_args)
            .await
            .map_err(|e| format!("Failed to fetch PRs: {}", e))?;

        Command::new("toot")
            .arg("post")
            .args(["-v", "private", "-t", "text/markdown"])
            .arg(&output)
            .output()
            .map_err(|_| "Failed to post to Fedi")?;

        Ok(output)
    }
}
