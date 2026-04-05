---
name: go-claude-agents
description: Build agentic AI applications in Go using the clawde library (Claude Agents SDK / Claude Code SDK). Streaming, tool use, agent patterns, and SSE integration.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go Claude Agents

## When to Use

- Building AI-powered features with Claude
- Implementing streaming chat interfaces
- Creating agents with tool use
- Integrating AI into Go web applications
- Real-time AI response streaming via SSE

---

Build agentic AI applications in Go using the clawde library. This skill covers streaming, tool use, agent patterns, and SSE integration for real-time web interfaces.

## Setup & Installation

### Install clawde Library
```bash
go get github.com/anthropics/clawde
```

### API Key Configuration
```bash
export ANTHROPIC_API_KEY="your-api-key"
```

Or load from `.env` file:
```go
import "github.com/joho/godotenv"

func init() {
    godotenv.Load()
}
```

### Project Structure
```
myagent/
├── cmd/
│   └── server/
│       └── main.go
├── internal/
│   ├── agent/
│   │   ├── agent.go      # Agent logic
│   │   └── tools.go      # Tool definitions
│   └── handlers/
│       └── chat.go       # HTTP handlers
├── views/
│   └── components/
│       └── chat.templ    # UI components
├── go.mod
└── .env
```

---

## Core Concepts

### Client Initialization
```go
import "github.com/anthropics/clawde"

// Basic client
client := clawde.NewClient()

// Client with options
client := clawde.NewClient(
    clawde.WithModel("claude-sonnet-4-20250514"),
    clawde.WithSystemPrompt("You are a helpful coding assistant."),
    clawde.WithMaxTurns(50),
)
```

### Available Models
```go
// Fast and capable (recommended for most tasks)
clawde.WithModel("claude-sonnet-4-20250514")

// Most capable (complex reasoning)
clawde.WithModel("claude-opus-4-20250514")

// Fast and efficient (simple tasks)
clawde.WithModel("claude-haiku-3-5-20241022")
```

### Message Types
The streaming API yields different message types:

| Type | Fields | Purpose |
|------|--------|---------|
| `*clawde.AssistantMessage` | `.Content`, `.Model` | Full assistant response |
| `*clawde.TextBlock` | `.Text` | Streaming text chunk |
| `*clawde.ThinkingBlock` | `.Text` | Extended thinking output |
| `*clawde.ToolUseMessage` | `.Name`, `.Input` | Tool invocation request |
| `*clawde.ToolResultMessage` | `.Name`, `.Result` | Tool execution result |
| `*clawde.ResultMessage` | `.StopReason`, `.Usage` | Completion metadata |

### Basic Streaming Query
```go
func chat(prompt string) error {
    client := clawde.NewClient(
        clawde.WithModel("claude-sonnet-4-20250514"),
    )

    ctx := context.Background()
    stream, err := client.Query(ctx, prompt)
    if err != nil {
        return err
    }

    for stream.Next() {
        switch msg := stream.Current().(type) {
        case *clawde.TextBlock:
            fmt.Print(msg.Text)
        case *clawde.ThinkingBlock:
            fmt.Printf("[thinking: %s]\n", msg.Text)
        }
    }

    return stream.Err()
}
```

---

## Tool Use / Function Calling

### Defining Tools
```go
tools := []clawde.Tool{
    {
        Name:        "read_file",
        Description: "Read contents of a file",
        InputSchema: map[string]any{
            "type": "object",
            "properties": map[string]any{
                "path": map[string]any{
                    "type":        "string",
                    "description": "File path to read",
                },
            },
            "required": []string{"path"},
        },
    },
    {
        Name:        "write_file",
        Description: "Write content to a file",
        InputSchema: map[string]any{
            "type": "object",
            "properties": map[string]any{
                "path": map[string]any{
                    "type":        "string",
                    "description": "File path to write",
                },
                "content": map[string]any{
                    "type":        "string",
                    "description": "Content to write",
                },
            },
            "required": []string{"path", "content"},
        },
    },
    {
        Name:        "list_directory",
        Description: "List files in a directory",
        InputSchema: map[string]any{
            "type": "object",
            "properties": map[string]any{
                "path": map[string]any{
                    "type":        "string",
                    "description": "Directory path",
                },
            },
            "required": []string{"path"},
        },
    },
}
```

