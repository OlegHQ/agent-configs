---
name: go-context-patterns
description: Context threading in Go. Cancellation, timeouts, values. All blocking ops take context. Use for any async or IO code.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go Context Patterns

## When to Use

- Any function doing I/O or blocking operations
- Implementing request timeouts
- Handling cancellation signals
- Passing request-scoped data (trace IDs)
- Managing graceful shutdown

---

## Context as First Parameter

Every function that does I/O or might block takes context as first param:

```go
// GOOD
func (r *UserRepo) FindByID(ctx context.Context, id primitive.ObjectID) (*User, error)
func (s *UserService) Create(ctx context.Context, email, name string) (*User, error)
func (c *HTTPClient) Get(ctx context.Context, url string) (*Response, error)

// BAD - no way to cancel or timeout
func (r *UserRepo) FindByID(id primitive.ObjectID) (*User, error)
```

## Timeouts

```go
// Set timeout for operation
ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
defer cancel()

user, err := repo.FindByID(ctx, id)
if errors.Is(err, context.DeadlineExceeded) {
    return nil, fmt.Errorf("find user timed out: %w", err)
}
```

## Cancellation

```go
// Create cancellable context
ctx, cancel := context.WithCancel(context.Background())

// Cancel from another goroutine (e.g., shutdown signal)
go func() {
    <-shutdownCh
    cancel()
}()

// Check cancellation in long operations
for _, item := range items {
    select {
    case <-ctx.Done():
        return ctx.Err()
    default:
    }
    process(item)
}
```

## HTTP Request Context

```go
func (h *UserHandler) Create(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()  // Get context from request

    // Context is cancelled when:
    // - Client disconnects
    // - Request timeout exceeded
    // - Server shuts down

    user, err := h.svc.Create(ctx, req.Email)
    if errors.Is(err, context.Canceled) {
        // Client disconnected, no point responding
        return
    }
}
```

## Context Values (Use Sparingly)

Only for request-scoped data that crosses API boundaries:

```go
// GOOD use cases for context values
type contextKey string

const (
    traceIDKey contextKey = "traceID"
    userIDKey  contextKey = "userID"
)

func WithTraceID(ctx context.Context, traceID string) context.Context {
    return context.WithValue(ctx, traceIDKey, traceID)
}

func TraceID(ctx context.Context) string {
    if v, ok := ctx.Value(traceIDKey).(string); ok {
        return v
    }
    return ""
}

// BAD - don't pass dependencies via context
ctx = context.WithValue(ctx, "db", db)      // NO! Pass as function param
ctx = context.WithValue(ctx, "logger", log) // NO! Pass as function param
```

## Never Store Context in Structs

```go
// BAD - context stored in struct
type Worker struct {
    ctx context.Context  // DON'T DO THIS
}

// GOOD - pass context to methods
type Worker struct {
    // no context field
}

func (w *Worker) Process(ctx context.Context, item Item) error {
    // use ctx for this operation
}
```

## Propagate Context Through Call Stack

```go
// Handler receives context from request
func (h *UserHandler) Create(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()
    user, err := h.svc.Create(ctx, req.Email)  // pass to service
}

// Service passes to repo
func (s *UserService) Create(ctx context.Context, email string) (*User, error) {
    return s.repo.Insert(ctx, user)  // pass to repo
}

// Repo passes to database driver
func (r *UserRepo) Insert(ctx context.Context, u *User) error {
    _, err := r.coll.InsertOne(ctx, u)  // pass to MongoDB
    return err
}
```

## Background Operations

```go
// When you need to outlive the request context
func (h *Handler) Create(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()

    // Do synchronous work with request context
    user, err := h.svc.Create(ctx, req)
    if err != nil {
        return
    }

    // Respond to client
    h.respondJSON(w, http.StatusCreated, user)

    // Fire-and-forget with new context (not tied to request)
    go func() {
        bgCtx, cancel := context.WithTimeout(context.Background(), time.Minute)
        defer cancel()
        h.analytics.Track(bgCtx, "user_created", user.ID)
    }()
}
```

## Quick Reference

```go
// Timeout
ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
defer cancel()

// Deadline
deadline := time.Now().Add(time.Minute)
ctx, cancel := context.WithDeadline(ctx, deadline)
defer cancel()

// Cancellation
ctx, cancel := context.WithCancel(ctx)
defer cancel()

// Check done
select {
case <-ctx.Done():
    return ctx.Err()
default:
}

// Get error reason
if ctx.Err() == context.Canceled {
    // explicitly cancelled
}
if ctx.Err() == context.DeadlineExceeded {
    // timeout
}
```

---

## Integration

This skill works with:
- **go-http-handlers**: Request context propagation
- **go-mongodb**: Database operation timeouts
- **go-htmx-sse**: SSE stream cancellation
- **go-concurrency-safe**: Goroutine lifecycle management

Reference this skill for any async or I/O code.
