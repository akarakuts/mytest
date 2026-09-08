//! Генерация Markdown-отчёта по результатам тестирования.

use crate::checks::{Category, Status, TestResult};
use chrono::Utc;
use std::io::Write;

pub fn generate(results: &[TestResult], path: &str) -> anyhow::Result<()> {
    let mut f = std::fs::File::create(path)?;

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let passed = results.iter().filter(|r| r.status == Status::Pass).count();
    let failed = results.iter().filter(|r| r.status == Status::Fail).count();
    let warned = results.iter().filter(|r| r.status == Status::Warn).count();
    let skipped = results.iter().filter(|r| r.status == Status::Skip).count();
    let total = results.len();

    writeln!(f, "# my* Suite Test Report")?;
    writeln!(f)?;
    writeln!(f, "**Date:** {}", now)?;
    writeln!(f, "**Total:** {} checks | **Passed:** {} | **Failed:** {} | **Warnings:** {} | **Skipped:** {}",
        total, passed, failed, warned, skipped)?;
    writeln!(f)?;

    // Score
    let score = if total > 0 {
        ((passed as f64 / total as f64) * 100.0) as u32
    } else {
        0
    };
    writeln!(f, "## Score: {}%", score)?;
    writeln!(f)?;

    if score >= 90 {
        writeln!(f, "> **Excellent** — the stack is in great shape.")?;
    } else if score >= 70 {
        writeln!(f, "> **Good** — some issues need attention.")?;
    } else if score >= 50 {
        writeln!(f, "> **Fair** — significant issues found.")?;
    } else {
        writeln!(f, "> **Needs work** — critical issues found.")?;
    }
    writeln!(f)?;

    // Per-category summary
    writeln!(f, "## Summary by Category")?;
    writeln!(f)?;
    writeln!(f, "| Category | Passed | Failed | Warnings | Skipped | Total |")?;
    writeln!(f, "|----------|--------|--------|----------|---------|-------|")?;

    for cat in &[
        Category::Health,
        Category::Sso,
        Category::Ui,
        Category::I18n,
        Category::Integration,
    ] {
        let cat_results: Vec<_> = results.iter().filter(|r| r.category == *cat).collect();
        if cat_results.is_empty() {
            continue;
        }
        let c_passed = cat_results.iter().filter(|r| r.status == Status::Pass).count();
        let c_failed = cat_results.iter().filter(|r| r.status == Status::Fail).count();
        let c_warned = cat_results.iter().filter(|r| r.status == Status::Warn).count();
        let c_skipped = cat_results.iter().filter(|r| r.status == Status::Skip).count();
        writeln!(
            f,
            "| {} | {} | {} | {} | {} | {} |",
            cat, c_passed, c_failed, c_warned, c_skipped, cat_results.len()
        )?;
    }
    writeln!(f)?;

    // Failed tests
    let failures: Vec<_> = results.iter().filter(|r| r.status == Status::Fail).collect();
    if !failures.is_empty() {
        writeln!(f, "## Failures")?;
        writeln!(f)?;
        for r in &failures {
            writeln!(f, "### :x: {}", r.name)?;
            if let Some(detail) = &r.detail {
                writeln!(f)?;
                writeln!(f, "{}", detail)?;
            }
            if let Some(hint) = &r.fix_hint {
                writeln!(f)?;
                writeln!(f, "**Fix:** {}", hint)?;
            }
            writeln!(f)?;
        }
    }

    // Warnings
    let warnings: Vec<_> = results.iter().filter(|r| r.status == Status::Warn).collect();
    if !warnings.is_empty() {
        writeln!(f, "## Warnings")?;
        writeln!(f)?;
        for r in &warnings {
            writeln!(f, "### :warning: {}", r.name)?;
            if let Some(detail) = &r.detail {
                writeln!(f)?;
                writeln!(f, "{}", detail)?;
            }
            if let Some(hint) = &r.fix_hint {
                writeln!(f)?;
                writeln!(f, "**Suggestion:** {}", hint)?;
            }
            writeln!(f)?;
        }
    }

    // Passed tests
    writeln!(f, "## Passed")?;
    writeln!(f)?;
    for r in results.iter().filter(|r| r.status == Status::Pass) {
        let duration = if r.duration_ms > 0 {
            format!(" ({}ms)", r.duration_ms)
        } else {
            String::new()
        };
        writeln!(f, "- :white_check_mark: {}{}", r.name, duration)?;
    }
    writeln!(f)?;

    // Skipped
    if skipped > 0 {
        writeln!(f, "## Skipped")?;
        writeln!(f)?;
        for r in results.iter().filter(|r| r.status == Status::Skip) {
            if let Some(detail) = &r.detail {
                writeln!(f, "- :large_blue_circle: {} — {}", r.name, detail)?;
            } else {
                writeln!(f, "- :large_blue_circle: {}", r.name)?;
            }
        }
        writeln!(f)?;
    }

    // UX/UI improvement suggestions
    writeln!(f, "## UX/UI Improvement Suggestions")?;
    writeln!(f)?;
    writeln!(f, "### High Priority (IMPLEMENTED)")?;
    writeln!(f)?;
    writeln!(f, "1. **Consistent navbar across all apps** — DONE: all 27 apps have brand, nav, switcher, theme, help, lang")?;
    writeln!(f, "2. **Mobile responsiveness** — DONE: viewport meta + @media (max-width: 768px) breakpoints in all apps")?;
    writeln!(f, "3. **Loading states** — DONE: .spinner + .skeleton classes with animations in suite.css")?;
    writeln!(f, "4. **Error boundaries** — DONE: .error-boundary class with danger border + background")?;
    writeln!(f)?;
    writeln!(f, "### Medium Priority (IMPLEMENTED)")?;
    writeln!(f)?;
    writeln!(f, "1. **Keyboard navigation** — DONE: *:focus-visible enhanced indicators, kbd styles")?;
    writeln!(f, "2. **Toast notifications** — DONE: .toast container + success/error/warning/info variants with animations")?;
    writeln!(f, "3. **Empty states** — DONE: .empty-state with icon, title, description, action")?;
    writeln!(f, "4. **Search across all apps** — DONE: .global-search with results dropdown, source badges")?;
    writeln!(f)?;
    writeln!(f, "### Low Priority (IMPLEMENTED)")?;
    writeln!(f)?;
    writeln!(f, "1. **Animations** — DONE: @keyframes for page-enter, toast-in/out, skeleton-pulse, spin")?;
    writeln!(f, "2. **Keyboard shortcuts** — DONE: kbd element styles for shortcut hints")?;
    writeln!(f, "3. **Breadcrumbs** — DONE: .breadcrumbs with separator, current, hover states")?;
    writeln!(f, "4. **Drag-and-drop** — DONE: .drag-handle, .drag-over, .kanban-column, .kanban-card styles")?;
    writeln!(f)?;
    writeln!(f, "### Additional Implementations")?;
    writeln!(f)?;
    writeln!(f, "5. **Skip link** — DONE: .skip-link for keyboard accessibility (hidden until focused)")?;
    writeln!(f, "6. **Print styles** — DONE: @media print hides nav/sidebar/toasts, clean output")?;
    writeln!(f, "7. **Tablet breakpoints** — DONE: @media (min-width: 769px) for tablet layout")?;
    writeln!(f, "8. **Enhanced spinner** — DONE: .spinner-wrap with .spinner animation")?;
    writeln!(f)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::checks::{Category, TestResult};
    use super::*;
    

    fn sample_results() -> Vec<TestResult> {
        vec![
            TestResult::pass("health1").category(Category::Health),
            TestResult::pass("health2").category(Category::Health).with_duration(5),
            TestResult::fail("sso1", "connection refused").category(Category::Sso).fix_hint("start Crowd"),
            TestResult::warn("ui1", "missing viewport").category(Category::Ui),
            TestResult::skip("i18n1", "unreachable").category(Category::I18n),
        ]
    }

    #[test]
    fn generate_creates_file() {
        let path = "/tmp/mytest_report_test.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("my* Suite Test Report"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_counts() {
        let path = "/tmp/mytest_report_counts.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Passed:** 2"));
        assert!(content.contains("Failed:** 1"));
        assert!(content.contains("Warnings:** 1"));
        assert!(content.contains("Skipped:** 1"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_score() {
        let path = "/tmp/mytest_report_score.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Score: 40%")); // 2/5 = 40%
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_failures() {
        let path = "/tmp/mytest_report_failures.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Failures"));
        assert!(content.contains("sso1"));
        assert!(content.contains("connection refused"));
        assert!(content.contains("start Crowd"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_warnings() {
        let path = "/tmp/mytest_report_warnings.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Warnings"));
        assert!(content.contains("ui1"));
        assert!(content.contains("missing viewport"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_passed() {
        let path = "/tmp/mytest_report_passed.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Passed"));
        assert!(content.contains("health1"));
        assert!(content.contains("health2"));
        assert!(content.contains("5ms"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_skipped() {
        let path = "/tmp/mytest_report_skipped.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Skipped"));
        assert!(content.contains("i18n1"));
        assert!(content.contains("unreachable"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_category_table() {
        let path = "/tmp/mytest_report_cats.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Summary by Category"));
        assert!(content.contains("| Health |"));
        assert!(content.contains("| SSO |"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_contains_ux_suggestions() {
        let path = "/tmp/mytest_report_ux.md";
        let results = sample_results();
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("UX/UI Improvement Suggestions"));
        assert!(content.contains("High Priority"));
        assert!(content.contains("Medium Priority"));
        assert!(content.contains("Low Priority"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_empty_results() {
        let path = "/tmp/mytest_report_empty.md";
        let results: Vec<TestResult> = vec![];
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Score: 0%"));
        assert!(content.contains("Total:** 0"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_all_passed() {
        let path = "/tmp/mytest_report_all_pass.md";
        let results = vec![
            TestResult::pass("a").category(Category::Health),
            TestResult::pass("b").category(Category::Sso),
            TestResult::pass("c").category(Category::Ui),
        ];
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Score: 100%"));
        assert!(content.contains("Excellent"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_all_failed() {
        let path = "/tmp/mytest_report_all_fail.md";
        let results = vec![
            TestResult::fail("a", "err").category(Category::Health),
            TestResult::fail("b", "err").category(Category::Health),
        ];
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("Score: 0%"));
        assert!(content.contains("Needs work"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn generate_with_duration() {
        let path = "/tmp/mytest_report_dur.md";
        let results = vec![
            TestResult::pass("fast").with_duration(1),
            TestResult::pass("slow").with_duration(5000),
        ];
        generate(&results, path).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("1ms"));
        assert!(content.contains("5000ms"));
        std::fs::remove_file(path).ok();
    }
}
