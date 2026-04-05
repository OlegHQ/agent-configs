---
name: go-http-handlers
description: Clean HTTP handlers with dependency injection. stdlib only. Linear flow. Use for HTTP code.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go HTTP Handlers

## When to Use

- Writing HTTP handlers for REST APIs
- Building JSON API endpoints
- Implementing request/response lifecycle
- Adding middleware for cross-cutting concerns
- Handling route parameters and query strings

---

## Handler Struct with Dependencies

```go
type UserHandler struct {
    svc *UserService
    log *slog.Logger
}

func NewUserHandler(svc *UserService, log *slog.Logger) *UserHandler {
    return &UserHandler{svc: svc, log: log}
}
```

## Linear Handler Flow

Parse → Validate → Execute → Respond

```go
func (h *UserHandler) Create(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()

    // 1. Parse
    var req CreateUserRequest
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        h.respondError(w, http.StatusBadRequest, "invalid json")
        return
    }

    // 2. Validate
    if req.Email == "" {
        h.respondError(w, http.StatusBadRequest, "email required")
        return
    }

    // 3. Execute
    user, err := h.svc.Create(ctx, req.Email, req.Name)
    if err != nil {
        h.handleError(w, err)
        return
    }

    // 4. Respond
    h.respondJSON(w, http.StatusCreated, user)
}
```

## Response Helpers

```go
func (h *UserHandler) respondJSON(w http.ResponseWriter, status int, data any) {
    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(status)
    json.NewEncoder(w).Encode(data)
}

func (h *UserHandler) respondError(w http.ResponseWriter, status int, message string) {
    h.respondJSON(w, status, map[string]string{"error": message})
}

func (h *UserHandler) handleError(w http.ResponseWriter, err error) {
    // Domain errors → appropriate status
    if errors.Is(err, ErrUserNotFound) {
        h.respondError(w, http.StatusNotFound, "user not found")
        return
    }
    if errors.Is(err, ErrDuplicateEmail) {
        h.respondError(w, http.StatusConflict, "email already exists")
        return
    }

    var valErr *ValidationError
    if errors.As(err, &valErr) {
        h.respondError(w, http.StatusBadRequest, valErr.Error())
        return
    }

    // Unknown error → log + generic response
    h.log.Error("handler error", "error", err)
    h.respondError(w, http.StatusInternalServerError, "internal error")
}
```

## HTMX HTML Responses

When building HTMX endpoints, return HTML fragments instead of JSON.

```go
// HTML response helper for templ components
func (h *Handler) respondHTML(w http.ResponseWriter, status int, component templ.Component) {
    w.Header().Set("Content-Type", "text/html; charset=utf-8")
    w.WriteHeader(status)
    component.Render(context.Background(), w)
}

// Alternative: render to string first (useful for SSE)
func renderComponent(component templ.Component) string {
    var buf bytes.Buffer
    component.Render(context.Background(), &buf)
    return buf.String()
}

func (h *Handler) respondHTMLString(w http.ResponseWriter, status int, html string) {
    w.Header().Set("Content-Type", "text/html; charset=utf-8")
    w.WriteHeader(status)
    w.Write([]byte(html))
}
```

### HTMX Handler Pattern

```go
func (h *Handler) ToggleFavorite(w http.ResponseWriter, r *http.Request) {
    id := r.PathValue("id")

    item, err := h.svc.ToggleFavorite(r.Context(), id)
    if err != nil {
        // Return error HTML, not JSON - HTMX expects HTML!
        h.respondHTML(w, http.StatusOK, templates.ErrorBanner(err.Error()))
        return
    }

    // Return updated component HTML for HTMX to swap
    h.respondHTML(w, http.StatusOK, templates.FavoriteButton(item.IsFavorite))
}
```

> **Important**: HTMX expects HTML responses. If you return JSON to an HTMX-triggered endpoint, it will render as raw text in the browser.

## Route Registration (Go 1.22+)