### Tool Handler Function
```go
func handleTool(name string, input map[string]any) (string, error) {
    switch name {
    case "read_file":
        path := input["path"].(string)
        data, err := os.ReadFile(path)
        if err != nil {
            return "", err
        }
        return string(data), nil

    case "write_file":
        path := input["path"].(string)
        content := input["content"].(string)
        err := os.WriteFile(path, []byte(content), 0644)
        if err != nil {
            return "", err
        }
        return "File written successfully", nil

    case "list_directory":
        path := input["path"].(string)
        entries, err := os.ReadDir(path)
        if err != nil {
            return "", err
        }
        var names []string
        for _, e := range entries {
            names = append(names, e.Name())
        }
        return strings.Join(names, "\n"), nil
    }

    return "", fmt.Errorf("unknown tool: %s", name)
}
```

### Client with Tools
```go
client := clawde.NewClient(
    clawde.WithModel("claude-sonnet-4-20250514"),
    clawde.WithSystemPrompt("You are a file management assistant."),
    clawde.WithTools(tools),
    clawde.WithToolHandler(handleTool),
    clawde.WithMaxTurns(10),
)
```

---

## Streaming Patterns

### Console Streaming
```go
func streamToConsole(prompt string) error {
    client := clawde.NewClient(
        clawde.WithModel("claude-sonnet-4-20250514"),
    )

    stream, err := client.Query(context.Background(), prompt)
    if err != nil {
        return err
    }

    for stream.Next() {
        switch msg := stream.Current().(type) {
        case *clawde.TextBlock:
            fmt.Print(msg.Text)
        case *clawde.ToolUseMessage:
            fmt.Printf("\n[Using tool: %s]\n", msg.Name)
        case *clawde.ToolResultMessage:
            fmt.Printf("[Tool result: %s]\n", msg.Name)
        case *clawde.ResultMessage:
            fmt.Printf("\n[Done: %s]\n", msg.StopReason)
        }
    }

    return stream.Err()
}
```

### Collecting Full Response
```go
func getFullResponse(prompt string) (string, error) {
    client := clawde.NewClient(
        clawde.WithModel("claude-sonnet-4-20250514"),
    )

    stream, err := client.Query(context.Background(), prompt)
    if err != nil {
        return "", err
    }

    var response strings.Builder
    for stream.Next() {
        if text, ok := stream.Current().(*clawde.TextBlock); ok {
            response.WriteString(text.Text)
        }
    }

    if err := stream.Err(); err != nil {
        return "", err
    }

    return response.String(), nil
}
```

---

## Agent Patterns

### Simple Chatbot
```go
type Chatbot struct {
    client *clawde.Client
}

func NewChatbot(systemPrompt string) *Chatbot {
    return &Chatbot{
        client: clawde.NewClient(
            clawde.WithModel("claude-sonnet-4-20250514"),
            clawde.WithSystemPrompt(systemPrompt),
        ),
    }
}

func (c *Chatbot) Chat(ctx context.Context, prompt string) (string, error) {
    stream, err := c.client.Query(ctx, prompt)
    if err != nil {
        return "", err
    }

    var response strings.Builder
    for stream.Next() {
        if text, ok := stream.Current().(*clawde.TextBlock); ok {
            response.WriteString(text.Text)
        }
    }

    return response.String(), stream.Err()
}
```

