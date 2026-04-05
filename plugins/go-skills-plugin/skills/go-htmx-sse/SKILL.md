---
name: go-htmx-sse
description: Server-Sent Events for real-time updates with Go and HTMX. Streaming responses, live notifications, and clawde integration.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go + HTMX Server-Sent Events

Build real-time features using Server-Sent Events. Stream updates from Go to browser without WebSocket complexity.

## When to Use

- Real-time notifications and alerts
- Live log streaming
- Progress indicators for long operations
- Agent/AI response streaming (clawde)
- Live dashboards with auto-updating data
- Chat message delivery

---

## CRITICAL: sse-swap is REQUIRED

> **WARNING**: The `sse-swap` attribute is **REQUIRED** for HTMX to process SSE events. Without it, HTMX connects to the endpoint but ignores all incoming events!

### Common Mistake

```html
<!-- WRONG - HTMX ignores all events! -->
<div hx-ext="sse" sse-connect="/events">
    <div id="output"></div>
</div>
```

### Correct Pattern

```html
<!-- CORRECT - Events are captured and processed -->
<div hx-ext="sse" sse-connect="/events">
    <div id="output" sse-swap="message,update" hx-swap="beforeend"></div>
</div>
```

### Requirements Checklist

1. `hx-ext="sse"` - Enable the SSE extension
2. `sse-connect` - URL to connect to
3. **`sse-swap`** - **REQUIRED** - Comma-separated event types to listen for
4. `hx-swap` - How to swap content (innerHTML, beforeend, etc.)

---

## SSE vs WebSocket vs Polling

| Feature | SSE | WebSocket | Polling |
|---------|-----|-----------|---------|
| Direction | Server → Client | Bidirectional | Client → Server |
| Complexity | Simple | Complex | Simple |
| Auto-reconnect | Built-in | Manual | N/A |
| Browser support | All modern | All modern | All |
| HTTP/2 multiplexing | Yes | No | Yes |
| Best for | Updates, streams | Chat, games | Legacy |

**Choose SSE when**: You need server-to-client updates (most cases).

---

## Basic SSE Handler

### Go Handler with http.Flusher

```go
func handleSSE(w http.ResponseWriter, r *http.Request) {
    // Set SSE headers
    w.Header().Set("Content-Type", "text/event-stream")
    w.Header().Set("Cache-Control", "no-cache")
    w.Header().Set("Connection", "keep-alive")

    // Get flusher for streaming
    flusher, ok := w.(http.Flusher)
    if !ok {
        http.Error(w, "SSE not supported", http.StatusInternalServerError)
        return
    }

    // Stream events until client disconnects
    ctx := r.Context()
    ticker := time.NewTicker(time.Second)
    defer ticker.Stop()

    for {
        select {
        case <-ctx.Done():
            return  // Client disconnected
        case t := <-ticker.C:
            fmt.Fprintf(w, "data: %s\n\n", t.Format(time.RFC3339))
            flusher.Flush()
        }
    }
}
```

### Register Route

```go
mux.HandleFunc("GET /events", handleSSE)
```

---

## SSE Event Format

### Basic Event

```
data: Hello World\n\n
```

### Event with Type

```
event: notification
data: {"message": "New item created"}

```

### Multi-line Data

```
data: Line 1
data: Line 2
data: Line 3

```

### Event with ID (for reconnection)

```
id: 123
event: update
data: {"count": 42}

```

### Retry Interval (milliseconds)

```
retry: 5000
data: reconnect in 5s if disconnected

```

---

## Go Event Helpers

### Event Writer

```go
type SSEWriter struct {
    w       http.ResponseWriter
    flusher http.Flusher
}

func NewSSEWriter(w http.ResponseWriter) (*SSEWriter, error) {
    flusher, ok := w.(http.Flusher)
    if !ok {
        return nil, fmt.Errorf("streaming not supported")
    }

    w.Header().Set("Content-Type", "text/event-stream")
    w.Header().Set("Cache-Control", "no-cache")
    w.Header().Set("Connection", "keep-alive")

    return &SSEWriter{w: w, flusher: flusher}, nil
}

func (s *SSEWriter) SendEvent(event, data string) {
    if event != "" {
        fmt.Fprintf(s.w, "event: %s\n", event)
    }
    fmt.Fprintf(s.w, "data: %s\n\n", data)
    s.flusher.Flush()
}

func (s *SSEWriter) SendData(data string) {
    fmt.Fprintf(s.w, "data: %s\n\n", data)
    s.flusher.Flush()
}

func (s *SSEWriter) SendHTML(event string, html string) {
    // Escape newlines in HTML for SSE
    escaped := strings.ReplaceAll(html, "\n", "")
    s.SendEvent(event, escaped)
}
```

