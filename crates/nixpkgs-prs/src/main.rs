use chrono::Utc;
use clap::{Parser, Subcommand};
use fetch_prs::{fetch_prs, FetchArgs, OutputFormat};
use post_bsky::BskyClient;
use post_fedi::FediClient;
use reqwest::Client;
use std::env;

/// Error type
type E = Box<dyn std::error::Error>;

#[derive(Parser)]
#[command(name = "nixpkgs-prs-bot", about = "Fetch and post merged PRs")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Fetch {
        #[arg(long, default_value = "markdown")]
        output_format: String,

        #[arg(long, default_value_t = false)]
        no_links: bool,

        #[arg(long, default_value_t = false)]
        yesterday: bool,
    },
    Bsky {
        #[arg(long)]
        email: Option<String>,

        #[arg(long)]
        password: Option<String>,
    },
    Fedi {
        #[arg(long)]
        instance: Option<String>,

        #[arg(long)]
        email: Option<String>,

        #[arg(long)]
        password: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), E> {
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
pub async fn execute(cli: Cli) -> Result<(), E> {
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
        Commands::Bsky { email, password } => {
            let bsky_email =
                email.unwrap_or(env::var("BSKY_EMAIL").expect("BSKY_USERNAME not set"));
            let bsky_password =
                password.unwrap_or(env::var("BSKY_PASSWORD").expect("BSKY_PASSWORD not set"));

            let bsky_client = BskyClient::new(bsky_email, bsky_password).await?;

            if let Err(e) = bsky_client.post_to_bsky(client).await {
                eprintln!("Error posting to Bluesky: {e}");
            }
        }
        Commands::Fedi {
            instance,
            email,
            password,
        } => {
            let fedi_instance =
                instance.unwrap_or(env::var("FEDI_INSTANCE").expect("FEDI_INSTANCE not set"));
            let fedi_email = email.unwrap_or(env::var("FEDI_EMAIL").expect("FEDI_EMAIL not set"));
            let fedi_password =
                password.unwrap_or(env::var("FEDI_PASSWORD").expect("FEDI_PASSWORD not set"));

            let fedi_client = FediClient::new(fedi_instance, fedi_email, fedi_password);

            if let Err(e) = fedi_client.post_to_fedi(client).await {
                eprintln!("Error posting to fedi: {e}");
            }
        }
    }

    Ok(())
}
