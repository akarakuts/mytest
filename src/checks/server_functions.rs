//! Проверки серверных функций: доступность endpoints, коды ответов.

use crate::checks::{Category, TestResult};
use crate::config::ServiceConfig;
use reqwest::Client;

fn http_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// Проверить доступность серверных функций для каждого сервиса.
pub async fn check_server_function_endpoints(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();
    let client = http_client();

    // Определяем серверные функции для проверки
    let server_fn_checks: Vec<(&str, Vec<(&str, &str)>)> = vec![
        // mychat: Video/Voice Call
        ("mychat", vec![
            ("initiate_call", "/api/initiate_call"),
            ("answer_call", "/api/answer_call"),
            ("reject_call", "/api/reject_call"),
            ("end_call", "/api/end_call"),
            ("call_signal", "/api/call_signal"),
        ]),
        // myflow: Variables, Scheduling
        ("myflow", vec![
            ("update_rule_variables", "/api/update_rule_variables"),
            ("set_schedule_preset", "/api/set_schedule_preset"),
            ("get_dry_run_trace", "/api/get_dry_run_trace"),
            ("list_schedule_presets", "/api/list_schedule_presets"),
        ]),
        // myalign: Milestones, Risks, Stakeholders
        ("myalign", vec![
            ("list_milestones", "/api/list_milestones"),
            ("create_milestone", "/api/create_milestone"),
            ("list_risks", "/api/list_risks"),
            ("create_risk", "/api/create_risk"),
            ("list_stakeholders", "/api/list_stakeholders"),
            ("create_stakeholder", "/api/create_stakeholder"),
            ("get_progress_report", "/api/get_progress_report"),
            ("list_epic_dependencies", "/api/list_epic_dependencies"),
            ("get_portfolio_report", "/api/get_portfolio_report"),
            ("get_cross_program_capacity", "/api/get_cross_program_capacity"),
        ]),
    ];

    for (svc_name, fns) in &server_fn_checks {
        // Находим сервис в конфиге
        let svc = config.services.iter().find(|s| s.name == *svc_name);
        let Some(svc) = svc else {
            results.push(
                TestResult::skip(
                    format!("{} server functions", svc_name),
                    "Service not in config",
                )
                .category(Category::Integration),
            );
            continue;
        };

        for (fn_name, endpoint) in fns {
            let url = format!("http://{}:{}{}", config.base_ip, svc.port, endpoint);
            match client.post(&url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    // Серверные функции должны принимать POST
                    // 200 = OK, 401/403 = нужна авторизация (но endpoint существует),
                    // 405 = Method Not Allowed (не POST), 422 = невалидный ввод
                    if status.is_success() || status.as_u16() == 401 || status.as_u16() == 403 || status.as_u16() == 422 {
                        results.push(
                            TestResult::pass(format!("{} {} endpoint", svc_name, fn_name))
                                .category(Category::Integration),
                        );
                    } else if status.as_u16() == 405 {
                        results.push(
                            TestResult::warn(
                                format!("{} {} endpoint", svc_name, fn_name),
                                format!("Method Not Allowed (405) — may need different method"),
                            )
                            .category(Category::Integration),
                        );
                    } else if status.as_u16() == 404 {
                        results.push(
                            TestResult::fail(
                                format!("{} {} endpoint", svc_name, fn_name),
                                format!("Not Found (404) — server function not registered"),
                            )
                            .category(Category::Integration)
                            .fix_hint(format!("Register {} in server/register_all.rs", fn_name)),
                        );
                    } else {
                        results.push(
                            TestResult::warn(
                                format!("{} {} endpoint", svc_name, fn_name),
                                format!("Unexpected status: {}", status),
                            )
                            .category(Category::Integration),
                        );
                    }
                }
                Err(e) => {
                    results.push(
                        TestResult::fail(
                            format!("{} {} endpoint", svc_name, fn_name),
                            format!("Connection error: {}", e),
                        )
                        .category(Category::Integration),
                    );
                }
            }
        }
    }

    results
}

/// Проверить доступность новых страниц для каждого сервиса.
pub async fn check_new_feature_pages(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();
    let client = http_client();

    // Определяем новые страницы для проверки
    let page_checks: Vec<(&str, Vec<&str>)> = vec![
        // mychat: новые страницы
        ("mychat", vec![
            "/search",
            "/files",
        ]),
        // myflow: новые страницы
        ("myflow", vec![
            "/",
            "/executions",
            "/login",
        ]),
        // myalign: новые страницы
        ("myalign", vec![
            "/",
            "/login",
        ]),
    ];

    for (svc_name, pages) in &page_checks {
        let svc = config.services.iter().find(|s| s.name == *svc_name);
        let Some(svc) = svc else {
            results.push(
                TestResult::skip(
                    format!("{} feature pages", svc_name),
                    "Service not in config",
                )
                .category(Category::Ui),
            );
            continue;
        };

        for page in pages {
            let url = format!("http://{}:{}{}", config.base_ip, svc.port, page);
            match client.get(&url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        results.push(
                            TestResult::pass(format!("{} {} page", svc_name, page))
                                .category(Category::Ui),
                        );
                    } else if status.as_u16() == 302 || status.as_u16() == 303 {
                        // Redirect — likely to login page (expected for protected pages)
                        results.push(
                            TestResult::pass(format!("{} {} page (redirect)", svc_name, page))
                                .category(Category::Ui),
                        );
                    } else if status.as_u16() == 404 {
                        results.push(
                            TestResult::fail(
                                format!("{} {} page", svc_name, page),
                                "Page not found (404)",
                            )
                            .category(Category::Ui)
                            .fix_hint("Add route in app.rs"),
                        );
                    } else {
                        results.push(
                            TestResult::warn(
                                format!("{} {} page", svc_name, page),
                                format!("Status: {}", status),
                            )
                            .category(Category::Ui),
                        );
                    }
                }
                Err(e) => {
                    results.push(
                        TestResult::fail(
                            format!("{} {} page", svc_name, page),
                            format!("Connection error: {}", e),
                        )
                        .category(Category::Ui),
                    );
                }
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_function_endpoints() {
        let config = crate::config::load_config().unwrap();
        let results = check_server_function_endpoints(&config).await;
        for r in &results {
            if r.status == crate::checks::Status::Fail {
                eprintln!("FAIL: {} - {}", r.name, r.detail.as_deref().unwrap_or(""));
            }
        }
        // At least some endpoints should exist
        let passed = results.iter().filter(|r| r.status == crate::checks::Status::Pass).count();
        assert!(passed > 0, "No server function endpoints found");
    }

    #[tokio::test]
    async fn test_new_feature_pages() {
        let config = crate::config::load_config().unwrap();
        let results = check_new_feature_pages(&config).await;
        for r in &results {
            if r.status == crate::checks::Status::Fail {
                eprintln!("FAIL: {} - {}", r.name, r.detail.as_deref().unwrap_or(""));
            }
        }
        // At least some pages should exist
        let passed = results.iter().filter(|r| r.status == crate::checks::Status::Pass).count();
        assert!(passed > 0, "No new feature pages found");
    }
}
