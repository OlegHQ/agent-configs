# TCP Connection Handler Refactoring: State Pattern

## Problem

The original code used a `&'static str` to represent connection state with a single monolithic `handle_event` method containing nested match arms. Adding a new state required modifying this one large function in multiple places, and the compiler could not verify that all states were handled correctly.

## Pattern Applied: State

The **State** pattern is the textbook fit here. The skill's signal table identifies this directly: "Object changes behavior based on internal state" maps to the State pattern, and "Conditional logic switching on type/variant" confirms it.

## Key Design Decisions

**`self: Box<Self>` for state transitions.** Each state's `handle_event` consumes the current state and returns the next one. This is the critical Rust-idiomatic innovation -- the old state ceases to exist after a transition, making invalid state reuse impossible at compile time.

**Context passed down, not stored.** Shared mutable data (`buffer`, `retry_count`, `session_id`) lives in a `ConnectionContext` struct owned by `TcpConnection`. States receive `&mut ConnectionContext` as a parameter rather than holding a reference to it. This follows the skill's core principle of passing context down to avoid borrow checker conflicts.

**`Option<Box<dyn State>>` for temporary ownership transfer.** `TcpConnection` stores the state in an `Option` so `handle_event` can `.take()` ownership, pass it to the state's method, and store the returned next state. This is the standard Rust pattern for owned state machines.

**Dynamic dispatch (`Box<dyn State>`).** Since the concrete state type changes at runtime based on events, dynamic dispatch is the natural choice. Static dispatch would require the state type to be a generic parameter, which doesn't work when the type changes during execution.

## What Changed

| Before | After |
|---|---|
| State is a `&'static str` -- no compile-time safety | Each state is a distinct type -- compiler enforces correctness |
| One 60-line nested match handles all states | Each state is ~20 lines in its own `impl State` block |
| Adding a state means editing the central match | Adding a state means adding a new struct + `impl State` -- zero changes to existing states |
| Invalid state strings (`"bogus"`) only caught by `panic!` at runtime | Invalid states are structurally impossible |

## Tradeoffs

- Adds a trait + 4 structs where there was one match. For a state machine this size, the overhead is modest and pays for itself at the first new state addition.
- Each state allocation uses `Box`. For a connection handler this is negligible -- if profiling showed it mattered, an enum-based approach could eliminate the allocation while keeping the separation of concerns.
