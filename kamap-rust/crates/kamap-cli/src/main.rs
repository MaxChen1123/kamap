mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "kamap", version, about = "Knowledge Asset Mapping — Git-based impact analysis framework")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new kamap project
    Init(commands::init::InitArgs),

    /// Scan for impacted assets based on Git changes
    Scan(commands::scan::ScanArgs),

    /// Policy check (CI-friendly, exits with non-zero on failure)
    Check(commands::check::CheckArgs),

    /// Explain a mapping or impact chain
    Explain(commands::explain::ExplainArgs),

    /// Output tool self-description for Agent consumption
    Describe(commands::describe::DescribeArgs),

    /// Mapping management (add, remove, list, validate, export, import)
    Mapping(commands::mapping::MappingArgs),

    /// Asset management (add, remove, list, check)
    Asset(commands::asset::AssetArgs),

    /// Index management (build, stats)
    Index(commands::index::IndexArgs),

    /// Provider management (list, info)
    Provider(commands::provider::ProviderArgs),

    /// Plugin management (list, info) — deprecated, use `provider` instead
    Plugin(commands::plugin::PluginArgs),
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init(args) => commands::init::run(args),
        Commands::Scan(args) => commands::scan::run(args),
        Commands::Check(args) => commands::check::run(args),
        Commands::Explain(args) => commands::explain::run(args),
        Commands::Describe(args) => commands::describe::run(args),
        Commands::Mapping(args) => commands::mapping::run(args),
        Commands::Asset(args) => commands::asset::run(args),
        Commands::Index(args) => commands::index::run(args),
        Commands::Provider(args) => commands::provider::run(args),
        Commands::Plugin(args) => commands::plugin::run(args),
    };

    if let Err(e) = result {
        eprintln!("{}", kamap_core::output::format_error_json("command_error", &format!("{:#}", e)));
        std::process::exit(1);
    }
}
