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

                println!("\n{}", "═══ Server Function Checks ═══".cyan().bold());
                let server_fn_results = checks::server_functions::check_server_function_endpoints(&config).await;
                for r in &server_fn_results {
                    print_result(r);
                }
                results.extend(server_fn_results);

                println!("\n{}", "═══ Feature Page Checks ═══".cyan().bold());
                let page_results = checks::server_functions::check_new_feature_pages(&config).await;
                for r in &page_results {
                    print_result(r);
                }
                results.extend(page_results);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_result_does_not_panic_for_pass() {
        let r = checks::TestResult::pass("test pass");
        print_result(&r);
    }

    #[test]
    fn print_result_does_not_panic_for_fail() {
        let r = checks::TestResult::fail("test fail", "something broke");
        print_result(&r);
    }

    #[test]
    fn print_result_does_not_panic_for_warn() {
        let r = checks::TestResult::warn("test warn", "heads up");
        print_result(&r);
    }

    #[test]
    fn print_result_does_not_panic_for_skip() {
        let r = checks::TestResult::skip("test skip", "not applicable");
        print_result(&r);
    }

    #[test]
    fn print_result_with_duration() {
        let r = checks::TestResult::pass("fast").with_duration(42);
        print_result(&r);
    }

    #[test]
    fn print_result_fail_with_multiline_detail() {
        let r = checks::TestResult::fail("multi", "line1\nline2\nline3");
        print_result(&r);
    }

    #[test]
    fn cli_parses_run_command() {
        let cli = Cli::parse_from(["mytest", "run"]);
        match cli.command {
            Commands::Run { health_only, sso_only, ui_only, i18n_only, output } => {
                assert!(!health_only);
                assert!(!sso_only);
                assert!(!ui_only);
                assert!(!i18n_only);
                assert_eq!(output, "reports");
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn cli_parses_run_health_only() {
        let cli = Cli::parse_from(["mytest", "run", "--health-only"]);
        match cli.command {
            Commands::Run { health_only, .. } => assert!(health_only),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn cli_parses_run_sso_only() {
        let cli = Cli::parse_from(["mytest", "run", "--sso-only"]);
        match cli.command {
            Commands::Run { sso_only, .. } => assert!(sso_only),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn cli_parses_run_ui_only() {
        let cli = Cli::parse_from(["mytest", "run", "--ui-only"]);
        match cli.command {
            Commands::Run { ui_only, .. } => assert!(ui_only),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn cli_parses_run_i18n_only() {
        let cli = Cli::parse_from(["mytest", "run", "--i18n-only"]);
        match cli.command {
            Commands::Run { i18n_only, .. } => assert!(i18n_only),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn cli_parses_run_custom_output() {
        let cli = Cli::parse_from(["mytest", "run", "--output", "/tmp/reports"]);
        match cli.command {
            Commands::Run { output, .. } => assert_eq!(output, "/tmp/reports"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn cli_parses_status_command() {
        let cli = Cli::parse_from(["mytest", "status"]);
        match cli.command {
            Commands::Status => {}
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn cli_parses_plan_command() {
        let cli = Cli::parse_from(["mytest", "plan"]);
        match cli.command {
            Commands::Plan { report, output } => {
                assert!(report.is_none());
                assert_eq!(output, "reports/fix-plan.md");
            }
            _ => panic!("Expected Plan command"),
        }
    }

    #[test]
    fn cli_parses_plan_with_report() {
        let cli = Cli::parse_from(["mytest", "plan", "--report", "/tmp/report.md"]);
        match cli.command {
            Commands::Plan { report, .. } => {
                assert_eq!(report.unwrap(), "/tmp/report.md");
            }
            _ => panic!("Expected Plan command"),
        }
    }

    #[test]
    fn cli_parses_plan_with_output() {
        let cli = Cli::parse_from(["mytest", "plan", "--output", "/tmp/plan.md"]);
        match cli.command {
            Commands::Plan { output, .. } => {
                assert_eq!(output, "/tmp/plan.md");
            }
            _ => panic!("Expected Plan command"),
        }
    }
}
