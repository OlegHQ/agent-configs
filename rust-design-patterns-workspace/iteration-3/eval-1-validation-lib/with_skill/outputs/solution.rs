// Validation Library
//
// Architecture: Strategy Pattern
//
// Each validation rule is a separate struct implementing the `ValidationRule` trait.
// Rules are composed into a `FieldRules` collection (`Vec<Box<dyn ValidationRule>>`)
// that runs all rules and collects all errors — not just the first.
//
// Adding a new validation rule requires only:
//   1. Define a new struct (with any config it needs)
//   2. Implement `ValidationRule` for it
//   3. Optionally add a convenience constructor function
// No existing code is modified.

use std::fmt;

// ---------------------------------------------------------------------------
// Core trait — the extension point for every validation rule (Strategy)
// ---------------------------------------------------------------------------

/// A single validation rule that can inspect a string value and return an
/// error message if the value is invalid.
pub trait ValidationRule: fmt::Debug {
    /// Returns `Some(error_message)` when validation fails, `None` when it passes.
    fn validate(&self, value: &str) -> Option<String>;
}

// ---------------------------------------------------------------------------
// ValidationError — a single error tied to a field name
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

// ---------------------------------------------------------------------------
// FieldRules — composes multiple rules for a single named field
// ---------------------------------------------------------------------------

pub struct FieldRules {
    field_name: String,
    rules: Vec<Box<dyn ValidationRule>>,
}

