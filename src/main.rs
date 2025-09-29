//! Main entry point for the Freegin AI service.
//!
//! Responsibilities:
//! - Handle basic CLI flags (`--help`, `--version`).
//! - Initialize logging and tracing.
//! - Load application configuration.
//! - Establish shared infrastructure (database, provider clients).
//! - Start the Axum web server and expose HTTP routes.

use std::{env, net::SocketAddr, process, sync::Arc};

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use freegin_ai::{
    config,
    credentials::CredentialStore,
    database,
    error::AppError,
    providers::{Provider, ProviderRouter},
    routes::{self, AppState},
    usage::UsageLogger,
};

enum CliCommand {
    Run,
    Help,
    Version,
    AddService(Provider),
    RemoveService(Provider),
    ListServices,
}

#[tokio::main]
async fn main() {
    let command = parse_cli_command();

    if matches!(command, CliCommand::Help) {
        print_help();
        return;
    }

    if matches!(command, CliCommand::Version) {
        print_version();
        return;
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

    // Load configuration (falls back to defaults if the secrets file is missing).
    let config = match config::AppConfig::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            error!(error = %err, "Failed to load configuration");
            eprintln!("freegin-ai: configuration error: {err}");
            process::exit(1);
        }
    };
    info!("Configuration loaded successfully");

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

fn parse_cli_command() -> CliCommand {
    let mut args = env::args();
    let _program = args.next();
    if let Some(flag) = args.next() {
        match flag.as_str() {
            "-h" | "--help" | "help" => CliCommand::Help,
            "-V" | "--version" | "version" => CliCommand::Version,
            "add-service" => {
                let Some(name) = args.next() else {
                    eprintln!("freegin-ai: add-service requires a provider name");
                    return CliCommand::Help;
                };
                match Provider::from_alias(&name) {
                    Some(provider) => CliCommand::AddService(provider),
                    None => {
                        eprintln!("freegin-ai: unknown provider '{name}'");
                        CliCommand::Help
                    }
                }
            }
            "remove-service" => {
                let Some(name) = args.next() else {
                    eprintln!("freegin-ai: remove-service requires a provider name");
                    return CliCommand::Help;
                };
                match Provider::from_alias(&name) {
                    Some(provider) => CliCommand::RemoveService(provider),
                    None => {
                        eprintln!("freegin-ai: unknown provider '{name}'");
                        CliCommand::Help
                    }
                }
            }
            "list-services" => CliCommand::ListServices,
            other => {
                eprintln!("freegin-ai: unknown argument '{other}'\n");
                CliCommand::Help
            }
        }
    } else {
        CliCommand::Run
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
    let stored: std::collections::HashSet<_> =
        store.stored_providers().await?.into_iter().collect();

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
