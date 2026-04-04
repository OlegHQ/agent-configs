use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Every event that can trigger notifications.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// Contextual payload carried by every notification.
#[derive(Debug, Clone)]
struct NotificationPayload {
    event: Event,
    recipient: String,
    message: String,
    metadata: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Channel trait (the extension point)
// ---------------------------------------------------------------------------

/// Any delivery mechanism implements this trait.
/// Adding Slack / SMS / push later is just another impl.
trait NotificationChannel: fmt::Debug {
    fn name(&self) -> &str;
    fn send(&self, payload: &NotificationPayload) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// Concrete channels
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct EmailChannel;

impl NotificationChannel for EmailChannel {
    fn name(&self) -> &str {
        "Email"
    }

    fn send(&self, payload: &NotificationPayload) -> Result<(), String> {
        println!(
            "  [Email] To: {} | Subject: {} | Body: {}",
            payload.recipient, payload.event, payload.message
        );
        Ok(())
    }
}

#[derive(Debug)]
struct SlackChannel {
    webhook_url: String,
}

impl SlackChannel {
    fn new(webhook_url: &str) -> Self {
        Self {
            webhook_url: webhook_url.to_string(),
        }
    }
}

impl NotificationChannel for SlackChannel {
    fn name(&self) -> &str {
        "Slack"
    }

    fn send(&self, payload: &NotificationPayload) -> Result<(), String> {
        println!(
            "  [Slack -> {}] #{}: {} (re: {})",
            self.webhook_url, payload.recipient, payload.message, payload.event
        );
        Ok(())
    }
}

#[derive(Debug)]
struct SmsChannel;

impl NotificationChannel for SmsChannel {
    fn name(&self) -> &str {
        "SMS"
    }

    fn send(&self, payload: &NotificationPayload) -> Result<(), String> {
        println!(
            "  [SMS] To: {} | {}: {}",
            payload.recipient, payload.event, payload.message
        );
        Ok(())
    }
}

#[derive(Debug)]
struct PushChannel;

impl NotificationChannel for PushChannel {
    fn name(&self) -> &str {
        "Push"
    }

    fn send(&self, payload: &NotificationPayload) -> Result<(), String> {
        println!(
            "  [Push] Device: {} | {}: {}",
            payload.recipient, payload.event, payload.message
        );
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Dispatcher — routes events to one or more channels
// ---------------------------------------------------------------------------

struct NotificationDispatcher {
    /// Map from event to the list of channels that should fire.
    routes: HashMap<Event, Vec<Box<dyn NotificationChannel>>>,
}

impl NotificationDispatcher {
    fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Register a channel for a specific event.
    /// The same event can have many channels (fan-out).
    fn subscribe(&mut self, event: Event, channel: Box<dyn NotificationChannel>) {
        self.routes.entry(event).or_default().push(channel);
    }

    /// Fire all channels registered for the given event.
    /// Returns a vec of per-channel results so the caller can handle
    /// partial failures (e.g. email succeeded but Slack timed out).
    fn dispatch(&self, payload: &NotificationPayload) -> Vec<Result<(), String>> {
        let mut results = Vec::new();

        if let Some(channels) = self.routes.get(&payload.event) {
            for channel in channels {
                let result = channel.send(payload);
                if let Err(ref e) = result {
                    eprintln!(
                        "  [WARN] {} channel failed for {}: {}",
                        channel.name(),
                        payload.event,
                        e
                    );
                }
                results.push(result);
            }
        } else {
            eprintln!(
                "  [INFO] No channels registered for event: {}",
                payload.event
            );
        }

        results
    }
}

// ---------------------------------------------------------------------------
// Convenience builder for setting up common routing rules
// ---------------------------------------------------------------------------

fn build_dispatcher() -> NotificationDispatcher {
    let mut dispatcher = NotificationDispatcher::new();

    // User signup -> email welcome
    dispatcher.subscribe(Event::UserSignup, Box::new(EmailChannel));

    // Order placed -> email confirmation
    dispatcher.subscribe(Event::OrderPlaced, Box::new(EmailChannel));

    // Payment failed -> email the user AND slack the ops team
    dispatcher.subscribe(Event::PaymentFailed, Box::new(EmailChannel));
    dispatcher.subscribe(
        Event::PaymentFailed,
        Box::new(SlackChannel::new("https://hooks.slack.com/services/ops-alerts")),
    );

    // Password reset -> email only
    dispatcher.subscribe(Event::PasswordReset, Box::new(EmailChannel));

    dispatcher
}

// ---------------------------------------------------------------------------
// Helper to create payloads
// ---------------------------------------------------------------------------

fn make_payload(event: Event, recipient: &str, message: &str) -> NotificationPayload {
    NotificationPayload {
        event,
        recipient: recipient.to_string(),
        message: message.to_string(),
        metadata: HashMap::new(),
    }
}

// ---------------------------------------------------------------------------
// Main — exercises every event path
// ---------------------------------------------------------------------------

fn main() {
    let dispatcher = build_dispatcher();

    println!("=== Notification System Demo ===\n");

    // 1. User signup
    println!("[Event] UserSignup");
    let payload = make_payload(
        Event::UserSignup,
        "alice@example.com",
        "Welcome to our platform, Alice!",
    );
    dispatcher.dispatch(&payload);
    println!();

    // 2. Order placed
    println!("[Event] OrderPlaced");
    let payload = make_payload(
        Event::OrderPlaced,
        "bob@example.com",
        "Your order #1042 has been confirmed.",
    );
    dispatcher.dispatch(&payload);
    println!();

    // 3. Payment failed — fan-out to email + slack
    println!("[Event] PaymentFailed (multi-channel)");
    let payload = make_payload(
        Event::PaymentFailed,
        "carol@example.com",
        "Payment for order #1043 was declined. Please update your card.",
    );
    let results = dispatcher.dispatch(&payload);
    println!("  Channels notified: {}", results.len());
    println!();

    // 4. Password reset
    println!("[Event] PasswordReset");
    let payload = make_payload(
        Event::PasswordReset,
        "dave@example.com",
        "Click here to reset your password (link expires in 15 min).",
    );
    dispatcher.dispatch(&payload);
    println!();

    // 5. Show extensibility: add SMS + Push at runtime
    println!("--- Adding SMS and Push channels at runtime ---\n");
    let mut extended = build_dispatcher();
    extended.subscribe(Event::OrderPlaced, Box::new(SmsChannel));
    extended.subscribe(Event::OrderPlaced, Box::new(PushChannel));

    println!("[Event] OrderPlaced (now with SMS + Push)");
    let payload = make_payload(
        Event::OrderPlaced,
        "eve@example.com",
        "Your order #1044 is confirmed.",
    );
    let results = extended.dispatch(&payload);
    println!("  Channels notified: {}", results.len());
}
