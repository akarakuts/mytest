//! Comprehensive server function and page tests for ALL 27 services.

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

/// All server function endpoints for each service.
fn all_server_functions() -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
    vec![
        // mycrowd - Identity/SSO
        ("mycrowd", vec![
            ("get_me", "/api/get_me"),
            ("list_users", "/api/list_users"),
            ("list_groups", "/api/list_groups"),
            ("list_applications", "/api/list_applications"),
            ("list_sessions", "/api/list_sessions"),
            ("list_audit_events", "/api/list_audit_events"),
            ("list_directories", "/api/list_directories"),
            ("get_oidc_config", "/api/get_oidc_config"),
        ]),
        // myconf - Knowledge Base
        ("myconf", vec![
            ("get_me", "/api/get_me"),
            ("list_spaces", "/api/list_spaces"),
            ("list_pages", "/api/list_pages"),
            ("search_pages", "/api/search_pages"),
            ("get_page_history", "/api/get_page_history"),
            ("list_templates", "/api/list_templates"),
            ("list_recent_pages", "/api/list_recent_pages"),
            ("get_page_restrictions", "/api/get_page_restrictions"),
        ]),
        // myjira - Issue Tracker
        ("myjira", vec![
            ("get_me", "/api/get_me"),
            ("list_projects", "/api/list_projects"),
            ("list_issues", "/api/list_issues"),
            ("search_issues", "/api/search_issues"),
            ("list_boards", "/api/list_boards"),
            ("list_sprints", "/api/list_sprints"),
            ("get_dashboard", "/api/get_dashboard"),
            ("list_filters", "/api/list_filters"),
            ("list_workflows", "/api/list_workflows"),
            ("list_custom_fields", "/api/list_custom_fields"),
        ]),
        // mybitbucket - Git Hosting
        ("mybitbucket", vec![
            ("get_me", "/api/get_me"),
            ("list_repos", "/api/list_repos"),
            ("list_pull_requests", "/api/list_pull_requests"),
            ("list_branches", "/api/list_branches"),
            ("list_commits", "/api/list_commits"),
            ("get_dashboard", "/api/get_dashboard"),
            ("list_webhooks", "/api/list_webhooks"),
        ]),
        // mybamboo - CI/CD
        ("mybamboo", vec![
            ("get_me", "/api/get_me"),
            ("list_plans", "/api/list_plans"),
            ("list_builds", "/api/list_builds"),
            ("list_deployments", "/api/list_deployments"),
            ("list_environments", "/api/list_environments"),
            ("list_artifacts", "/api/list_artifacts"),
        ]),
        // myportal - Suite Portal
        ("myportal", vec![
            ("get_me", "/api/get_me"),
            ("get_suite_apps", "/api/get_suite_apps"),
            ("get_activity_feed", "/api/get_activity_feed"),
            ("get_status_widget", "/api/get_status_widget"),
        ]),
        // myopsgenie - Alerts
        ("myopsgenie", vec![
            ("get_me", "/api/get_me"),
            ("list_alerts", "/api/list_alerts"),
            ("list_incidents", "/api/list_incidents"),
            ("list_schedules", "/api/list_schedules"),
            ("list_escalations", "/api/list_escalations"),
            ("list_heartbeats", "/api/list_heartbeats"),
            ("list_policies", "/api/list_policies"),
            ("list_dedup_policies", "/api/list_dedup_policies"),
            ("list_notification_policies", "/api/list_notification_policies"),
        ]),
        // myservicedesk - Service Desk
        ("myservicedesk", vec![
            ("get_me", "/api/get_me"),
            ("list_queues", "/api/list_queues"),
            ("list_requests", "/api/list_requests"),
            ("list_slas", "/api/list_slas"),
            ("list_automation_rules", "/api/list_automation_rules"),
            ("list_approval_rules", "/api/list_approval_rules"),
            ("list_assets", "/api/list_assets"),
            ("list_knowledge_base", "/api/list_knowledge_base"),
            ("get_customer_portal", "/api/get_customer_portal"),
        ]),
        // mycompass - Service Catalog
        ("mycompass", vec![
            ("get_me", "/api/get_me"),
            ("list_components", "/api/list_components"),
            ("list_scorecards", "/api/list_scorecards"),
            ("list_teams", "/api/list_teams"),
            ("list_goals", "/api/list_goals"),
            ("list_surveys", "/api/list_surveys"),
            ("list_audit_events", "/api/list_audit_events"),
        ]),
        // mycalendars - Team Calendars
        ("mycalendars", vec![
            ("get_me", "/api/get_me"),
            ("list_events", "/api/list_events"),
            ("list_calendars", "/api/list_calendars"),
            ("list_holidays", "/api/list_holidays"),
            ("list_leaves", "/api/list_leaves"),
            ("find_time_slots", "/api/find_time_slots"),
            ("list_subscriptions", "/api/list_subscriptions"),
        ]),
        // mystatuspage - Status Page
        ("mystatuspage", vec![
            ("get_me", "/api/get_me"),
            ("list_services", "/api/list_services"),
            ("list_incidents", "/api/list_incidents"),
            ("list_monitors", "/api/list_monitors"),
            ("list_uptime", "/api/list_uptime"),
            ("get_status_summary", "/api/get_status_summary"),
        ]),
        // mycrm - CRM
        ("mycrm", vec![
            ("get_me", "/api/get_me"),
            ("list_contacts", "/api/list_contacts"),
            ("list_deals", "/api/list_deals"),
            ("list_pipelines", "/api/list_pipelines"),
            ("list_invoices", "/api/list_invoices"),
            ("list_activities", "/api/list_activities"),
        ]),
        // myrovo - AI Assistant
        ("myrovo", vec![
            ("get_me", "/api/get_me"),
            ("rovo_enabled", "/api/rovo_enabled"),
            ("ask_rovo", "/api/ask_rovo"),
            ("get_conversations", "/api/get_conversations"),
            ("get_tools", "/api/get_tools"),
        ]),
        // myanalytics - BI Dashboards
        ("myanalytics", vec![
            ("get_me", "/api/get_me"),
            ("list_dashboards", "/api/list_dashboards"),
            ("list_widgets", "/api/list_widgets"),
            ("list_reports", "/api/list_reports"),
            ("get_dora_metrics", "/api/get_dora_metrics"),
        ]),
        // mymarketplace - Plugin Catalog
        ("mymarketplace", vec![
            ("get_me", "/api/get_me"),
            ("list_plugins", "/api/list_plugins"),
            ("list_products", "/api/list_products"),
            ("get_plugin_detail", "/api/get_plugin_detail"),
            ("list_installed_plugins", "/api/list_installed_plugins"),
        ]),
        // myalign - Jira Align
        ("myalign", vec![
            ("get_me", "/api/get_me"),
            ("list_programs", "/api/list_programs"),
            ("get_program_detail", "/api/get_program_detail"),
            ("list_milestones", "/api/list_milestones"),
            ("list_risks", "/api/list_risks"),
            ("list_stakeholders", "/api/list_stakeholders"),
            ("get_progress_report", "/api/get_progress_report"),
            ("list_epic_dependencies", "/api/list_epic_dependencies"),
            ("get_portfolio_report", "/api/get_portfolio_report"),
            ("get_cross_program_capacity", "/api/get_cross_program_capacity"),
            ("list_strategic_themes", "/api/list_strategic_themes"),
        ]),
        // mynotifications - Notification Hub
        ("mynotifications", vec![
            ("get_me", "/api/get_me"),
            ("list_notifications", "/api/list_notifications"),
            ("list_channels", "/api/list_channels"),
            ("get_unread_count", "/api/get_unread_count"),
            ("list_delivery_log", "/api/list_delivery_log"),
            ("get_delivery_stats", "/api/get_delivery_stats"),
        ]),
        // mychat - Team Messenger
        ("mychat", vec![
            ("get_me", "/api/get_me"),
            ("list_channels", "/api/list_channels"),
            ("list_messages", "/api/list_messages"),
            ("search_messages", "/api/search_messages"),
            ("list_pinned_messages", "/api/list_pinned_messages"),
            ("list_saved_messages", "/api/list_saved_messages"),
            ("list_files", "/api/list_files"),
            ("initiate_call", "/api/initiate_call"),
            ("answer_call", "/api/answer_call"),
            ("reject_call", "/api/reject_call"),
            ("end_call", "/api/end_call"),
            ("call_signal", "/api/call_signal"),
        ]),
        // mytrello - Kanban Boards
        ("mytrello", vec![
            ("get_me", "/api/get_me"),
            ("list_boards", "/api/list_boards"),
            ("list_cards", "/api/list_cards"),
            ("list_labels", "/api/list_labels"),
            ("list_checklists", "/api/list_checklists"),
            ("get_board_activity", "/api/get_board_activity"),
        ]),
        // mydiscovery - Idea Funnel
        ("mydiscovery", vec![
            ("get_me", "/api/get_me"),
            ("list_ideas", "/api/list_ideas"),
            ("list_fields", "/api/list_fields"),
            ("list_views", "/api/list_views"),
            ("get_roadmap", "/api/get_roadmap"),
            ("get_scoring", "/api/get_scoring"),
        ]),
        // myatlas - Team Updates
        ("myatlas", vec![
            ("get_me", "/api/get_me"),
            ("list_projects", "/api/list_projects"),
            ("list_updates", "/api/list_updates"),
            ("list_templates", "/api/list_templates"),
            ("list_goals", "/api/list_goals"),
            ("get_weekly_digest", "/api/get_weekly_digest"),
            ("list_smart_links", "/api/list_smart_links"),
        ]),
        // myflow - Automation
        ("myflow", vec![
            ("get_me", "/api/get_me"),
            ("list_rules", "/api/list_rules"),
            ("list_executions", "/api/list_executions"),
            ("list_templates", "/api/list_templates"),
            ("get_analytics", "/api/get_analytics"),
            ("list_shared_rules", "/api/list_shared_rules"),
            ("update_rule_variables", "/api/update_rule_variables"),
            ("set_schedule_preset", "/api/set_schedule_preset"),
            ("get_dry_run_trace", "/api/get_dry_run_trace"),
            ("list_schedule_presets", "/api/list_schedule_presets"),
        ]),
        // mysearch - Global Search
        ("mysearch", vec![
            ("get_me", "/api/get_me"),
            ("global_search", "/api/global_search"),
            ("list_saved_searches", "/api/list_saved_searches"),
            ("get_search_history", "/api/get_search_history"),
            ("get_search_analytics", "/api/get_search_analytics"),
        ]),
        // myjam - Whiteboard
        ("myjam", vec![
            ("get_me", "/api/get_me"),
            ("list_boards", "/api/list_boards"),
            ("get_board_items", "/api/get_board_items"),
            ("list_templates", "/api/list_templates"),
            ("export_board_svg", "/api/export_board_svg"),
        ]),
        // myrunbook - Incident Runbooks
        ("myrunbook", vec![
            ("get_me", "/api/get_me"),
            ("list_runbooks", "/api/list_runbooks"),
            ("list_runs", "/api/list_runs"),
            ("list_escalations", "/api/list_escalations"),
            ("get_run_analytics", "/api/get_run_analytics"),
        ]),
        // mytimesheets - Time Tracking
        ("mytimesheets", vec![
            ("get_me", "/api/get_me"),
            ("list_entries", "/api/list_entries"),
            ("list_reports", "/api/list_reports"),
            ("list_invoices", "/api/list_invoices"),
            ("list_approvals", "/api/list_approvals"),
            ("get_utilization", "/api/get_utilization"),
            ("list_budgets", "/api/list_budgets"),
        ]),
        // myforms - Form Builder
        ("myforms", vec![
            ("get_me", "/api/get_me"),
            ("list_forms", "/api/list_forms"),
            ("list_submissions", "/api/list_submissions"),
            ("list_templates", "/api/list_templates"),
            ("get_analytics", "/api/get_analytics"),
            ("list_conditions", "/api/list_conditions"),
        ]),
    ]
}