### Usage

```go
func handleStream(w http.ResponseWriter, r *http.Request) {
    sse, err := NewSSEWriter(w)
    if err != nil {
        http.Error(w, err.Error(), 500)
        return
    }

    for msg := range messages {
        sse.SendEvent("message", msg)
    }
}
```

---

## HTMX SSE Extension

### Load Extension

```html
<script src="/static/js/htmx.min.js"></script>
<script src="/static/js/sse.js"></script>
```

Or from CDN:
```html
<script src="https://unpkg.com/htmx-ext-sse@2.2.2/sse.js"></script>
```

### Connect to SSE Endpoint

Use a wrapper element for the connection and child elements for event handling:

```html
<!-- Wrapper establishes connection -->
<div hx-ext="sse" sse-connect="/events">
    <!-- Child handles specific events -->
    <div sse-swap="message" hx-swap="beforeend">
        <!-- Messages appear here -->
    </div>
</div>
```

### Multiple Event Types

List event types in comma-separated format:

```html
<div hx-ext="sse" sse-connect="/events">
    <div sse-swap="message,notification,alert" hx-swap="beforeend">
        <!-- All three event types append here -->
    </div>
</div>
```

### Different Handlers for Different Events

```html
<div hx-ext="sse" sse-connect="/events">
    <div id="messages" sse-swap="message" hx-swap="beforeend">
        <!-- Messages append here -->
    </div>
    <div id="status" sse-swap="status" hx-swap="innerHTML">
        <!-- Status replaces content -->
    </div>
</div>
```

---

## OOB-Only Events

Some SSE events only update elements via Out-of-Band swaps (using `hx-swap-oob` in the HTML content). These events still need to be captured by `sse-swap`, even though they don't need a primary swap target.

### The Problem

OOB attributes in the HTML response are **only processed if the event is captured** via `sse-swap`. If no element has `sse-swap` for that event type, the event is ignored entirely.

### Solution: Hidden Handler

Use a hidden div with `hx-swap="none"` to capture OOB-only events:

```html
<div hx-ext="sse" sse-connect="/events">
    <!-- Events that append content to the stream -->
    <div id="stream" sse-swap="message,tool_use" hx-swap="beforeend">
        <!-- Messages and tool cards appear here -->
    </div>

    <!-- OOB-only events (update other elements, no primary swap target) -->
    <div sse-swap="tool_result,status,counter" hx-swap="none" style="display:none;"></div>
</div>

<!-- These elements are updated via hx-swap-oob in the event HTML -->
<div id="tool-output-123"><!-- Updated by tool_result --></div>
<div id="current-status"><!-- Updated by status --></div>
<span id="message-count"><!-- Updated by counter --></span>
```

### Server-Side OOB HTML

```go
// tool_result event HTML with OOB swap
func renderToolResult(toolID, output string) string {
    return fmt.Sprintf(`<div id="tool-output-%s" hx-swap-oob="innerHTML:#tool-output-%s">
        <pre>%s</pre>
    </div>`, toolID, toolID, output)
}
```

### Common OOB Use Cases

| Event Type | OOB Target | Use Case |
|------------|------------|----------|
| `tool_result` | `#tool-output-{id}` | Update tool card with output |
| `counter` | `#message-count` | Update running count |
| `status` | `#status-badge` | Update status indicator |
| `todos` | `#todo-list` | Replace todo panel |
| `result` | `#result-banner` | Show completion banner |

---

## Templ Components for SSE

### Complete Stream Container

