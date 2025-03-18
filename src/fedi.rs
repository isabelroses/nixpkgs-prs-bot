use crate::github::{fetch_prs, FetchArgs, OutputFormat};
use chrono::Utc;
use megalodon::{generator, megalodon::AppInputOptions, SNS};
use reqwest::{multipart, Url};

pub struct FediClient {
    //client: Box<dyn megalodon::Megalodon + Send + Sync>,
    api_client: reqwest::Client,
    instance: String,
    access_token: Option<String>,
}

impl FediClient {
    pub fn new(api_client: reqwest::Client, instance: &str, client_token: String) -> Self {
        //let client = generator(
        //    SNS::Pleroma,
        //    instance.to_string(),
        //    Some(client_token.clone()),
        //    None,
        //)?;

        FediClient {
            //client,
            api_client,
            instance: instance.to_string(),
            access_token: Some(client_token),
        }
    }

    pub async fn bootstrap(instance: String) -> Result<String, crate::Error> {
        let client = generator(SNS::Pleroma, instance.clone(), None, None)?;

        let opts = AppInputOptions {
            scopes: Some(["write".to_string()].to_vec()),
            ..Default::default()
        };

        let app_data = client
            .register_app("nixpkgs-prs-bot".to_string(), &opts)
            .await?;

        let code = {
            // don't unwrap or this, it will not work lolllllll
            let str_out = format!("Enter authorization code from {}: ", app_data.url.unwrap());
            println!("{str_out}");
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

        println!("Access token: {token}");

        Ok(token)
    }

    pub async fn post_to_fedi(&self) -> Result<(), crate::Error> {
        let date = Utc::now()
            .date_naive()
            .pred_opt()
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();

        let fetch_args = FetchArgs {
            client: &self.api_client,
            date: &date,
            output_format: OutputFormat::Markdown,
            no_links: false,
        };

        let output = fetch_prs(fetch_args)
            .await
            .map_err(|e| format!("Failed to fetch PRs: {e}"))?;

        self.post_status(output).await?;

        Ok(())
    }

    async fn post_status(&self, status: String) -> Result<(), crate::Error> {
        let url_str = format!("{}/api/v1/statuses", self.instance);
        let url = Url::parse(&url_str)?;
        let req = self
            .api_client
            .post(url)
            .bearer_auth(self.access_token.as_ref().unwrap());

        let params = multipart::Form::new()
            .text("status", status)
            .text("visibility", "public")
            .text("language", "en")
            .text("content_type", "text/markdown");

        req.multipart(params).send().await?;

        Ok(())
    }
}
