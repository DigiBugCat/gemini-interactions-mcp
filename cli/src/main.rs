use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Instant;

const INTERACTIONS_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta/interactions";
const MODEL: &str = "gemini-3-flash-preview";

#[derive(Parser)]
#[command(name = "gemini-ask")]
#[command(about = "CLI for Gemini Interactions API with grounding", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Search query (shorthand for search command)
    #[arg(long, conflicts_with = "ask_query", conflicts_with = "think_query")]
    search: Option<String>,

    /// Ask query (shorthand for ask command)
    #[arg(long, conflicts_with = "search", conflicts_with = "think_query")]
    ask: Option<String>,

    /// Think query (shorthand for ask-thinking command)
    #[arg(long, conflicts_with = "search", conflicts_with = "ask_query")]
    think: Option<String>,

    /// Previous interaction ID for follow-up
    #[arg(short, long)]
    interaction: Option<String>,

    /// Files to include (can be repeated)
    #[arg(short, long, action = clap::ArgAction::Append)]
    file: Vec<String>,

    /// Output format
    #[arg(short, long, default_value = "text")]
    output: OutputFormat,
}

#[derive(Subcommand)]
enum Commands {
    /// Quick search with minimal thinking
    Search {
        query: String,
        #[arg(long, default_value = "10")]
        max_results: u32,
    },
    /// Get grounded answer with balanced reasoning
    Ask {
        query: String,
        #[arg(short, long)]
        interaction: Option<String>,
    },
    /// Deep reasoning for complex problems
    Think {
        query: String,
        #[arg(short, long)]
        interaction: Option<String>,
    },
    /// Continue previous conversation
    FollowUp {
        query: String,
        #[arg(short, long)]
        interaction: String,
        #[arg(long, default_value = "medium")]
        thinking_level: ThinkingLevel,
    },
    /// Check status of async interaction
    Status { interaction_id: String },
    /// Cancel async interaction
    Cancel { interaction_id: String },
}

#[derive(Clone, Debug, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, clap::ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
enum ThinkingLevel {
    Minimal,
    Low,
    Medium,
    High,
}

impl std::fmt::Display for ThinkingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThinkingLevel::Minimal => write!(f, "minimal"),
            ThinkingLevel::Low => write!(f, "low"),
            ThinkingLevel::Medium => write!(f, "medium"),
            ThinkingLevel::High => write!(f, "high"),
        }
    }
}

#[derive(Serialize)]
struct InteractionRequest {
    model: String,
    input: serde_json::Value,
    store: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_interaction_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<bool>,
    generation_config: GenerationConfig,
    tools: Vec<Tool>,
}

#[derive(Serialize)]
struct GenerationConfig {
    thinking_level: String,
    max_output_tokens: u32,
}

