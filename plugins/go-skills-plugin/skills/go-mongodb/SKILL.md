---
name: go-mongodb
description: MongoDB patterns in Go. Official driver, proper indexes, same struct for bson+json. Use for database code.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go MongoDB Patterns

## When to Use

- Implementing repository layer with MongoDB
- CRUD operations with the official driver
- Creating indexes for performance
- Handling duplicate key errors
- Running aggregation pipelines

---

## Connection

```go
package db

import (
    "context"
    "fmt"

    "go.mongodb.org/mongo-driver/mongo"
    "go.mongodb.org/mongo-driver/mongo/options"
)

func Connect(ctx context.Context, uri, dbName string) (*mongo.Database, error) {
    client, err := mongo.Connect(ctx, options.Client().ApplyURI(uri))
    if err != nil {
        return nil, fmt.Errorf("connect mongo: %w", err)
    }

    if err := client.Ping(ctx, nil); err != nil {
        return nil, fmt.Errorf("ping mongo: %w", err)
    }

    return client.Database(dbName), nil
}
```

## One Struct for Everything

Same struct for JSON responses, MongoDB documents, and domain logic:

```go
type User struct {
    ID        primitive.ObjectID `bson:"_id,omitempty" json:"id"`
    Email     string             `bson:"email" json:"email"`
    Name      string             `bson:"name" json:"name"`
    Role      string             `bson:"role" json:"role"`
    CreatedAt time.Time          `bson:"created_at" json:"createdAt"`
    UpdatedAt time.Time          `bson:"updated_at" json:"updatedAt"`
}
```

## Repository Pattern

```go
type UserRepo struct {
    coll *mongo.Collection
}

func NewUserRepo(db *mongo.Database) *UserRepo {
    return &UserRepo{coll: db.Collection("users")}
}
```

## Find By ID

```go
func (r *UserRepo) FindByID(ctx context.Context, id primitive.ObjectID) (*User, error) {
    var user User
    err := r.coll.FindOne(ctx, bson.M{"_id": id}).Decode(&user)
    if errors.Is(err, mongo.ErrNoDocuments) {
        return nil, ErrUserNotFound
    }
    if err != nil {
        return nil, fmt.Errorf("find user %s: %w", id, err)
    }
    return &user, nil
}
```

## Find By Field

```go
func (r *UserRepo) FindByEmail(ctx context.Context, email string) (*User, error) {
    var user User
    err := r.coll.FindOne(ctx, bson.M{"email": email}).Decode(&user)
    if errors.Is(err, mongo.ErrNoDocuments) {
        return nil, ErrUserNotFound
    }
    if err != nil {
        return nil, fmt.Errorf("find user by email: %w", err)
    }
    return &user, nil
}
```

## List with Pagination

```go
func (r *UserRepo) List(ctx context.Context, limit, offset int) ([]*User, error) {
    opts := options.Find().
        SetLimit(int64(limit)).
        SetSkip(int64(offset)).
        SetSort(bson.D{{Key: "created_at", Value: -1}})

    cursor, err := r.coll.Find(ctx, bson.M{}, opts)
    if err != nil {
        return nil, fmt.Errorf("list users: %w", err)
    }
    defer cursor.Close(ctx)

    var users []*User
    if err := cursor.All(ctx, &users); err != nil {
        return nil, fmt.Errorf("decode users: %w", err)
    }
    return users, nil
}
```

## Insert

```go
func (r *UserRepo) Insert(ctx context.Context, u *User) error {
    u.ID = primitive.NewObjectID()
    u.CreatedAt = time.Now()
    u.UpdatedAt = u.CreatedAt

    _, err := r.coll.InsertOne(ctx, u)
    if mongo.IsDuplicateKeyError(err) {
        return ErrDuplicateEmail
    }
    if err != nil {
        return fmt.Errorf("insert user: %w", err)
    }
    return nil
}
```

## Update

