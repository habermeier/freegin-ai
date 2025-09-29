//! Main entry point for the Freegin AI service.
//!
//! Responsibilities:
//! - Handle basic CLI flags (`--help`, `--version`).
//! - Initialize logging and tracing.
//! - Load application configuration.
//! - Establish shared infrastructure (database, provider clients).
//! - Start the Axum web server and expose HTTP routes.

use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::{self, Read},
    net::SocketAddr,
    path::PathBuf,
    process,
    sync::Arc,
};

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use freegin_ai::{
    config,
    credentials::CredentialStore,
    database,
    error::AppError,
    models::{
        AIRequest, RequestComplexity, RequestGuardrail, RequestHints, RequestQuality, RequestSpeed,
        ResponseFormat,
    },
    providers::{Provider, ProviderRouter},
    routes::{self, AppState},
    usage::UsageLogger,
};
use serde_json::json;

enum CliCommand {
    Run,
    Help,
    Version,
    Generate(GenerateOptions),
    AddService(Provider),
    RemoveService(Provider),
    ListServices,
}

#[derive(Default, Clone, Debug)]
struct GenerateOptions {
    prompt: Option<String>,
    prompt_file: Option<PathBuf>,
    output_file: Option<PathBuf>,
    context_files: Vec<PathBuf>,
    metadata: HashMap<String, String>,
    tags: Vec<String>,
    hints: RequestHints,
    response_format: Option<ResponseFormat>,
    provider_override: Option<String>,
    model: Option<String>,
    emit_metadata: bool,
}

