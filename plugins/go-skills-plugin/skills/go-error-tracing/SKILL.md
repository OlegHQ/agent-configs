---
name: go-error-tracing
description: Error wrapping for fast debugging. Every error shows its path. Custom types for domain errors. Use for error handling.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go Error Tracing

## When to Use

- Wrapping errors with context
- Defining domain-specific errors
- Translating database errors
- Handling validation errors
- Logging errors with full trace

---

## Wrap Every Error

```go
// BAD - loses context
return err

// GOOD - shows path: "create user: insert document: connection refused"
return fmt.Errorf("create user: %w", err)
```

## Stack of Wraps

Each layer adds context so you can trace the error path instantly:

```go
// repo.go
func (r *UserRepo) Insert(ctx context.Context, u *User) error {
    _, err := r.coll.InsertOne(ctx, u)
    if err != nil {
        return fmt.Errorf("insert user %s: %w", u.ID, err)
    }
    return nil
}

// service.go
func (s *UserService) Create(ctx context.Context, u *User) error {
    if err := s.repo.Insert(ctx, u); err != nil {
        return fmt.Errorf("create user: %w", err)
    }
    return nil
}

// handler.go
func (h *UserHandler) Create(w http.ResponseWriter, r *http.Request) {
    user, err := h.svc.Create(ctx, u)
    if err != nil {
        // Error: "create user: insert user abc123: connection refused"
        // ^ Instantly know: handler → service → repo → mongo failed
        h.log.Error("create user failed", "error", err)
        h.respondError(w, 500, "internal error")
        return
    }
}
```

## Domain Errors

Define sentinel errors for domain-specific cases:

```go
// users/errors.go
var (
    ErrUserNotFound   = errors.New("user not found")
    ErrDuplicateEmail = errors.New("email already exists")
    ErrInvalidEmail   = errors.New("invalid email format")
)

// Check with errors.Is()
if errors.Is(err, ErrUserNotFound) {
    h.respondError(w, http.StatusNotFound, "user not found")
    return
}
```

## Translate Database Errors

Convert database errors to domain errors in the repository:

```go
func (r *UserRepo) FindByID(ctx context.Context, id primitive.ObjectID) (*User, error) {
    var user User
    err := r.coll.FindOne(ctx, bson.M{"_id": id}).Decode(&user)
    if errors.Is(err, mongo.ErrNoDocuments) {
        return nil, ErrUserNotFound  // domain error, not mongo error
    }
    if err != nil {
        return nil, fmt.Errorf("find user %s: %w", id, err)
    }
    return &user, nil
}

func (r *UserRepo) Insert(ctx context.Context, u *User) error {
    _, err := r.coll.InsertOne(ctx, u)
    if mongo.IsDuplicateKeyError(err) {
        return ErrDuplicateEmail  // domain error
    }
    if err != nil {
        return fmt.Errorf("insert user: %w", err)
    }
    return nil
}
```

## Validation Errors

For field-level validation, use a custom error type:

```go
type ValidationError struct {
    Field   string
    Message string
}

func (e *ValidationError) Error() string {
    return fmt.Sprintf("%s: %s", e.Field, e.Message)
}

// Usage
if req.Email == "" {
    return &ValidationError{Field: "email", Message: "required"}
}

// Check with errors.As()
var valErr *ValidationError
if errors.As(err, &valErr) {
    h.respondError(w, http.StatusBadRequest, valErr.Error())
    return
}
```

## Error Handling in Handlers

```go
func (h *UserHandler) Create(w http.ResponseWriter, r *http.Request) {
    ctx := r.Context()

    user, err := h.svc.Create(ctx, req.Email, req.Name)
    if err != nil {
        // Check domain errors first
        if errors.Is(err, ErrDuplicateEmail) {
            h.respondError(w, http.StatusConflict, "email already exists")
            return
        }

        var valErr *ValidationError
        if errors.As(err, &valErr) {
            h.respondError(w, http.StatusBadRequest, valErr.Error())
            return
        }

        // Unknown error - log full error, return generic message
        h.log.Error("create user failed", "error", err)
        h.respondError(w, http.StatusInternalServerError, "internal error")
        return
    }

    h.respondJSON(w, http.StatusCreated, user)
}
```

## Never Panic

```go
// BAD
func GetUser(id string) *User {
    user, err := repo.FindByID(id)
    if err != nil {
        panic(err)  // crashes the server!
    }
    return user
}

// GOOD
func GetUser(ctx context.Context, id string) (*User, error) {
    user, err := repo.FindByID(ctx, id)
    if err != nil {
        return nil, fmt.Errorf("get user: %w", err)
    }
    return user, nil
}
```

---

## Integration

This skill works with:
- **go-http-handlers**: Error response patterns
- **go-mongodb**: Database error translation
- **go-config-logging**: Error logging with slog
- **go-testing-simple**: Testing error paths

Reference this skill for all error handling code.