```go
func (r *UserRepo) Update(ctx context.Context, u *User) error {
    u.UpdatedAt = time.Now()

    result, err := r.coll.UpdateOne(ctx,
        bson.M{"_id": u.ID},
        bson.M{"$set": bson.M{
            "name":       u.Name,
            "role":       u.Role,
            "updated_at": u.UpdatedAt,
        }},
    )
    if err != nil {
        return fmt.Errorf("update user: %w", err)
    }
    if result.MatchedCount == 0 {
        return ErrUserNotFound
    }
    return nil
}
```

## Delete

```go
func (r *UserRepo) Delete(ctx context.Context, id primitive.ObjectID) error {
    result, err := r.coll.DeleteOne(ctx, bson.M{"_id": id})
    if err != nil {
        return fmt.Errorf("delete user: %w", err)
    }
    if result.DeletedCount == 0 {
        return ErrUserNotFound
    }
    return nil
}
```

## Indexes

Create indexes on startup:

```go
func (r *UserRepo) EnsureIndexes(ctx context.Context) error {
    indexes := []mongo.IndexModel{
        {
            Keys:    bson.D{{Key: "email", Value: 1}},
            Options: options.Index().SetUnique(true),
        },
        {
            Keys: bson.D{{Key: "created_at", Value: -1}},
        },
    }

    _, err := r.coll.Indexes().CreateMany(ctx, indexes)
    if err != nil {
        return fmt.Errorf("create indexes: %w", err)
    }
    return nil
}
```

## Count

```go
func (r *UserRepo) Count(ctx context.Context) (int64, error) {
    count, err := r.coll.CountDocuments(ctx, bson.M{})
    if err != nil {
        return 0, fmt.Errorf("count users: %w", err)
    }
    return count, nil
}
```

## Aggregation

```go
func (r *UserRepo) CountByRole(ctx context.Context) (map[string]int64, error) {
    pipeline := []bson.M{
        {"$group": bson.M{
            "_id":   "$role",
            "count": bson.M{"$sum": 1},
        }},
    }

    cursor, err := r.coll.Aggregate(ctx, pipeline)
    if err != nil {
        return nil, fmt.Errorf("aggregate: %w", err)
    }
    defer cursor.Close(ctx)

    var results []struct {
        Role  string `bson:"_id"`
        Count int64  `bson:"count"`
    }
    if err := cursor.All(ctx, &results); err != nil {
        return nil, fmt.Errorf("decode aggregate: %w", err)
    }

    counts := make(map[string]int64)
    for _, r := range results {
        counts[r.Role] = r.Count
    }
    return counts, nil
}
```

## Transactions

```go
func (r *UserRepo) TransferRole(ctx context.Context, fromID, toID primitive.ObjectID, role string) error {
    session, err := r.coll.Database().Client().StartSession()
    if err != nil {
        return fmt.Errorf("start session: %w", err)
    }
    defer session.EndSession(ctx)

    _, err = session.WithTransaction(ctx, func(sessCtx mongo.SessionContext) (any, error) {
        // Remove role from first user
        _, err := r.coll.UpdateOne(sessCtx,
            bson.M{"_id": fromID},
            bson.M{"$unset": bson.M{"role": ""}},
        )
        if err != nil {
            return nil, err
        }

        // Add role to second user
        _, err = r.coll.UpdateOne(sessCtx,
            bson.M{"_id": toID},
            bson.M{"$set": bson.M{"role": role}},
        )
        if err != nil {
            return nil, err
        }

        return nil, nil
    })

    if err != nil {
        return fmt.Errorf("transaction: %w", err)
    }
    return nil
}
```

## Domain Errors

```go
// users/errors.go
var (
    ErrUserNotFound   = errors.New("user not found")
    ErrDuplicateEmail = errors.New("email already exists")
)
```

---

## Integration

This skill works with:
- **go-clean-architecture**: Repository layer in the stack
- **go-context-patterns**: Database operation timeouts
- **go-error-tracing**: Translating database errors
- **go-testing-simple**: Integration test setup

Reference this skill when writing any MongoDB code.