```templ
templ StreamContainer(endpoint string) {
    <!-- SSE Connection Wrapper -->
    <div hx-ext="sse" sse-connect={ endpoint }>
        <!-- Stream content - events append here -->
        <div
            id="stream"
            class="space-y-2"
            sse-swap="message,notification,tool_use"
            hx-swap="beforeend"
        >
            <div class="text-gray-500">Connecting...</div>
        </div>

        <!-- Hidden handler for OOB-only events -->
        <div
            sse-swap="tool_result,status,done"
            hx-swap="none"
            style="display:none;"
        ></div>
    </div>

    <!-- OOB targets (outside SSE wrapper is fine) -->
    <div id="status-panel"><!-- Updated by status event --></div>
    <div id="result-banner" style="display:none;"><!-- Updated by done event --></div>
}
```

### Live Notifications

```templ
templ LiveNotifications() {
    <div hx-ext="sse" sse-connect="/notifications/stream">
        <div
            sse-swap="notification"
            hx-swap="beforeend"
            class="space-y-2"
        >
            <!-- Notifications appear here -->
        </div>
    </div>
}
```

### Notification Fragment

```templ
templ NotificationItem(msg Notification) {
    <div class="p-4 bg-blue-50 rounded-lg border border-blue-200 animate-fade-in">
        <p class="font-medium">{ msg.Title }</p>
        <p class="text-sm text-gray-600">{ msg.Body }</p>
    </div>
}
```

### Handler Sending HTML Fragment

```go
func handleNotificationStream(w http.ResponseWriter, r *http.Request) {
    sse, _ := NewSSEWriter(w)
    ctx := r.Context()

    for {
        select {
        case <-ctx.Done():
            return
        case notification := <-notificationChan:
            var buf bytes.Buffer
            NotificationItem(notification).Render(ctx, &buf)
            sse.SendHTML("notification", buf.String())
        }
    }
}
```

---

## Clawde Integration

### Streaming Agent Responses

```go
func handleAgentStream(w http.ResponseWriter, r *http.Request) {
    sse, err := NewSSEWriter(w)
    if err != nil {
        http.Error(w, err.Error(), 500)
        return
    }

    client := clawde.NewClient(
        clawde.WithModel("claude-sonnet-4-20250514"),
        clawde.WithSystemPrompt("You are a helpful assistant."),
    )

    prompt := r.URL.Query().Get("prompt")
    ctx := r.Context()

    stream, err := client.Query(ctx, prompt)
    if err != nil {
        sse.SendEvent("error", err.Error())
        return
    }

    for stream.Next() {
        msg := stream.Current()

        switch m := msg.(type) {
        case *clawde.AssistantMessage:
            // Send text chunks
            for _, block := range m.Content {
                if text, ok := block.(*clawde.TextBlock); ok {
                    var buf bytes.Buffer
                    MessageChunk(text.Text).Render(ctx, &buf)
                    sse.SendHTML("chunk", buf.String())
                }
            }

        case *clawde.ToolUseMessage:
            // Send tool use notification
            var buf bytes.Buffer
            ToolUseCard(m.Name, m.Input).Render(ctx, &buf)
            sse.SendHTML("tool", buf.String())

        case *clawde.ResultMessage:
            // Send completion
            sse.SendEvent("done", `{"status": "complete"}`)
        }
    }

    if err := stream.Err(); err != nil {
        sse.SendEvent("error", err.Error())
    }
}
```

### Message Components

```templ
templ MessageChunk(text string) {
    <span>{ text }</span>
}

templ ToolUseCard(name string, input string) {
    <div class="bg-gray-100 rounded p-2 my-2 text-sm">
        <span class="font-mono text-blue-600">{ name }</span>
    </div>
}

templ AgentStreamContainer(prompt string) {
    <!-- SSE Connection Wrapper -->
    <div hx-ext="sse" sse-connect={ "/agent/stream?prompt=" + prompt }>
        <!-- Stream events that append content -->
        <div
            id="output"
            class="space-y-2"
            sse-swap="chunk,tool,thinking"
            hx-swap="beforeend"
        >
            <!-- Streaming text and tool cards appear here -->
        </div>

        <!-- OOB-only events (status updates, completion) -->
        <div
            sse-swap="done,error,tool_result"
            hx-swap="none"
            style="display:none;"
        ></div>
    </div>

    <!-- OOB targets -->
    <div id="status"><!-- Updated by done/error events --></div>
}
```