#[derive(Serialize)]
struct Tool {
    r#type: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct InteractionResponse {
    id: Option<String>,
    status: Option<String>,
    outputs: Option<Vec<Output>>,
    usage: Option<Usage>,
    #[serde(default)]
    error: Option<ApiError>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Output {
    r#type: String,
    text: Option<String>,
    annotations: Option<Vec<Annotation>>,
    result: Option<Vec<SearchResult>>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Annotation {
    source: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct SearchResult {
    url: Option<String>,
    title: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Usage {
    total_input_tokens: Option<u32>,
    total_output_tokens: Option<u32>,
    total_reasoning_tokens: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
struct ApiError {
    message: Option<String>,
    code: Option<String>,
}

fn get_api_key() -> Result<String> {
    env::var("GEMINI_API_KEY").context(
        "GEMINI_API_KEY environment variable not set. Get your key from https://aistudio.google.com/app/apikey",
    )
}

async fn create_interaction(
    query: &str,
    thinking_level: &str,
    previous_interaction_id: Option<&str>,
    system_instruction: Option<&str>,
    max_tokens: u32,
    background: bool,
) -> Result<InteractionResponse> {
    let api_key = get_api_key()?;
    let client = reqwest::Client::new();

    let request = InteractionRequest {
        model: MODEL.to_string(),
        input: serde_json::Value::String(query.to_string()),
        store: true,
        system_instruction: system_instruction.map(|s| s.to_string()),
        previous_interaction_id: previous_interaction_id.map(|s| s.to_string()),
        background: if background { Some(true) } else { None },
        generation_config: GenerationConfig {
            thinking_level: thinking_level.to_string(),
            max_output_tokens: max_tokens,
        },
        tools: vec![
            Tool { r#type: "google_search".to_string() },
            Tool { r#type: "url_context".to_string() },
        ],
    };

    let start = Instant::now();
    let response = client
        .post(INTERACTIONS_ENDPOINT)
        .header("x-goog-api-key", &api_key)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send request")?;

    let elapsed = start.elapsed();
    eprintln!("Request completed in {:.2}s", elapsed.as_secs_f64());

    let data: InteractionResponse = response.json().await.context("Failed to parse response")?;
    Ok(data)
}

async fn get_interaction(interaction_id: &str) -> Result<InteractionResponse> {
    let api_key = get_api_key()?;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/{}", INTERACTIONS_ENDPOINT, interaction_id))
        .header("x-goog-api-key", &api_key)
        .send()
        .await
        .context("Failed to send request")?;

    let data: InteractionResponse = response.json().await.context("Failed to parse response")?;
    Ok(data)
}

async fn cancel_interaction(interaction_id: &str) -> Result<InteractionResponse> {
    let api_key = get_api_key()?;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/{}/cancel", INTERACTIONS_ENDPOINT, interaction_id))
        .header("x-goog-api-key", &api_key)
        .header("Content-Type", "application/json")
        .send()
        .await
        .context("Failed to send request")?;

    let data: InteractionResponse = response.json().await.context("Failed to parse response")?;
    Ok(data)
}

async fn resolve_redirect_url(url: String) -> String {
    if !url.contains("vertexaisearch.cloud.google.com/grounding-api-redirect") {
        return url;
    }

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(5))
        .build();

    if let Ok(client) = client {
        if let Ok(response) = client.head(&url).send().await {
            if let Some(location) = response.headers().get("location") {
                if let Ok(resolved) = location.to_str() {
                    return resolved.to_string();
                }
            }
        }
    }
    url
}

async fn format_response(response: &InteractionResponse, format: &OutputFormat) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(response).unwrap_or_default(),
        OutputFormat::Text => {
            let mut output = String::new();

            // Check for error
            if let Some(error) = &response.error {
                output.push_str(&format!(
                    "Error: {}\n",
                    error.message.as_deref().unwrap_or("Unknown error")
                ));
                return output;
            }

            // Extract text
            if let Some(outputs) = &response.outputs {
                for out in outputs {
                    if out.r#type == "text" {
                        if let Some(text) = &out.text {
                            output.push_str(text);
                        }
                    }
                }
            }

            // Collect sources
            let mut sources: Vec<(String, String)> = Vec::new(); // (title, url)
            if let Some(outputs) = &response.outputs {
                for out in outputs {
                    // From annotations
                    if let Some(anns) = &out.annotations {
                        for ann in anns {
                            if let Some(source) = &ann.source {
                                let entry = ("Source".to_string(), source.clone());
                                if !sources.iter().any(|(_, u)| u == source) {
                                    sources.push(entry);
                                }
                            }
                        }
                    }
                    // From search results
                    if out.r#type == "google_search_result" {
                        if let Some(results) = &out.result {
                            for result in results {
                                if let Some(url) = &result.url {
                                    let title = result.title.clone().unwrap_or_else(|| "Untitled".to_string());
                                    if !sources.iter().any(|(_, u)| u == url) {
                                        sources.push((title, url.clone()));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Resolve redirect URLs in parallel
            if !sources.is_empty() {
                let resolve_futures: Vec<_> = sources
                    .iter()
                    .map(|(title, url)| {
                        let title = title.clone();
                        let url = url.clone();
                        async move {
                            let resolved = resolve_redirect_url(url).await;
                            (title, resolved)
                        }
                    })
                    .collect();

                let resolved_sources = futures::future::join_all(resolve_futures).await;

                output.push_str("\n\nSources:\n");
                for (i, (title, url)) in resolved_sources.iter().enumerate() {
                    output.push_str(&format!("{}. [{}]({})\n", i + 1, title, url));
                }
            }

            // Add follow-up instructions
            output.push_str("\n---\n");
            if let Some(id) = &response.id {
                output.push_str(&format!("To follow up, use interaction_id: {}\n", id));
            }

            output
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle shorthand flags
    // search and ask are blocking, think runs in background
    let result = if let Some(query) = &cli.search {
        let system_instruction = format!(
            "Search for the query and return results in this exact format:\n\n---\nTITLE: [page title]\nURL: [full url]\nSNIPPET: [2-3 sentence excerpt]\n---\n\nReturn up to 10 results. No additional commentary or analysis."
        );
        create_interaction(
            query,
            "minimal",
            cli.interaction.as_deref(),
            Some(&system_instruction),
            4096,
            false, // blocking
        )
        .await?
    } else if let Some(query) = &cli.ask {
        create_interaction(
            query,
            "medium",
            cli.interaction.as_deref(),
            Some("Be concise and factual. Cite sources when using web information."),
            8192,
            false, // blocking
        )
        .await?
    } else if let Some(query) = &cli.think {
        create_interaction(
            query,
            "high",
            cli.interaction.as_deref(),
            Some("Think step by step. Be thorough and cite sources."),
            16384,
            false, // blocking (background only for agent interactions)
        )
        .await?
    } else if let Some(command) = &cli.command {
        match command {
            Commands::Search { query, max_results } => {
                let system_instruction = format!(
                    "Search for the query and return results in this exact format:\n\n---\nTITLE: [page title]\nURL: [full url]\nSNIPPET: [2-3 sentence excerpt]\n---\n\nReturn up to {} results. No additional commentary or analysis.",
                    max_results
                );
                create_interaction(
                    query,
                    "minimal",
                    None,
                    Some(&system_instruction),
                    4096,
                    false, // blocking
                )
                .await?
            }
            Commands::Ask { query, interaction } => {
                create_interaction(
                    query,
                    "medium",
                    interaction.as_deref(),
                    Some("Be concise and factual. Cite sources when using web information."),
                    8192,
                    false, // blocking
                )
                .await?
            }
            Commands::Think { query, interaction } => {
                create_interaction(
                    query,
                    "high",
                    interaction.as_deref(),
                    Some("Think step by step. Be thorough and cite sources."),
                    16384,
                    false, // blocking
                )
                .await?
            }
            Commands::FollowUp {
                query,
                interaction,
                thinking_level,
            } => {
                create_interaction(
                    query,
                    &thinking_level.to_string(),
                    Some(interaction),
                    None,
                    8192,
                    false, // blocking
                )
                .await?
            }
            Commands::Status { interaction_id } => get_interaction(interaction_id).await?,
            Commands::Cancel { interaction_id } => cancel_interaction(interaction_id).await?,
        }
    } else {
        eprintln!("No command or query provided. Use --help for usage.");
        std::process::exit(1);
    };

    println!("{}", format_response(&result, &cli.output).await);
    Ok(())
}
