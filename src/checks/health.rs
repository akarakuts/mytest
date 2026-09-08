//! Health checks для всех сервисов my*.

use crate::checks::{Category, TestResult};
use colored::Colorize;
use crate::config::ServiceConfig;
use std::time::Instant;

/// Проверить health endpoint одного сервиса.
async fn check_health(
    svc: &crate::config::Service,
    config: &ServiceConfig,
) -> TestResult {
    let url = config.health_url(svc);
    let start = Instant::now();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    match client.get(&url).send().await {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();

            if status.is_success() {
                TestResult::pass(format!("{} healthz", svc.name))
                    .category(Category::Health)
                    .with_duration(elapsed)
            } else if status.is_redirection() {
                TestResult::warn(
                    format!("{} healthz", svc.name),
                    format!("HTTP {} — redirect (expected 200)", status.as_u16()),
                )
                .category(Category::Health)
                .with_duration(elapsed)
                .fix_hint(format!(
                    "Check {} healthz endpoint — returns redirect instead of 200",
                    svc.name
                ))
            } else {
                TestResult::fail(
                    format!("{} healthz", svc.name),
                    format!("HTTP {}: {}", status.as_u16(), &body[..body.len().min(200)]),
                )
                .category(Category::Health)
                .with_duration(elapsed)
                .fix_hint(format!("Fix {} healthz endpoint — returns {}", svc.name, status.as_u16()))
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            TestResult::fail(
                format!("{} healthz", svc.name),
                format!("Connection failed: {}", e),
            )
            .category(Category::Health)
            .with_duration(elapsed)
            .fix_hint(format!(
                "Start {} service on port {} or check network",
                svc.name, svc.port
            ))
        }
    }
}

/// Проверить доступность через HTTPS (Caddy).
async fn check_https(svc: &crate::config::Service, config: &ServiceConfig) -> TestResult {
    let url = config.base_url_https(svc);
    let start = Instant::now();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    match client.get(&url).send().await {
        Ok(resp) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let status = resp.status();
            if status.is_success() || status.is_redirection() {
                TestResult::pass(format!("{} HTTPS ({})", svc.name, svc.domain))
                    .category(Category::Health)
                    .with_duration(elapsed)
            } else {
                TestResult::warn(
                    format!("{} HTTPS ({})", svc.name, svc.domain),
                    format!("HTTP {}", status.as_u16()),
                )
                .category(Category::Health)
                .with_duration(elapsed)
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            TestResult::fail(
                format!("{} HTTPS ({})", svc.name, svc.domain),
                format!("Connection failed: {}", e),
            )
            .category(Category::Health)
            .with_duration(elapsed)
            .fix_hint("Check Caddy configuration and DNS entries for home.local domains")
        }
    }
}

/// Проверить наличие стандартных security headers.
async fn check_security_headers(
    svc: &crate::config::Service,
    config: &ServiceConfig,
) -> TestResult {
    let url = config.health_url(svc);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    match client.get(&url).send().await {
        Ok(resp) => {
            let headers = resp.headers();
            let mut missing = Vec::new();

            if !headers.contains_key("x-content-type-options") {
                missing.push("X-Content-Type-Options");
            }
            if !headers.contains_key("x-frame-options") {
                missing.push("X-Frame-Options");
            }
            if !headers.contains_key("referrer-policy") {
                missing.push("Referrer-Policy");
            }
            if !headers.contains_key("strict-transport-security") {
                missing.push("Strict-Transport-Security");
            }

            if missing.is_empty() {
                TestResult::pass(format!("{} security headers", svc.name))
                    .category(Category::Health)
            } else {
                TestResult::warn(
                    format!("{} security headers", svc.name),
                    format!("Missing: {}", missing.join(", ")),
                )
                .category(Category::Health)
                .fix_hint("Add missing security headers in Axum middleware")
            }
        }
        Err(_) => TestResult::skip(
            format!("{} security headers", svc.name),
            "Service unreachable",
        ),
    }
}

/// Проверить response time — предупреждение при > 1000ms.
async fn check_response_time(
    svc: &crate::config::Service,
    config: &ServiceConfig,
) -> TestResult {
    let url = config.health_url(svc);
    let start = Instant::now();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    match client.get(&url).send().await {
        Ok(_) => {
            let elapsed = start.elapsed().as_millis() as u64;
            if elapsed < 500 {
                TestResult::pass(format!("{} response time", svc.name))
                    .category(Category::Health)
                    .with_duration(elapsed)
            } else if elapsed < 2000 {
                TestResult::warn(
                    format!("{} response time", svc.name),
                    format!("{}ms (target: <500ms)", elapsed),
                )
                .category(Category::Health)
                .with_duration(elapsed)
                .fix_hint("Investigate slow healthz response — possible DB connection issue")
            } else {
                TestResult::fail(
                    format!("{} response time", svc.name),
                    format!("{}ms (target: <500ms)", elapsed),
                )
                .category(Category::Health)
                .with_duration(elapsed)
                .fix_hint("Service is very slow — check database connectivity and resource usage")
            }
        }
        Err(_) => TestResult::skip(
            format!("{} response time", svc.name),
            "Service unreachable",
        ),
    }
}

/// Запустить все health checks.
pub async fn run_all(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();

    // Check all services concurrently (batches of 7 to avoid connection limits)
    for chunk in config.services.chunks(7) {
        let mut handles = Vec::new();
        for svc in chunk {
            let svc = svc.clone();
            handles.push(tokio::spawn(async move {
                let mut r = Vec::new();
                r.push(check_health(&svc, &ServiceConfig::default()).await);
                r.push(check_https(&svc, &ServiceConfig::default()).await);
                r.push(check_security_headers(&svc, &ServiceConfig::default()).await);
                r.push(check_response_time(&svc, &ServiceConfig::default()).await);
                r
            }));
        }
        for handle in handles {
            if let Ok(r) = handle.await {
                results.extend(r);
            }
        }
    }

    results
}

/// Вывести статус всех сервисов в консоль.
pub async fn print_status(config: &ServiceConfig) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    println!(
        "  {:<22} {:<6} {:<25} {:<8} {}",
        "Service", "Port", "Domain", "HTTP", "HTTPS"
    );
    println!("  {}", "─".repeat(80));

    for svc in &config.services {
        let http_status = match client.get(&config.health_url(svc)).send().await {
            Ok(r) => format!("{}", r.status().as_u16()),
            Err(_) => "DOWN".to_string(),
        };
        let https_status = match client.get(&config.base_url_https(svc)).send().await {
            Ok(r) => format!("{}", r.status().as_u16()),
            Err(_) => "DOWN".to_string(),
        };

        let http_color = if http_status.starts_with('2') {
            http_status.green().to_string()
        } else if http_status.starts_with('3') {
            http_status.yellow().to_string()
        } else {
            http_status.red().to_string()
        };

        let https_color = if https_status.starts_with('2') || https_status.starts_with('3') {
            https_status.green().to_string()
        } else {
            https_status.red().to_string()
        };

        println!(
            "  {:<22} {:<6} {:<25} {:<8} {}",
            svc.name, svc.port, svc.domain, http_color, https_color
        );
    }
}
