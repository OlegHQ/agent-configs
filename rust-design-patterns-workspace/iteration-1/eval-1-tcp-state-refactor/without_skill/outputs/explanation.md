## Refactoring: nested match arms to the State pattern

### Problem

The original code encoded state as a `&'static str` and used a large nested
match statement in a single `handle_event` method. Adding a new state required
touching every branch of the outer match plus potentially several inner matches,
all in one growing function. It was also easy to introduce typos in the string
literals with no compile-time safety net.

### What changed

The refactored version applies the **State pattern** (also called the typestate
pattern when used at the type level, though here we use trait objects for runtime
flexibility):

1. **A `State` trait** with a single method `handle_event` that consumes the
   current state (`self: Box<Self>`) and returns the next state. Each state
   is responsible only for its own transitions.

2. **One struct per state** (`Disconnected`, `Connecting`, `Connected`, `Error`).
   State-specific data lives on the struct that needs it -- for example
   `retry_count` lives only on `Connecting`, not on the connection as a whole.

3. **`SharedData`** holds fields that persist across state transitions (`buffer`,
   `session_id`). The state's `handle_event` receives a `&mut SharedData`
   reference so it can read/write shared fields without owning the whole
   connection.

4. **`TcpConnection`** becomes a thin facade: it owns a `Box<dyn State>` and
   the shared data, and delegates events to whichever state is current.

### Benefits

- **Adding a new state** means writing one new struct and one `impl State` block.
  No existing code needs to change.
- **Compile-time safety**: states are types, not strings. A misspelled state name
  is a compile error, not a silent bug.
- **State-local data**: `retry_count` only exists while the connection is in the
  `Connecting` state. It is impossible to accidentally read it from `Connected`.
- **Easier testing**: each state can be unit-tested in isolation by constructing
  it directly and calling `handle_event`.
