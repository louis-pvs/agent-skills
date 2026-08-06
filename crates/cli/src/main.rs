use agent_skills_core::path_safety::get_repo_root;
use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod commands;
use commands::adr::{run_adr_command, AdrSubcommand};
use commands::agent_council::{run_agent_council_command, AgentCouncilSubcommand};
use commands::architecture_auditor::{
    run_architecture_auditor_command, ArchitectureAuditorSubcommand,
};
use commands::context_gatherer::{run_context_gatherer_command, ContextGathererSubcommand};
use commands::depgraph::{run_depgraph, DepgraphArgs};
use commands::domain_modeling::{run_domain_modeling_command, DomainModelingSubcommand};
use commands::install::{run_installer, InstallArgs};
use commands::lint_scripts::{run_lint_scripts, LintScriptsArgs};
use commands::skill_creator::{scaffold_skill, validate_skill, SkillCreatorSubcommand};
use commands::tdd::{run_tdd_command, TddArgs};
use commands::tech_doc_writer::{run_tech_doc_writer_audit, TechDocWriterSubcommand};

#[derive(Parser, Debug)]
#[command(
    name = "agent-skills",
    author,
    version,
    about = "Unified CLI for Agent Skills automation & management"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Skill Creator tooling (scaffold, validate Agent Skills)
    SkillCreator {
        #[command(subcommand)]
        subcommand: SkillCreatorSubcommand,
    },
    /// Architecture Decision Records (ADR) CLI utility (init, new, supersede, reindex, validate)
    Adr {
        #[command(subcommand)]
        subcommand: AdrSubcommand,
    },
    /// Agent Council multi-agent prompt synthesis (start, wait, results, clean)
    AgentCouncil {
        #[command(subcommand)]
        subcommand: AgentCouncilSubcommand,
    },
    /// Architecture Auditor design principle audit tool (check, analyze)
    ArchitectureAuditor {
        #[command(subcommand)]
        subcommand: ArchitectureAuditorSubcommand,
    },
    /// Context Gatherer extraction tools (git-coupling, symbol-nav, ast-search)
    ContextGatherer {
        #[command(subcommand)]
        subcommand: ContextGathererSubcommand,
    },
    /// Domain Modeling DDD tools (check, scaffold-entity)
    DomainModeling {
        #[command(subcommand)]
        subcommand: DomainModelingSubcommand,
    },
    /// Test-Driven Development (TDD) runner and state verifier (--detect, --verify-red, --verify-green)
    Tdd(TddArgs),
    /// Re-run global skill symlinking installer across supported AI agents
    Install(InstallArgs),
    /// Skill dependency graph resolver and lockfile generator/verifier
    Depgraph(DepgraphArgs),
    /// Script compliance auditor (ADR 0001 stdlib, ADR 0004 path safety, ADR 0005 config)
    LintScripts(LintScriptsArgs),
    /// Technical documentation quality & GFM compliance auditor
    TechDocWriter {
        #[command(subcommand)]
        subcommand: TechDocWriterSubcommand,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let repo_root = get_repo_root(None);

    match cli.command {
        Commands::SkillCreator { subcommand } => match subcommand {
            SkillCreatorSubcommand::Scaffold(args) => match scaffold_skill(&args, Some(&repo_root))
            {
                Ok(path) => {
                    println!("🎉 Successfully scaffolded skill at: {}", path.display());
                    ExitCode::SUCCESS
                }
                Err(err) => {
                    eprintln!("{err}");
                    ExitCode::FAILURE
                }
            },
            SkillCreatorSubcommand::Validate(args) => {
                let (is_valid, issues) = validate_skill(&args, Some(&repo_root));
                if is_valid {
                    println!(
                        "✅ Skill at '{}' is VALID according to agentskills.io standard!",
                        args.path
                    );
                    ExitCode::SUCCESS
                } else {
                    eprintln!("❌ Skill validation FAILED for '{}':", args.path);
                    for issue in &issues {
                        eprintln!("  - {issue}");
                    }
                    ExitCode::FAILURE
                }
            }
        },
        Commands::Adr { subcommand } => match run_adr_command(&subcommand, &repo_root) {
            Ok(_) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("❌ ADR operation failed: {err}");
                ExitCode::FAILURE
            }
        },
        Commands::AgentCouncil { subcommand } => {
            match run_agent_council_command(&subcommand, &repo_root) {
                Ok(_) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("❌ Agent council failed: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        Commands::ArchitectureAuditor { subcommand } => {
            match run_architecture_auditor_command(&subcommand, &repo_root) {
                Ok(_) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("❌ Architecture auditor failed: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        Commands::ContextGatherer { subcommand } => {
            match run_context_gatherer_command(&subcommand, &repo_root) {
                Ok(_) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("❌ Context gatherer failed: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        Commands::DomainModeling { subcommand } => {
            match run_domain_modeling_command(&subcommand, &repo_root) {
                Ok(_) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("❌ Domain modeling failed: {err}");
                    ExitCode::FAILURE
                }
            }
        }
        Commands::Tdd(args) => match run_tdd_command(&args, &repo_root) {
            Ok(_) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
        },
        Commands::Install(args) => match run_installer(&args, &repo_root) {
            Ok(_) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("❌ Installation failed: {err}");
                ExitCode::FAILURE
            }
        },
        Commands::Depgraph(args) => match run_depgraph(&args, &repo_root) {
            Ok(_) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("❌ Depgraph execution failed: {err}");
                ExitCode::FAILURE
            }
        },
        Commands::LintScripts(args) => match run_lint_scripts(&args, &repo_root) {
            Ok(_) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("{err}");
                ExitCode::FAILURE
            }
        },
        Commands::TechDocWriter { subcommand } => match subcommand {
            TechDocWriterSubcommand::Audit(args) => {
                match run_tech_doc_writer_audit(&args, &repo_root) {
                    Ok(_) => ExitCode::SUCCESS,
                    Err(err) => {
                        eprintln!("{err}");
                        ExitCode::FAILURE
                    }
                }
            }
        },
    }
}