/// All feature pages for each service.
fn all_feature_pages() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("mycrowd", vec!["/", "/login", "/users", "/groups", "/applications", "/directories", "/sessions", "/audit", "/compliance", "/api_keys", "/secrets", "/offboard", "/dashboard"]),
        ("myconf", vec!["/", "/login", "/search"]),
        ("myjira", vec!["/", "/login", "/backlog", "/board", "/dashboard"]),
        ("mybitbucket", vec!["/", "/login", "/dashboard"]),
        ("mybamboo", vec!["/", "/login", "/plans", "/deployments"]),
        ("myportal", vec!["/", "/login", "/admin", "/admin/dashboard", "/performance"]),
        ("myopsgenie", vec!["/", "/login", "/alerts", "/incidents", "/schedules", "/escalations", "/heartbeats", "/dashboard", "/notification-policies", "/incident-timeline"]),
        ("myservicedesk", vec!["/", "/login", "/portal", "/queue", "/dashboard", "/sla", "/automation", "/assets", "/knowledge-base", "/customer-portal", "/approvals", "/branding", "/business-hours", "/email-routes", "/agent-performance"]),
        ("mycompass", vec!["/", "/login", "/catalog", "/scorecards", "/teams", "/goals", "/surveys", "/audit", "/templates", "/webhooks", "/map"]),
        ("mycalendars", vec!["/", "/login", "/month", "/leaves", "/holidays", "/find-time", "/settings", "/subscriptions", "/advanced"]),
        ("mystatuspage", vec!["/", "/login", "/admin", "/dashboard", "/incidents", "/uptime"]),
        ("mycrm", vec!["/", "/login", "/contacts", "/deals", "/pipelines", "/invoices", "/reports"]),
        ("myrovo", vec!["/", "/login", "/conversations", "/tools", "/settings"]),
        ("myanalytics", vec!["/", "/login", "/dashboard", "/reports", "/reports-gallery", "/dora", "/admin"]),
        ("mymarketplace", vec!["/", "/login", "/catalog", "/publish", "/sdk", "/plugin"]),
        ("myalign", vec!["/", "/login"]),
        ("mynotifications", vec!["/", "/login", "/inbox", "/channels", "/settings"]),
        ("mychat", vec!["/", "/login", "/admin", "/search", "/files"]),
        ("mytrello", vec!["/", "/login", "/boards"]),
        ("mydiscovery", vec!["/", "/login", "/ideas", "/roadmap", "/backlog"]),
        ("myatlas", vec!["/", "/login", "/projects", "/updates", "/templates", "/goals", "/digest"]),
        ("myflow", vec!["/", "/login", "/executions"]),
        ("mysearch", vec!["/", "/login", "/history"]),
        ("myjam", vec!["/", "/login", "/boards"]),
        ("myrunbook", vec!["/", "/login", "/runbooks", "/analytics"]),
        ("mytimesheets", vec!["/", "/login", "/timesheet", "/reports", "/invoices", "/calendar", "/utilization", "/approvals"]),
        ("myforms", vec!["/", "/login", "/forms", "/templates", "/analytics"]),
    ]
}