#[tokio::main]
async fn main() {
    let command = match parse_cli_command() {
        Ok(cmd) => cmd,
        Err(err) => {
            eprintln!("freegin-ai: {err}");
            print_help();
            return;
        }
    };

    match command {
        CliCommand::Help => {
            print_help();
            return;
        }
        CliCommand::Version => {
            print_version();
            return;
        }
        _ => {}
    }

    // Load configuration (falls back to defaults if the secrets file is missing).

    // Load configuration (falls back to defaults if the secrets file is missing).
    let config = match config::AppConfig::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!(error = %err, "Failed to load configuration");
            eprintln!("freegin-ai: configuration error: {err}");
            process::exit(1);
        }
    };
    let db_pool = match database::init_db(&config.database.url).await {
        Ok(pool) => Arc::new(pool),
        Err(err) => {
            error!(error = %err, "Failed to connect to database");
            eprintln!("freegin-ai: database connection failed: {err}");
            process::exit(1);
        }
    };

    if let Err(err) = database::ensure_schema(db_pool.as_ref()).await {
        error!(error = %err, "Failed to ensure database schema");
        eprintln!("freegin-ai: database schema error: {err}");
        process::exit(1);
    }

    let credential_store = match CredentialStore::new(Arc::clone(&db_pool)).await {
        Ok(store) => store,
        Err(err) => {
            error!(error = %err, "Failed to initialise credential store");
            eprintln!("freegin-ai: credential store error: {err}");
            process::exit(1);
        }
    };

    match command {
        CliCommand::Generate(options) => {
            let usage_logger = UsageLogger::new(Arc::clone(&db_pool));
            if let Err(err) =
                handle_generate(options, &config, &credential_store, Some(usage_logger)).await
            {
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
            return;
        }
        CliCommand::AddService(provider) => {
            if let Err(err) = handle_add_service(provider, &credential_store).await {
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
            return;
        }
        CliCommand::RemoveService(provider) => {
            if let Err(err) = handle_remove_service(provider, &credential_store).await {
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
            return;
        }
        CliCommand::ListServices => {
            if let Err(err) = handle_list_services(&credential_store, &config).await {
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
            return;
        }
        CliCommand::Run => {}
        CliCommand::Help | CliCommand::Version => unreachable!(),
    }

    // Initialize tracing based on RUST_LOG or the fallback filter.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "freegin_ai=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Freegin AI server...");

    let usage_logger = Some(UsageLogger::new(Arc::clone(&db_pool)));

    let provider_router =
        match ProviderRouter::from_config(&config, &credential_store, usage_logger).await {
            Ok(router) => Arc::new(router),
            Err(err) => {
                error!(error = %err, "Failed to initialise provider router");
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
        };

    // Initialize the database connection pool once migrations exist.
    // info!("Database connection pool established");

    // Build the HTTP router.
    let state = AppState::new(provider_router);
    let app = routes::api_router(state);

    let addr_str = format!("{}:{}", config.server.host, config.server.port);
    let addr: SocketAddr = addr_str.parse().expect("Invalid server address format");

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(err) => {
            error!(error = %err, "Failed to bind TCP listener");
            eprintln!("freegin-ai: cannot bind to {addr} ({err})");
            process::exit(1);
        }
    };

    info!("Server listening on {addr}");

    axum::serve(listener, app).await.expect("Server crashed");
}

fn parse_cli_command() -> Result<CliCommand, String> {
    let mut args: Vec<String> = env::args().collect();
    if args.is_empty() {
        return Ok(CliCommand::Run);
    }

    let _program = args.remove(0);
    if args.is_empty() {
        return Ok(CliCommand::Run);
    }

    let mut iter = args.into_iter();
    let first = iter.next().unwrap();
    match first.as_str() {
        "-h" | "--help" | "help" => Ok(CliCommand::Help),
        "-V" | "--version" | "version" => Ok(CliCommand::Version),
        "generate" => {
            let remaining: Vec<String> = iter.collect();
            let options = parse_generate_options(&remaining)?;
            Ok(CliCommand::Generate(options))
        }
        "add-service" => {
            let name = iter
                .next()
                .ok_or_else(|| "add-service requires a provider name".to_string())?;
            let provider = parse_provider(&name)?;
            Ok(CliCommand::AddService(provider))
        }
        "remove-service" => {
            let name = iter
                .next()
                .ok_or_else(|| "remove-service requires a provider name".to_string())?;
            let provider = parse_provider(&name)?;
            Ok(CliCommand::RemoveService(provider))
        }
        "list-services" => Ok(CliCommand::ListServices),
        other if other.starts_with('-') => Err(format!("Unknown option '{other}'")),
        _ => Ok(CliCommand::Run),
    }
}

fn parse_generate_options(args: &[String]) -> Result<GenerateOptions, String> {
    let mut options = GenerateOptions::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--prompt" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--prompt requires an argument".to_string())?;
                options.prompt = Some(value.clone());
            }
            "--prompt-file" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--prompt-file requires a path".to_string())?;
                options.prompt_file = Some(PathBuf::from(value));
            }
            "--output-file" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--output-file requires a path".to_string())?;
                options.output_file = Some(PathBuf::from(value));
            }
            "--context-file" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--context-file requires a path".to_string())?;
                options.context_files.push(PathBuf::from(value));
            }
            "--metadata" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--metadata requires key=value".to_string())?;
                let (key, val) = value
                    .split_once('=')
                    .ok_or_else(|| format!("Metadata '{value}' must be in key=value form"))?;
                drop(
                    options
                        .metadata
                        .insert(key.trim().to_string(), val.trim().to_string()),
                );
            }
            "--tag" | "--tags" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--tag requires a value".to_string())?;
                options.tags.push(value.trim().to_string());
            }
            "--complexity" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--complexity requires low|medium|high".to_string())?;
                options.hints.complexity = Some(parse_complexity(value)?);
            }
            "--quality" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--quality requires standard|balanced|premium".to_string())?;
                options.hints.quality = Some(parse_quality(value)?);
            }
            "--speed" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--speed requires fast|normal".to_string())?;
                options.hints.speed = Some(parse_speed(value)?);
            }
            "--guardrail" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--guardrail requires strict|lenient".to_string())?;
                options.hints.guardrail = Some(parse_guardrail(value)?);
            }
            "--format" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--format requires text|markdown|json".to_string())?;
                let format = parse_response_format(value)?;
                options.response_format = Some(format);
                options.hints.response_format = Some(format);
            }
            "--provider" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--provider requires a provider name".to_string())?;
                let provider = parse_provider(value)?;
                let alias = provider.as_str().to_string();
                options.provider_override = Some(alias.clone());
                options.hints.provider = Some(alias.clone());
                options.tags.push(format!("provider:{alias}"));
            }
            "--model" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--model requires a value".to_string())?;
                options.model = Some(value.clone());
            }
            "--emit-metadata" => {
                options.emit_metadata = true;
            }
            other => {
                return Err(format!("Unknown generate option '{other}'"));
            }
        }
    }

    if options.prompt.is_some() && options.prompt_file.is_some() {
        return Err("Use either --prompt or --prompt-file, not both".into());
    }

    Ok(options)
}

fn parse_provider(name: &str) -> Result<Provider, String> {
    Provider::from_alias(name).ok_or_else(|| format!("Unknown provider '{name}'"))
}