```go
func main() {
    mux := http.NewServeMux()

    // Method + path pattern
    mux.HandleFunc("GET /users/{id}", userHandler.Get)
    mux.HandleFunc("POST /users", userHandler.Create)
    mux.HandleFunc("PUT /users/{id}", userHandler.Update)
    mux.HandleFunc("DELETE /users/{id}", userHandler.Delete)

    // Path parameters
    // r.PathValue("id") returns the {id} value
}

func (h *UserHandler) Get(w http.ResponseWriter, r *http.Request) {
    id := r.PathValue("id")  // Go 1.22+
    // ...
}
```

## Query Parameters

```go
func (h *UserHandler) List(w http.ResponseWriter, r *http.Request) {
    // Parse query params
    limit := 20
    if l := r.URL.Query().Get("limit"); l != "" {
        if parsed, err := strconv.Atoi(l); err == nil && parsed > 0 {
            limit = min(parsed, 100)  // cap at 100
        }
    }

    offset := 0
    if o := r.URL.Query().Get("offset"); o != "" {
        if parsed, err := strconv.Atoi(o); err == nil && parsed >= 0 {
            offset = parsed
        }
    }

    users, err := h.svc.List(ctx, limit, offset)
    // ...
}
```

## Middleware

Only for cross-cutting concerns:

```go
// Logging middleware
func LoggingMiddleware(log *slog.Logger) func(http.Handler) http.Handler {
    return func(next http.Handler) http.Handler {
        return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
            start := time.Now()
            next.ServeHTTP(w, r)
            log.Info("request",
                "method", r.Method,
                "path", r.URL.Path,
                "duration", time.Since(start),
            )
        })
    }
}

// Auth middleware
func AuthMiddleware(verifier TokenVerifier) func(http.Handler) http.Handler {
    return func(next http.Handler) http.Handler {
        return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
            token := r.Header.Get("Authorization")
            userID, err := verifier.Verify(token)
            if err != nil {
                http.Error(w, "unauthorized", http.StatusUnauthorized)
                return
            }
            ctx := WithUserID(r.Context(), userID)
            next.ServeHTTP(w, r.WithContext(ctx))
        })
    }
}

// Apply middleware
func main() {
    mux := http.NewServeMux()
    mux.HandleFunc("GET /users", userHandler.List)

    handler := LoggingMiddleware(log)(mux)
    handler = AuthMiddleware(verifier)(handler)

    http.ListenAndServe(":8080", handler)
}
```

## Request/Response Types

```go
// Request type - only fields you accept
type CreateUserRequest struct {
    Email string `json:"email"`
    Name  string `json:"name"`
}

// Response uses domain type directly
// User struct has json tags, use it as response
func (h *UserHandler) Create(w http.ResponseWriter, r *http.Request) {
    user, err := h.svc.Create(ctx, req.Email, req.Name)
    h.respondJSON(w, http.StatusCreated, user)  // User has json tags
}

// For lists, wrap in object
type ListResponse struct {
    Users  []*User `json:"users"`
    Total  int     `json:"total"`
    Limit  int     `json:"limit"`
    Offset int     `json:"offset"`
}
```

## Graceful Shutdown

```go
func main() {
    srv := &http.Server{
        Addr:    ":8080",
        Handler: mux,
    }

    // Start server
    go func() {
        if err := srv.ListenAndServe(); err != http.ErrServerClosed {
            log.Error("server error", "error", err)
        }
    }()

    // Wait for interrupt
    quit := make(chan os.Signal, 1)
    signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
    <-quit

    // Graceful shutdown with timeout
    ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
    defer cancel()

    if err := srv.Shutdown(ctx); err != nil {
        log.Error("shutdown error", "error", err)
    }
}
```

---

## Integration

This skill works with:
- **go-clean-architecture**: Handler layer in the architecture stack
- **go-context-patterns**: Context threading through handlers
- **go-error-tracing**: Error handling and response patterns
- **go-testing-simple**: Testing HTTP handlers with httptest

Reference this skill when writing any HTTP endpoint.
