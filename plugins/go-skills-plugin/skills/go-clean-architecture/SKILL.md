---
name: go-clean-architecture
description: Clean Go architecture. Repository/Service/Handler layers, acyclic packages, functional composition, no globals. Use for project structure.
allowed-tools: Read, Glob, Grep
---

# Go Clean Architecture

## When to Use

- Starting a new Go project
- Structuring packages and dependencies
- Deciding where to put business logic
- Wiring dependencies in main.go
- Refactoring for better separation

---

## Layers

```
handler → service → repository → database
    ↓         ↓           ↓
  (deps passed explicitly via function params)
```

Each layer has ONE responsibility:
- **Handler:** Parse HTTP, validate input, call service, write response
- **Service:** Business logic, orchestrate repository calls
- **Repository:** Database access, translate domain errors

## Package Structure (Acyclic Tree)

```
app/
├── config/      # Env loading, no deps
├── db/          # MongoDB connection
├── users/       # Feature package
│   ├── types.go     # User struct (bson + json tags)
│   ├── errors.go    # ErrUserNotFound, etc.
│   ├── repo.go      # Repository (DB access)
│   ├── service.go   # Business logic
│   └── handler.go   # HTTP handlers
├── orders/      # Another feature
│   ├── types.go
│   ├── repo.go
│   ├── service.go
│   └── handler.go
└── main.go      # Wires everything
```

## Dependency Injection via Functions

```go
// handler.go - receives service as param
func NewUserHandler(svc *UserService, log *slog.Logger) *UserHandler {
    return &UserHandler{svc: svc, log: log}
}

// service.go - receives repo as param
func NewUserService(repo *UserRepo) *UserService {
    return &UserService{repo: repo}
}

// repo.go - receives db as param
func NewUserRepo(db *mongo.Database) *UserRepo {
    return &UserRepo{coll: db.Collection("users")}
}

// main.go - wires the graph
func main() {
    cfg := config.LoadConfig()
    log := logging.NewLogger(cfg.LogJSON)

    db, _ := db.Connect(ctx, cfg.MongoURI)

    // Wire users feature
    userRepo := users.NewUserRepo(db)
    userSvc := users.NewUserService(userRepo)
    userHandler := users.NewUserHandler(userSvc, log)

    // Wire orders feature
    orderRepo := orders.NewOrderRepo(db)
    orderSvc := orders.NewOrderService(orderRepo, userRepo)
    orderHandler := orders.NewOrderHandler(orderSvc, log)

    // Routes
    mux := http.NewServeMux()
    mux.HandleFunc("POST /users", userHandler.Create)
    mux.HandleFunc("POST /orders", orderHandler.Create)
}
```

## Rules

### NO Global Variables
```go
// BAD
var db *mongo.Client  // global state

// GOOD
type UserRepo struct {
    coll *mongo.Collection  // injected dependency
}
```

### NO init() Functions for State
```go
// BAD
func init() {
    db, _ = mongo.Connect(...)  // hidden initialization
}

// GOOD
func main() {
    db, err := db.Connect(ctx, cfg.MongoURI)  // explicit
    if err != nil {
        log.Fatal(err)
    }
}
```

### Dependencies Flow Down Only (Acyclic)
```
config ← db ← users ← main
              ↑
         orders ←─┘

NEVER: users → orders → users (cycle)
```

### One Struct for JSON + BSON + Domain
```go
type User struct {
    ID        primitive.ObjectID `bson:"_id,omitempty" json:"id"`
    Email     string             `bson:"email" json:"email"`
    Name      string             `bson:"name" json:"name"`
    CreatedAt time.Time          `bson:"created_at" json:"createdAt"`
}
// Same struct used in: HTTP response, MongoDB, service logic
```

### Interface Only When 2+ Implementations Exist
```go
// BAD - interface for one implementation
type UserRepository interface {
    FindByID(ctx context.Context, id string) (*User, error)
}

// GOOD - concrete type, no interface needed
type UserRepo struct {
    coll *mongo.Collection
}
```

---

## Integration

This skill works with:
- **go-mental-model**: Complexity validation
- **go-http-handlers**: Handler layer implementation
- **go-mongodb**: Repository layer patterns
- **go-project-bootstrap**: Initial project structure

Reference this skill when designing project architecture.