fn parse_complexity(value: &str) -> Result<RequestComplexity, String> {
    match value.to_lowercase().as_str() {
        "low" => Ok(RequestComplexity::Low),
        "medium" => Ok(RequestComplexity::Medium),
        "high" => Ok(RequestComplexity::High),
        _ => Err(format!("Invalid complexity '{value}'")),
    }
}

fn parse_quality(value: &str) -> Result<RequestQuality, String> {
    match value.to_lowercase().as_str() {
        "standard" => Ok(RequestQuality::Standard),
        "balanced" => Ok(RequestQuality::Balanced),
        "premium" => Ok(RequestQuality::Premium),
        _ => Err(format!("Invalid quality '{value}'")),
    }
}

fn parse_speed(value: &str) -> Result<RequestSpeed, String> {
    match value.to_lowercase().as_str() {
        "fast" => Ok(RequestSpeed::Fast),
        "normal" => Ok(RequestSpeed::Normal),
        _ => Err(format!("Invalid speed '{value}'")),
    }
}

fn parse_guardrail(value: &str) -> Result<RequestGuardrail, String> {
    match value.to_lowercase().as_str() {
        "strict" => Ok(RequestGuardrail::Strict),
        "lenient" => Ok(RequestGuardrail::Lenient),
        _ => Err(format!("Invalid guardrail '{value}'")),
    }
}

fn parse_response_format(value: &str) -> Result<ResponseFormat, String> {
    match value.to_lowercase().as_str() {
        "text" => Ok(ResponseFormat::Text),
        "markdown" => Ok(ResponseFormat::Markdown),
        "json" => Ok(ResponseFormat::Json),
        _ => Err(format!("Invalid format '{value}'")),
    }
}

fn print_help() {
    println!(
        "{name} {version}\n\nUsage:\n  {name} [OPTIONS]\n  {name} add-service <provider>\n  {name} remove-service <provider>\n  {name} list-services\n\nOptions:\n  -h, --help       Show this help message and exit\n  -V, --version    Print version information\n\nProviders:\n  huggingface      Hugging Face Inference API",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
    );
}

fn print_version() {
    println!(
        "{name} {version}",
        name = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION")
    );
}

async fn handle_add_service(provider: Provider, store: &CredentialStore) -> Result<(), AppError> {
    match provider {
        Provider::HuggingFace => {
            println!("Visit https://huggingface.co/settings/tokens to create a token.");
            let token = rpassword::prompt_password("Enter Hugging Face token (input hidden): ")
                .map_err(|err| AppError::ConfigError(format!("Failed to read token: {err}")))?;
            let token = token.trim().to_string();
            if token.is_empty() {
                return Err(AppError::ConfigError("Token cannot be empty".into()));
            }
            println!("Captured token ({} characters).", token.chars().count());
            store.set_token(Provider::HuggingFace, &token).await?;
            println!("Hugging Face token saved locally. It is stored encrypted on disk.");
            Ok(())
        }
        other => Err(AppError::ConfigError(format!(
            "Adding credentials for {other} is not yet supported"
        ))),
    }
}

async fn handle_remove_service(
    provider: Provider,
    store: &CredentialStore,
) -> Result<(), AppError> {
    match provider {
        Provider::HuggingFace => {
            let removed = store.remove_token(provider).await?;
            if removed {
                println!("Removed Hugging Face token from local store.");
            } else {
                println!("No stored token found for Hugging Face.");
            }
            Ok(())
        }
        other => Err(AppError::ConfigError(format!(
            "Removing credentials for {other} is not yet supported"
        ))),
    }
}

async fn handle_list_services(
    store: &CredentialStore,
    config: &config::AppConfig,
) -> Result<(), AppError> {
    let stored: HashSet<_> = store.stored_providers().await?.into_iter().collect();

    println!("Provider       Configured   Stored");
    println!("--------------------------------");
    for provider in [
        Provider::HuggingFace,
        Provider::Google,
        Provider::OpenAI,
        Provider::Anthropic,
        Provider::Cohere,
    ] {
        let configured = match provider {
            Provider::HuggingFace => config
                .providers
                .hugging_face
                .as_ref()
                .map(|cfg| !cfg.api_key.trim().is_empty())
                .unwrap_or(false),
            Provider::Google => config
                .providers
                .google
                .as_ref()
                .map(|cfg| !cfg.api_key.trim().is_empty())
                .unwrap_or(false),
            Provider::OpenAI => config
                .providers
                .openai
                .as_ref()
                .map(|cfg| !cfg.api_key.trim().is_empty())
                .unwrap_or(false),
            Provider::Anthropic => config
                .providers
                .anthropic
                .as_ref()
                .map(|cfg| !cfg.api_key.trim().is_empty())
                .unwrap_or(false),
            Provider::Cohere => config
                .providers
                .cohere
                .as_ref()
                .map(|cfg| !cfg.api_key.trim().is_empty())
                .unwrap_or(false),
        };

        println!(
            "{:<13} {:<11} {:<6}",
            provider.as_str(),
            if configured { "yes" } else { "no" },
            if stored.contains(&provider) {
                "yes"
            } else {
                "no"
            }
        );
    }

    Ok(())
}