---

## Fan-Out Hub Pattern

For broadcasting to multiple clients.

### Hub Implementation

```go
type Hub struct {
    clients    map[chan string]bool
    broadcast  chan string
    register   chan chan string
    unregister chan chan string
    mu         sync.RWMutex
}

func NewHub() *Hub {
    h := &Hub{
        clients:    make(map[chan string]bool),
        broadcast:  make(chan string, 256),
        register:   make(chan chan string),
        unregister: make(chan chan string),
    }
    go h.run()
    return h
}

func (h *Hub) run() {
    for {
        select {
        case client := <-h.register:
            h.mu.Lock()
            h.clients[client] = true
            h.mu.Unlock()

        case client := <-h.unregister:
            h.mu.Lock()
            if _, ok := h.clients[client]; ok {
                delete(h.clients, client)
                close(client)
            }
            h.mu.Unlock()

        case msg := <-h.broadcast:
            h.mu.RLock()
            for client := range h.clients {
                select {
                case client <- msg:
                default:
                    // Client buffer full, skip
                }
            }
            h.mu.RUnlock()
        }
    }
}

func (h *Hub) Subscribe() chan string {
    ch := make(chan string, 16)
    h.register <- ch
    return ch
}

func (h *Hub) Unsubscribe(ch chan string) {
    h.unregister <- ch
}

func (h *Hub) Broadcast(msg string) {
    h.broadcast <- msg
}
```

### Usage

```go
var hub = NewHub()

func handleBroadcastSSE(w http.ResponseWriter, r *http.Request) {
    sse, _ := NewSSEWriter(w)
    ctx := r.Context()

    ch := hub.Subscribe()
    defer hub.Unsubscribe(ch)

    for {
        select {
        case <-ctx.Done():
            return
        case msg := <-ch:
            sse.SendData(msg)
        }
    }
}

func handlePostMessage(w http.ResponseWriter, r *http.Request) {
    msg := r.FormValue("message")
    hub.Broadcast(msg)
    w.WriteHeader(http.StatusOK)
}
```

---

## Progress Streaming

### Long Operation with Progress

```go
func handleUploadProcess(w http.ResponseWriter, r *http.Request) {
    sse, _ := NewSSEWriter(w)
    ctx := r.Context()

    // Simulate processing stages
    stages := []string{"Uploading", "Processing", "Analyzing", "Complete"}

    for i, stage := range stages {
        select {
        case <-ctx.Done():
            return
        default:
            progress := (i + 1) * 100 / len(stages)
            var buf bytes.Buffer
            ProgressBar(stage, progress).Render(ctx, &buf)
            sse.SendHTML("progress", buf.String())
            time.Sleep(time.Second)
        }
    }

    sse.SendEvent("done", "true")
}
```

### Progress Component

```templ
templ ProgressBar(stage string, percent int) {
    <div class="space-y-2">
        <div class="flex justify-between text-sm">
            <span>{ stage }</span>
            <span>{ fmt.Sprintf("%d%%", percent) }</span>
        </div>
        <div class="w-full bg-gray-200 rounded-full h-2">
            <div
                class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                style={ fmt.Sprintf("width: %d%%", percent) }
            ></div>
        </div>
    </div>
}

templ ProgressContainer() {
    <div
        hx-ext="sse"
        sse-connect="/upload/process"
        sse-swap="progress"
    >
        @ProgressBar("Starting...", 0)
    </div>
}
```

---

## Auto-Scroll on New Messages

### Hyperscript Pattern (Recommended)

Use hyperscript's built-in scroll command for automatic scrolling:

```html
<!-- Scroll on every new message -->
<div
    id="stream"
    sse-swap="message"
    hx-swap="beforeend"
    _="on htmx:afterSwap scroll me to bottom"
>
    <!-- Messages appear here -->
</div>
```

For containers that need scroll on initial load AND new content:

```html
<!-- Scroll on load AND on new messages -->
<div
    id="stream"
    sse-swap="message"
    hx-swap="beforeend"
    _="on load scroll me to bottom then on htmx:afterSwap scroll me to bottom"
>
    <!-- Pre-populated history + new SSE messages -->
</div>
```

