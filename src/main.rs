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
    catalog::CatalogStore,
    config,
    credentials::CredentialStore,
    database::{self, DbPool},
    error::AppError,
    health::HealthTracker,
    models::{
        AIRequest, RequestComplexity, RequestGuardrail, RequestHints, RequestQuality, RequestSpeed,
        ResponseFormat, Workload,
    },
    providers::{Provider, ProviderRouter},
    routes::{self, AppState},
    usage::UsageLogger,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

enum CliCommand {
    Run,
    Help,
    Version,
    Init,
    Generate(GenerateOptions),
    RefreshModels(RefreshOptions),
    ListModels(ListModelsOptions),
    AdoptModel(AdoptModelOptions),
    AddService(Provider),
    RemoveService(Provider),
    ListServices,
    Status(StatusOptions),
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
    verbose: bool,
}

#[derive(Default, Clone, Debug)]
struct RefreshOptions {
    provider: Option<Provider>,
    workload: Option<Workload>,
    dry_run: bool,
}

#[derive(Default, Clone, Debug)]
struct ListModelsOptions {
    provider: Option<Provider>,
    workload: Option<Workload>,
    include_suggestions: bool,
}

#[derive(Clone, Debug)]
struct AdoptModelOptions {
    provider: Provider,
    workload: Workload,
    model: String,
    priority: i64,
}

#[derive(Default, Clone, Debug)]
struct StatusOptions {
    provider: Option<Provider>,
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
        CliCommand::Init => {
            if let Err(err) = handle_init().await {
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
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

    let catalog = CatalogStore::new(Arc::clone(&db_pool));

    // Seed default models if needed
    if let Err(err) = catalog.seed_defaults().await {
        error!(error = %err, "Failed to seed default models");
        // Non-fatal; continue anyway
    }

    match command {
        CliCommand::Generate(options) => {
            let usage_logger = UsageLogger::new(Arc::clone(&db_pool));
            if let Err(err) = handle_generate(
                options,
                &config,
                &credential_store,
                &catalog,
                Some(usage_logger),
            )
            .await
            {
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
            return;
        }
        CliCommand::RefreshModels(options) => {
            let usage_logger = UsageLogger::new(Arc::clone(&db_pool));
            if let Err(err) =
                handle_refresh_models(options, &config, &credential_store, &catalog, usage_logger)
                    .await
            {
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
            return;
        }
        CliCommand::ListModels(options) => {
            if let Err(err) = handle_list_models(&catalog, options).await {
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
            return;
        }
        CliCommand::AdoptModel(options) => {
            if let Err(err) = handle_adopt_model(&catalog, options).await {
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
        CliCommand::Status(options) => {
            if let Err(err) = handle_status(&catalog, Arc::clone(&db_pool), options).await {
                eprintln!("freegin-ai: {err}");
                process::exit(1);
            }
            return;
        }
        CliCommand::Run => {}
        CliCommand::Help | CliCommand::Version | CliCommand::Init => unreachable!(),
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

    let provider_router = match ProviderRouter::from_config(
        &config,
        &credential_store,
        usage_logger,
        Some(catalog.clone()),
    )
    .await
    {
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
        "--init" | "init" => Ok(CliCommand::Init),
        "generate" => {
            let remaining: Vec<String> = iter.collect();
            let options = parse_generate_options(&remaining)?;
            Ok(CliCommand::Generate(options))
        }
        "refresh-models" => {
            let remaining: Vec<String> = iter.collect();
            let options = parse_refresh_options(&remaining)?;
            Ok(CliCommand::RefreshModels(options))
        }
        "list-models" => {
            let remaining: Vec<String> = iter.collect();
            let options = parse_list_models_options(&remaining)?;
            Ok(CliCommand::ListModels(options))
        }
        "adopt-model" => {
            let remaining: Vec<String> = iter.collect();
            let options = parse_adopt_model_options(&remaining)?;
            Ok(CliCommand::AdoptModel(options))
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
        "status" => {
            let remaining: Vec<String> = iter.collect();
            let options = parse_status_options(&remaining)?;
            Ok(CliCommand::Status(options))
        }
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
            "--verbose" | "-v" => {
                options.verbose = true;
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

fn parse_refresh_options(args: &[String]) -> Result<RefreshOptions, String> {
    let mut options = RefreshOptions::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--provider requires a provider name".to_string())?;
                options.provider = Some(parse_provider(value)?);
            }
            "--workload" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--workload requires a value".to_string())?;
                options.workload = Some(parse_workload(value)?);
            }
            "--dry-run" => options.dry_run = true,
            other => return Err(format!("Unknown refresh-models option '{other}'")),
        }
    }
    Ok(options)
}

fn parse_list_models_options(args: &[String]) -> Result<ListModelsOptions, String> {
    let mut options = ListModelsOptions::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--provider requires a provider name".to_string())?;
                options.provider = Some(parse_provider(value)?);
            }
            "--workload" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--workload requires a value".to_string())?;
                options.workload = Some(parse_workload(value)?);
            }
            "--include-suggestions" => options.include_suggestions = true,
            other => return Err(format!("Unknown list-models option '{other}'")),
        }
    }
    Ok(options)
}

fn parse_adopt_model_options(args: &[String]) -> Result<AdoptModelOptions, String> {
    let mut iter = args.iter();
    let provider = iter
        .next()
        .ok_or_else(|| "adopt-model requires a provider".to_string())?;
    let model = iter
        .next()
        .ok_or_else(|| "adopt-model requires a model identifier".to_string())?;

    let provider = parse_provider(provider)?;
    let mut workload = Workload::Chat;
    let mut priority = 100;

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--workload" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--workload requires a value".to_string())?;
                workload = parse_workload(value)?;
            }
            "--priority" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--priority requires an integer".to_string())?;
                priority = value
                    .parse()
                    .map_err(|_| "--priority must be an integer".to_string())?;
            }
            other => return Err(format!("Unknown adopt-model option '{other}'")),
        }
    }

    Ok(AdoptModelOptions {
        provider,
        workload,
        model: model.clone(),
        priority,
    })
}

