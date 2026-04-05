---
name: go-testing-simple
description: Simple Go testing. Table-driven, -race flag, no assertion libs. Use for writing tests.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go Testing

## When to Use

- Writing unit tests for Go functions
- Testing HTTP handlers with httptest
- Creating table-driven test cases
- Setting up test fixtures and helpers
- Running tests with race detector

---

## Always Run with Race Detector

```bash
go test -race ./...
```

## Table-Driven Tests

```go
func TestValidateEmail(t *testing.T) {
    tests := []struct {
        name    string
        email   string
        wantErr bool
    }{
        {"valid", "test@example.com", false},
        {"valid with plus", "test+tag@example.com", false},
        {"empty", "", true},
        {"no at", "testexample.com", true},
        {"no domain", "test@", true},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            err := ValidateEmail(tt.email)
            if (err != nil) != tt.wantErr {
                t.Errorf("ValidateEmail(%q) error = %v, wantErr %v", tt.email, err, tt.wantErr)
            }
        })
    }
}
```

## Standard Assertions (No Libraries)

```go
// Check equality
if got != want {
    t.Errorf("got %v, want %v", got, want)
}

// Check error
if err != nil {
    t.Fatalf("unexpected error: %v", err)
}

// Check no error
if err == nil {
    t.Fatal("expected error, got nil")
}

// Check specific error
if !errors.Is(err, ErrUserNotFound) {
    t.Errorf("got error %v, want ErrUserNotFound", err)
}

// Check nil
if user == nil {
    t.Fatal("expected user, got nil")
}

// Check slice length
if len(users) != 3 {
    t.Errorf("got %d users, want 3", len(users))
}
```

## Test Setup and Teardown

```go
func TestUserRepo(t *testing.T) {
    // Setup
    ctx := context.Background()
    db := setupTestDB(t)
    repo := NewUserRepo(db)

    // Cleanup after all subtests
    t.Cleanup(func() {
        db.Drop(ctx)
    })

    t.Run("Insert", func(t *testing.T) {
        user := &User{Email: "test@example.com", Name: "Test"}
        err := repo.Insert(ctx, user)
        if err != nil {
            t.Fatalf("Insert() error = %v", err)
        }
        if user.ID.IsZero() {
            t.Error("Insert() did not set ID")
        }
    })

    t.Run("FindByID", func(t *testing.T) {
        // ... uses user from Insert test
    })
}
```

## Test Helpers

```go
func setupTestDB(t *testing.T) *mongo.Database {
    t.Helper()

    ctx := context.Background()
    client, err := mongo.Connect(ctx, options.Client().ApplyURI("mongodb://localhost:27017"))
    if err != nil {
        t.Fatalf("connect: %v", err)
    }

    dbName := fmt.Sprintf("test_%d", time.Now().UnixNano())
    return client.Database(dbName)
}

func createTestUser(t *testing.T, repo *UserRepo, email string) *User {
    t.Helper()

    user := &User{Email: email, Name: "Test User"}
    if err := repo.Insert(context.Background(), user); err != nil {
        t.Fatalf("create test user: %v", err)
    }
    return user
}
```

## Testing HTTP Handlers

```go
func TestUserHandler_Create(t *testing.T) {
    // Setup
    svc := &mockUserService{}
    log := slog.New(slog.NewTextHandler(io.Discard, nil))
    handler := NewUserHandler(svc, log)

    tests := []struct {
        name       string
        body       string
        wantStatus int
    }{
        {
            name:       "valid request",
            body:       `{"email":"test@example.com","name":"Test"}`,
            wantStatus: http.StatusCreated,
        },
        {
            name:       "invalid json",
            body:       `{invalid}`,
            wantStatus: http.StatusBadRequest,
        },
        {
            name:       "missing email",
            body:       `{"name":"Test"}`,
            wantStatus: http.StatusBadRequest,
        },
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            req := httptest.NewRequest("POST", "/users", strings.NewReader(tt.body))
            req.Header.Set("Content-Type", "application/json")
            rec := httptest.NewRecorder()

            handler.Create(rec, req)

            if rec.Code != tt.wantStatus {
                t.Errorf("status = %d, want %d", rec.Code, tt.wantStatus)
            }
        })
    }
}
```

## Mocks (Manual, No Libraries)

```go
type mockUserService struct {
    createFunc func(ctx context.Context, email, name string) (*User, error)
}

func (m *mockUserService) Create(ctx context.Context, email, name string) (*User, error) {
    if m.createFunc != nil {
        return m.createFunc(ctx, email, name)
    }
    return &User{ID: primitive.NewObjectID(), Email: email, Name: name}, nil
}

// Usage in test
svc := &mockUserService{
    createFunc: func(ctx context.Context, email, name string) (*User, error) {
        return nil, ErrDuplicateEmail
    },
}
```

## Test Data Files

```
users/
├── service.go
├── service_test.go
└── testdata/
    ├── user.json
    └── users.json
```

```go
func TestParseUser(t *testing.T) {
    data, err := os.ReadFile("testdata/user.json")
    if err != nil {
        t.Fatalf("read testdata: %v", err)
    }

    var user User
    if err := json.Unmarshal(data, &user); err != nil {
        t.Fatalf("unmarshal: %v", err)
    }

    if user.Email != "test@example.com" {
        t.Errorf("email = %q, want test@example.com", user.Email)
    }
}
```

## Skip Tests Conditionally

```go
func TestWithMongo(t *testing.T) {
    if os.Getenv("MONGO_URI") == "" {
        t.Skip("MONGO_URI not set, skipping integration test")
    }
    // ... test with real MongoDB
}
```

## Parallel Tests

```go
func TestParallel(t *testing.T) {
    tests := []struct {
        name  string
        input int
        want  int
    }{
        {"double 1", 1, 2},
        {"double 2", 2, 4},
        {"double 3", 3, 6},
    }

    for _, tt := range tests {
        tt := tt // capture range variable
        t.Run(tt.name, func(t *testing.T) {
            t.Parallel() // run in parallel
            got := Double(tt.input)
            if got != tt.want {
                t.Errorf("Double(%d) = %d, want %d", tt.input, got, tt.want)
            }
        })
    }
}
```

## Quick Checklist

- [ ] All tests pass: `go test ./...`
- [ ] Race detector passes: `go test -race ./...`
- [ ] Table-driven for multiple cases
- [ ] `t.Helper()` in helper functions
- [ ] `t.Parallel()` where possible
- [ ] `t.Cleanup()` for teardown
- [ ] No testify or assertion libraries

---

## Integration

This skill works with:
- **go-http-handlers**: HTTP handler testing patterns
- **go-mongodb**: Database integration tests
- **go-concurrency-safe**: Race detector usage
- **go-error-tracing**: Testing error paths

Reference this skill when writing any test file.