### Conversational Agent (with History)
```go
type ConversationAgent struct {
    client   *clawde.Client
    messages []clawde.Message
}

func NewConversationAgent(systemPrompt string) *ConversationAgent {
    return &ConversationAgent{
        client: clawde.NewClient(
            clawde.WithModel("claude-sonnet-4-20250514"),
            clawde.WithSystemPrompt(systemPrompt),
        ),
        messages: []clawde.Message{},
    }
}

func (a *ConversationAgent) Chat(ctx context.Context, userInput string) (string, error) {
    // Add user message to history
    a.messages = append(a.messages, clawde.Message{
        Role:    "user",
        Content: userInput,
    })

    // Query with full history
    stream, err := a.client.QueryWithMessages(ctx, a.messages)
    if err != nil {
        return "", err
    }

    var response strings.Builder
    for stream.Next() {
        if text, ok := stream.Current().(*clawde.TextBlock); ok {
            response.WriteString(text.Text)
        }
    }

    if err := stream.Err(); err != nil {
        return "", err
    }

    // Add assistant response to history
    a.messages = append(a.messages, clawde.Message{
        Role:    "assistant",
        Content: response.String(),
    })

    return response.String(), nil
}

func (a *ConversationAgent) Reset() {
    a.messages = []clawde.Message{}
}
```

### ReAct Agent (Reason + Act)
```go
type ReActAgent struct {
    client        *clawde.Client
    tools         []clawde.Tool
    toolHandler   func(string, map[string]any) (string, error)
    maxIterations int
}

func NewReActAgent(systemPrompt string, tools []clawde.Tool, handler func(string, map[string]any) (string, error)) *ReActAgent {
    return &ReActAgent{
        client: clawde.NewClient(
            clawde.WithModel("claude-sonnet-4-20250514"),
            clawde.WithSystemPrompt(systemPrompt),
            clawde.WithTools(tools),
        ),
        tools:         tools,
        toolHandler:   handler,
        maxIterations: 10,
    }
}

func (a *ReActAgent) Run(ctx context.Context, task string) (string, error) {
    messages := []clawde.Message{
        {Role: "user", Content: task},
    }

    for i := 0; i < a.maxIterations; i++ {
        stream, err := a.client.QueryWithMessages(ctx, messages)
        if err != nil {
            return "", err
        }

        var response strings.Builder
        var toolCalls []struct {
            Name  string
            Input map[string]any
        }

        for stream.Next() {
            switch msg := stream.Current().(type) {
            case *clawde.TextBlock:
                response.WriteString(msg.Text)
            case *clawde.ToolUseMessage:
                toolCalls = append(toolCalls, struct {
                    Name  string
                    Input map[string]any
                }{
                    Name:  msg.Name,
                    Input: msg.Input,
                })
            case *clawde.ResultMessage:
                // If no tool calls and end of turn, we're done
                if msg.StopReason == "end_turn" && len(toolCalls) == 0 {
                    return response.String(), nil
                }
            }
        }

        if err := stream.Err(); err != nil {
            return "", err
        }

        // Add assistant response
        messages = append(messages, clawde.Message{
            Role:    "assistant",
            Content: response.String(),
        })

        // Execute tools and add results
        for _, tc := range toolCalls {
            result, err := a.toolHandler(tc.Name, tc.Input)
            if err != nil {
                result = fmt.Sprintf("Error: %v", err)
            }
            messages = append(messages, clawde.Message{
                Role:    "user",
                Content: fmt.Sprintf("[Tool Result: %s]\n%s", tc.Name, result),
            })
        }
    }

    return "", fmt.Errorf("max iterations (%d) reached", a.maxIterations)
}
```

---

## SSE Integration with HTMX

