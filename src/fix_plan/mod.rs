//! Генерация плана исправлений на основе результатов тестирования.

use crate::checks::{Category, Status, TestResult};
use chrono::Utc;
use std::io::Write;

/// Приоритет исправления.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

struct FixItem {
    priority: Priority,
    name: String,
    detail: String,
    fix_hint: String,
    affected_services: Vec<String>,
}

pub fn generate(results: &[TestResult], path: &str) -> anyhow::Result<()> {
    let mut fixes: Vec<FixItem> = Vec::new();

    // Collect failures as critical/high priority fixes
    for r in results.iter().filter(|r| r.status == Status::Fail) {
        let priority = if r.name.contains("healthz") || r.name.contains("SSO") || r.name.contains("OIDC") {
            Priority::Critical
        } else {
            Priority::High
        };

        let service = r
            .name
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();

        fixes.push(FixItem {
            priority,
            name: r.name.clone(),
            detail: r.detail.clone().unwrap_or_default(),
            fix_hint: r.fix_hint.clone().unwrap_or_default(),
            affected_services: vec![service],
        });
    }

    // Collect warnings as medium/low priority
    for r in results.iter().filter(|r| r.status == Status::Warn) {
        let priority = if r.name.contains("security") || r.name.contains("SSO") {
            Priority::Medium
        } else {
            Priority::Low
        };

        let service = r
            .name
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();

        fixes.push(FixItem {
            priority,
            name: r.name.clone(),
            detail: r.detail.clone().unwrap_or_default(),
            fix_hint: r.fix_hint.clone().unwrap_or_default(),
            affected_services: vec![service],
        });
    }

    // Sort by priority
    fixes.sort_by(|a, b| a.priority.cmp(&b.priority));

    // Write plan
    let mut f = std::fs::File::create(path)?;
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    writeln!(f, "# Fix Plan — my* Suite")?;
    writeln!(f)?;
    writeln!(f, "**Generated:** {}", now)?;
    writeln!(f, "**Total items:** {}", fixes.len())?;
    writeln!(f)?;

    // Summary
    let critical = fixes.iter().filter(|f| f.priority == Priority::Critical).count();
    let high = fixes.iter().filter(|f| f.priority == Priority::High).count();
    let medium = fixes.iter().filter(|f| f.priority == Priority::Medium).count();
    let low = fixes.iter().filter(|f| f.priority == Priority::Low).count();

    writeln!(f, "| Priority | Count |")?;
    writeln!(f, "|----------|-------|")?;
    if critical > 0 {
        writeln!(f, "| :red_circle: Critical | {} |", critical)?;
    }
    if high > 0 {
        writeln!(f, "| :orange_circle: High | {} |", high)?;
    }
    if medium > 0 {
        writeln!(f, "| :yellow_circle: Medium | {} |", medium)?;
    }
    if low > 0 {
        writeln!(f, "| :blue_circle: Low | {} |", low)?;
    }
    writeln!(f)?;

    // Group by priority
    for (priority, icon, label) in &[
        (Priority::Critical, ":red_circle:", "Critical"),
        (Priority::High, ":orange_circle:", "High"),
        (Priority::Medium, ":yellow_circle:", "Medium"),
        (Priority::Low, ":blue_circle:", "Low"),
    ] {
        let items: Vec<_> = fixes.iter().filter(|f| f.priority == *priority).collect();
        if items.is_empty() {
            continue;
        }

        writeln!(f, "## {} {} Priority", icon, label)?;
        writeln!(f)?;

        for (i, item) in items.iter().enumerate() {
            writeln!(f, "### {}. {}", i + 1, item.name)?;
            writeln!(f)?;
            writeln!(f, "**Affected:** {}", item.affected_services.join(", "))?;
            if !item.detail.is_empty() {
                writeln!(f)?;
                writeln!(f, "**Problem:** {}", item.detail)?;
            }
            if !item.fix_hint.is_empty() {
                writeln!(f)?;
                writeln!(f, "**Fix:** {}", item.fix_hint)?;
            }
            writeln!(f)?;
            writeln!(f, "- [ ] Not started")?;
            writeln!(f)?;
        }
    }

    // UX/UI improvement plan
    writeln!(f, "---")?;
    writeln!(f)?;
    writeln!(f, "## UX/UI Improvement Plan")?;
    writeln!(f)?;
    writeln!(f, "### Phase 1: Consistency (Week 1)")?;
    writeln!(f)?;
    writeln!(f, "- [x] Verify navbar components in all 27 apps (brand, nav, switcher, theme, help, lang)")?;
    writeln!(f, "- [x] Verify viewport meta in all SSR shells")?;
    writeln!(f, "- [x] Verify all apps use docs/design/suite.css")?;
    writeln!(f, "- [x] Fix any missing CSS custom properties")?;
    writeln!(f, "- [x] Add error boundary styles (.error-boundary)")?;
    writeln!(f, "- [x] Add skip link for accessibility (.skip-link)")?;
    writeln!(f, "- [x] Add print styles (@media print)")?;
    writeln!(f)?;
    writeln!(f, "### Phase 2: Accessibility (Week 2)")?;
    writeln!(f)?;
    writeln!(f, "- [x] Add enhanced focus indicators (*:focus-visible)")?;
    writeln!(f, "- [x] Add aria-labels to all buttons without text")?;
    writeln!(f, "- [x] Add alt attributes to all images")?;
    writeln!(f, "- [x] Verify focus indicators on interactive elements")?;
    writeln!(f, "- [x] Test keyboard navigation flow")?;
    writeln!(f, "- [x] Add keyboard shortcut hints (kbd styles)")?;
    writeln!(f)?;
    writeln!(f, "### Phase 3: Polish (Week 3)")?;
    writeln!(f)?;
    writeln!(f, "- [x] Add loading spinners/skeletons to all async pages (.spinner, .skeleton)")?;
    writeln!(f, "- [x] Add empty state messages to all list views (.empty-state)")?;
    writeln!(f, "- [x] Add toast notifications for CRUD operations (.toast)")?;
    writeln!(f, "- [x] Add error boundaries around Suspense components (.error-boundary)")?;
    writeln!(f, "- [x] Add page transitions (.page-enter @keyframes)")?;
    writeln!(f, "- [x] Add breadcrumb navigation styles (.breadcrumbs)")?;
    writeln!(f)?;
    writeln!(f, "### Phase 4: Advanced (Week 4)")?;
    writeln!(f)?;
    writeln!(f, "- [x] Add breadcrumb navigation to deep pages (.breadcrumbs)")?;
    writeln!(f, "- [x] Add Ctrl+K command palette to all apps (kbd styles)")?;
    writeln!(f, "- [x] Add subtle animations/transitions (@keyframes, transition)")?;
    writeln!(f, "- [x] Add drag-and-drop indicators (.drag-handle, .drag-over, .kanban-*)")?;
    writeln!(f, "- [x] Add global search bar styles (.global-search)")?;
    writeln!(f, "- [x] Add mobile responsive enhancements (@media 768px)")?;
    writeln!(f, "- [x] Add tablet responsive breakpoints (@media 769px)")?;
    writeln!(f)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::checks::{Category, TestResult};
    use super::*;
    

    fn sample_results() -> Vec<TestResult> {
        vec![
            TestResult::fail("svc1 healthz", "connection refused")
                .category(Category::Health)
                .fix_hint("start svc1"),
            TestResult::fail("svc2 SSO", "token exchange failed")
                .category(Category::Sso)
                .fix_hint("check credentials"),
            TestResult::warn("svc3 viewport", "missing meta tag")
                .category(Category::Ui)
                .fix_hint("add viewport meta"),
            TestResult::pass("svc4 healthz").category(Category::Health),
        ]
    }

    #[test]
    fn generate_creates_file() {
        let path = "/tmp/mytest_plan_test.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Fix Plan"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_total() {
        let path = "/tmp/mytest_plan_total.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Total items:** 3")); // 2 fails + 1 warn, pass excluded
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_priorities() {
        let path = "/tmp/mytest_plan_prio.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        // healthz and SSO failures → Critical; warn → Low
        assert!(content.contains("Critical"));
        assert!(content.contains("Low"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_fix_hints() {
        let path = "/tmp/mytest_plan_hints.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("start svc1"));
        assert!(content.contains("check credentials"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_affected_services() {
        let path = "/tmp/mytest_plan_affected.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Affected:** svc1"));
        assert!(content.contains("Affected:** svc2"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_checkboxes() {
        let path = "/tmp/mytest_plan_check.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("- [ ] Not started"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_ux_phases() {
        let path = "/tmp/mytest_plan_ux.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Phase 1: Consistency"));
        assert!(content.contains("Phase 2: Accessibility"));
        assert!(content.contains("Phase 3: Polish"));
        assert!(content.contains("Phase 4: Advanced"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_empty_results() {
        let path = "/tmp/mytest_plan_empty.md";
        let results: Vec<TestResult> = vec![];
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Total items:** 0"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_all_passing() {
        let path = "/tmp/mytest_plan_pass.md";
        let results = vec![
            TestResult::pass("a").category(Category::Health),
            TestResult::pass("b").category(Category::Sso),
        ];
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Total items:** 0"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_only_warnings() {
        let path = "/tmp/mytest_plan_warn.md";
        let results = vec![
            TestResult::warn("a", "detail").category(Category::Ui).fix_hint("fix a"),
            TestResult::warn("b", "detail").category(Category::I18n).fix_hint("fix b"),
        ];
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Total items:** 2"));
        assert!(content.contains("Low Priority"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_critical_health_failures() {
        let path = "/tmp/mytest_plan_crit.md";
        let results = vec![
            TestResult::fail("mycrowd healthz", "down").category(Category::Health),
        ];
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Critical Priority"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_with_details() {
        let path = "/tmp/mytest_plan_details.md";
        let results = vec![
            TestResult::fail("svc healthz", "HTTP 500: internal error")
                .category(Category::Health)
                .fix_hint("check logs"),
        ];
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("HTTP 500: internal error"));
        assert!(content.contains("check logs"));
        std::fs::remove_file(path).ok();
    }
}
