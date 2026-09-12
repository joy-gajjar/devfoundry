mod config;
mod runtime;
mod update;

use clap::{Parser, Subcommand};
use devfoundry_schema::{ProjectId, SessionId};
use devfoundry_storage::SqliteStore;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "devfoundry", version, about = "A local-first AI coding agent")]
struct Cli {
    /// Project directory. Defaults to the current directory.
    #[arg(long, global = true, value_name = "PATH")]
    dir: Option<PathBuf>,

    /// Print machine-readable diagnostics.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check the public release metadata for a newer version.
    Update {
        #[command(subcommand)]
        command: UpdateCommand,
    },
    /// Print the resolved project directory.
    Doctor,
    /// Make one bounded GitHub Copilot smoke request.
    CopilotSmoke {
        /// Explicitly allow the external network request.
        #[arg(long, conflicts_with = "mock")]
        allow_network: bool,
        /// Use the deterministic in-process provider without reading credentials.
        #[arg(long, conflicts_with = "allow_network")]
        mock: bool,
        /// Model to request. Defaults to the configured model or gpt-4o-mini.
        #[arg(long)]
        model: Option<String>,
    },
    /// Inspect or back up a SQLite database.
    Database {
        #[command(subcommand)]
        command: DatabaseCommand,
    },
    /// Export or import a portable session JSON file.
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
    /// Start the local HTTP API.
    Serve {
        #[arg(long, default_value = "127.0.0.1:4096")]
        bind: String,
        #[arg(long, default_value = "devfoundry.db")]
        database: String,
    },
}

#[derive(Debug, Subcommand)]
enum UpdateCommand {
    /// Check for a newer release without downloading or installing it.
    Check,
}