### HTTP Handler for Streaming
```go
type ChatHandler struct {
    client *clawde.Client
}

func NewChatHandler() *ChatHandler {
    return &ChatHandler{
        client: clawde.NewClient(
            clawde.WithModel("claude-sonnet-4-20250514"),
            clawde.WithSystemPrompt("You are a helpful assistant."),
        ),
    }
}

func (h *ChatHandler) StreamChat(w http.ResponseWriter, r *http.Request) {
    // Set SSE headers
    w.Header().Set("Content-Type", "text/event-stream")
    w.Header().Set("Cache-Control", "no-cache")
    w.Header().Set("Connection", "keep-alive")

    flusher, ok := w.(http.Flusher)
    if !ok {
        http.Error(w, "SSE not supported", http.StatusInternalServerError)
        return
    }

    prompt := r.FormValue("prompt")
    if prompt == "" {
        fmt.Fprintf(w, "event: error\ndata: prompt required\n\n")
        flusher.Flush()
        return
    }

    stream, err := h.client.Query(r.Context(), prompt)
    if err != nil {
        fmt.Fprintf(w, "event: error\ndata: %s\n\n", err.Error())
        flusher.Flush()
        return
    }

    for stream.Next() {
        switch msg := stream.Current().(type) {
        case *clawde.TextBlock:
            // Escape for SSE (replace newlines)
            escaped := strings.ReplaceAll(msg.Text, "\n", "\\n")
            fmt.Fprintf(w, "event: text\ndata: %s\n\n", escaped)
            flusher.Flush()

        case *clawde.ThinkingBlock:
            escaped := strings.ReplaceAll(msg.Text, "\n", "\\n")
            fmt.Fprintf(w, "event: thinking\ndata: %s\n\n", escaped)
            flusher.Flush()

        case *clawde.ToolUseMessage:
            data := fmt.Sprintf("%s|%v", msg.Name, msg.Input)
            fmt.Fprintf(w, "event: tool_use\ndata: %s\n\n", data)
            flusher.Flush()

        case *clawde.ToolResultMessage:
            fmt.Fprintf(w, "event: tool_result\ndata: %s\n\n", msg.Name)
            flusher.Flush()
        }
    }

    if err := stream.Err(); err != nil {
        fmt.Fprintf(w, "event: error\ndata: %s\n\n", err.Error())
    } else {
        fmt.Fprintf(w, "event: done\ndata: complete\n\n")
    }
    flusher.Flush()
}
```

### HTMX Chat Component (Templ)
```templ
// views/components/chat.templ

templ ChatContainer() {
    <div id="chat-container" class="container">
        <article>
            <header>AI Chat</header>
            <div id="messages"></div>
            <form
                hx-ext="sse"
                sse-connect="/api/chat/stream"
                sse-swap="text"
                hx-target="#messages"
                hx-swap="beforeend"
            >
                <input
                    type="text"
                    name="prompt"
                    placeholder="Ask something..."
                    required
                />
                <button type="submit">Send</button>
            </form>
        </article>
    </div>
}

templ ChatMessage(content string) {
    <div class="message assistant">
        { content }
    </div>
}

templ ThinkingIndicator() {
    <div class="thinking">
        <progress></progress>
        <small>Thinking...</small>
    </div>
}

templ ToolUseIndicator(toolName string) {
    <div class="tool-use">
        <kbd>{ toolName }</kbd>
        <small>Running tool...</small>
    </div>
}
```

### Route Setup
```go
func main() {
    mux := http.NewServeMux()

    chatHandler := NewChatHandler()
    mux.HandleFunc("POST /api/chat/stream", chatHandler.StreamChat)

    // Serve static files and templates...

    log.Println("Server starting on :8080")
    http.ListenAndServe(":8080", mux)
}
```

---

## Advanced Patterns

### Extended Thinking
```go
// Enable extended thinking for complex reasoning
client := clawde.NewClient(
    clawde.WithModel("claude-sonnet-4-20250514"),
    clawde.WithExtendedThinking(true),
)

stream, _ := client.Query(ctx, "Solve this complex problem...")

for stream.Next() {
    switch msg := stream.Current().(type) {
    case *clawde.ThinkingBlock:
        // Display thinking process to user
        fmt.Printf("[Reasoning: %s]\n", msg.Text)
    case *clawde.TextBlock:
        fmt.Print(msg.Text)
    }
}
```

