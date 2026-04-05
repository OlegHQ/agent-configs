// Notification System
//
// Design patterns applied:
// - Trait (open set) for NotificationChannel -- new channels added without modifying existing code
// - Enum (closed set) for Event -- app-controlled event types with exhaustive matching
// - Observer/Registry for routing events to multiple channels
// - Result-based error handling throughout

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Every event the system can emit. Closed set -- the app controls these.
/// Adding a new event is: add a variant here, then register its channel routing.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum Event {
    UserSignup,
    OrderPlaced,
    PaymentFailed,
    PasswordReset,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::UserSignup => write!(f, "UserSignup"),
            Event::OrderPlaced => write!(f, "OrderPlaced"),
            Event::PaymentFailed => write!(f, "PaymentFailed"),
            Event::PasswordReset => write!(f, "PasswordReset"),
        }
    }
}

/// Context delivered with every notification. Channels use what they need.
#[derive(Debug, Clone)]
struct NotificationContext {
    event: Event,
    recipient: String,
    subject: String,
    body: String,
    metadata: HashMap<String, String>,
}

impl NotificationContext {
    fn new(event: Event, recipient: impl Into<String>, subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            event,
            recipient: recipient.into(),
            subject: subject.into(),
            body: body.into(),
            metadata: HashMap::new(),
        }
    }

    fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ---------------------------------------------------------------------------
// NotificationChannel trait -- the open extension point
// ---------------------------------------------------------------------------

/// Trait for notification delivery channels. Each channel is its own type.
/// Adding a new channel (Slack, SMS, push) means adding a new struct + impl.
/// No existing code needs to change.
trait NotificationChannel: fmt::Display {
    /// Attempt to send a notification. Returns Ok(()) on success or an error message.
    fn send(&self, ctx: &NotificationContext) -> Result<(), NotificationError>;

    /// Human-readable name for logging.
    fn channel_name(&self) -> &str;
}

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct NotificationError {
    channel: String,
    message: String,
}

impl NotificationError {
    fn new(channel: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.channel, self.message)
    }
}

/// Collects results from dispatching to multiple channels.
#[derive(Debug)]
struct DispatchReport {
    event: Event,
    successes: Vec<String>,
    failures: Vec<NotificationError>,
}

impl DispatchReport {
    fn is_fully_successful(&self) -> bool {
        self.failures.is_empty()
    }
}

impl fmt::Display for DispatchReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Event [{}]: {} sent, {} failed", self.event, self.successes.len(), self.failures.len())?;
        for err in &self.failures {
            write!(f, "\n  - {}", err)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Concrete channels
// ---------------------------------------------------------------------------

struct EmailChannel {
    smtp_host: String,
}

impl EmailChannel {
    fn new(smtp_host: impl Into<String>) -> Self {
        Self {
            smtp_host: smtp_host.into(),
        }
    }
}

impl NotificationChannel for EmailChannel {
    fn send(&self, ctx: &NotificationContext) -> Result<(), NotificationError> {
        // In production: connect to SMTP, build MIME message, send.
        println!(
            "  [Email] Sending to {} via {}: \"{}\"",
            ctx.recipient, self.smtp_host, ctx.subject
        );
        Ok(())
    }

    fn channel_name(&self) -> &str {
        "Email"
    }
}

impl fmt::Display for EmailChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Email({})", self.smtp_host)
    }
}

struct SlackChannel {
    webhook_url: String,
    default_channel: String,
}

impl SlackChannel {
    fn new(webhook_url: impl Into<String>, default_channel: impl Into<String>) -> Self {
        Self {
            webhook_url: webhook_url.into(),
            default_channel: default_channel.into(),
        }
    }
}

impl NotificationChannel for SlackChannel {
    fn send(&self, ctx: &NotificationContext) -> Result<(), NotificationError> {
        let target_channel = ctx
            .metadata
            .get("slack_channel")
            .unwrap_or(&self.default_channel);
        // In production: POST JSON to webhook URL.
        println!(
            "  [Slack] Posting to #{}: \"{}\"",
            target_channel, ctx.subject
        );
        Ok(())
    }

    fn channel_name(&self) -> &str {
        "Slack"
    }
}

impl fmt::Display for SlackChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Slack(#{})", self.default_channel)
    }
}

struct SmsChannel {
    api_key: String,
}

impl SmsChannel {
    fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }
}

impl NotificationChannel for SmsChannel {
    fn send(&self, ctx: &NotificationContext) -> Result<(), NotificationError> {
        let phone = ctx.metadata.get("phone").ok_or_else(|| {
            NotificationError::new("SMS", "No phone number in metadata")
        })?;
        // In production: call SMS API.
        println!("  [SMS] Sending to {}: \"{}\"", phone, ctx.subject);
        Ok(())
    }

    fn channel_name(&self) -> &str {
        "SMS"
    }
}

impl fmt::Display for SmsChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SMS")
    }
}

struct PushChannel {
    app_id: String,
}

impl PushChannel {
    fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
        }
    }
}