/// Test all server function endpoints.
pub async fn test_all_server_functions(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();
    let client = http_client();

    for (svc_name, fns) in all_server_functions() {
        let svc = config.services.iter().find(|s| s.name == svc_name);
        let Some(svc) = svc else {
            for (fn_name, _) in fns {
                results.push(
                    TestResult::skip(format!("{} {}", svc_name, fn_name), "Service not in config")
                        .category(Category::Integration),
                );
            }
            continue;
        };

        for (fn_name, endpoint) in fns {
            let url = format!("http://{}:{}{}", config.base_ip, svc.port, endpoint);
            match client.post(&url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() || status.as_u16() == 401 || status.as_u16() == 403 || status.as_u16() == 422 {
                        results.push(
                            TestResult::pass(format!("{} {}", svc_name, fn_name))
                                .category(Category::Integration),
                        );
                    } else if status.as_u16() == 404 {
                        results.push(
                            TestResult::fail(format!("{} {}", svc_name, fn_name), "Not Found (404)")
                                .category(Category::Integration)
                                .fix_hint(format!("Register {} server function", fn_name)),
                        );
                    } else {
                        results.push(
                            TestResult::warn(format!("{} {}", svc_name, fn_name), format!("Status: {}", status))
                                .category(Category::Integration),
                        );
                    }
                }
                Err(e) => {
                    results.push(
                        TestResult::fail(format!("{} {}", svc_name, fn_name), format!("Error: {}", e))
                            .category(Category::Integration),
                    );
                }
            }
        }
    }

    results
}

