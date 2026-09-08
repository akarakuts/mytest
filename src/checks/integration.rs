//! Кросс-приложенческие проверки: webhook endpoints, portal widget, SUITE_APPS links.

use crate::checks::{Category, TestResult};
use crate::config::ServiceConfig;
use reqwest::Client;

fn http_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Проверить что все сервисы в app-switcher доступны.
async fn check_suite_apps_links(config: &ServiceConfig) -> TestResult {
    let client = http_client();
    let mut broken = Vec::new();

    // Check portal page for SUITE_APPS links
    let portal_url = format!("http://{}:3004", config.base_ip);
    match client.get(&portal_url).send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            for svc in &config.services {
                if svc.name == "myportal" {
                    continue;
                }
                let link_pattern = format!("http://{}:{}", config.base_ip, svc.port);
                if !body.contains(&link_pattern) && !body.contains(svc.domain) {
                    broken.push(svc.name);
                }
            }

            if broken.is_empty() {
                TestResult::pass("SUITE_APPS completeness").category(Category::Integration)
            } else {
                TestResult::warn(
                    "SUITE_APPS completeness",
                    format!("Missing from portal: {}", broken.join(", ")),
                )
                .category(Category::Integration)
                .fix_hint("Check SUITE_APPS env variable in myportal .env")
            }
        }
        Err(_) => TestResult::skip("SUITE_APPS completeness", "Portal unreachable")
            .category(Category::Integration),
    }
}

/// Проверить widget endpoints (GET /api/v1/widget) на всех сервисах.
async fn check_widget_endpoints(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();
    let client = http_client();

    // Original 12 apps + mycrm may not have widget endpoints
    let no_widget = ["mycrowd", "myconf", "myjira", "mybitbucket", "mybamboo", "myportal"];

    for svc in &config.services {
        if no_widget.contains(&svc.name) {
            continue;
        }

        let url = format!("http://{}:{}/api/v1/widget", config.base_ip, svc.port);
        match client.get(&url).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    results.push(
                        TestResult::pass(format!("{} widget endpoint", svc.name))
                            .category(Category::Integration),
                    );
                } else if resp.status().is_client_error() {
                    results.push(
                        TestResult::warn(
                            format!("{} widget endpoint", svc.name),
                            format!("HTTP {} — widget endpoint not found", resp.status().as_u16()),
                        )
                        .category(Category::Integration)
                        .fix_hint("Add GET /api/v1/widget endpoint for portal integration"),
                    );
                } else {
                    results.push(
                        TestResult::fail(
                            format!("{} widget endpoint", svc.name),
                            format!("HTTP {}", resp.status().as_u16()),
                        )
                        .category(Category::Integration),
                    );
                }
            }
            Err(_) => results.push(
                TestResult::skip(format!("{} widget endpoint", svc.name), "Service unreachable")
                    .category(Category::Integration),
            ),
        }
    }

    results
}

/// Проверить что Crowd SSO discovery доступен для всех OIDC-клиентов.
async fn check_oidc_client_configs(config: &ServiceConfig) -> TestResult {
    // Verify Crowd discovery is accessible via the URL configured in clients
    let client = http_client();
    let discovery_url = format!("{}/.well-known/openid-configuration", config.crowd_url);

    match client.get(&discovery_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let issuer = body.get("issuer").and_then(|v| v.as_str()).unwrap_or("");

                // Check if OIDC clients can reach the issuer
                if issuer.starts_with("http") {
                    match client.get(issuer).send().await {
                        Ok(r) if r.status().is_success() || r.status().is_redirection() => {
                            TestResult::pass("OIDC issuer reachable").category(Category::Integration)
                        }
                        Ok(r) => TestResult::warn(
                            "OIDC issuer reachable",
                            format!("Issuer {} returned HTTP {}", issuer, r.status().as_u16()),
                        )
                        .category(Category::Integration)
                        .fix_hint("Check Crowd PUBLIC_BASE_URL matches reachable URL"),
                        Err(e) => TestResult::fail(
                            "OIDC issuer reachable",
                            format!("Cannot reach issuer {}: {}", issuer, e),
                        )
                        .category(Category::Integration)
                        .fix_hint("Set Crowd PUBLIC_BASE_URL to a reachable URL"),
                    }
                } else {
                    TestResult::fail("OIDC issuer reachable", format!("Invalid issuer: {}", issuer))
                        .category(Category::Integration)
                }
            } else {
                TestResult::fail(
                    "OIDC issuer reachable",
                    format!("Discovery HTTP {}", resp.status().as_u16()),
                )
                .category(Category::Integration)
            }
        }
        Err(e) => TestResult::fail("OIDC issuer reachable", format!("{}", e))
            .category(Category::Integration),
    }
}

/// Проверить Cross-Origin headers для статических ресурсов.
async fn check_cors_headers(config: &ServiceConfig) -> TestResult {
    let client = http_client();
    let svc = config.by_name("myjira").unwrap();
    let url = format!("http://{}:{}/pkg/myjira.js", config.base_ip, svc.port);

    match client.get(&url).send().await {
        Ok(resp) => {
            let headers = resp.headers();
            if headers.contains_key("access-control-allow-origin") {
                TestResult::pass("CORS headers on static assets").category(Category::Integration)
            } else {
                TestResult::warn(
                    "CORS headers on static assets",
                    "No CORS headers — cross-origin requests may fail",
                )
                .category(Category::Integration)
            }
        }
        Err(_) => TestResult::skip("CORS headers", "Service unreachable")
            .category(Category::Integration),
    }
}

/// Запустить все integration checks.
pub async fn run_all(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();

    results.push(check_suite_apps_links(config).await);
    results.extend(check_widget_endpoints(config).await);
    results.push(check_oidc_client_configs(config).await);
    results.push(check_cors_headers(config).await);

    results
}
