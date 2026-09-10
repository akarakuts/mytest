//! Модули проверок: health, SSO, UI, i18n, integration, server_functions, server_functions_all.

pub mod health;
pub mod i18n;
pub mod integration;
pub mod server_functions;
pub mod server_functions_all;
pub mod sso;
pub mod ui;

/// Результат одного теста.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub category: Category,
    pub status: Status,
    pub detail: Option<String>,
    pub duration_ms: u64,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    Warn,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Category {
    Health,
    Sso,
    Ui,
    I18n,
    Integration,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Health => write!(f, "Health"),
            Category::Sso => write!(f, "SSO"),
            Category::Ui => write!(f, "UI/UX"),
            Category::I18n => write!(f, "i18n"),
            Category::Integration => write!(f, "Integration"),
        }
    }
}

impl TestResult {
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: Category::Health,
            status: Status::Pass,
            detail: None,
            duration_ms: 0,
            fix_hint: None,
        }
    }

    pub fn fail(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: Category::Health,
            status: Status::Fail,
            detail: Some(detail.into()),
            duration_ms: 0,
            fix_hint: None,
        }
    }

    pub fn warn(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: Category::Health,
            status: Status::Warn,
            detail: Some(detail.into()),
            duration_ms: 0,
            fix_hint: None,
        }
    }

    pub fn skip(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: Category::Health,
            status: Status::Skip,
            detail: Some(reason.into()),
            duration_ms: 0,
            fix_hint: None,
        }
    }

    pub fn category(mut self, cat: Category) -> Self {
        self.category = cat;
        self
    }

    pub fn fix_hint(mut self, hint: impl Into<String>) -> Self {
        self.fix_hint = Some(hint.into());
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_creates_passing_result() {
        let r = TestResult::pass("test");
        assert_eq!(r.status, Status::Pass);
        assert_eq!(r.name, "test");
        assert!(r.detail.is_none());
        assert_eq!(r.duration_ms, 0);
        assert!(r.fix_hint.is_none());
    }

    #[test]
    fn fail_creates_failing_result() {
        let r = TestResult::fail("test", "broken");
        assert_eq!(r.status, Status::Fail);
        assert_eq!(r.detail.as_deref(), Some("broken"));
    }

    #[test]
    fn warn_creates_warning_result() {
        let r = TestResult::warn("test", "heads up");
        assert_eq!(r.status, Status::Warn);
        assert_eq!(r.detail.as_deref(), Some("heads up"));
    }

    #[test]
    fn skip_creates_skipped_result() {
        let r = TestResult::skip("test", "not applicable");
        assert_eq!(r.status, Status::Skip);
        assert_eq!(r.detail.as_deref(), Some("not applicable"));
    }

    #[test]
    fn category_sets_category() {
        let r = TestResult::pass("t").category(Category::Sso);
        assert_eq!(r.category, Category::Sso);

        let r = TestResult::pass("t").category(Category::Ui);
        assert_eq!(r.category, Category::Ui);

        let r = TestResult::pass("t").category(Category::I18n);
        assert_eq!(r.category, Category::I18n);

        let r = TestResult::pass("t").category(Category::Integration);
        assert_eq!(r.category, Category::Integration);
    }

    #[test]
    fn fix_hint_sets_hint() {
        let r = TestResult::fail("t", "d").fix_hint("do this");
        assert_eq!(r.fix_hint.as_deref(), Some("do this"));
    }

    #[test]
    fn with_duration_sets_duration() {
        let r = TestResult::pass("t").with_duration(42);
        assert_eq!(r.duration_ms, 42);
    }

    #[test]
    fn builder_chaining() {
        let r = TestResult::warn("name", "detail")
            .category(Category::I18n)
            .fix_hint("hint")
            .with_duration(100);
        assert_eq!(r.status, Status::Warn);
        assert_eq!(r.category, Category::I18n);
        assert_eq!(r.fix_hint.as_deref(), Some("hint"));
        assert_eq!(r.duration_ms, 100);
    }

    #[test]
    fn category_display() {
        assert_eq!(format!("{}", Category::Health), "Health");
        assert_eq!(format!("{}", Category::Sso), "SSO");
        assert_eq!(format!("{}", Category::Ui), "UI/UX");
        assert_eq!(format!("{}", Category::I18n), "i18n");
        assert_eq!(format!("{}", Category::Integration), "Integration");
    }

    #[test]
    fn status_equality() {
        assert_eq!(Status::Pass, Status::Pass);
        assert_ne!(Status::Pass, Status::Fail);
        assert_ne!(Status::Warn, Status::Skip);
    }

    #[test]
    fn category_equality() {
        assert_eq!(Category::Health, Category::Health);
        assert_ne!(Category::Health, Category::Sso);
    }

    #[test]
    fn test_result_clone() {
        let r = TestResult::pass("t").category(Category::Ui).fix_hint("h");
        let r2 = r.clone();
        assert_eq!(r2.name, "t");
        assert_eq!(r2.category, Category::Ui);
    }

    #[test]
    fn test_result_debug() {
        let r = TestResult::pass("t");
        let dbg = format!("{:?}", r);
        assert!(dbg.contains("TestResult"));
        assert!(dbg.contains("Pass"));
    }
}
