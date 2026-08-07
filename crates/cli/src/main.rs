use agent_skills_core::path_safety::get_repo_root;
use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod commands;
use commands::adr::{run_adr_command, AdrSubcommand};
use commands::agent_council::{run_agent_council_command, AgentCouncilSubcommand};
use commands::architecture_auditor::{
    run_architecture_auditor_command, ArchitectureAuditorSubcommand,
};
use commands::benchmarking::{run_benchmarking_command, BenchmarkingSubcommand};
use commands::capability_gap_analyzer::{
    run_capability_gap_analyzer_command, CapabilityGapAnalyzerSubcommand,
};
use commands::code_janitor::{run_code_janitor_command, CodeJanitorSubcommand};
use commands::context_gatherer::{run_context_gatherer_command, ContextGathererSubcommand};
use commands::depgraph::{run_depgraph, DepgraphArgs};
use commands::domain_modeling::{run_domain_modeling_command, DomainModelingSubcommand};
use commands::git_conflict_resolver::{
    run_git_conflict_resolver_command, GitConflictResolverSubcommand,
};
use commands::install::{run_installer, InstallArgs};
use commands::lint_scripts::{run_lint_scripts, LintScriptsArgs};
use commands::self_annealer::{run_self_annealer_command, SelfAnnealerSubcommand};
use commands::self_progress::{run_self_progress_command, SelfProgressSubcommand};
use commands::skill_creator::{scaffold_skill, validate_skill, SkillCreatorSubcommand};
use commands::tdd::{run_tdd_command, TddArgs};
use commands::tech_doc_writer::{run_tech_doc_writer_audit, TechDocWriterSubcommand};
use commands::what_if_analysis::{run_what_if_analysis_command, WhatIfAnalysisSubcommand};

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
    /// Benchmarking empirical performance verification tool (check, run)
    Benchmarking {
        #[command(subcommand)]
        subcommand: BenchmarkingSubcommand,
    },
    /// Capability Gap Analyzer checklist & domain matrix scanner (check, analyze)
    CapabilityGapAnalyzer {
        #[command(subcommand)]
        subcommand: CapabilityGapAnalyzerSubcommand,
    },
    /// Code Janitor automated code hygiene scanner (check, scan)
    CodeJanitor {
        #[command(subcommand)]
        subcommand: CodeJanitorSubcommand,
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
    /// Git Conflict Resolver 3-way merge/rebase analysis tool (check, analyze)
    GitConflictResolver {
        #[command(subcommand)]
        subcommand: GitConflictResolverSubcommand,
    },
    /// Self Annealer bounded self-healing repair loop (check, run)
    SelfAnnealer {
        #[command(subcommand)]
        subcommand: SelfAnnealerSubcommand,
    },
    /// Self Progress session retrospective tool (check, analyze)
    SelfProgress {
        #[command(subcommand)]
        subcommand: SelfProgressSubcommand,
    },
    /// What-If Analysis blast radius & sensitivity modeling tool (check, impact)
    WhatIfAnalysis {
        #[command(subcommand)]
        subcommand: WhatIfAnalysisSubcommand,
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

/// Run a command that returns `anyhow::Result<()>`, converting to ExitCode.
fn run(result: anyhow::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("❌ {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let repo_root = get_repo_root(None);

    match cli.command {
        Commands::SkillCreator { subcommand } => match subcommand {
            SkillCreatorSubcommand::Scaffold(args) => run(scaffold_skill(&args, Some(&repo_root))
                .map(|path| {
                    println!("🎉 Successfully scaffolded skill at: {}", path.display());
                })),
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
        Commands::Adr { subcommand } => run(run_adr_command(&subcommand, &repo_root)),
        Commands::AgentCouncil { subcommand } => {
            run(run_agent_council_command(&subcommand, &repo_root))
        }
        Commands::ArchitectureAuditor { subcommand } => {
            run(run_architecture_auditor_command(&subcommand, &repo_root))
        }
        Commands::Benchmarking { subcommand } => {
            run(run_benchmarking_command(&subcommand, &repo_root))
        }
        Commands::CapabilityGapAnalyzer { subcommand } => {
            run(run_capability_gap_analyzer_command(&subcommand, &repo_root))
        }
        Commands::CodeJanitor { subcommand } => {
            run(run_code_janitor_command(&subcommand, &repo_root))
        }
        Commands::ContextGatherer { subcommand } => {
            run(run_context_gatherer_command(&subcommand, &repo_root))
        }
        Commands::DomainModeling { subcommand } => {
            run(run_domain_modeling_command(&subcommand, &repo_root))
        }
        Commands::GitConflictResolver { subcommand } => {
            run(run_git_conflict_resolver_command(&subcommand, &repo_root))
        }
        Commands::SelfAnnealer { subcommand } => {
            run(run_self_annealer_command(&subcommand, &repo_root))
        }
        Commands::SelfProgress { subcommand } => {
            run(run_self_progress_command(&subcommand, &repo_root))
        }
        Commands::WhatIfAnalysis { subcommand } => {
            run(run_what_if_analysis_command(&subcommand, &repo_root))
        }
        Commands::Tdd(args) => run(run_tdd_command(&args, &repo_root)),
        Commands::Install(args) => run(run_installer(&args, &repo_root)),
        Commands::Depgraph(args) => run(run_depgraph(&args, &repo_root)),
        Commands::LintScripts(args) => run(run_lint_scripts(&args, &repo_root)),
        Commands::TechDocWriter { subcommand } => match subcommand {
            TechDocWriterSubcommand::Audit(args) => {
                run(run_tech_doc_writer_audit(&args, &repo_root))
            }
        },
    }
}
