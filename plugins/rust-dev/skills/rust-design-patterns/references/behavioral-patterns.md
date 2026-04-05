# Behavioral Patterns in Rust

Behavioral patterns manage algorithms and responsibilities between objects. In Rust, most require
dynamic dispatch (`dyn Trait`) because behavior is inherently runtime-determined. However, several
have idiomatic functional alternatives using closures.

---

## Chain of Responsibility

**Problem:** Pass a request through a chain of handlers, each deciding whether to process or forward.

```rust
trait Handler {
    fn handle(&mut self, patient: &mut Patient) -> Option<String>;
    fn set_next(&mut self, next: Box<dyn Handler>);
}

struct Reception {
    next: Option<Box<dyn Handler>>,
}

impl Handler for Reception {
    fn handle(&mut self, patient: &mut Patient) -> Option<String> {
        if patient.registered {
            // Already registered — pass to next handler
            self.next.as_mut()?.handle(patient)
        } else {
            patient.registered = true;
            println!("Patient registered at reception");
            self.next.as_mut()?.handle(patient)
        }
    }

    fn set_next(&mut self, next: Box<dyn Handler>) {
        self.next = Some(next);
    }
}
```

**Alternative — Vec pipeline:**
```rust
fn process(patient: &mut Patient, handlers: &mut [Box<dyn Handler>]) {
    for handler in handlers.iter_mut() {
        if let Some(result) = handler.handle(patient) {
            println!("{result}");
            return;
        }
    }
}
```

**When to use:** Request processing pipelines, middleware stacks, validation chains.

---

## Command

**Problem:** Encapsulate actions as objects — for undo, queuing, logging, or deferred execution.

```rust
trait Command {
    /// Key Rust insight: pass context as parameter, don't store a reference to it.
    /// This avoids borrow checker conflicts from stored mutable references.
    fn execute(&mut self, app: &mut Editor);
    fn undo(&mut self, app: &mut Editor);
}

struct CopyCommand {
    backup: String,
}

impl Command for CopyCommand {
    fn execute(&mut self, app: &mut Editor) {
        self.backup = app.clipboard.clone();
        app.clipboard = app.get_selection().to_string();
    }

    fn undo(&mut self, app: &mut Editor) {
        app.clipboard = self.backup.clone();
    }
}

struct Editor {
    text: String,
    clipboard: String,
    history: Vec<Box<dyn Command>>,
}

impl Editor {
    fn execute_command(&mut self, mut cmd: Box<dyn Command>) {
        cmd.execute(self);
        self.history.push(cmd);
    }

    fn undo(&mut self) {
        if let Some(mut cmd) = self.history.pop() {
            cmd.undo(self);
        }
    }
}
```

**Critical Rust detail:** A command instance should NOT hold a permanent `&mut` reference to the
editor/context. Instead, the context is passed into `execute()` and `undo()` from the top down.
This is the idiomatic Rust approach and avoids fighting the borrow checker.

---

## Iterator

**Problem:** Traverse elements without exposing underlying data structure.

```rust
struct UserCollection {
    users: Vec<User>,
}

impl UserCollection {
    fn iter(&self) -> impl Iterator<Item = &User> {
        self.users.iter()
    }
}

// Or implement Iterator trait for custom traversal
struct FibonacciIterator { a: u64, b: u64 }

impl Iterator for FibonacciIterator {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let result = self.a;
        let new_b = self.a + self.b;
        self.a = self.b;
        self.b = new_b;
        Some(result)
    }
}
```

**Already built into Rust** — the `Iterator` trait, combinators (`.map()`, `.filter()`, `.fold()`,
`.flat_map()`), and `for` loops are the standard iteration mechanism. Implement `Iterator` for
custom sequences; delegate to inner `iter()` for collections.

---

## Mediator

**Problem:** Reduce chaotic dependencies between objects by centralizing communication.

**This is the hardest pattern in Rust** because the classic OOP version requires multiple objects
holding mutable cross-references — exactly what the borrow checker prevents.

