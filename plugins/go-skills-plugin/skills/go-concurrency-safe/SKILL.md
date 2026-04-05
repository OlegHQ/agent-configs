---
name: go-concurrency-safe
description: Correct concurrent Go. Avoid goroutine leaks, races, deadlocks. Real bugs documented. Use for any concurrent code.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go Concurrency Safety

## When to Use

- Writing any code with goroutines
- Managing shared state with mutexes
- Implementing worker pools
- Handling channel communication
- Before running race detector

---

These patterns come from real bugs found in production code.

## Always Defer Cancel

```go
// GOOD - cancel is deferred immediately
ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
defer cancel()
result, err := doWork(ctx)
```

## Never Defer Inside Loops

```go
// BAD - defer in loop accumulates, resources leak until function returns
for _, item := range items {
    ctx, cancel := context.WithTimeout(ctx, time.Second)
    defer cancel()  // LEAK! defers stack up until loop ends
    doWork(ctx, item)
}

// GOOD - explicit cancel in loop
for _, item := range items {
    ctx, cancel := context.WithTimeout(ctx, time.Second)
    doWork(ctx, item)
    cancel()  // explicit, immediate cleanup
}

// GOOD - extract to function if cleanup is complex
for _, item := range items {
    if err := processItem(ctx, item); err != nil {
        return err
    }
}

func processItem(ctx context.Context, item Item) error {
    ctx, cancel := context.WithTimeout(ctx, time.Second)
    defer cancel()  // OK - function scope is small
    return doWork(ctx, item)
}
```

## Initialize Before Spawning Goroutines

```go
// BAD - struct fields may be nil when goroutine accesses them
s := &Stream{}
go s.readLoop()  // s.msgCh might be nil!
s.msgCh = make(chan Message)

// GOOD - initialize everything first
s := &Stream{
    msgCh:  make(chan Message),
    doneCh: make(chan struct{}),
}
go s.readLoop()  // all fields ready
```

## One Reader Per Channel

```go
// BAD - multiple readers race
go reader1(ch)  // might get message
go reader2(ch)  // might get same message race!

// GOOD - single reader, fan out if needed
go func() {
    for msg := range ch {
        reader1(msg)
        reader2(msg)
    }
}()
```

## Never Silent Drop with Select Default

```go
// BAD - silently drops messages when channel is full
select {
case ch <- msg:
case default:  // message lost, no one knows!
}

// GOOD - block or error explicitly
select {
case ch <- msg:
case <-ctx.Done():
    return ctx.Err()
}

// GOOD - if dropping is intentional, log it
select {
case ch <- msg:
default:
    log.Warn("dropping message, channel full")
}
```

## Bound Channel Buffers

```go
// BAD - unbounded can grow forever
pending := make(map[string]chan Response)  // grows without limit

// GOOD - bounded with clear limit
const maxPending = 1000
pending := make(map[string]chan Response)
if len(pending) >= maxPending {
    return ErrTooManyPending
}
```

## Always Use -race in Tests

```bash
# ALWAYS run tests with race detector
go test -race ./...

# In CI/CD
go test -race -v ./...
```

## Mutex Patterns

```go
// GOOD - RWMutex for read-heavy workloads
type Cache struct {
    mu    sync.RWMutex
    items map[string]Item
}

func (c *Cache) Get(key string) (Item, bool) {
    c.mu.RLock()
    defer c.mu.RUnlock()
    item, ok := c.items[key]
    return item, ok
}

func (c *Cache) Set(key string, item Item) {
    c.mu.Lock()
    defer c.mu.Unlock()
    c.items[key] = item
}

// GOOD - minimize lock scope
func (c *Cache) GetOrCreate(key string, create func() Item) Item {
    // Try read first
    c.mu.RLock()
    if item, ok := c.items[key]; ok {
        c.mu.RUnlock()
        return item
    }
    c.mu.RUnlock()

    // Need to create - get write lock
    c.mu.Lock()
    defer c.mu.Unlock()

    // Double-check (another goroutine might have created it)
    if item, ok := c.items[key]; ok {
        return item
    }

    item := create()
    c.items[key] = item
    return item
}
```

## Goroutine Lifecycle

```go
// GOOD - clear ownership and shutdown
type Worker struct {
    doneCh chan struct{}
    wg     sync.WaitGroup
}

func (w *Worker) Start() {
    w.doneCh = make(chan struct{})
    w.wg.Add(1)
    go w.run()
}

func (w *Worker) run() {
    defer w.wg.Done()
    for {
        select {
        case <-w.doneCh:
            return
        case work := <-w.workCh:
            w.process(work)
        }
    }
}

func (w *Worker) Stop() {
    close(w.doneCh)
    w.wg.Wait()  // wait for clean shutdown
}
```

## Quick Checklist

Before any concurrent code:
- [ ] `defer cancel()` after every `context.With*`?
- [ ] No `defer` inside loops?
- [ ] All struct fields initialized before `go`?
- [ ] Single reader per channel?
- [ ] No silent drops in `select`?
- [ ] Channel buffers bounded?
- [ ] Tests run with `-race`?

---

## Integration

This skill works with:
- **go-context-patterns**: Context cancellation in goroutines
- **go-htmx-sse**: Fan-out hub pattern
- **go-testing-simple**: Race detector in tests
- **go-claude-agents**: Agent streaming patterns

Reference this skill before writing any concurrent code.