> **Tip**: Use `then` to chain multiple event handlers in hyperscript.

### Alternative: hx-swap scroll modifier

HTMX also supports scroll modifiers in `hx-swap`:

```html
<div
    sse-swap="message"
    hx-swap="beforeend scroll:bottom"
>
```

However, hyperscript provides more control and works reliably with SSE.

See **go-hyperscript-patterns** for more hyperscript patterns.

---

## SSE with Pre-populated History

A common pattern: load existing data from database, then stream new updates via SSE.

### The Pattern

1. Server renders existing history into the container
2. SSE connection appends new events via `beforeend`
3. Hyperscript scrolls to bottom on load and new content

### Templ Implementation

```templ
templ StreamWithHistory(streamID string, history []Message) {
    <div hx-ext="sse" sse-connect={ "/events?id=" + streamID }>
        <div
            id="stream"
            class="message-stream"
            sse-swap="message,tool_use"
            hx-swap="beforeend"
            _="on load scroll me to bottom then on htmx:afterSwap scroll me to bottom"
        >
            // Pre-render existing history from database
            for _, msg := range history {
                @MessageCard(msg)
            }
        </div>
        // Hidden handler for OOB-only events
        <div sse-swap="status,result" hx-swap="none" style="display:none;"></div>
    </div>
}

templ MessageCard(msg Message) {
    <div class="message">
        <span class="role">{ msg.Role }</span>
        <div class="content">{ msg.Content }</div>
    </div>
}
```

### Handler

```go
func handleRunDetail(w http.ResponseWriter, r *http.Request) {
    runID := r.PathValue("id")

    // Load run with history from database
    run, err := repo.GetRun(r.Context(), runID)
    if err != nil {
        http.Error(w, err.Error(), 500)
        return
    }

    // Check if run is still active (use DB status, not in-memory state)
    isLive := run.Status == "running" || run.Status == "paused"

    data := RunDetailData{
        Run:    run,
        IsLive: isLive,
    }

    templates.RunDetail(data).Render(r.Context(), w)
}
```

### Key Insights

1. **Use database status for "live" detection** - Don't rely on in-memory executor state; it may not reflect reality after server restart or when viewing from different process.

2. **SSE `beforeend` appends** - Pre-populated content stays, new events append after.

3. **Scroll on load AND afterSwap** - Use hyperscript `then` to chain both handlers:
   ```html
   _="on load scroll me to bottom then on htmx:afterSwap scroll me to bottom"
   ```

4. **OOB events update other elements** - Status updates, result banners, etc. can be updated via OOB swaps while messages stream.

### Live vs Static Views

```templ
templ RunDetail(data RunDetailData) {
    if data.IsLive {
        // Live: SSE connection with pre-populated history
        <div hx-ext="sse" sse-connect={ "/events?id=" + data.Run.ID }>
            <div
                id="stream"
                sse-swap="message"
                hx-swap="beforeend"
                _="on load scroll me to bottom then on htmx:afterSwap scroll me to bottom"
            >
                for _, msg := range data.Run.Messages {
                    @MessageCard(msg)
                }
            </div>
        </div>
    } else {
        // Completed: Static history only
        <div id="stream" class="message-stream">
            for _, msg := range data.Run.Messages {
                @MessageCard(msg)
            }
        </div>
    }
}
```

---

## Error Handling

### Retry Configuration

```go
func handleSSE(w http.ResponseWriter, r *http.Request) {
    sse, _ := NewSSEWriter(w)

    // Tell client to retry in 3 seconds if disconnected
    fmt.Fprintf(sse.w, "retry: 3000\n\n")
    sse.flusher.Flush()

    // ... stream events
}
```

### Error Event

```go
func handleSSE(w http.ResponseWriter, r *http.Request) {
    sse, _ := NewSSEWriter(w)

    err := doSomething()
    if err != nil {
        sse.SendEvent("error", fmt.Sprintf(`{"message": "%s"}`, err.Error()))
        return
    }
}
```

### Client Error Handling

```html
<div
    hx-ext="sse"
    sse-connect="/events"
    sse-swap="update"
    hx-on::sse-error="handleSSEError(event)"
>
```