impl NotificationChannel for PushChannel {
    fn send(&self, ctx: &NotificationContext) -> Result<(), NotificationError> {
        let device_token = ctx.metadata.get("device_token").ok_or_else(|| {
            NotificationError::new("Push", "No device_token in metadata")
        })?;
        // In production: send via APNs / FCM.
        println!(
            "  [Push] Sending to device {}: \"{}\"",
            device_token, ctx.subject
        );
        Ok(())
    }

    fn channel_name(&self) -> &str {
        "Push"
    }
}

impl fmt::Display for PushChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Push({})", self.app_id)
    }
}

// ---------------------------------------------------------------------------
// NotificationDispatcher -- the observer registry
// ---------------------------------------------------------------------------

/// Routes events to one or more notification channels.
///
/// Registration is per-event: each event maps to a list of channels.
/// Dispatching fans out to all channels registered for that event,
/// collecting successes and failures into a report.
struct NotificationDispatcher {
    routes: HashMap<Event, Vec<Box<dyn NotificationChannel>>>,
}

impl NotificationDispatcher {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Register a channel for a specific event.
    fn register(&mut self, event: Event, channel: Box<dyn NotificationChannel>) {
        self.routes.entry(event).or_default().push(channel);
    }

    /// Register a channel for multiple events at once.
    fn register_for_events(
        &mut self,
        events: &[Event],
        channel_factory: impl Fn() -> Box<dyn NotificationChannel>,
    ) {
        for event in events {
            self.register(event.clone(), channel_factory());
        }
    }

    /// Dispatch a notification to all channels registered for its event.
    fn dispatch(&self, ctx: &NotificationContext) -> DispatchReport {
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        if let Some(channels) = self.routes.get(&ctx.event) {
            for channel in channels {
                match channel.send(ctx) {
                    Ok(()) => successes.push(channel.channel_name().to_string()),
                    Err(e) => failures.push(e),
                }
            }
        }

        DispatchReport {
            event: ctx.event.clone(),
            successes,
            failures,
        }
    }

    /// List all registered routes (useful for debugging / admin).
    fn describe_routes(&self) {
        println!("Notification routes:");
        for (event, channels) in &self.routes {
            let names: Vec<_> = channels.iter().map(|c| c.to_string()).collect();
            println!("  {} -> [{}]", event, names.join(", "));
        }
    }
}

// ---------------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------------

fn main() {
    // -- Build the dispatcher and configure routing --
    let mut dispatcher = NotificationDispatcher::new();

    // UserSignup: email the user
    dispatcher.register(
        Event::UserSignup,
        Box::new(EmailChannel::new("smtp.example.com")),
    );

    // OrderPlaced: email the user
    dispatcher.register(
        Event::OrderPlaced,
        Box::new(EmailChannel::new("smtp.example.com")),
    );

    // PaymentFailed: email the user AND slack the ops team
    dispatcher.register(
        Event::PaymentFailed,
        Box::new(EmailChannel::new("smtp.example.com")),
    );
    dispatcher.register(
        Event::PaymentFailed,
        Box::new(SlackChannel::new(
            "https://hooks.slack.com/services/XXX",
            "ops-alerts",
        )),
    );

    // PasswordReset: email the user
    dispatcher.register(
        Event::PasswordReset,
        Box::new(EmailChannel::new("smtp.example.com")),
    );

    dispatcher.describe_routes();
    println!();

    // -- Dispatch some events --

    // 1. User signs up
    let ctx = NotificationContext::new(
        Event::UserSignup,
        "alice@example.com",
        "Welcome to our platform!",
        "Hi Alice, thanks for signing up.",
    );
    let report = dispatcher.dispatch(&ctx);
    println!("{}\n", report);

    // 2. Order placed
    let ctx = NotificationContext::new(
        Event::OrderPlaced,
        "bob@example.com",
        "Order #1234 confirmed",
        "Your order has been placed successfully.",
    );
    let report = dispatcher.dispatch(&ctx);
    println!("{}\n", report);

    // 3. Payment failed -- goes to email AND slack
    let ctx = NotificationContext::new(
        Event::PaymentFailed,
        "charlie@example.com",
        "Payment failed for order #5678",
        "We could not process your payment.",
    )
    .with_metadata("slack_channel", "ops-alerts")
    .with_metadata("order_id", "5678");
    let report = dispatcher.dispatch(&ctx);
    println!("{}\n", report);

    // 4. Password reset
    let ctx = NotificationContext::new(
        Event::PasswordReset,
        "dave@example.com",
        "Password reset requested",
        "Click the link to reset your password.",
    );
    let report = dispatcher.dispatch(&ctx);
    println!("{}\n", report);

    // -- Demonstrate adding SMS for payment failures (future expansion) --
    println!("--- After adding SMS channel for PaymentFailed ---\n");
    dispatcher.register(
        Event::PaymentFailed,
        Box::new(SmsChannel::new("sms-api-key-123")),
    );

    let ctx = NotificationContext::new(
        Event::PaymentFailed,
        "eve@example.com",
        "Payment failed for order #9999",
        "We could not process your payment.",
    )
    .with_metadata("phone", "+1-555-0199")
    .with_metadata("slack_channel", "ops-alerts");
    let report = dispatcher.dispatch(&ctx);
    println!("{}", report);
}
