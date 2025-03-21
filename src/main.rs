use chrono::Utc;
use clap::{Parser, Subcommand};
use reqwest::Client;

mod github;
use github::{fetch_prs, FetchArgs, OutputFormat};

#[cfg(any(feature = "post-bsky", feature = "post-fedi"))]
use std::env;

#[cfg(feature = "post-bsky")]
mod bsky;
#[cfg(feature = "post-bsky")]
use bsky::BskyClient;

#[cfg(feature = "post-fedi")]
mod fedi;
#[cfg(feature = "post-fedi")]
use fedi::FediClient;

/// Error type
type Error = Box<dyn std::error::Error>;

#[derive(Parser)]
#[command(name = "nixpkgs-prs", about = "Fetch and post merged PRs", version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// fetch info from nixpkgs prs
    Fetch {
        /// output format for pr info
        #[arg(short, long, default_value = "markdown")]
        output_format: String,

        /// don't include links in pr list
        #[arg(long, default_value_t = false)]
        no_links: bool,

        /// fetch prs since yesterday
        #[arg(long, default_value_t = false)]
        yesterday: bool,
    },

    /// post pr info to bksy
    #[cfg(feature = "post-bsky")]
    Bsky {
        /// user email
        #[arg(short, long)]
        email: Option<String>,

        /// user password
        #[arg(short, long)]
        password: Option<String>,
    },

    /// post pr info to bksy
    #[cfg(feature = "post-fedi")]
    Fedi {
        /// url to fedi instance (e.g., <https://mastodon.social>), falls back to `$FEDI_INSTANCE`
        #[arg(short, long)]
        instance: Option<String>,

        /// access token (returned after initial login), falls back to `$FEDI_TOKEN`
        #[arg(short, long)]
        token: Option<String>,
    },

    #[cfg(feature = "post-fedi")]
    /// Create the fedi client token
    FediBootstrap {
        #[arg(short, long)]
        /// The instance to generate the token for
        instance: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();
    execute(args).await?;
    Ok(())
}

/// Execute the command
///
/// # Arguments
/// * `cli` - The CLI arguments
///
/// # Returns
/// Ok if successful, an error otherwise
///
/// # Errors
/// If the request fails
///
/// # Panics
/// If there is no yesterday
pub async fn execute(cli: Cli) -> Result<(), Error> {
    let client = Client::builder().user_agent("nixpkgs-pr-bot").build()?;

    match cli.command {
        Commands::Fetch {
            output_format,
            no_links,
            yesterday,
        } => {
            let date = if yesterday {
                Utc::now()
                    .date_naive()
                    .pred_opt()
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string()
            } else {
                Utc::now().date_naive().format("%Y-%m-%d").to_string()
            };

            let output_format = match output_format.as_str() {
                "plain" => OutputFormat::PlainText,
                _ => OutputFormat::Markdown,
            };

            let args = FetchArgs {
                client: &client,
                date: &date,
                output_format,
                no_links,
            };

            match fetch_prs(args).await {
                Ok(output) => println!("{output}"),
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        #[cfg(feature = "post-bsky")]
        Commands::Bsky { email, password } => {
            let bsky_email = match email {
                Some(email) => email,
                None => env::var("BSKY_EMAIL")
                    .map_err(|_| "Email not provided and BSKY_EMAIL not set")?,
            };

            let bsky_password = match password {
                Some(password) => password,
                None => env::var("BSKY_PASSWORD")
                    .map_err(|_| "Password not provided and BSKY_PASSWORD not set")?,
            };

            let bsky_client = BskyClient::new(bsky_email, bsky_password).await?;

            if let Err(e) = bsky_client.post_to_bsky(client).await {
                eprintln!("Error posting to Bluesky: {e}");
            }
        }
        #[cfg(feature = "post-fedi")]
        Commands::Fedi { instance, token } => {
            let fedi_instance = match instance {
                Some(instance) => instance,
                None => env::var("FEDI_INSTANCE")
                    .map_err(|_| "Instance not provided and FEDI_INSTANCE not set")?,
            };

            let fedi_token = match token {
                Some(token) => token,
                None => env::var("FEDI_TOKEN")
                    .map_err(|_| "Token not provided and FEDI_TOKEN not set")?,
            };

            let fedi_client = FediClient::new(client, &fedi_instance, fedi_token);

            if let Err(e) = fedi_client.post_to_fedi().await {
                eprintln!("Error posting to fedi: {e}");
            }
        }
        #[cfg(feature = "post-fedi")]
        Commands::FediBootstrap { instance } => {
            let fedi_instance = match instance {
                Some(instance) => instance,
                None => env::var("FEDI_INSTANCE")
                    .map_err(|_| "Instance not provided and FEDI_INSTANCE not set")?,
            };

            FediClient::bootstrap(fedi_instance).await?;
        }
    }

    Ok(())
}