/// Test all feature pages.
pub async fn test_all_feature_pages(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();
    let client = http_client();

    for (svc_name, pages) in all_feature_pages() {
        let svc = config.services.iter().find(|s| s.name == svc_name);
        let Some(svc) = svc else {
            for page in pages {
                results.push(
                    TestResult::skip(format!("{} {}", svc_name, page), "Service not in config")
                        .category(Category::Ui),
                );
            }
            continue;
        };

        for page in pages {
            let url = format!("http://{}:{}{}", config.base_ip, svc.port, page);
            match client.get(&url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() || status.as_u16() == 302 || status.as_u16() == 303 {
                        results.push(
                            TestResult::pass(format!("{} {} page", svc_name, page))
                                .category(Category::Ui),
                        );
                    } else if status.as_u16() == 404 {
                        results.push(
                            TestResult::fail(format!("{} {} page", svc_name, page), "Not Found (404)")
                                .category(Category::Ui)
                                .fix_hint("Add route in app.rs"),
                        );
                    } else {
                        results.push(
                            TestResult::warn(format!("{} {} page", svc_name, page), format!("Status: {}", status))
                                .category(Category::Ui),
                        );
                    }
                }
                Err(e) => {
                    results.push(
                        TestResult::fail(format!("{} {} page", svc_name, page), format!("Error: {}", e))
                            .category(Category::Ui),
                    );
                }
            }
        }
    }

    results
}

