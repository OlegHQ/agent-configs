use std::time::Duration;

// --- State definitions ---

/// Each state is its own struct, holding only the data relevant to that state.
/// Adding a new state means: define a struct, implement `State` for it, done.

trait State {
    /// Handle an event, returning the next state (boxed) and optionally mutating
    /// shared connection data.
    fn handle_event(self: Box<Self>, event: &str, shared: &mut SharedData) -> Box<dyn State>;

    fn name(&self) -> &'static str;
}

/// Data that travels with the connection regardless of state.
struct SharedData {
    buffer: Vec<u8>,
    session_id: Option<String>,
}

// --- Concrete states ---

struct Disconnected;

impl State for Disconnected {
    fn name(&self) -> &'static str {
        "disconnected"
    }

    fn handle_event(self: Box<Self>, event: &str, _shared: &mut SharedData) -> Box<dyn State> {
        match event {
            "connect" => {
                println!("Initiating connection...");
                Box::new(Connecting { retry_count: 0 })
            }
            _ => {
                println!("Ignoring event {} in disconnected state", event);
                self
            }
        }
    }
}

struct Connecting {
    retry_count: u32,
}

impl State for Connecting {
    fn name(&self) -> &'static str {
        "connecting"
    }

    fn handle_event(self: Box<Self>, event: &str, shared: &mut SharedData) -> Box<dyn State> {
        match event {
            "connected" => {
                shared.session_id = Some(format!("sess_{}", rand::random::<u32>()));
                println!("Connected with session {:?}", shared.session_id);
                Box::new(Connected)
            }
            "timeout" => {
                let new_count = self.retry_count + 1;
                if new_count >= 3 {
                    println!("Max retries exceeded");
                    Box::new(Error)
                } else {
                    println!("Retrying... attempt {}", new_count);
                    Box::new(Connecting {
                        retry_count: new_count,
                    })
                }
            }
            _ => {
                println!("Ignoring event {} in connecting state", event);
                self
            }
        }
    }
}

struct Connected;

impl State for Connected {
    fn name(&self) -> &'static str {
        "connected"
    }

    fn handle_event(self: Box<Self>, event: &str, shared: &mut SharedData) -> Box<dyn State> {
        match event {
            "data" => {
                shared.buffer.extend_from_slice(b"received data");
                println!("Data received, buffer size: {}", shared.buffer.len());
                self
            }
            "disconnect" => {
                shared.buffer.clear();
                shared.session_id = None;
                println!("Disconnected");
                Box::new(Disconnected)
            }
            "error" => {
                println!("Connection error");
                Box::new(Error)
            }
            _ => {
                println!("Ignoring event {} in connected state", event);
                self
            }
        }
    }
}

struct Error;

impl State for Error {
    fn name(&self) -> &'static str {
        "error"
    }

    fn handle_event(self: Box<Self>, event: &str, shared: &mut SharedData) -> Box<dyn State> {
        match event {
            "reset" => {
                shared.buffer.clear();
                shared.session_id = None;
                println!("Reset to disconnected");
                Box::new(Disconnected)
            }
            _ => {
                println!("Ignoring event {} in error state", event);
                self
            }
        }
    }
}

// --- Connection facade ---

struct TcpConnection {
    state: Box<dyn State>,
    shared: SharedData,
}

impl TcpConnection {
    fn new() -> Self {
        TcpConnection {
            state: Box::new(Disconnected),
            shared: SharedData {
                buffer: Vec::new(),
                session_id: None,
            },
        }
    }

    fn handle_event(&mut self, event: &str) {
        // Take ownership of the current state, replacing it with a temporary.
        // This avoids needing an Option wrapper or unsafe code.
        let current_state = std::mem::replace(&mut self.state, Box::new(Disconnected));
        self.state = current_state.handle_event(event, &mut self.shared);
    }

    fn state_name(&self) -> &'static str {
        self.state.name()
    }
}