#[derive(Debug, Subcommand)]
enum SessionCommand {
    /// Export normalized session history without secrets or runtime state.
    Export {
        #[arg(long, default_value = ".devfoundry.db")]
        database: PathBuf,
        #[arg(long)]
        session: String,
        #[arg(long, value_name = "PATH")]
        output: PathBuf,
    },
    /// Import a session into an existing project without overwriting records.
    Import {
        #[arg(long, default_value = ".devfoundry.db")]
        database: PathBuf,
        #[arg(long)]
        project: String,
        #[arg(long, value_name = "PATH")]
        input: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum DatabaseCommand {
    /// Print bounded, content-free SQLite health diagnostics.
    Diagnose {
        #[arg(long, default_value = ".devfoundry.db")]
        database: PathBuf,
    },
    /// Create a consistent backup without overwriting an existing file.
    Backup {
        #[arg(long, default_value = ".devfoundry.db")]
        database: PathBuf,
        #[arg(long, value_name = "PATH")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let requested_directory = cli
        .dir
        .unwrap_or_else(|| std::env::current_dir().expect("current directory is unavailable"));
    let directory = config::discover_root(&requested_directory).unwrap_or(requested_directory);
    let loaded_config = config::load_from_root(&directory);
    let json = cli.json;

    match cli.command {
        Some(Command::Update {
            command: UpdateCommand::Check,
        }) => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime initialization failed");
            runtime.block_on(update::check());
        }
        Some(Command::Doctor) => {
            if json {
                match loaded_config {
                    Ok(config) => println!(
                        "{}",
                        serde_json::json!({
                            "project_directory": directory,
                            "config_loaded": true,
                            "configured_model": config.model,
                            "provider_kind": config.provider.kind,
                            "provider_base_url": config.provider.base_url,
                            "provider_token_configured": config.provider.token_env.as_deref().and_then(std::env::var_os).is_some()
                        })
                    ),
                    Err(error) => {
                        eprintln!("config_error={error}");
                        std::process::exit(2);
                    }
                }
                return;
            }
            println!("project_directory={}", directory.display());
            match loaded_config {
                Ok(config) => {
                    println!("config_loaded=true");
                    println!(
                        "configured_model={}",
                        config.model.as_deref().unwrap_or("none")
                    );
                    println!("provider_kind={}", config.provider.kind);
                    println!("provider_base_url={}", config.provider.base_url);
                    println!(
                        "provider_token_configured={}",
                        config
                            .provider
                            .token_env
                            .as_deref()
                            .and_then(std::env::var_os)
                            .is_some()
                    );
                }
                Err(error) => {
                    eprintln!("config_error={error}");
                    std::process::exit(2);
                }
            }
        }
        Some(Command::CopilotSmoke {
            allow_network,
            mock,
            model,
        }) => {
            if !allow_network && !mock {
                eprintln!(
                    "choose --mock for an offline smoke test or --allow-network for a real provider request"
                );
                std::process::exit(2);
            }
            let config = match loaded_config {
                Ok(config) => config,
                Err(error) => {
                    eprintln!("config_error={error}");
                    std::process::exit(2);
                }
            };
            let model = model
                .or(config.model.clone())
                .unwrap_or_else(|| "gpt-4o-mini".into());
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("runtime initialization failed");
            if let Err(error) =
                runtime.block_on(runtime::copilot_smoke(config, model, cli.json, mock))
            {
                if cli.json {
                    println!("{}", serde_json::json!({"status": "error", "error": error}));
                } else {
                    eprintln!("copilot_smoke_error={error}");
                }
                std::process::exit(1);
            }
        }
        Some(Command::Serve { bind, database }) => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("runtime initialization failed");
            if let Err(error) = runtime.block_on(runtime::serve(&database, &bind)) {
                eprintln!("server_error={error}");
                std::process::exit(1);
            }
        }
        Some(Command::Database { command }) => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("runtime initialization failed");
            let result = runtime.block_on(async move {
                match command {
                    DatabaseCommand::Diagnose { database } => {
                        let store = SqliteStore::connect_path(database).await?;
                        let diagnostics = store.diagnostics().await?;
                        if json {
                            println!("{}", serde_json::to_string(&diagnostics)?);
                        } else {
                            println!("foreign_keys={}", diagnostics.foreign_keys);
                            println!("integrity_check={}", diagnostics.integrity_check);
                            for (table, count) in diagnostics.row_counts {
                                println!("rows_{table}={count}");
                            }
                        }
                        Ok::<(), Box<dyn std::error::Error>>(())
                    }
                    DatabaseCommand::Backup { database, output } => {
                        let store = SqliteStore::connect_path(database).await?;
                        store.backup_to(&output).await?;
                        println!("backup_created={}", output.display());
                        Ok(())
                    }
                }
            });
            if let Err(error) = result {
                eprintln!("database_error={error}");
                std::process::exit(1);
            }
        }
        Some(Command::Session { command }) => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("runtime initialization failed");
            let result = runtime.block_on(async move {
                match command {
                    SessionCommand::Export {
                        database,
                        session,
                        output,
                    } => {
                        let session_id: SessionId = session.parse().map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "invalid session ID",
                            )
                        })?;
                        let store = SqliteStore::connect_path(database).await?;
                        let export = store.export_session(session_id).await?;
                        if output.exists() {
                            return Err::<(), Box<dyn std::error::Error>>(
                                "output already exists".into(),
                            );
                        }
                        std::fs::write(&output, serde_json::to_vec_pretty(&export)?)?;
                        println!("session_exported={}", output.display());
                        Ok(())
                    }
                    SessionCommand::Import {
                        database,
                        project,
                        input,
                    } => {
                        let project_id: ProjectId = project.parse().map_err(|_| {
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                "invalid project ID",
                            )
                        })?;
                        let export: devfoundry_schema::SessionExport =
                            serde_json::from_slice(&std::fs::read(input)?)?;
                        let store = SqliteStore::connect_path(database).await?;
                        let session = store.import_session(project_id, &export).await?;
                        println!("session_imported={}", session.id);
                        Ok(())
                    }
                }
            });
            if let Err(error) = result {
                eprintln!("session_error={error}");
                std::process::exit(1);
            }
        }
        None => {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("runtime initialization failed");
            if let Err(error) = runtime.block_on(runtime::interactive(directory)) {
                eprintln!("tui_error={error}");
                std::process::exit(1);
            }
        }
    }
}