fn parse_status_options(args: &[String]) -> Result<StatusOptions, String> {
    let mut options = StatusOptions::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--provider" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--provider requires a provider name".to_string())?;
                options.provider = Some(parse_provider(value)?);
            }
            other => return Err(format!("Unknown status option '{other}'")),
        }
    }
    Ok(options)
}

fn parse_provider(name: &str) -> Result<Provider, String> {
    Provider::from_alias(name).ok_or_else(|| format!("Unknown provider '{name}'"))
}

fn parse_workload(value: &str) -> Result<Workload, String> {
    match value.to_lowercase().as_str() {
        "chat" => Ok(Workload::Chat),
        "summarization" | "summary" => Ok(Workload::Summarization),
        "code" => Ok(Workload::Code),
        "extraction" => Ok(Workload::Extraction),
        "creative" => Ok(Workload::Creative),
        "classification" => Ok(Workload::Classification),
        other => Err(format!(
            "Unknown workload '{other}'. Expected one of {:?}",
            Workload::variants()
        )),
    }
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
        "{name} {version}

Usage:
  {name} [OPTIONS]
  {name} --init
  {name} generate [GENERATE_OPTIONS]
  {name} add-service <provider>
  {name} remove-service <provider>
  {name} list-services
  {name} list-models [OPTIONS]
  {name} adopt-model --provider <provider> --workload <workload> --model <model> [OPTIONS]
  {name} refresh-models --provider <provider> --workload <workload>
  {name} status [--provider <provider>]

Commands:
  --init             Interactive setup wizard for provider credentials
  generate           Run a single inference request
  add-service        Add encrypted provider credentials
  remove-service     Remove provider credentials
  list-services      Show configured providers
  list-models        List active models and suggestions
  adopt-model        Add a model to the active roster
  refresh-models     Discover new models using LLM
  status             Show provider health and model status

Generate Options:
  --prompt <text>           Inline prompt text
  --prompt-file <file>      Read prompt from file
  --output-file <file>      Write response to file
  --context-file <file>     Add context (repeatable)
  --complexity <level>      low|medium|high
  --quality <level>         standard|balanced|premium
  --speed <level>           fast|normal
  --provider <name>         Force specific provider
  --model <name>            Override model selection
  --format <format>         text|markdown|json
  -v, --verbose             Show provider and model metadata
  --emit-metadata           Output metadata as JSON

Options:
  -h, --help       Show this help message and exit
  -V, --version    Print version information

Providers:
  groq             Groq (ultra-fast, 14.4K requests/day free) - https://console.groq.com/keys
  deepseek         DeepSeek (pay-per-use, very low cost) - https://platform.deepseek.com/api_keys
  together         Together AI ($5 deposit, then free models) - https://api.together.xyz/settings/api-keys
  huggingface      Hugging Face Inference API - https://huggingface.co/settings/tokens
  google           Google Gemini - https://makersuite.google.com/app/apikey
  openai           OpenAI - https://platform.openai.com/api-keys
  anthropic        Anthropic Claude - https://console.anthropic.com/
  cohere           Cohere - https://dashboard.cohere.com/api-keys",
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

async fn handle_init() -> Result<(), AppError> {
    println!("=== freegin-ai Provider Setup ===\n");
    println!("This wizard will help you configure AI providers with encrypted credential storage.");
    println!("You can skip any provider by pressing Enter without typing a key.\n");

    // Initialize database and credential store
    let config = config::AppConfig::load()
        .map_err(|e| AppError::ConfigError(format!("Failed to load config: {}", e)))?;
    let db_pool = Arc::new(
        database::init_db(&config.database.url)
            .await
            .map_err(|e| AppError::DatabaseError(e))?,
    );
    database::ensure_schema(db_pool.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let store = CredentialStore::new(Arc::clone(&db_pool)).await?;

    // Get already configured providers
    let stored = store.stored_providers().await?;
    let stored_set: HashSet<_> = stored.into_iter().collect();

    // Define providers with their details
    let providers = vec![
        (
            Provider::Groq,
            "Groq",
            "Ultra-fast inference (14,400 requests/day free)",
            "https://console.groq.com/keys",
        ),
        (
            Provider::DeepSeek,
            "DeepSeek",
            "Unlimited free tier with powerful reasoning",
            "https://platform.deepseek.com/api_keys",
        ),
        (
            Provider::Together,
            "Together AI",
            "Requires $5 deposit, then free models available (Llama 3.3 70B)",
            "https://api.together.xyz/settings/api-keys",
        ),
        (
            Provider::Google,
            "Google Gemini",
            "60 requests/min, 1M tokens/min free",
            "https://makersuite.google.com/app/apikey",
        ),
        (
            Provider::HuggingFace,
            "Hugging Face",
            "Rate-limited serverless API",
            "https://huggingface.co/settings/tokens",
        ),
        (
            Provider::OpenAI,
            "OpenAI",
            "Pay-as-you-go (no free tier)",
            "https://platform.openai.com/api-keys",
        ),
        (
            Provider::Anthropic,
            "Anthropic Claude",
            "Pay-as-you-go with limited free credits",
            "https://console.anthropic.com/",
        ),
        (
            Provider::Cohere,
            "Cohere",
            "Free tier for experimentation",
            "https://dashboard.cohere.com/api-keys",
        ),
    ];

    let mut configured_count = 0;

    for (provider, name, description, url) in providers {
        if stored_set.contains(&provider) {
            println!("✓ {} - Already configured (stored)", name);
            configured_count += 1;
            continue;
        }

        println!("\n--- {} ---", name);
        println!("Description: {}", description);
        println!("Sign up: {}", url);

        let prompt = format!("Enter {} API key (or press Enter to skip): ", name);
        let token = rpassword::prompt_password(&prompt)
            .map_err(|err| AppError::ConfigError(format!("Failed to read input: {err}")))?;
        let token = token.trim().to_string();

        if token.is_empty() {
            println!("  Skipped");
            continue;
        }

        println!("  Saving encrypted API key ({} characters)...", token.chars().count());
        store.set_token(provider, &token).await?;
        println!("  ✓ {} configured successfully!", name);
        configured_count += 1;
    }

    println!("\n=== Setup Complete ===");
    println!("Configured {} provider(s)", configured_count);
    println!("\nNext steps:");
    println!("  • Run 'freegin-ai list-services' to verify configuration");
    println!("  • Run 'freegin-ai status' to check provider health");
    println!("  • Run 'freegin-ai generate --prompt \"Hello!\"' to test");
    println!("\nYou can re-run 'freegin-ai --init' anytime to add more providers.");

    Ok(())
}

async fn handle_add_service(provider: Provider, store: &CredentialStore) -> Result<(), AppError> {
    let (url, prompt) = match provider {
        Provider::Groq => (
            "https://console.groq.com/keys",
            "Enter Groq API key (input hidden): ",
        ),
        Provider::DeepSeek => (
            "https://platform.deepseek.com/api_keys",
            "Enter DeepSeek API key (input hidden): ",
        ),
        Provider::Together => (
            "https://api.together.xyz/settings/api-keys",
            "Enter Together AI API key (input hidden): ",
        ),
        Provider::HuggingFace => (
            "https://huggingface.co/settings/tokens",
            "Enter Hugging Face token (input hidden): ",
        ),
        Provider::Google => (
            "https://makersuite.google.com/app/apikey",
            "Enter Google API key (input hidden): ",
        ),
        Provider::OpenAI => (
            "https://platform.openai.com/api-keys",
            "Enter OpenAI API key (input hidden): ",
        ),
        Provider::Anthropic => (
            "https://console.anthropic.com/",
            "Enter Anthropic API key (input hidden): ",
        ),
        Provider::Cohere => (
            "https://dashboard.cohere.com/api-keys",
            "Enter Cohere API key (input hidden): ",
        ),
    };

    println!("Visit {} to create an API key.", url);
    let token = rpassword::prompt_password(prompt)
        .map_err(|err| AppError::ConfigError(format!("Failed to read token: {err}")))?;
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(AppError::ConfigError("API key cannot be empty".into()));
    }
    println!("Captured API key ({} characters).", token.chars().count());
    store.set_token(provider, &token).await?;
    println!(
        "{} API key saved locally. It is stored encrypted on disk.",
        provider.as_str()
    );
    Ok(())
}

async fn handle_remove_service(
    provider: Provider,
    store: &CredentialStore,
) -> Result<(), AppError> {
    let removed = store.remove_token(provider).await?;
    if removed {
        println!("Removed {} API key from local store.", provider.as_str());
    } else {
        println!("No stored API key found for {}.", provider.as_str());
    }
    Ok(())
}

async fn handle_list_services(
    store: &CredentialStore,
    config: &config::AppConfig,
) -> Result<(), AppError> {
    let stored: HashSet<_> = store.stored_providers().await?.into_iter().collect();

    println!("Provider       Configured");
    println!("---------------------------");
    for provider in [
        Provider::Groq,
        Provider::DeepSeek,
        Provider::Together,
        Provider::HuggingFace,
        Provider::Google,
        Provider::OpenAI,
        Provider::Anthropic,
        Provider::Cohere,
    ] {
        let has_config_key = match provider {
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
            Provider::Groq => config
                .providers
                .groq
                .as_ref()
                .map(|cfg| !cfg.api_key.trim().is_empty())
                .unwrap_or(false),
            Provider::DeepSeek => config
                .providers
                .deepseek
                .as_ref()
                .map(|cfg| !cfg.api_key.trim().is_empty())
                .unwrap_or(false),
            Provider::Together => config
                .providers
                .together
                .as_ref()
                .map(|cfg| !cfg.api_key.trim().is_empty())
                .unwrap_or(false),
        };

        let status = if stored.contains(&provider) {
            "stored"
        } else if has_config_key {
            "environment"
        } else {
            "none"
        };

        println!("{:<13} {}", provider.as_str(), status);
    }

    Ok(())
}

async fn handle_generate(
    options: GenerateOptions,
    config: &config::AppConfig,
    credential_store: &CredentialStore,
    catalog: &CatalogStore,
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

    let router = ProviderRouter::from_config(
        config,
        credential_store,
        usage_logger,
        Some(catalog.clone()),
    )
    .await?;
    let response = router.generate(&request).await?;

    let output_string = match options.response_format.unwrap_or(ResponseFormat::Text) {
        ResponseFormat::Json => serde_json::to_string_pretty(&json!({
            "provider": response.provider.as_str(),
            "content": response.content,
        }))
        .map_err(|err| AppError::ApiError(err.to_string()))?,
        ResponseFormat::Markdown | ResponseFormat::Text => response.content.clone(),
    };

    // Output metadata header for verbose mode (before content)
    if options.verbose && !matches!(options.response_format, Some(ResponseFormat::Json)) {
        eprintln!("=== Metadata ===");
        eprintln!("Provider: {}", response.provider.as_str());
        eprintln!("\n=== Response ===");
    }

    // Output main content
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

    // Output JSON metadata for --emit-metadata mode (after content)
    if options.emit_metadata && !matches!(options.response_format, Some(ResponseFormat::Json)) {
        let metadata_json = serde_json::to_string(&json!({
            "provider": response.provider.as_str(),
        }))
        .map_err(|err| AppError::ApiError(err.to_string()))?;
        println!("{}", metadata_json);
    }
    // By default: no metadata output (clean response only)

    Ok(())
}

async fn handle_list_models(
    catalog: &CatalogStore,
    options: ListModelsOptions,
) -> Result<(), AppError> {
    let models = catalog
        .list_models(options.provider, options.workload)
        .await?;

    if options.include_suggestions {
        let suggestions = catalog
            .list_suggestions(options.provider, options.workload)
            .await?;

        // Group by provider/workload
        let mut groups: HashMap<(String, String), (Vec<_>, Vec<_>)> = HashMap::new();

        for model in models {
            let key = (
                model.provider.as_str().to_string(),
                format!("{:?}", model.workload),
            );
            groups
                .entry(key)
                .or_insert_with(|| (Vec::new(), Vec::new()))
                .0
                .push(model);
        }

        for suggestion in suggestions {
            let key = (
                suggestion.provider.as_str().to_string(),
                format!("{:?}", suggestion.workload),
            );
            groups
                .entry(key)
                .or_insert_with(|| (Vec::new(), Vec::new()))
                .1
                .push(suggestion);
        }

        for ((provider, workload), (models, suggestions)) in groups {
            println!("\nProvider: {} | Workload: {}", provider, workload);

            if !models.is_empty() {
                println!("  Active:");
                for m in models {
                    let rationale = m.rationale.as_deref().unwrap_or("");
                    println!("    {:3}  {} — {}", m.priority, m.model, rationale);
                }
            }

            if !suggestions.is_empty() {
                println!("  Suggestions:");
                for s in suggestions {
                    let rationale = s.rationale.as_deref().unwrap_or("");
                    println!("    {}  {} — {}", s.status, s.model, rationale);
                }
            }
        }
    } else {
        // Simple list: active models only
        let mut current_group = ("".to_string(), "".to_string());

        for model in models {
            let key = (
                model.provider.as_str().to_string(),
                format!("{:?}", model.workload),
            );
            if key != current_group {
                println!("\nProvider: {} | Workload: {}", key.0, key.1);
                current_group = key;
            }
            let rationale = model.rationale.as_deref().unwrap_or("");
            println!("  {:3}  {} — {}", model.priority, model.model, rationale);
        }
    }

    Ok(())
}

async fn handle_adopt_model(
    catalog: &CatalogStore,
    options: AdoptModelOptions,
) -> Result<(), AppError> {
    catalog
        .adopt_model(
            options.provider,
            options.workload,
            options.model.clone(),
            None, // rationale can be added later
            None, // metadata can be added later
            options.priority,
        )
        .await?;

    println!(
        "Adopted model '{}' for provider '{}' and workload '{:?}' with priority {}",
        options.model,
        options.provider.as_str(),
        options.workload,
        options.priority
    );

    // Show the new roster ordering
    let models = catalog
        .active_models(options.provider, Some(options.workload))
        .await?;
    if !models.is_empty() {
        println!(
            "\nActive models for {} / {:?}:",
            options.provider.as_str(),
            options.workload
        );
        for m in models {
            let rationale = m.rationale.as_deref().unwrap_or("");
            println!("  {:3}  {} — {}", m.priority, m.model, rationale);
        }
    }

    Ok(())
}

async fn handle_refresh_models(
    options: RefreshOptions,
    config: &config::AppConfig,
    credential_store: &CredentialStore,
    catalog: &CatalogStore,
    usage_logger: UsageLogger,
) -> Result<(), AppError> {
    let provider = options.provider.unwrap_or(Provider::HuggingFace);
    let workload = options.workload.unwrap_or(Workload::Chat);

    println!(
        "Refreshing model suggestions for {} / {:?}...",
        provider.as_str(),
        workload
    );

    // 1. Build context with current roster
    let current_models = catalog.active_models(provider, Some(workload)).await?;

    // 2. Get usage statistics
    let stats = catalog
        .usage_stats(provider, Some(workload))
        .await
        .unwrap_or_else(|_| {
            // Default stats if no usage data exists
            freegin_ai::catalog::UsageStats {
                total_calls: 0,
                successful_calls: 0,
                success_rate: 0.0,
                avg_latency_ms: 0.0,
                max_latency_ms: 0,
            }
        });

    // 3. Build prompt for LLM
    let context = RefreshContext {
        provider: provider.as_str().to_string(),
        workload: format!("{:?}", workload),
        current_models: current_models
            .iter()
            .map(|m| ModelInfo {
                model: m.model.clone(),
                priority: m.priority,
                rationale: m.rationale.clone(),
            })
            .collect(),
        usage_stats: StatsInfo {
            total_calls: stats.total_calls,
            success_rate: stats.success_rate,
            avg_latency_ms: stats.avg_latency_ms,
        },
    };

    let context_json = serde_json::to_string_pretty(&context)
        .map_err(|e| AppError::ApiError(format!("Failed to serialize context: {}", e)))?;

    let prompt = format!(
        r#"You are a model selection assistant. Analyze the following context and suggest 3-5 candidate models for the given provider and workload.

Context:
{}

Requirements:
- Respond with ONLY valid JSON matching this schema:
{{
  "suggestions": [
    {{
      "model": "provider/model-name",
      "workload": "Chat|Code|Summarization|Extraction|Creative|Classification",
      "rationale": "Brief explanation (max 40 words)",
      "production_ready": true|false,
      "notes": "Optional additional notes",
      "metadata": {{"est_cost_per_1k_tokens": 0.15}}
    }}
  ]
}}

- Consider current models and usage statistics
- Prioritize models with good cost/performance balance
- Include newer models that might outperform current roster
- Ensure model names are valid for the provider

Output only the JSON, no other text."#,
        context_json
    );

    // 4. Call router to get suggestions (using the most reliable model available)
    let request = AIRequest {
        model: String::new(), // Let router pick
        prompt,
        tags: vec!["model-refresh".to_string()],
        context: vec![],
        metadata: HashMap::new(),
        hints: RequestHints {
            complexity: Some(RequestComplexity::Medium),
            quality: Some(RequestQuality::Premium),
            speed: Some(RequestSpeed::Normal),
            guardrail: Some(RequestGuardrail::Strict),
            response_format: Some(ResponseFormat::Json),
            provider: None,
            workload: None,
        },
    };

    println!("Querying LLM for model suggestions...");
    let router = ProviderRouter::from_config(
        config,
        credential_store,
        Some(usage_logger),
        Some(catalog.clone()),
    )
    .await?;
    let response = router.generate(&request).await?;

    // 5. Parse JSON response
    let suggestions: SuggestionSet = serde_json::from_str(&response.content).map_err(|e| {
        AppError::ApiError(format!(
            "Failed to parse LLM response as JSON: {}. Response was: {}",
            e, response.content
        ))
    })?;

    if options.dry_run {
        println!("\n=== DRY RUN MODE ===");
        println!(
            "Would insert {} suggestions:\n",
            suggestions.suggestions.len()
        );
        for (i, s) in suggestions.suggestions.iter().enumerate() {
            println!("{}. {} ({})", i + 1, s.model, s.workload);
            println!("   Rationale: {}", s.rationale.as_deref().unwrap_or("N/A"));
            println!(
                "   Production ready: {}",
                s.production_ready.unwrap_or(false)
            );
            if let Some(notes) = &s.notes {
                println!("   Notes: {}", notes);
            }
            println!();
        }
        return Ok(());
    }

    // 6. Insert suggestions into database
    let mut inserted = 0;
    for suggestion in suggestions.suggestions {
        let workload_enum = match suggestion.workload.to_lowercase().as_str() {
            "chat" => Workload::Chat,
            "code" => Workload::Code,
            "summarization" => Workload::Summarization,
            "extraction" => Workload::Extraction,
            "creative" => Workload::Creative,
            "classification" => Workload::Classification,
            _ => continue, // Skip invalid workloads
        };

        let metadata_str = suggestion
            .metadata
            .and_then(|m| serde_json::to_string(&m).ok());

        catalog
            .upsert_suggestion(
                provider,
                workload_enum,
                suggestion.model.clone(),
                suggestion.rationale.clone(),
                metadata_str,
                "pending",
            )
            .await?;

        inserted += 1;
    }

    println!("Successfully inserted {} model suggestions.", inserted);
    println!("Run 'freegin-ai list-models --include-suggestions' to review them.");
    println!("Use 'freegin-ai adopt-model <provider> <model> --workload <type> --priority <num>' to adopt.");

    Ok(())
}

#[derive(Serialize)]
struct RefreshContext {
    provider: String,
    workload: String,
    current_models: Vec<ModelInfo>,
    usage_stats: StatsInfo,
}

#[derive(Serialize)]
struct ModelInfo {
    model: String,
    priority: i64,
    rationale: Option<String>,
}

#[derive(Serialize)]
struct StatsInfo {
    total_calls: i64,
    success_rate: f64,
    avg_latency_ms: f64,
}

#[derive(Deserialize)]
struct SuggestionSet {
    suggestions: Vec<Suggestion>,
}

#[derive(Deserialize)]
struct Suggestion {
    model: String,
    workload: String,
    rationale: Option<String>,
    production_ready: Option<bool>,
    notes: Option<String>,
    metadata: Option<serde_json::Value>,
}

async fn handle_status(
    catalog: &CatalogStore,
    db_pool: Arc<DbPool>,
    options: StatusOptions,
) -> Result<(), AppError> {
    let health_tracker = HealthTracker::new(db_pool);

    let providers = if let Some(p) = options.provider {
        vec![p]
    } else {
        vec![
            Provider::Groq,
            Provider::DeepSeek,
            Provider::Together,
            Provider::HuggingFace,
            Provider::Google,
            Provider::OpenAI,
            Provider::Anthropic,
            Provider::Cohere,
        ]
    };

    for provider in providers {
        // Get provider health status
        let health = health_tracker.get_health(provider).await?;

        // Format health status
        let health_icon = match health.status {
            freegin_ai::health::HealthStatus::Available => "✓",
            freegin_ai::health::HealthStatus::Degraded => "⚠",
            freegin_ai::health::HealthStatus::Unavailable => "✗",
        };
        let health_text = match health.status {
            freegin_ai::health::HealthStatus::Available => "AVAILABLE",
            freegin_ai::health::HealthStatus::Degraded => "DEGRADED",
            freegin_ai::health::HealthStatus::Unavailable => "UNAVAILABLE",
        };

        println!(
            "\n═══ {} {} {} ═══",
            provider.as_str().to_uppercase(),
            health_icon,
            health_text
        );

        // Show health details if there are issues
        if health.status != freegin_ai::health::HealthStatus::Available {
            if let Some(error) = &health.last_error {
                println!("    Last error: {}", error);
            }
            if let Some(retry_after) = health.retry_after {
                use chrono::Utc;
                let now = Utc::now();
                if retry_after > now {
                    let duration = retry_after - now;
                    let minutes = duration.num_minutes();
                    let seconds = duration.num_seconds() % 60;
                    println!(
                        "    Retry after: {}m {}s (at {})",
                        minutes,
                        seconds,
                        retry_after.format("%H:%M:%S")
                    );
                }
            }
            if health.consecutive_failures > 0 {
                println!("    Consecutive failures: {}", health.consecutive_failures);
            }
        }

        // Get all workloads
        let workloads = [
            Workload::Chat,
            Workload::Code,
            Workload::Summarization,
            Workload::Extraction,
            Workload::Creative,
            Workload::Classification,
        ];

        for workload in workloads {
            // Get active models
            let active = catalog.active_models(provider, Some(workload)).await?;

            // Get suggestions
            let suggestions = catalog
                .list_suggestions(Some(provider), Some(workload))
                .await?;

            // Get usage stats
            let stats = catalog
                .usage_stats(provider, Some(workload))
                .await
                .unwrap_or(freegin_ai::catalog::UsageStats {
                    total_calls: 0,
                    successful_calls: 0,
                    success_rate: 0.0,
                    avg_latency_ms: 0.0,
                    max_latency_ms: 0,
                });

            // Only show workload if there's data
            if !active.is_empty() || !suggestions.is_empty() || stats.total_calls > 0 {
                println!("\n┌─ {:?}", workload);

                // Show currently favored (active roster)
                if !active.is_empty() {
                    println!("│ ▶ ACTIVE (currently favored):");
                    for model in &active {
                        let rationale = model
                            .rationale
                            .as_deref()
                            .unwrap_or("")
                            .chars()
                            .take(50)
                            .collect::<String>();
                        println!(
                            "│   [{:3}] {} {}",
                            model.priority,
                            model.model,
                            if !rationale.is_empty() {
                                format!("— {}", rationale)
                            } else {
                                String::new()
                            }
                        );
                    }
                }

                // Show known models (suggestions)
                if !suggestions.is_empty() {
                    println!("│ ◆ KNOWN (suggestions):");
                    for sugg in &suggestions {
                        let rationale = sugg
                            .rationale
                            .as_deref()
                            .unwrap_or("")
                            .chars()
                            .take(50)
                            .collect::<String>();
                        println!(
                            "│   [{}] {} {}",
                            sugg.status,
                            sugg.model,
                            if !rationale.is_empty() {
                                format!("— {}", rationale)
                            } else {
                                String::new()
                            }
                        );
                    }
                }

                // Show usage stats
                if stats.total_calls > 0 {
                    println!("│ ⚡ USAGE:");
                    println!(
                        "│   Calls: {} | Success: {:.1}% | Avg latency: {:.0}ms",
                        stats.total_calls, stats.success_rate, stats.avg_latency_ms
                    );
                }

                println!("└─");
            }
        }
    }

    println!();
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