**Top-Down Ownership (recommended):**
```rust
struct TrainStation {
    trains: Vec<Train>,
    platform_occupied: bool,
}

impl TrainStation {
    fn notify_about_arrival(&mut self, train_idx: usize) {
        if self.platform_occupied {
            println!("Train {} waiting", self.trains[train_idx].name);
        } else {
            self.platform_occupied = true;
            self.trains[train_idx].arrive();
        }
    }

    fn notify_about_departure(&mut self, train_idx: usize) {
        self.trains[train_idx].depart();
        self.platform_occupied = false;
        // Notify other trains the platform is free
    }
}
```

**Key insight:** The mediator OWNS all components. Components don't store references to the mediator.
Instead, the mediator passes itself (or relevant data) into component methods when needed. Control
flows top-down from `main()`.

**Rc<RefCell<...>> approach (use sparingly):**
```rust
use std::rc::Rc;
use std::cell::RefCell;

// Components hold Rc<RefCell<Mediator>> — runtime borrow checking
// Works but can panic at runtime if borrows overlap
// Only use when top-down ownership truly doesn't fit
```

---

## Memento

**Problem:** Save and restore previous state without exposing internal details.

**Manual approach:**
```rust
struct EditorState {
    content: String,
    cursor_position: usize,
}

struct Editor {
    content: String,
    cursor_position: usize,
}

impl Editor {
    fn save(&self) -> EditorState {
        EditorState {
            content: self.content.clone(),
            cursor_position: self.cursor_position,
        }
    }

    fn restore(&mut self, state: EditorState) {
        self.content = state.content;
        self.cursor_position = state.cursor_position;
    }
}
```

**Serde approach (more flexible):**
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Editor {
    content: String,
    cursor_position: usize,
}

impl Editor {
    fn save(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn restore(snapshot: &str) -> Self {
        serde_json::from_str(snapshot).unwrap()
    }
}
```

**When to use:** Undo systems, checkpoints, transaction rollback. The serde approach is more
flexible but has serialization overhead.

---

## Observer

**Problem:** Objects need to react to events from another object without tight coupling.

**Function pointer approach (idiomatic Rust):**
```rust
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Clone)]
enum Event { Open, Save, Close }

struct EventPublisher {
    listeners: HashMap<Event, Vec<fn(String)>>,
}

impl EventPublisher {
    fn subscribe(&mut self, event: Event, listener: fn(String)) {
        self.listeners.entry(event).or_default().push(listener);
    }

    fn unsubscribe(&mut self, event: Event, listener: fn(String)) {
        if let Some(listeners) = self.listeners.get_mut(&event) {
            listeners.retain(|&l| l != listener);
        }
    }

    fn notify(&self, event: &Event, data: String) {
        if let Some(listeners) = self.listeners.get(event) {
            for listener in listeners {
                listener(data.clone());
            }
        }
    }
}
```

**When closures are needed (capturing state):**
```rust
struct EventPublisher {
    listeners: HashMap<Event, Vec<Box<dyn Fn(String)>>>,
}
// Note: Box<dyn Fn> closures cannot be compared for equality,
// so unsubscription requires an ID-based approach instead
```

**When to use:** Event systems, UI updates, logging hooks, plugin architectures. Prefer function
pointers for simple cases; `Box<dyn Fn>` when closures need captured state.

---

## State

**Problem:** Object changes behavior based on internal state — replacing state-conditional logic.

```rust
trait State {
    fn play(self: Box<Self>, player: &mut Player) -> Box<dyn State>;
    fn stop(self: Box<Self>, player: &mut Player) -> Box<dyn State>;
}

struct StoppedState;
struct PlayingState;

impl State for StoppedState {
    fn play(self: Box<Self>, player: &mut Player) -> Box<dyn State> {
        player.start_playback();
        Box::new(PlayingState)  // Transition: Stopped -> Playing
    }
    fn stop(self: Box<Self>, _player: &mut Player) -> Box<dyn State> {
        self  // Already stopped — no transition
    }
}

