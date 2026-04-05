---
name: go-config-logging
description: Environment config and structured logging. JSON_LOGS for JSON output. No globals. Use for app setup.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go Config and Logging

## When to Use

- Loading configuration from environment
- Setting up structured logging with slog
- Configuring log output format (JSON/text)
- Wiring dependencies in main.go
- Adding request logging middleware

---

## Config from Environment

```go
package config

import "os"

type Config struct {
    // Server
    Port string
    Host string

    // Database
    MongoURI  string
    MongoName string

    // Logging
    LogJSON bool
    LogLevel string

    // Features
    EnableMetrics bool
}

func Load() *Config {
    return &Config{
        Port:          getEnv("PORT", "8080"),
        Host:          getEnv("HOST", "0.0.0.0"),
        MongoURI:      getEnv("MONGO_URI", "mongodb://localhost:27017"),
        MongoName:     getEnv("MONGO_NAME", "myapp"),
        LogJSON:       getEnv("JSON_LOGS", "") != "",
        LogLevel:      getEnv("LOG_LEVEL", "info"),
        EnableMetrics: getEnv("ENABLE_METRICS", "") != "",
    }
}

func getEnv(key, defaultVal string) string {
    if v := os.Getenv(key); v != "" {
        return v
    }
    return defaultVal
}
```

## Required Config

For required values, fail fast:

```go
func Load() (*Config, error) {
    cfg := &Config{
        Port:     getEnv("PORT", "8080"),
        MongoURI: os.Getenv("MONGO_URI"),  // required
        LogJSON:  getEnv("JSON_LOGS", "") != "",
    }

    if cfg.MongoURI == "" {
        return nil, fmt.Errorf("MONGO_URI environment variable required")
    }

    return cfg, nil
}
```

## Structured Logging with slog

```go
package logging

import (
    "log/slog"
    "os"
)

func New(jsonOutput bool, level string) *slog.Logger {
    var lvl slog.Level
    switch level {
    case "debug":
        lvl = slog.LevelDebug
    case "warn":
        lvl = slog.LevelWarn
    case "error":
        lvl = slog.LevelError
    default:
        lvl = slog.LevelInfo
    }

    opts := &slog.HandlerOptions{Level: lvl}

    var handler slog.Handler
    if jsonOutput {
        handler = slog.NewJSONHandler(os.Stdout, opts)
    } else {
        handler = slog.NewTextHandler(os.Stdout, opts)
    }

    return slog.New(handler)
}
```

## Logging Output

```go
// With JSON_LOGS=1
{"time":"2024-01-15T10:30:00Z","level":"INFO","msg":"user created","user_id":"abc123","email":"test@example.com"}

// Without JSON_LOGS (default)
time=2024-01-15T10:30:00Z level=INFO msg="user created" user_id=abc123 email=test@example.com
```

## Logging Patterns

```go
// Add context to logger
log := log.With("request_id", requestID)

// Log with structured attributes
log.Info("user created",
    "user_id", user.ID,
    "email", user.Email,
)

// Log errors with full error chain
log.Error("create user failed",
    "error", err,
    "email", req.Email,
)

// Debug for development
log.Debug("processing request",
    "method", r.Method,
    "path", r.URL.Path,
)
```

## main.go Wiring

```go
func main() {
    // 1. Load config
    cfg, err := config.Load()
    if err != nil {
        fmt.Fprintf(os.Stderr, "config error: %v\n", err)
        os.Exit(1)
    }

    // 2. Setup logging
    log := logging.New(cfg.LogJSON, cfg.LogLevel)

    // 3. Connect database
    ctx := context.Background()
    db, err := db.Connect(ctx, cfg.MongoURI, cfg.MongoName)
    if err != nil {
        log.Error("database connection failed", "error", err)
        os.Exit(1)
    }

    // 4. Wire dependencies
    userRepo := users.NewUserRepo(db)
    if err := userRepo.EnsureIndexes(ctx); err != nil {
        log.Error("index creation failed", "error", err)
        os.Exit(1)
    }

    userSvc := users.NewUserService(userRepo)
    userHandler := users.NewUserHandler(userSvc, log)

    // 5. Setup routes
    mux := http.NewServeMux()
    mux.HandleFunc("GET /users/{id}", userHandler.Get)
    mux.HandleFunc("POST /users", userHandler.Create)

    // 6. Start server
    addr := cfg.Host + ":" + cfg.Port
    log.Info("server starting", "addr", addr)

    if err := http.ListenAndServe(addr, mux); err != nil {
        log.Error("server error", "error", err)
        os.Exit(1)
    }
}
```

## Request Logging Middleware

```go
func RequestLogger(log *slog.Logger) func(http.Handler) http.Handler {
    return func(next http.Handler) http.Handler {
        return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
            start := time.Now()

            // Wrap response writer to capture status
            wrapped := &responseWriter{ResponseWriter: w, status: 200}

            next.ServeHTTP(wrapped, r)

            log.Info("request",
                "method", r.Method,
                "path", r.URL.Path,
                "status", wrapped.status,
                "duration_ms", time.Since(start).Milliseconds(),
            )
        })
    }
}

type responseWriter struct {
    http.ResponseWriter
    status int
}

func (w *responseWriter) WriteHeader(status int) {
    w.status = status
    w.ResponseWriter.WriteHeader(status)
}
```

## Child Logger in Handler

```go
type UserHandler struct {
    svc *UserService
    log *slog.Logger
}

func (h *UserHandler) Create(w http.ResponseWriter, r *http.Request) {
    // Create child logger with request context
    log := h.log.With(
        "handler", "UserHandler.Create",
        "request_id", r.Header.Get("X-Request-ID"),
    )

    log.Debug("parsing request")

    // ... handler logic ...

    if err != nil {
        log.Error("create failed", "error", err)
        return
    }

    log.Info("user created", "user_id", user.ID)
}
```

## Environment Files (Local Dev)

Create `.env` for local development:

```bash
# .env (git-ignored)
PORT=8080
MONGO_URI=mongodb://localhost:27017
MONGO_NAME=myapp_dev
LOG_LEVEL=debug
```

Load with a tool like `direnv` or source before running:

```bash
source .env && go run .
```

---

## Integration

This skill works with:
- **go-project-bootstrap**: Initial configuration setup
- **go-clean-architecture**: main.go wiring patterns
- **go-http-handlers**: Request logging middleware
- **go-error-tracing**: Error logging patterns

Reference this skill when setting up application startup.