impl FieldRules {
    pub fn new(field_name: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            rules: Vec::new(),
        }
    }

    /// Add a rule to this field's rule set. Chainable.
    pub fn add(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Run every rule against `value` and return *all* errors (not just the first).
    pub fn validate(&self, value: &str) -> Vec<ValidationError> {
        self.rules
            .iter()
            .filter_map(|rule| {
                rule.validate(value).map(|msg| ValidationError {
                    field: self.field_name.clone(),
                    message: msg,
                })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// FormValidator — composes FieldRules across an entire form
// ---------------------------------------------------------------------------

pub struct FormValidator {
    fields: Vec<FieldRules>,
}

impl FormValidator {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn field(mut self, rules: FieldRules) -> Self {
        self.fields.push(rules);
        self
    }

    /// Validate all fields against a lookup function that returns the value for
    /// a given field name. Returns all errors across all fields.
    pub fn validate<F>(&self, get_value: F) -> Vec<ValidationError>
    where
        F: Fn(&str) -> &str,
    {
        self.fields
            .iter()
            .flat_map(|fr| fr.validate(get_value(&fr.field_name)))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Built-in rules — each is its own struct + impl (Strategy pattern instances)
// ---------------------------------------------------------------------------

// --- Required ---

#[derive(Debug)]
pub struct Required;

impl ValidationRule for Required {
    fn validate(&self, value: &str) -> Option<String> {
        if value.trim().is_empty() {
            Some("this field is required".to_string())
        } else {
            None
        }
    }
}

pub fn required() -> Box<dyn ValidationRule> {
    Box::new(Required)
}

// --- MinLength ---

#[derive(Debug)]
pub struct MinLength {
    min: usize,
}

impl ValidationRule for MinLength {
    fn validate(&self, value: &str) -> Option<String> {
        if value.len() < self.min {
            Some(format!("must be at least {} characters", self.min))
        } else {
            None
        }
    }
}

pub fn min_length(min: usize) -> Box<dyn ValidationRule> {
    Box::new(MinLength { min })
}

// --- MaxLength ---

#[derive(Debug)]
pub struct MaxLength {
    max: usize,
}

impl ValidationRule for MaxLength {
    fn validate(&self, value: &str) -> Option<String> {
        if value.len() > self.max {
            Some(format!("must be at most {} characters", self.max))
        } else {
            None
        }
    }
}

pub fn max_length(max: usize) -> Box<dyn ValidationRule> {
    Box::new(MaxLength { max })
}

// --- EmailFormat ---

#[derive(Debug)]
pub struct EmailFormat;

impl ValidationRule for EmailFormat {
    fn validate(&self, value: &str) -> Option<String> {
        // Lightweight check: non-empty local part, `@`, non-empty domain with a dot.
        // Production code would use a dedicated email-validation crate.
        if value.is_empty() {
            return None; // emptiness is the job of `Required`
        }
        let parts: Vec<&str> = value.splitn(2, '@').collect();
        if parts.len() != 2 {
            return Some("must be a valid email address".to_string());
        }
        let (local, domain) = (parts[0], parts[1]);
        if local.is_empty() || domain.is_empty() || !domain.contains('.') {
            return Some("must be a valid email address".to_string());
        }
        None
    }
}

pub fn email_format() -> Box<dyn ValidationRule> {
    Box::new(EmailFormat)
}

// --- Regex ---

#[derive(Debug)]
pub struct RegexRule {
    pattern: String,
    message: String,
}

impl RegexRule {
    /// Simple pattern matching without pulling in the `regex` crate.
    /// Supports a deliberately minimal subset useful for common form validation:
    ///   - Literal characters match themselves
    ///   - `.` matches any character
    ///   - `^` / `$` anchor start / end (always applied — the pattern must match
    ///     the full string when both are present)
    ///   - `+` means "one or more" of the preceding character class
    ///   - `*` means "zero or more" of the preceding character class
    ///   - `[…]` character classes (no nesting)
    ///   - `\d`, `\w`, `\s` shorthands
    ///
    /// For production use, swap the body for `regex::Regex`.
    fn matches(&self, value: &str) -> bool {
        // Delegate to a basic regex engine. For a real crate you'd use `regex::Regex`.
        simple_regex_match(&self.pattern, value)
    }
}

impl ValidationRule for RegexRule {
    fn validate(&self, value: &str) -> Option<String> {
        if value.is_empty() {
            return None; // emptiness is the job of `Required`
        }
        if !self.matches(value) {
            Some(self.message.clone())
        } else {
            None
        }
    }
}

pub fn regex_rule(pattern: &str, message: &str) -> Box<dyn ValidationRule> {
    Box::new(RegexRule {
        pattern: pattern.to_string(),
        message: message.to_string(),
    })
}

// --- NumericRange ---

#[derive(Debug)]
pub struct NumericRange {
    min: f64,
    max: f64,
}

impl ValidationRule for NumericRange {
    fn validate(&self, value: &str) -> Option<String> {
        if value.is_empty() {
            return None;
        }
        match value.parse::<f64>() {
            Ok(n) if n < self.min || n > self.max => {
                Some(format!("must be between {} and {}", self.min, self.max))
            }
            Err(_) => Some("must be a valid number".to_string()),
            _ => None,
        }
    }
}

pub fn numeric_range(min: f64, max: f64) -> Box<dyn ValidationRule> {
    Box::new(NumericRange { min, max })
}

// --- Matches (cross-field confirmation) ---

#[derive(Debug)]
pub struct Matches {
    other_value: String,
    other_field_name: String,
}

impl ValidationRule for Matches {
    fn validate(&self, value: &str) -> Option<String> {
        if value != self.other_value {
            Some(format!("must match {}", self.other_field_name))
        } else {
            None
        }
    }
}

/// Creates a rule that checks `value` equals `other_value`.
/// `other_field_name` is used only for the error message.
pub fn matches_field(other_field_name: &str, other_value: &str) -> Box<dyn ValidationRule> {
    Box::new(Matches {
        other_value: other_value.to_string(),
        other_field_name: other_field_name.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Minimal regex engine (avoids external dependency for this self-contained file)
// ---------------------------------------------------------------------------

/// Very small regex subset — enough for form-validation patterns like
/// `^\d+$`, `^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`, etc.
/// In production, use the `regex` crate instead.
fn simple_regex_match(pattern: &str, input: &str) -> bool {
    let chars: Vec<char> = pattern.chars().collect();
    let input_chars: Vec<char> = input.chars().collect();

    // Strip anchors — we always do a full-string match when both ^ and $ are present.
    let (pat, anchored_start, anchored_end) = {
        let mut start = 0;
        let mut end = chars.len();
        let mut a_start = false;
        let mut a_end = false;
        if !chars.is_empty() && chars[0] == '^' {
            start = 1;
            a_start = true;
        }
        if end > start && chars[end - 1] == '$' {
            end -= 1;
            a_end = true;
        }
        (&chars[start..end], a_start, a_end)
    };

    if anchored_start && anchored_end {
        regex_match_here(pat, &input_chars, 0).map_or(false, |end| end == input_chars.len())
    } else if anchored_start {
        regex_match_here(pat, &input_chars, 0).is_some()
    } else if anchored_end {
        for start in 0..=input_chars.len() {
            if let Some(end) = regex_match_here(pat, &input_chars, start) {
                if end == input_chars.len() {
                    return true;
                }
            }
        }
        false
    } else {
        for start in 0..=input_chars.len() {
            if regex_match_here(pat, &input_chars, start).is_some() {
                return true;
            }
        }
        false
    }
}

/// Try to match `pat` starting at `input[pos]`. Returns the position *after*
/// the last consumed character on success, or `None`.
fn regex_match_here(pat: &[char], input: &[char], pos: usize) -> Option<usize> {
    if pat.is_empty() {
        return Some(pos);
    }

    // Character class [...]
    if pat[0] == '[' {
        if let Some(close) = pat.iter().position(|&c| c == ']') {
            let class = &pat[1..close];
            let rest = &pat[close + 1..];
            let (quantifier, rest_after_q) = get_quantifier(rest);
            return match_quantified(class, quantifier, rest_after_q, input, pos, true);
        }
    }

    // Escaped character classes: \d, \w, \s
    if pat[0] == '\\' && pat.len() >= 2 {
        let esc = &pat[0..2];
        let rest = &pat[2..];
        let (quantifier, rest_after_q) = get_quantifier(rest);
        return match_quantified(esc, quantifier, rest_after_q, input, pos, false);
    }

    // Dot
    if pat[0] == '.' {
        let rest = &pat[1..];
        let (quantifier, rest_after_q) = get_quantifier(rest);
        let class: &[char] = &['.'];
        return match_quantified(class, quantifier, rest_after_q, input, pos, false);
    }

    // {n,m} quantifier on a literal — handle after checking quantifier
    let rest = &pat[1..];
    let (quantifier, rest_after_q) = get_quantifier(rest);
    if quantifier != Quantifier::One {
        let class = &pat[0..1];
        return match_quantified(class, quantifier, rest_after_q, input, pos, false);
    }

    // Plain literal
    if pos < input.len() && input[pos] == pat[0] {
        regex_match_here(&pat[1..], input, pos + 1)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Quantifier {
    One,
    ZeroOrMore,
    OneOrMore,
    Repeat(usize, usize), // {min, max}
}

fn get_quantifier(rest: &[char]) -> (Quantifier, &[char]) {
    if rest.is_empty() {
        return (Quantifier::One, rest);
    }
    match rest[0] {
        '+' => (Quantifier::OneOrMore, &rest[1..]),
        '*' => (Quantifier::ZeroOrMore, &rest[1..]),
        '{' => {
            // Parse {n} or {n,m}
            if let Some(close) = rest.iter().position(|&c| c == '}') {
                let inside: String = rest[1..close].iter().collect();
                let after = &rest[close + 1..];
                if let Some(comma) = inside.find(',') {
                    let min = inside[..comma].parse().unwrap_or(0);
                    let max = inside[comma + 1..].parse().unwrap_or(min);
                    (Quantifier::Repeat(min, max), after)
                } else if let Ok(n) = inside.parse::<usize>() {
                    (Quantifier::Repeat(n, n), after)
                } else {
                    (Quantifier::One, rest)
                }
            } else {
                (Quantifier::One, rest)
            }
        }
        _ => (Quantifier::One, rest),
    }
}

fn char_matches_class(class: &[char], ch: char, is_bracket: bool) -> bool {
    if is_bracket {
        // bracket expression: handle ranges like a-z
        let mut i = 0;
        while i < class.len() {
            if i + 2 < class.len() && class[i + 1] == '-' {
                if ch >= class[i] && ch <= class[i + 2] {
                    return true;
                }
                i += 3;
            } else {
                if ch == class[i] {
                    return true;
                }
                i += 1;
            }
        }
        false
    } else if class.len() == 2 && class[0] == '\\' {
        match class[1] {
            'd' => ch.is_ascii_digit(),
            'w' => ch.is_ascii_alphanumeric() || ch == '_',
            's' => ch.is_ascii_whitespace(),
            other => ch == other,
        }
    } else if class.len() == 1 && class[0] == '.' {
        true // dot matches anything
    } else if class.len() == 1 {
        ch == class[0]
    } else {
        false
    }
}

fn match_quantified(
    class: &[char],
    quantifier: Quantifier,
    rest: &[char],
    input: &[char],
    pos: usize,
    is_bracket: bool,
) -> Option<usize> {
    let (min, max) = match quantifier {
        Quantifier::One => (1, 1),
        Quantifier::ZeroOrMore => (0, usize::MAX),
        Quantifier::OneOrMore => (1, usize::MAX),
        Quantifier::Repeat(lo, hi) => (lo, hi),
    };

    // Greedy: consume as many as possible, then backtrack
    let mut count = 0;
    let mut cur = pos;
    while cur < input.len() && count < max && char_matches_class(class, input[cur], is_bracket) {
        count += 1;
        cur += 1;
    }

    // Try from longest match downward
    while count >= min {
        let try_pos = pos + count;
        if let Some(end) = regex_match_here(rest, input, try_pos) {
            return Some(end);
        }
        if count == 0 {
            break;
        }
        count -= 1;
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // -- Individual rule tests --

    #[test]
    fn test_required_passes_on_non_empty() {
        assert!(Required.validate("hello").is_none());
    }

    #[test]
    fn test_required_fails_on_empty() {
        assert!(Required.validate("").is_some());
        assert!(Required.validate("   ").is_some());
    }

    #[test]
    fn test_min_length() {
        let rule = MinLength { min: 3 };
        assert!(rule.validate("ab").is_some());
        assert!(rule.validate("abc").is_none());
        assert!(rule.validate("abcd").is_none());
    }

    #[test]
    fn test_max_length() {
        let rule = MaxLength { max: 5 };
        assert!(rule.validate("12345").is_none());
        assert!(rule.validate("123456").is_some());
    }

    #[test]
    fn test_email_format_valid() {
        assert!(EmailFormat.validate("user@example.com").is_none());
        assert!(EmailFormat.validate("a@b.c").is_none());
    }

    #[test]
    fn test_email_format_invalid() {
        assert!(EmailFormat.validate("not-an-email").is_some());
        assert!(EmailFormat.validate("@no-local.com").is_some());
        assert!(EmailFormat.validate("no-domain@").is_some());
        assert!(EmailFormat.validate("no-dot@domain").is_some());
    }

    #[test]
    fn test_email_format_skips_empty() {
        // Empty check is the job of Required, not EmailFormat
        assert!(EmailFormat.validate("").is_none());
    }

    #[test]
    fn test_numeric_range() {
        let rule = NumericRange { min: 1.0, max: 100.0 };
        assert!(rule.validate("50").is_none());
        assert!(rule.validate("0").is_some());
        assert!(rule.validate("101").is_some());
        assert!(rule.validate("abc").is_some());
        assert!(rule.validate("").is_none()); // empty = not our concern
    }

    #[test]
    fn test_matches_field_pass() {
        let rule = Matches {
            other_value: "secret123".to_string(),
            other_field_name: "password".to_string(),
        };
        assert!(rule.validate("secret123").is_none());
    }

    #[test]
    fn test_matches_field_fail() {
        let rule = Matches {
            other_value: "secret123".to_string(),
            other_field_name: "password".to_string(),
        };
        let err = rule.validate("different").unwrap();
        assert!(err.contains("must match password"));
    }

    #[test]
    fn test_regex_rule_digits_only() {
        let rule = RegexRule {
            pattern: r"^\d+$".to_string(),
            message: "digits only".to_string(),
        };
        assert!(rule.validate("12345").is_none());
        assert!(rule.validate("123a5").is_some());
    }

    // -- FieldRules composition tests --

    #[test]
    fn test_field_rules_collects_all_errors() {
        let rules = FieldRules::new("email")
            .add(required())
            .add(email_format())
            .add(max_length(255));

        // Valid email — no errors
        let errors = rules.validate("user@example.com");
        assert!(errors.is_empty());

        // Invalid email — should get email format error
        let errors = rules.validate("not-an-email");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("email"));
    }

    #[test]
    fn test_field_rules_empty_value_collects_required_error() {
        let rules = FieldRules::new("email")
            .add(required())
            .add(email_format())
            .add(max_length(255));

        let errors = rules.validate("");
        // Only `required` fires — email_format and max_length skip empty strings
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("required"));
    }

    #[test]
    fn test_field_rules_multiple_failures() {
        let rules = FieldRules::new("username")
            .add(required())
            .add(min_length(5))
            .add(max_length(3)); // intentionally contradictory for test

        let errors = rules.validate("ab");
        // min_length fails (2 < 5), that's 1 error. required passes, max_length passes (2 <= 3).
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("at least 5"));
    }

    // -- FormValidator integration test --

    #[test]
    fn test_form_validator() {
        let password = "secret123";
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
            );

        let mut values = HashMap::new();
        values.insert("email", "bad");
        values.insert("password", "secret123");
        values.insert("password_confirm", "mismatch");

        let errors = form.validate(|field| values.get(field).copied().unwrap_or(""));
        // email: invalid format. password_confirm: doesn't match.
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.field == "email"));
        assert!(errors.iter().any(|e| e.field == "password_confirm"));
    }

    // -- Extensibility demonstration --
    // Adding a new rule requires ZERO changes to existing code — just a new struct + impl.

    #[derive(Debug)]
    struct ContainsUppercase;

    impl ValidationRule for ContainsUppercase {
        fn validate(&self, value: &str) -> Option<String> {
            if value.chars().any(|c| c.is_uppercase()) {
                None
            } else {
                Some("must contain at least one uppercase letter".to_string())
            }
        }
    }

    #[test]
    fn test_custom_rule_composes_with_builtins() {
        let rules = FieldRules::new("password")
            .add(required())
            .add(min_length(8))
            .add(Box::new(ContainsUppercase));

        let errors = rules.validate("alllowercase");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("uppercase"));

        let errors = rules.validate("HasUpper123");
        assert!(errors.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Example main — shows the API in action
// ---------------------------------------------------------------------------

fn main() {
    // Example: validate an email field
    let rules = FieldRules::new("email")
        .add(required())
        .add(email_format())
        .add(max_length(255));

    let errors = rules.validate("not-an-email");
    println!("Email validation errors:");
    for e in &errors {
        println!("  - {}", e);
    }

    // Example: full form validation
    let password = "secret123";
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
            FieldRules::new("age")
                .add(required())
                .add(numeric_range(0.0, 150.0)),
        )
        .field(
            FieldRules::new("password_confirm")
                .add(required())
                .add(matches_field("password", password)),
        );

    let mut values = std::collections::HashMap::new();
    values.insert("email", "user@example.com");
    values.insert("password", "secret123");
    values.insert("age", "25");
    values.insert("password_confirm", "secret123");

    let errors = form.validate(|field| values.get(field).copied().unwrap_or(""));
    if errors.is_empty() {
        println!("\nForm is valid!");
    } else {
        println!("\nForm errors:");
        for e in &errors {
            println!("  - {}", e);
        }
    }
}