```javascript
function handleSSEError(event) {
    console.error('SSE error:', event.detail);
    // Show user-friendly error message
}
```

---

## Graceful Shutdown

### Context Cancellation

```go
func handleSSE(w http.ResponseWriter, r *http.Request) {
    sse, _ := NewSSEWriter(w)
    ctx := r.Context()

    // Create cancellable context
    ctx, cancel := context.WithCancel(ctx)
    defer cancel()

    // Listen for server shutdown
    go func() {
        <-serverShutdown
        cancel()
    }()

    for {
        select {
        case <-ctx.Done():
            // Send close event before returning
            sse.SendEvent("close", "server shutting down")
            return
        case data := <-dataChan:
            sse.SendData(data)
        }
    }
}
```

---

## Best Practices

### 1. Use Named Events

```go
// Good - client can filter
sse.SendEvent("user-joined", userData)
sse.SendEvent("user-left", userData)
sse.SendEvent("message", messageData)

// Bad - client processes everything
sse.SendData(genericData)
```

### 2. Keep Events Small

```go
// Good - send only what changed
sse.SendEvent("count", fmt.Sprintf("%d", count))

// Bad - send entire state
sse.SendEvent("state", entireAppState)
```

### 3. Use Event IDs for Reconnection

```go
func (s *SSEWriter) SendEventWithID(id, event, data string) {
    fmt.Fprintf(s.w, "id: %s\n", id)
    fmt.Fprintf(s.w, "event: %s\n", event)
    fmt.Fprintf(s.w, "data: %s\n\n", data)
    s.flusher.Flush()
}
```

### 4. Heartbeat to Detect Disconnects

```go
func handleSSE(w http.ResponseWriter, r *http.Request) {
    sse, _ := NewSSEWriter(w)
    ctx := r.Context()

    heartbeat := time.NewTicker(30 * time.Second)
    defer heartbeat.Stop()

    for {
        select {
        case <-ctx.Done():
            return
        case <-heartbeat.C:
            sse.SendEvent("heartbeat", "ping")
        case data := <-dataChan:
            sse.SendData(data)
        }
    }
}
```

---

## Common Mistakes

### 1. Missing sse-swap

```html
<!-- WRONG -->
<div hx-ext="sse" sse-connect="/events">
    <div id="output"></div>  <!-- No sse-swap! -->
</div>

<!-- CORRECT -->
<div hx-ext="sse" sse-connect="/events">
    <div id="output" sse-swap="message" hx-swap="beforeend"></div>
</div>
```

### 2. sse-swap on the connection element

```html
<!-- WRONG - sse-swap should be on a child element -->
<div hx-ext="sse" sse-connect="/events" sse-swap="message">
</div>

<!-- CORRECT - sse-swap on child -->
<div hx-ext="sse" sse-connect="/events">
    <div sse-swap="message" hx-swap="beforeend"></div>
</div>
```

### 3. OOB events not captured

```html
<!-- WRONG - tool_result events ignored, OOB never processed -->
<div hx-ext="sse" sse-connect="/events">
    <div sse-swap="message" hx-swap="beforeend"></div>
</div>

<!-- CORRECT - Hidden handler captures OOB events -->
<div hx-ext="sse" sse-connect="/events">
    <div sse-swap="message" hx-swap="beforeend"></div>
    <div sse-swap="tool_result" hx-swap="none" style="display:none;"></div>
</div>
```

### 4. Returning JSON instead of HTML

```go
// WRONG - HTMX expects HTML
func handlePause(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "application/json")
    json.NewEncoder(w).Encode(map[string]string{"status": "paused"})
}

// CORRECT - Return HTML fragment
func handlePause(w http.ResponseWriter, r *http.Request) {
    w.Header().Set("Content-Type", "text/html")
    templates.PauseButton(true).Render(r.Context(), w)
}
```

---

## Integration

This skill works with:
- **go-project-bootstrap**: Initial project setup
- **go-htmx-core**: Handler patterns and routing
- **go-templ-components**: HTML fragment rendering
- **go-htmx-dashboard**: Real-time dashboard updates
- **go-hyperscript-patterns**: Client-side behavior (scroll, modals, toggles)
- **go-pico-embed**: Asset embedding and deployment

Reference this skill when implementing real-time features.