async fn handle_generate(
    options: GenerateOptions,
    config: &config::AppConfig,
    credential_store: &CredentialStore,
    usage_logger: Option<UsageLogger>,
) -> Result<(), AppError> {
    let mut prompt = if let Some(p) = options.prompt.clone() {
        p
    } else if let Some(path) = &options.prompt_file {
        fs::read_to_string(path).map_err(|err| {
            AppError::ConfigError(format!(
                "Failed to read prompt file {}: {err}",
                path.display()
            ))
        })?
    } else {
        let mut buffer = String::new();
        let _ = io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|err| AppError::ConfigError(format!("Failed to read stdin: {err}")))?;
        buffer
    };

    if prompt.trim().is_empty() {
        return Err(AppError::ConfigError("Prompt cannot be empty".into()));
    }

    let mut context_blocks = Vec::new();
    for path in &options.context_files {
        let content = fs::read_to_string(path).map_err(|err| {
            AppError::ConfigError(format!(
                "Failed to read context file {}: {err}",
                path.display()
            ))
        })?;
        context_blocks.push(content);
    }

    if !context_blocks.is_empty() {
        let combined_context = context_blocks
            .iter()
            .enumerate()
            .map(|(idx, ctx)| format!("Context {}:\n{}", idx + 1, ctx))
            .collect::<Vec<_>>()
            .join("\n\n");
        prompt = format!("{}\n\n{}", combined_context, prompt);
    }

    let mut hints = options.hints.clone();
    if let Some(provider_override) = options.provider_override.clone() {
        hints.provider = Some(provider_override.clone());
    }

    let mut tags = options.tags.clone();
    if let Some(provider_hint) = hints.provider.as_ref() {
        tags.push(format!("provider:{}", provider_hint));
    }

    let mut metadata = options.metadata.clone();
    drop(metadata.insert("cli".into(), "true".into()));

    let request = AIRequest {
        model: options.model.clone().unwrap_or_default(),
        prompt,
        tags,
        context: context_blocks,
        metadata,
        hints,
    };

    let router = ProviderRouter::from_config(config, credential_store, usage_logger).await?;
    let response = router.generate(&request).await?;

    let output_string = match options.response_format.unwrap_or(ResponseFormat::Text) {
        ResponseFormat::Json => serde_json::to_string_pretty(&json!({
            "provider": response.provider.as_str(),
            "content": response.content,
        }))
        .map_err(|err| AppError::ApiError(err.to_string()))?,
        ResponseFormat::Markdown | ResponseFormat::Text => response.content.clone(),
    };

    if let Some(path) = &options.output_file {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                AppError::ConfigError(format!(
                    "Failed to create output directory {}: {err}",
                    parent.display()
                ))
            })?;
        }
        fs::write(path, output_string).map_err(|err| {
            AppError::ConfigError(format!(
                "Failed to write output file {}: {err}",
                path.display()
            ))
        })?;
    } else {
        println!("{}", output_string);
    }

    if options.emit_metadata && !matches!(options.response_format, Some(ResponseFormat::Json)) {
        let metadata_json = serde_json::to_string(&json!({
            "provider": response.provider.as_str(),
        }))
        .map_err(|err| AppError::ApiError(err.to_string()))?;
        println!("{}", metadata_json);
    } else {
        eprintln!("provider: {}", response.provider.as_str());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_generate_options_supports_hints() {
        let args = vec![
            "--prompt".to_string(),
            "Hello".to_string(),
            "--complexity".to_string(),
            "high".to_string(),
            "--provider".to_string(),
            "huggingface".to_string(),
        ];

        let opts = parse_generate_options(&args).expect("parse");
        assert_eq!(opts.prompt.as_deref(), Some("Hello"));
        assert_eq!(opts.hints.complexity, Some(RequestComplexity::High));
        assert_eq!(opts.provider_override.as_deref(), Some("huggingface"));
    }
}
