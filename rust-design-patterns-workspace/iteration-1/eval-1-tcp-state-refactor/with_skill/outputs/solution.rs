use std::fmt;

// --- Shared context: data that all states can read/modify ---

struct ConnectionContext {
    buffer: Vec<u8>,
    retry_count: u32,
    session_id: Option<String>,
}

impl ConnectionContext {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            retry_count: 0,
            session_id: None,
        }
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.session_id = None;
        self.retry_count = 0;
    }
}

// --- State trait: each state consumes itself and returns the next state ---

trait State: fmt::Debug {
    /// Handle an event, consuming the current state and returning the next one.
    /// The context is passed down (not stored) per Rust idiom.
    fn handle_event(self: Box<Self>, event: &str, ctx: &mut ConnectionContext) -> Box<dyn State>;
}

// --- Concrete states ---

#[derive(Debug)]
struct Disconnected;

#[derive(Debug)]
struct Connecting;

#[derive(Debug)]
struct Connected;

#[derive(Debug)]
struct Error;

impl State for Disconnected {
    fn handle_event(self: Box<Self>, event: &str, ctx: &mut ConnectionContext) -> Box<dyn State> {
        match event {
            "connect" => {
                println!("Initiating connection...");
                ctx.retry_count = 0;
                Box::new(Connecting)
            }
            _ => {
                println!("Ignoring event {} in disconnected state", event);
                self
            }
        }
    }
}

impl State for Connecting {
    fn handle_event(self: Box<Self>, event: &str, ctx: &mut ConnectionContext) -> Box<dyn State> {
        match event {
            "connected" => {
                ctx.session_id = Some(format!("sess_{}", rand::random::<u32>()));
                println!("Connected with session {:?}", ctx.session_id);
                Box::new(Connected)
            }
            "timeout" => {
                ctx.retry_count += 1;
                if ctx.retry_count >= 3 {
                    println!("Max retries exceeded");
                    Box::new(Error)
                } else {
                    println!("Retrying... attempt {}", ctx.retry_count);
                    self
                }
            }
            _ => {
                println!("Ignoring event {} in connecting state", event);
                self
            }
        }
    }
}

impl State for Connected {
    fn handle_event(self: Box<Self>, event: &str, ctx: &mut ConnectionContext) -> Box<dyn State> {
        match event {
            "data" => {
                ctx.buffer.extend_from_slice(b"received data");
                println!("Data received, buffer size: {}", ctx.buffer.len());
                self
            }
            "disconnect" => {
                ctx.buffer.clear();
                ctx.session_id = None;
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

impl State for Error {
    fn handle_event(self: Box<Self>, event: &str, ctx: &mut ConnectionContext) -> Box<dyn State> {
        match event {
            "reset" => {
                ctx.reset();
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

// --- TcpConnection: owns the current state and context ---

struct TcpConnection {
    state: Option<Box<dyn State>>,
    ctx: ConnectionContext,
}

impl TcpConnection {
    fn new() -> Self {
        Self {
            state: Some(Box::new(Disconnected)),
            ctx: ConnectionContext::new(),
        }
    }

    fn handle_event(&mut self, event: &str) {
        // Take ownership of the current state, pass context down,
        // then store the returned next state.
        if let Some(state) = self.state.take() {
            self.state = Some(state.handle_event(event, &mut self.ctx));
        }
    }
}