### Context Window Management
```go
type ManagedAgent struct {
    client     *clawde.Client
    messages   []clawde.Message
    maxTokens  int
    tokenCount int
}

func (a *ManagedAgent) addMessage(msg clawde.Message) {
    // Rough token estimate (4 chars per token)
    tokens := len(msg.Content) / 4
    a.tokenCount += tokens

    // Trim old messages if approaching limit
    for a.tokenCount > a.maxTokens*80/100 && len(a.messages) > 2 {
        removed := a.messages[0]
        a.messages = a.messages[1:]
        a.tokenCount -= len(removed.Content) / 4
    }

    a.messages = append(a.messages, msg)
}
```

### Rate Limiting & Retries
```go
func queryWithRetry(client *clawde.Client, ctx context.Context, prompt string) (*clawde.Stream, error) {
    var lastErr error
    backoff := time.Second

    for attempt := 0; attempt < 3; attempt++ {
        stream, err := client.Query(ctx, prompt)
        if err == nil {
            return stream, nil
        }

        lastErr = err

        // Check if retryable
        if strings.Contains(err.Error(), "rate_limit") {
            time.Sleep(backoff)
            backoff *= 2
            continue
        }

        return nil, err
    }

    return nil, fmt.Errorf("max retries exceeded: %w", lastErr)
}
```

### Error Recovery
```go
func (a *Agent) RunWithRecovery(ctx context.Context, task string) (string, error) {
    defer func() {
        if r := recover(); r != nil {
            log.Printf("Agent recovered from panic: %v", r)
        }
    }()

    result, err := a.Run(ctx, task)
    if err != nil {
        // Log error for debugging
        log.Printf("Agent error: %v", err)

        // Try simpler fallback
        return a.SimpleFallback(ctx, task)
    }

    return result, nil
}
```

---

## Complete Examples

### File Assistant Agent
```go
package main

import (
    "context"
    "fmt"
    "log"
    "os"
    "strings"

    "github.com/anthropics/clawde"
)

var fileTools = []clawde.Tool{
    {
        Name:        "read_file",
        Description: "Read a file's contents",
        InputSchema: map[string]any{
            "type": "object",
            "properties": map[string]any{
                "path": map[string]any{"type": "string"},
            },
            "required": []string{"path"},
        },
    },
    {
        Name:        "write_file",
        Description: "Write content to a file",
        InputSchema: map[string]any{
            "type": "object",
            "properties": map[string]any{
                "path":    map[string]any{"type": "string"},
                "content": map[string]any{"type": "string"},
            },
            "required": []string{"path", "content"},
        },
    },
    {
        Name:        "list_files",
        Description: "List files in directory",
        InputSchema: map[string]any{
            "type": "object",
            "properties": map[string]any{
                "path": map[string]any{"type": "string"},
            },
            "required": []string{"path"},
        },
    },
}

func fileToolHandler(name string, input map[string]any) (string, error) {
    switch name {
    case "read_file":
        data, err := os.ReadFile(input["path"].(string))
        if err != nil {
            return "", err
        }
        return string(data), nil
    case "write_file":
        return "written", os.WriteFile(
            input["path"].(string),
            []byte(input["content"].(string)),
            0644,
        )
    case "list_files":
        entries, err := os.ReadDir(input["path"].(string))
        if err != nil {
            return "", err
        }
        var names []string
        for _, e := range entries {
            names = append(names, e.Name())
        }
        return strings.Join(names, "\n"), nil
    }
    return "", fmt.Errorf("unknown tool")
}

func main() {
    client := clawde.NewClient(
        clawde.WithModel("claude-sonnet-4-20250514"),
        clawde.WithSystemPrompt("You are a file management assistant. Help users read, write, and organize files."),
        clawde.WithTools(fileTools),
        clawde.WithToolHandler(fileToolHandler),
        clawde.WithMaxTurns(20),
    )

    ctx := context.Background()
    stream, err := client.Query(ctx, "List the files in the current directory and summarize what you see")
    if err != nil {
        log.Fatal(err)
    }

    for stream.Next() {
        switch msg := stream.Current().(type) {
        case *clawde.TextBlock:
            fmt.Print(msg.Text)
        case *clawde.ToolUseMessage:
            fmt.Printf("\n[Using: %s]\n", msg.Name)
        }
    }

    if err := stream.Err(); err != nil {
        log.Fatal(err)
    }
}
```