/// Test widget endpoints for all services.
pub async fn test_all_widgets(config: &ServiceConfig) -> Vec<TestResult> {
    let mut results = Vec::new();
    let client = http_client();

    for svc in &config.services {
        let url = format!("http://{}:{}/api/v1/widget", config.base_ip, svc.port);
        match client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    results.push(
                        TestResult::pass(format!("{} widget", svc.name))
                            .category(Category::Integration),
                    );
                } else if status.as_u16() == 405 {
                    results.push(
                        TestResult::pass(format!("{} widget (405)", svc.name))
                            .category(Category::Integration),
                    );
                } else if status.as_u16() == 404 {
                    results.push(
                        TestResult::warn(format!("{} widget", svc.name), "Widget endpoint not found")
                            .category(Category::Integration),
                    );
                } else {
                    results.push(
                        TestResult::warn(format!("{} widget", svc.name), format!("Status: {}", status))
                            .category(Category::Integration),
                    );
                }
            }
            Err(e) => {
                results.push(
                    TestResult::fail(format!("{} widget", svc.name), format!("Error: {}", e))
                        .category(Category::Integration),
                );
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_functions_count() {
        let fns = all_server_functions();
        let total: usize = fns.iter().map(|(_, v)| v.len()).sum();
        assert!(total >= 200, "Should have at least 200 server function tests, got {}", total);
    }

    #[tokio::test]
    async fn test_feature_pages_count() {
        let pages = all_feature_pages();
        let total: usize = pages.iter().map(|(_, v)| v.len()).sum();
        assert!(total >= 150, "Should have at least 150 page tests, got {}", total);
    }
}