impl State for PlayingState {
    fn play(self: Box<Self>, _player: &mut Player) -> Box<dyn State> {
        self  // Already playing
    }
    fn stop(self: Box<Self>, player: &mut Player) -> Box<dyn State> {
        player.stop_playback();
        Box::new(StoppedState)  // Transition: Playing -> Stopped
    }
}

struct Player {
    state: Option<Box<dyn State>>,  // Option for temporary ownership transfer
}

impl Player {
    fn play(&mut self) {
        if let Some(state) = self.state.take() {
            self.state = Some(state.play(self));
        }
    }
}
```

**Critical Rust innovation:** `self: Box<Self>` — the method *consumes* the current state and
returns the next one. Invalid state transitions become impossible at the type level because the
old state no longer exists after transition.

**When to use vs. enums:** Use the State pattern when states have significantly different behavior
and new states may be added. Use enums (`match`) when the state set is small, fixed, and behavior
differences are minor.

---

## Strategy

**Problem:** Define a family of interchangeable algorithms.

**Trait-based (for complex strategies):**
```rust
trait RouteStrategy {
    fn build_route(&self, from: &str, to: &str) -> Vec<String>;
    fn estimated_time(&self, from: &str, to: &str) -> Duration;
}

struct WalkingStrategy;
struct DrivingStrategy;

impl RouteStrategy for WalkingStrategy {
    fn build_route(&self, from: &str, to: &str) -> Vec<String> { /* ... */ }
    fn estimated_time(&self, from: &str, to: &str) -> Duration { /* ... */ }
}

struct Navigator<S: RouteStrategy> {
    strategy: S,
}
```

**Functional (preferred for single operations):**
```rust
fn navigate(from: &str, to: &str, strategy: fn(&str, &str) -> Vec<String>) -> Vec<String> {
    strategy(from, to)
}

// Usage — closures and function pointers work directly
navigate("A", "B", walking_route);
navigate("A", "B", |from, to| driving_route(from, to));
```

**This is everywhere in Rust:** `.sort_by()`, `.filter()`, `.map()` — all Strategy pattern via
closures. Use the trait-based approach only when the strategy has multiple methods or needs
associated state.

---

## Template Method

**Problem:** Define algorithm skeleton, let subtypes override specific steps.

```rust
trait DataMiner {
    /// Template method — defines the algorithm skeleton
    fn mine(&self, path: &str) -> Data {
        let file = self.open_file(path);
        let raw = self.extract_data(&file);
        let data = self.parse_data(&raw);
        self.analyze(&data);  // Default step
        data
    }

    // Steps that subtypes must implement
    fn open_file(&self, path: &str) -> File;
    fn extract_data(&self, file: &File) -> RawData;
    fn parse_data(&self, raw: &RawData) -> Data;

    // Step with default implementation — override if needed
    fn analyze(&self, data: &Data) {
        println!("Basic analysis: {} records", data.len());
    }
}
```

**Rust's default trait methods** make this pattern natural. Required methods (no body) are the
"abstract" steps; methods with bodies are the optional hooks.

---

## Visitor

**Problem:** Define new operations on a structure without modifying its types.

```rust
trait Visitor {
    fn visit_circle(&mut self, circle: &Circle);
    fn visit_rectangle(&mut self, rect: &Rectangle);
}

trait Shape {
    fn accept(&self, visitor: &mut dyn Visitor);
}

struct Circle { radius: f64 }
impl Shape for Circle {
    fn accept(&self, visitor: &mut dyn Visitor) { visitor.visit_circle(self); }
}

struct AreaCalculator { total: f64 }
impl Visitor for AreaCalculator {
    fn visit_circle(&mut self, c: &Circle) {
        self.total += std::f64::consts::PI * c.radius * c.radius;
    }
    fn visit_rectangle(&mut self, r: &Rectangle) {
        self.total += r.width * r.height;
    }
}
```

**When to use:** When you need to perform many different operations on a structure of objects and
don't want to pollute their types with every operation. Common for AST processing, serialization,
code generation.