### Web Chat Server
```go
package main

import (
    "fmt"
    "log"
    "net/http"
    "strings"

    "github.com/anthropics/clawde"
)

func main() {
    client := clawde.NewClient(
        clawde.WithModel("claude-sonnet-4-20250514"),
        clawde.WithSystemPrompt("You are a helpful assistant."),
    )

    mux := http.NewServeMux()

    // SSE chat endpoint
    mux.HandleFunc("POST /chat", func(w http.ResponseWriter, r *http.Request) {
        w.Header().Set("Content-Type", "text/event-stream")
        w.Header().Set("Cache-Control", "no-cache")

        flusher, _ := w.(http.Flusher)

        prompt := r.FormValue("prompt")
        stream, err := client.Query(r.Context(), prompt)
        if err != nil {
            fmt.Fprintf(w, "event: error\ndata: %s\n\n", err)
            flusher.Flush()
            return
        }

        for stream.Next() {
            if text, ok := stream.Current().(*clawde.TextBlock); ok {
                escaped := strings.ReplaceAll(text.Text, "\n", "\\n")
                fmt.Fprintf(w, "event: text\ndata: %s\n\n", escaped)
                flusher.Flush()
            }
        }

        fmt.Fprintf(w, "event: done\ndata: \n\n")
        flusher.Flush()
    })

    // Serve index page
    mux.HandleFunc("GET /", func(w http.ResponseWriter, r *http.Request) {
        w.Header().Set("Content-Type", "text/html")
        fmt.Fprint(w, `<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css">
    <script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/htmx-ext-sse@2.2.2/sse.js"></script>
</head>
<body>
    <main class="container">
        <h1>AI Chat</h1>
        <article>
            <div id="output"></div>
            <form hx-post="/chat" hx-ext="sse" sse-connect="/chat" hx-target="#output" hx-swap="beforeend">
                <input type="text" name="prompt" placeholder="Ask something..." required>
                <button type="submit">Send</button>
            </form>
        </article>
    </main>
</body>
</html>`)
    })

    log.Println("Server at http://localhost:8080")
    log.Fatal(http.ListenAndServe(":8080", mux))
}
```

---

## Integration with Other Skills

- **go-htmx-sse**: Use SSE patterns for real-time streaming UI
- **go-templ-components**: Build chat UI components with Templ
- **go-htmx-dashboard**: Add AI widgets to dashboards
- **go-project-bootstrap**: Scaffold a complete project with agent support

---

## Quick Reference

### Client Options
```go
clawde.WithModel(model)           // Set model
clawde.WithSystemPrompt(prompt)   // Set system prompt
clawde.WithTools(tools)           // Add tools
clawde.WithToolHandler(fn)        // Set tool handler
clawde.WithMaxTurns(n)            // Limit tool iterations
clawde.WithExtendedThinking(bool) // Enable thinking
```

### Stream Iteration
```go
stream, err := client.Query(ctx, prompt)
for stream.Next() {
    msg := stream.Current()
    // Handle message types...
}
err = stream.Err()
```

### SSE Event Format
```
event: text
data: Hello world

event: thinking
data: Reasoning...

event: tool_use
data: tool_name|{"arg": "value"}

event: done
data: complete
```

---

## Integration

This skill works with:
- **go-htmx-sse**: SSE streaming for real-time AI responses
- **go-templ-components**: Chat UI components
- **go-htmx-dashboard**: AI widgets in dashboards
- **go-project-bootstrap**: Project scaffolding with agent support
- **go-concurrency-safe**: Managing agent goroutine lifecycles

Reference this skill when building AI features in Go.
