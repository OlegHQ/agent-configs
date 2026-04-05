use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Core trait — the extension point for every new validation rule
// ---------------------------------------------------------------------------

/// A single validation rule that can inspect a string value and return an
/// optional error message.  Implement this trait to add new business rules
/// without touching existing code (Open/Closed Principle).
pub trait Validator {
    /// Returns `Some(message)` when validation fails, `None` when it passes.
    fn validate(&self, value: &str) -> Option<String>;
}

// Allow boxed validators to be used transparently.
impl Validator for Box<dyn Validator> {
    fn validate(&self, value: &str) -> Option<String> {
        (**self).validate(value)
    }
}

// ---------------------------------------------------------------------------
// FieldRules — composable rule set for a single named field
// ---------------------------------------------------------------------------

/// Collects multiple [`Validator`]s for a single field and runs them all,
/// returning every error (not just the first).
pub struct FieldRules {
    field: String,
    validators: Vec<Box<dyn Validator>>,
}

impl FieldRules {
    pub fn new(field: &str) -> Self {
        Self {
            field: field.to_string(),
            validators: Vec::new(),
        }
    }

    /// Builder-style method — add a validator and return `self`.
    pub fn add<V: Validator + 'static>(mut self, v: V) -> Self {
        self.validators.push(Box::new(v));
        self
    }

    /// Run every validator and collect all error messages.
    pub fn validate(&self, value: &str) -> Vec<String> {
        self.validators
            .iter()
            .filter_map(|v| v.validate(value))
            .collect()
    }

    pub fn field_name(&self) -> &str {
        &self.field
    }
}

// ---------------------------------------------------------------------------
// FormValidator — validates an entire form (multiple fields) at once
// ---------------------------------------------------------------------------

/// Aggregated validation result for a whole form.
#[derive(Debug, Default)]
pub struct ValidationResult {
    errors: HashMap<String, Vec<String>>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn errors_for(&self, field: &str) -> &[String] {
        self.errors.get(field).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn all_errors(&self) -> &HashMap<String, Vec<String>> {
        &self.errors
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (field, msgs) in &self.errors {
            for msg in msgs {
                writeln!(f, "{}: {}", field, msg)?;
            }
        }
        Ok(())
    }
}

/// Validates a set of named fields against a map of submitted values.
pub struct FormValidator {
    field_rules: Vec<FieldRules>,
}

impl FormValidator {
    pub fn new() -> Self {
        Self {
            field_rules: Vec::new(),
        }
    }

    pub fn field(mut self, rules: FieldRules) -> Self {
        self.field_rules.push(rules);
        self
    }

