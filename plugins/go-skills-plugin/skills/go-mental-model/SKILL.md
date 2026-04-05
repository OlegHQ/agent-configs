---
name: go-mental-model
description: Reduce code complexity for human understanding. Minimize indirection, file jumps, and mental load. Use before writing any Go code.
allowed-tools: Read, Glob, Grep
---

# Go Mental Model

## When to Use

- Before designing a new feature or module
- When reviewing code for complexity
- Deciding whether to add an abstraction
- Evaluating pull requests for indirection
- Onboarding to understand code flow

---

**Core Principle: If you can't explain the code path in one sentence, it's too complex.**

## Indirection Budget

```
MAX 3 layers between HTTP request and database
Handler → Service → Repository → MongoDB
   ↓         ↓           ↓
 (each layer does ONE clear thing)
```

## Mental Model Complexity Metrics

| Metric | Target | Why |
|--------|--------|-----|
| File jumps to trace a request | ≤ 4 | Reader can hold in head |
| Interfaces per package | ≤ 1 | Fewer abstractions to learn |
| Function params | ≤ 5 | Easy to understand signature |
| Nested conditionals | ≤ 2 | Flat code, early returns |
| Lines per function | ≤ 50 | Fits on screen |

## Indirection Anti-Patterns

```go
// BAD - 7 layers of indirection
Handler → Controller → UseCase → Service → Repository → DAO → Database
// Reader has to jump 7 files to understand ONE operation

// GOOD - 3 layers, each clear
Handler → Service → Repository
// Reader jumps 3 files max
```

## "Where's the code?" Test

```
Q: Where does user creation happen?
A: users/service.go:Create()

If you can't answer in < 5 seconds: TOO MUCH INDIRECTION
```

## Code Locality Rules

- Related code lives in same file (types + functions that use them)
- One feature = one package (`users/`, `orders/`)
- No "utils" or "helpers" packages (put code where it's used)
- No "models" package (put types with their behavior)

## Abstraction Rules

- **Interface:** Only if 2+ real implementations exist TODAY
- **Factory:** Only if construction is genuinely complex
- **Builder:** Almost never (use struct literals)
- **Wrapper:** Only if original API is genuinely bad

## Reading Order Test

A new developer should understand your code by reading:
1. `main.go` - see the full dependency graph
2. One handler - see request flow end to end
3. That's it. If they need more, code is too complex.

## Before Writing Code, Ask

1. How many files will a reader jump to trace this?
2. Can I explain the data flow in one sentence?
3. Would removing this abstraction make code simpler?
4. Is this interface justified by 2+ implementations?

## Quick Reference

```go
// GOOD - clear, direct
func (h *UserHandler) Create(w http.ResponseWriter, r *http.Request) {
    user, err := h.svc.Create(ctx, req.Email)  // 1 hop to service
    // svc.Create calls repo.Insert             // 1 hop to repo
    // repo.Insert calls MongoDB                // 1 hop to DB
    // Total: 3 hops, traceable in head
}

// BAD - too many hops
func (h *UserHandler) Create(w http.ResponseWriter, r *http.Request) {
    result := h.controller.Handle(ctx, req)     // hop 1
    // controller calls usecase                 // hop 2
    // usecase calls service                    // hop 3
    // service calls repository                 // hop 4
    // repository calls dao                     // hop 5
    // dao calls database                       // hop 6
    // Total: 6 hops, impossible to hold in head
}
```

---

## Integration

This skill works with:
- **go-clean-architecture**: Validates layer count and indirection
- **go-http-handlers**: Ensures handler simplicity
- **go-project-bootstrap**: Guides initial structure decisions

Reference this skill before writing any Go code to validate complexity.
