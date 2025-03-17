use crate::github::{fetch_prs, FetchArgs, OutputFormat};
use chrono::Utc;
use megalodon::{
    entities::status::StatusVisibility,
    generator,
    megalodon::{AppInputOptions, PostStatusInputOptions},
    SNS,
};

pub struct FediClient {
    client: Box<dyn megalodon::Megalodon + Send + Sync>,
}

impl FediClient {
    pub async fn new(instance: String, client_token: String) -> Result<Self, crate::Error> {
        let client = generator(SNS::Pleroma, instance.clone(), Some(client_token), None)?;

        Ok(FediClient { client })
    }

    pub async fn bootstrap(instance: String) -> Result<String, crate::Error> {
        let client = generator(SNS::Pleroma, instance.clone(), None, None)?;

        let opts = AppInputOptions {
            scopes: Some(["read".to_string(), "write".to_string()].to_vec()),
            ..Default::default()
        };

        let app_data = match client
            .register_app("nixpkgs-prs-bot".to_string(), &opts)
            .await
        {
            Ok(data) => data,
            Err(e) => return Err(Box::new(e)),
        };

        let code = {
            // don't unwrap or this, it will not work lolllllll
            let str_out = format!("Enter authorization code from {}: ", app_data.url.unwrap());
            println!("{}", str_out);
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line)?;
            line.trim().to_string()
        };

        let token_data = client
            .fetch_access_token(
                app_data.client_id,
                app_data.client_secret,
                code,
                megalodon::default::NO_REDIRECT.to_string(),
            )
            .await?;

        let token = token_data.access_token;

        println!("Access token: {}", token);

        Ok(token)
    }

    pub async fn post_to_fedi(&self, client: reqwest::Client) -> Result<(), crate::Error> {
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

        self.client
            .post_status(
                output,
                Some(&PostStatusInputOptions {
                    visibility: Some(StatusVisibility::Public),
                    language: Some("en".to_string()),
                    ..Default::default()
                }),
            )
            .await?;

        Ok(())
    }
}