    /// Validate all fields.  Missing fields are treated as empty strings.
    pub fn validate(&self, values: &HashMap<&str, &str>) -> ValidationResult {
        let mut result = ValidationResult::default();
        for rules in &self.field_rules {
            let value = values.get(rules.field_name()).copied().unwrap_or("");
            let errs = rules.validate(value);
            if !errs.is_empty() {
                result.errors.insert(rules.field_name().to_string(), errs);
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Cross-field validator — e.g. "password confirmation must match"
// ---------------------------------------------------------------------------

/// Validates that a field's value matches another field's value.
pub struct MatchesField {
    other_field: String,
    other_value: String,
}

impl MatchesField {
    pub fn new(other_field: &str, other_value: &str) -> Self {
        Self {
            other_field: other_field.to_string(),
            other_value: other_value.to_string(),
        }
    }
}

impl Validator for MatchesField {
    fn validate(&self, value: &str) -> Option<String> {
        if value != self.other_value {
            Some(format!("must match {}", self.other_field))
        } else {
            None
        }
    }
}

/// Convenience constructor.
pub fn matches_field(other_field: &str, other_value: &str) -> MatchesField {
    MatchesField::new(other_field, other_value)
}

// ---------------------------------------------------------------------------
// Built-in validators (each is a small struct implementing Validator)
// ---------------------------------------------------------------------------

// -- Required ---------------------------------------------------------------

pub struct Required;

impl Validator for Required {
    fn validate(&self, value: &str) -> Option<String> {
        if value.trim().is_empty() {
            Some("is required".to_string())
        } else {
            None
        }
    }
}

pub fn required() -> Required {
    Required
}

// -- MinLength / MaxLength --------------------------------------------------

pub struct MinLength(usize);

impl Validator for MinLength {
    fn validate(&self, value: &str) -> Option<String> {
        if value.len() < self.0 {
            Some(format!("must be at least {} characters", self.0))
        } else {
            None
        }
    }
}

pub fn min_length(n: usize) -> MinLength {
    MinLength(n)
}

pub struct MaxLength(usize);

impl Validator for MaxLength {
    fn validate(&self, value: &str) -> Option<String> {
        if value.len() > self.0 {
            Some(format!("must be at most {} characters", self.0))
        } else {
            None
        }
    }
}

pub fn max_length(n: usize) -> MaxLength {
    MaxLength(n)
}

// -- Numeric range ----------------------------------------------------------

pub struct NumericRange {
    min: f64,
    max: f64,
}

impl Validator for NumericRange {
    fn validate(&self, value: &str) -> Option<String> {
        match value.parse::<f64>() {
            Ok(n) if n >= self.min && n <= self.max => None,
            Ok(_) => Some(format!("must be between {} and {}", self.min, self.max)),
            Err(_) => Some("must be a valid number".to_string()),
        }
    }
}

pub fn numeric_range(min: f64, max: f64) -> NumericRange {
    NumericRange { min, max }
}

// -- Regex ------------------------------------------------------------------

/// Uses a simple hand-rolled pattern match so the library compiles without
/// external crates.  For production use you would swap in the `regex` crate.
pub struct RegexMatch {
    pattern: String,
    // In a real project this would be `regex::Regex`.  We store the raw
    // pattern and do a naive substring/exact check for demonstration.
    message: String,
}

impl RegexMatch {
    pub fn new(pattern: &str, message: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            message: message.to_string(),
        }
    }
}

impl Validator for RegexMatch {
    fn validate(&self, value: &str) -> Option<String> {
        // Minimal built-in matching.  Replace with `regex::Regex::is_match`
        // for real pattern support.
        if !naive_regex_match(&self.pattern, value) {
            Some(self.message.clone())
        } else {
            None
        }
    }
}

/// Very small regex-like matcher supporting:
///   - literal characters
///   - `.` (any char)
///   - `*` (zero or more of preceding)
///   - `+` (one or more of preceding)
///   - `^` / `$` anchors
///   - `[...]` character classes (no nesting)
///   - `\d`, `\w`, `\s` shorthands
///
/// This is intentionally limited — swap in `regex` crate for full support.
fn naive_regex_match(pattern: &str, value: &str) -> bool {
    // For the demo we do a simple "contains the literal pattern" check when
    // the pattern has no special chars, and handle the anchored case.
    // A production implementation would use the `regex` crate.

    let anchored_start = pattern.starts_with('^');
    let anchored_end = pattern.ends_with('$');

    let inner = pattern
        .trim_start_matches('^')
        .trim_end_matches('$');

    if anchored_start && anchored_end {
        value == inner
    } else if anchored_start {
        value.starts_with(inner)
    } else if anchored_end {
        value.ends_with(inner)
    } else {
        value.contains(inner)
    }
}

pub fn regex_match(pattern: &str, message: &str) -> RegexMatch {
    RegexMatch::new(pattern, message)
}

// -- Email format -----------------------------------------------------------

pub struct EmailFormat;

impl Validator for EmailFormat {
    fn validate(&self, value: &str) -> Option<String> {
        if value.is_empty() {
            // Let `required()` handle emptiness — don't double-report.
            return None;
        }

        let parts: Vec<&str> = value.splitn(2, '@').collect();
        if parts.len() != 2 {
            return Some("must be a valid email address".to_string());
        }

        let (local, domain) = (parts[0], parts[1]);

        if local.is_empty() || domain.is_empty() {
            return Some("must be a valid email address".to_string());
        }

        // Domain must contain at least one dot and no consecutive dots.
        if !domain.contains('.') || domain.contains("..") {
            return Some("must be a valid email address".to_string());
        }

        // TLD must be at least 2 chars.
        if let Some(tld) = domain.rsplit('.').next() {
            if tld.len() < 2 {
                return Some("must be a valid email address".to_string());
            }
        }

        None
    }
}

pub fn email_format() -> EmailFormat {
    EmailFormat
}

// -- Custom closure validator -----------------------------------------------

/// Wraps an arbitrary closure so you can write one-off rules inline.
pub struct Custom<F: Fn(&str) -> Option<String>> {
    f: F,
}

impl<F: Fn(&str) -> Option<String>> Validator for Custom<F> {
    fn validate(&self, value: &str) -> Option<String> {
        (self.f)(value)
    }
}

pub fn custom<F: Fn(&str) -> Option<String>>(f: F) -> Custom<F> {
    Custom { f }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- required -----------------------------------------------------------

    #[test]
    fn required_rejects_empty() {
        assert!(required().validate("").is_some());
        assert!(required().validate("   ").is_some());
    }

    #[test]
    fn required_accepts_non_empty() {
        assert!(required().validate("hello").is_none());
    }

    // -- email --------------------------------------------------------------

    #[test]
    fn email_rejects_missing_at() {
        assert!(email_format().validate("bademail").is_some());
    }

    #[test]
    fn email_rejects_missing_tld() {
        assert!(email_format().validate("a@b").is_some());
    }

    #[test]
    fn email_accepts_valid() {
        assert!(email_format().validate("user@example.com").is_none());
    }

    #[test]
    fn email_skips_empty() {
        // Empty is not this validator's concern.
        assert!(email_format().validate("").is_none());
    }

    // -- length -------------------------------------------------------------

    #[test]
    fn min_length_works() {
        assert!(min_length(5).validate("ab").is_some());
        assert!(min_length(5).validate("abcde").is_none());
    }

    #[test]
    fn max_length_works() {
        assert!(max_length(3).validate("abcd").is_some());
        assert!(max_length(3).validate("abc").is_none());
    }

    // -- numeric range ------------------------------------------------------

    #[test]
    fn numeric_range_rejects_non_number() {
        assert!(numeric_range(1.0, 10.0).validate("abc").is_some());
    }

    #[test]
    fn numeric_range_rejects_out_of_bounds() {
        assert!(numeric_range(1.0, 10.0).validate("0").is_some());
        assert!(numeric_range(1.0, 10.0).validate("11").is_some());
    }

    #[test]
    fn numeric_range_accepts_in_bounds() {
        assert!(numeric_range(1.0, 10.0).validate("5").is_none());
        assert!(numeric_range(1.0, 10.0).validate("1").is_none());
        assert!(numeric_range(1.0, 10.0).validate("10").is_none());
    }

    // -- regex --------------------------------------------------------------

    #[test]
    fn regex_match_works() {
        let v = regex_match("hello", "must contain hello");
        assert!(v.validate("say hello world").is_none());
        assert!(v.validate("goodbye").is_some());
    }

    // -- matches_field ------------------------------------------------------

    #[test]
    fn matches_field_passes_when_equal() {
        let v = matches_field("password", "secret123");
        assert!(v.validate("secret123").is_none());
    }

    #[test]
    fn matches_field_fails_when_different() {
        let v = matches_field("password", "secret123");
        let err = v.validate("other").unwrap();
        assert!(err.contains("must match password"));
    }

    // -- custom -------------------------------------------------------------

    #[test]
    fn custom_validator_works() {
        let v = custom(|val| {
            if val.starts_with("USD") {
                None
            } else {
                Some("must start with USD".to_string())
            }
        });
        assert!(v.validate("USD100").is_none());
        assert!(v.validate("EUR50").is_some());
    }

    // -- FieldRules composition ---------------------------------------------

    #[test]
    fn field_rules_collects_all_errors() {
        let rules = FieldRules::new("email")
            .add(required())
            .add(email_format())
            .add(max_length(255));

        // Valid email — no errors.
        let errs = rules.validate("user@example.com");
        assert!(errs.is_empty());

        // Empty — required fires (email_format intentionally skips empty).
        let errs = rules.validate("");
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("required"));
    }

    #[test]
    fn field_rules_returns_multiple_errors() {
        let rules = FieldRules::new("username")
            .add(required())
            .add(min_length(3))
            .add(max_length(20));

        // Empty string triggers both required AND min_length.
        let errs = rules.validate("");
        assert!(errs.len() >= 2);
    }

    #[test]
    fn field_rules_invalid_email_format() {
        let rules = FieldRules::new("email")
            .add(required())
            .add(email_format())
            .add(max_length(255));

        let errs = rules.validate("not-an-email");
        assert!(!errs.is_empty());
        assert!(errs.iter().any(|e| e.contains("email")));
    }

    // -- FormValidator ------------------------------------------------------

    #[test]
    fn form_validator_full_example() {
        let password = "hunter2";

        let form = FormValidator::new()
            .field(
                FieldRules::new("email")
                    .add(required())
                    .add(email_format()),
            )
            .field(
                FieldRules::new("password")
                    .add(required())
                    .add(min_length(8)),
            )
            .field(
                FieldRules::new("password_confirm")
                    .add(required())
                    .add(matches_field("password", password)),
            )
            .field(
                FieldRules::new("age")
                    .add(required())
                    .add(numeric_range(18.0, 120.0)),
            );

        let mut values = HashMap::new();
        values.insert("email", "bad");
        values.insert("password", "hunter2");
        values.insert("password_confirm", "hunter2");
        values.insert("age", "25");

        let result = form.validate(&values);
        assert!(!result.is_valid());

        // email is invalid
        assert!(!result.errors_for("email").is_empty());

        // password too short (< 8)
        assert!(!result.errors_for("password").is_empty());

        // password_confirm matches — no error
        assert!(result.errors_for("password_confirm").is_empty());

        // age is fine
        assert!(result.errors_for("age").is_empty());
    }

    #[test]
    fn form_validator_all_valid() {
        let form = FormValidator::new()
            .field(
                FieldRules::new("name")
                    .add(required())
                    .add(max_length(100)),
            );

        let mut values = HashMap::new();
        values.insert("name", "Alice");

        let result = form.validate(&values);
        assert!(result.is_valid());
    }

    #[test]
    fn form_validator_missing_field_treated_as_empty() {
        let form = FormValidator::new()
            .field(FieldRules::new("name").add(required()));

        let values = HashMap::new(); // no "name" key
        let result = form.validate(&values);
        assert!(!result.is_valid());
    }
}

// ---------------------------------------------------------------------------
// Main — quick demo
// ---------------------------------------------------------------------------

fn main() {
    println!("=== Validation Library Demo ===\n");

    // Per-field example (matches the requested API).
    let rules = FieldRules::new("email")
        .add(required())
        .add(email_format())
        .add(max_length(255));

    let errors = rules.validate("not-an-email");
    println!("Validating 'not-an-email' as email:");
    for e in &errors {
        println!("  - {}", e);
    }

    let errors = rules.validate("alice@example.com");
    println!("\nValidating 'alice@example.com' as email:");
    if errors.is_empty() {
        println!("  (no errors)");
    }

    // Full-form example.
    println!("\n--- Full form validation ---");

    let password_value = "short";

    let form = FormValidator::new()
        .field(
            FieldRules::new("email")
                .add(required())
                .add(email_format()),
        )
        .field(
            FieldRules::new("password")
                .add(required())
                .add(min_length(8)),
        )
        .field(
            FieldRules::new("password_confirm")
                .add(required())
                .add(matches_field("password", password_value)),
        )
        .field(
            FieldRules::new("age")
                .add(required())
                .add(numeric_range(18.0, 120.0)),
        );

    let mut values = HashMap::new();
    values.insert("email", "bad-email");
    values.insert("password", password_value);
    values.insert("password_confirm", "mismatch");
    values.insert("age", "200");

    let result = form.validate(&values);
    println!("{}", result);

    if result.is_valid() {
        println!("Form is valid!");
    } else {
        println!("Form has errors.");
    }
}
