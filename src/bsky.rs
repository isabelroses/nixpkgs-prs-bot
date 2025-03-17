use crate::github::{fetch_prs, FetchArgs, OutputFormat};
use bsky_sdk::{
    api::{
        app::bsky::feed::post::{RecordData, ReplyRef, ReplyRefData},
        com::atproto::repo::strong_ref,
        types::{string::Datetime, Object},
    },
    BskyAgent,
};
use chrono::Utc;
use ipld_core::ipld::Ipld;
use reqwest::Client;

pub struct BskyClient {
    pub agent: BskyAgent,
}

impl BskyClient {
    pub async fn new(email: String, password: String) -> Result<Self, crate::Error> {
        let agent = BskyAgent::builder().build().await?;
        let _ = agent.login(email, password).await?;
        Ok(Self { agent })
    }

    pub async fn post_to_bsky(&self, client: Client) -> Result<(), String> {
        let date = Utc::now()
            .date_naive()
            .pred_opt()
            .unwrap()
            .format("%Y-%m-%d")
            .to_string();

        let args = FetchArgs {
            client: &client,
            date: &date,
            output_format: OutputFormat::PlainText,
            no_links: true,
        };

        let full_content = fetch_prs(args)
            .await
            .map_err(|e| format!("Failed to fetch PRs: {e}"))?;

        // Split content into chunks of approximately 300 words
        let chunks = self.split_into_chunks(full_content, 300);

        // Post the first chunk as the main post
        let main_post: Object<bsky_sdk::api::com::atproto::repo::create_record::OutputData> = self
            .agent
            .create_record(RecordData {
                created_at: Datetime::now(),
                embed: None,
                entities: None,
                facets: None,
                labels: None,
                langs: None,
                reply: None,
                tags: None,
                text: chunks[0].clone(),
            })
            .await
            .map_err(|e| format!("Failed to post main content to bsky: {e}"))?;

        // Get the URI and CID of the main post for reply references
        let main_uri = main_post.uri.clone();
        let main_cid = main_post.cid.clone();

        // Post any additional chunks as replies
        let mut last_uri = main_uri.clone();
        let mut last_cid = main_cid.clone();

        for chunk in chunks.iter().skip(1) {
            // Create a reply reference to the previous post
            let reply = ReplyRef {
                data: ReplyRefData {
                    parent: Object {
                        data: strong_ref::MainData {
                            cid: last_cid.clone(),
                            uri: last_uri.clone(),
                        },
                        extra_data: Ipld::Null,
                    },
                    root: Object {
                        data: strong_ref::MainData {
                            cid: main_cid.clone(),
                            uri: main_uri.clone(),
                        },
                        extra_data: Ipld::Null,
                    },
                },
                extra_data: Ipld::Null,
            };

            // Post the reply
            let reply_post = self
                .agent
                .create_record(RecordData {
                    created_at: Datetime::now(),
                    embed: None,
                    entities: None,
                    facets: None,
                    labels: None,
                    langs: None,
                    reply: Some(reply),
                    tags: None,
                    text: chunk.clone(),
                })
                .await
                .map_err(|e| format!("Failed to post reply to bsky: {e}"))?;

            // Update the references for the next reply
            last_uri = reply_post.uri.clone();
            last_cid = reply_post.cid.clone();
        }

        Ok(())
    }

    // Helper function to split content into chunks of approximately `max_chars` characters
    fn split_into_chunks(&self, content: String, max_chars: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        for line in content.split_inclusive('\n') {
            if current_chunk.len() + line.len() > max_chars {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
            }

            if current_chunk.len() + line.len() <= max_chars {
                current_chunk.push_str(line);
            } else {
                let mut words = line.split_whitespace().peekable();
                while let Some(word) = words.next() {
                    if !current_chunk.is_empty() && current_chunk.len() + word.len() + 1 > max_chars
                    {
                        chunks.push(current_chunk.clone());
                        current_chunk.clear();
                    }

                    if !current_chunk.is_empty() {
                        current_chunk.push(' ');
                    }

                    current_chunk.push_str(word);
                    if words.peek().is_none() && line.ends_with('\n') {
                        current_chunk.push('\n');
                    }
                }
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }
}
