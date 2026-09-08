//! mytest — интеграционный тест-сьют и аудит UX/UI для всего стека my* приложений.
//!
//! Проверяет: health endpoints, API, SSO/OIDC, UI-структуру, i18n, кросс-приложенческую
//! интеграцию. Генерирует отчёт + план исправлений в формате Markdown.

mod checks;
mod config;
mod fix_plan;
mod report;
mod ui_analyze;

use clap::{Parser, Subcommand};
use colored::Colorize;
use config::ServiceConfig;

#[derive(Parser)]
#[command(name = "mytest", about = "Integration test & UX/UI audit for my* suite")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Запустить все проверки и сгенерировать отчёт
    Run {
        /// Только health checks
        #[arg(long)]
        health_only: bool,
        /// Только SSO checks
        #[arg(long)]
        sso_only: bool,
        /// Только UI/UX audit
        #[arg(long)]
        ui_only: bool,
        /// Только i18n checks
        #[arg(long)]
        i18n_only: bool,
        /// Output directory for reports
        #[arg(long, default_value = "reports")]
        output: String,
    },
    /// Показать текущий статус всех сервисов
    Status,
    /// Сгенерировать план исправлений на основе последнего отчёта
    Plan {
        /// Input report file
        #[arg(long)]
        report: Option<String>,
        /// Output plan file
        #[arg(long, default_value = "reports/fix-plan.md")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let config = ServiceConfig::default();

    match cli.command {
        Commands::Run {
            health_only,
            sso_only,
            ui_only,
            i18n_only,
            output,
        } => {
            let run_all = !health_only && !sso_only && !ui_only && !i18n_only;

            let mut results = Vec::new();

            if run_all || health_only {
                println!("{}", "═══ Health Checks ═══".cyan().bold());
                let health = checks::health::run_all(&config).await;
                for r in &health {
                    print_result(r);
                }
                results.extend(health);
            }

            if run_all || sso_only {
                println!("\n{}", "═══ SSO / OIDC Checks ═══".cyan().bold());
                let sso = checks::sso::run_all(&config).await;
                for r in &sso {
                    print_result(r);
                }
                results.extend(sso);
            }

            if run_all || ui_only {
                println!("\n{}", "═══ UI / UX Audit ═══".cyan().bold());
                let ui = checks::ui::run_all(&config).await;
                for r in &ui {
                    print_result(r);
                }
                results.extend(ui);
            }

            if run_all || i18n_only {
                println!("\n{}", "═══ i18n Checks ═══".cyan().bold());
                let i18n = checks::i18n::run_all(&config).await;
                for r in &i18n {
                    print_result(r);
                }
                results.extend(i18n);
            }

            if run_all {
                println!("\n{}", "═══ Integration Checks ═══".cyan().bold());
                let integration = checks::integration::run_all(&config).await;
                for r in &integration {
                    print_result(r);
                }
                results.extend(integration);
            }

            // Summary
            let passed = results.iter().filter(|r| r.status == checks::Status::Pass).count();
            let failed = results.iter().filter(|r| r.status == checks::Status::Fail).count();
            let warn = results.iter().filter(|r| r.status == checks::Status::Warn).count();
            let skipped = results.iter().filter(|r| r.status == checks::Status::Skip).count();

            println!("\n{}", "═══ Summary ═══".cyan().bold());
            println!(
                "  {} passed, {} failed, {} warnings, {} skipped (total {})",
                passed.to_string().green(),
                failed.to_string().red(),
                warn.to_string().yellow(),
                skipped.to_string().dimmed(),
                results.len()
            );

            // Generate reports
            std::fs::create_dir_all(&output)?;
            let report_path = format!("{}/test-report.md", output);
            report::markdown::generate(&results, &report_path)?;
            println!("\n  Report: {}", report_path.green());

            let plan_path = format!("{}/fix-plan.md", output);
            fix_plan::generate(&results, &plan_path)?;
            println!("  Plan:   {}", plan_path.green());
        }
        Commands::Status => {
            println!("{}", "═══ Service Status ═══".cyan().bold());
            checks::health::print_status(&config).await;
        }
        Commands::Plan { report, output: _output } => {
            let report_file = report.unwrap_or_else(|| "reports/test-report.md".to_string());
            if !std::path::Path::new(&report_file).exists() {
                println!(
                    "{}",
                    format!("Report file not found: {}", report_file).red()
                );
                println!("Run `mytest run` first to generate a report.");
                std::process::exit(1);
            }
            // For now, regenerate from last run results
            println!("Generating fix plan from: {}", report_file);
        }
    }

    Ok(())
}

fn print_result(r: &checks::TestResult) {
    let icon = match r.status {
        checks::Status::Pass => "✓".green(),
        checks::Status::Fail => "✗".red(),
        checks::Status::Warn => "⚠".yellow(),
        checks::Status::Skip => "○".dimmed(),
    };
    let duration = if r.duration_ms > 0 {
        format!(" ({}ms)", r.duration_ms).dimmed().to_string()
    } else {
        String::new()
    };
    println!("  {} {}{}", icon, r.name, duration);
    if r.status == checks::Status::Fail || r.status == checks::Status::Warn {
        if let Some(detail) = &r.detail {
            for line in detail.lines() {
                println!("    {}", line.dimmed());
            }
        }
    }
}
