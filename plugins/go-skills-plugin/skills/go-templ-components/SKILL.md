---
name: go-templ-components
description: Type-safe Templ templates for Go web applications. JSX-like syntax with compile-time error checking.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go Templ Components

Build type-safe, composable UI components using Templ. Catch template errors at compile time, not runtime.

## When to Use

- Building Go web applications with templates
- Need type-safe template rendering
- Want JSX-like component syntax
- Prefer compile-time errors over runtime panics

---

## Setup

### Install Templ CLI

```bash
go install github.com/a-h/templ/cmd/templ@latest
```

### Add to Project

```bash
go get github.com/a-h/templ
```

### VS Code Extension

Install `templ-vscode` for syntax highlighting and LSP support.

---

## File Organization

```
templates/
├── layouts/
│   └── base.templ       # Main layout wrapper
├── pages/
│   ├── home.templ       # Full page components
│   └── dashboard.templ
├── partials/
│   ├── header.templ     # Shared sections
│   ├── nav.templ
│   └── footer.templ
└── components/
    ├── button.templ     # Reusable UI components
    ├── card.templ
    ├── badge.templ
    └── form/
        ├── input.templ
        └── select.templ
```

---

## Basic Syntax

### Simple Component

```templ
// templates/components/greeting.templ
package components

templ Greeting(name string) {
    <h1>Hello, { name }!</h1>
}
```

### Expressions

Use `{ }` for Go expressions:

```templ
templ UserCard(user User) {
    <div class="card">
        <h2>{ user.Name }</h2>
        <p>{ user.Email }</p>
        <span>Joined { user.CreatedAt.Format("Jan 2, 2006") }</span>
        <span>{ fmt.Sprintf("%d posts", user.PostCount) }</span>
    </div>
}
```

### Conditionals

```templ
templ Alert(message string, isError bool) {
    if isError {
        <div class="alert alert-error">{ message }</div>
    } else {
        <div class="alert alert-info">{ message }</div>
    }
}

templ UserStatus(user User) {
    switch user.Role {
        case "admin":
            <span class="badge-red">Admin</span>
        case "mod":
            <span class="badge-blue">Moderator</span>
        default:
            <span class="badge-gray">User</span>
    }
}
```

### Loops

```templ
templ ItemList(items []Item) {
    <ul>
        for _, item := range items {
            <li>{ item.Name }</li>
        }
    </ul>
}

templ Table(rows []Row) {
    <table>
        for i, row := range rows {
            <tr class={ templ.KV("bg-gray-50", i%2 == 0) }>
                <td>{ row.Name }</td>
            </tr>
        }
    </table>
}
```

---

## Pico CSS Styling

Pico CSS styles semantic HTML elements directly - minimal classes needed.

### Button Variants

```templ
// Pico button variants: default (primary), secondary, contrast, outline
templ Button(text string, variant string, disabled bool) {
    if variant == "" {
        <button disabled?={ disabled } aria-busy?={ disabled }>{ text }</button>
    } else {
        <button class={ variant } disabled?={ disabled } aria-busy?={ disabled }>{ text }</button>
    }
}

// Usage:
// @Button("Submit", "", false)           // Primary
// @Button("Cancel", "secondary", false)  // Secondary
// @Button("Delete", "contrast", false)   // High contrast
// @Button("Edit", "outline", false)      // Outline
```

### Card Component

```templ
// Pico uses semantic <article> for card-like styling
templ Card(title string) {
    <article>
        <header>
            <h3>{ title }</h3>
        </header>
        { children... }
    </article>
}

// Card with highlight using data attribute
templ CardHighlight(title string, highlight bool) {
    <article data-highlight?={ highlight }>
        <header>
            <h3>{ title }</h3>
        </header>
        { children... }
    </article>
}
```

### Status Badge

```templ
// Use semantic HTML elements for status colors
templ StatusBadge(status string, label string) {
    switch status {
        case "success":
            <ins>{ label }</ins>
        case "error":
            <del>{ label }</del>
        case "warning":
            <mark>{ label }</mark>
        default:
            <span>{ label }</span>
    }
}
```

### Dynamic Classes (when needed)

```templ
// Use templ.KV for conditional classes with Pico
templ AlertBox(message string, isError bool) {
    <div
        role="alert"
        class={ templ.KV("contrast", isError) }
    >
        { message }
    </div>
}
```

---

## Component Composition

### Calling Components

Use `@` to render other components:

```templ
templ HomePage(user User, items []Item) {
    @Header()
    <main>
        @UserCard(user)
        @ItemList(items)
    </main>
    @Footer()
}
```

### Children (Slots)

Accept children using `templ.Component` or `{ children... }`:

```templ
// Card that wraps content
templ Card(title string) {
    <div class="card">
        <h3 class="card-title">{ title }</h3>
        <div class="card-body">
            { children... }
        </div>
    </div>
}

// Usage
templ Dashboard() {
    @Card("User Stats") {
        <p>Total users: 1,234</p>
        <p>Active today: 89</p>
    }
}
```

### Named Slots via Component Parameter

```templ
templ Modal(title string, footer templ.Component) {
    <div class="modal">
        <div class="modal-header">{ title }</div>
        <div class="modal-body">
            { children... }
        </div>
        <div class="modal-footer">
            @footer
        </div>
    </div>
}

// Usage
templ DeleteConfirm(itemID string) {
    @Modal("Confirm Delete", modalFooter(itemID)) {
        <p>Are you sure you want to delete this item?</p>
    }
}

templ modalFooter(itemID string) {
    <button hx-delete={ "/items/" + itemID }>Delete</button>
    <button onclick="closeModal()">Cancel</button>
}
```

---

## Layouts

### Base Layout

```templ
// templates/layouts/base.templ
package layouts

templ Base(title string) {
    <!DOCTYPE html>
    <html lang="en" data-theme="light">
    <head>
        <meta charset="UTF-8"/>
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <title>{ title }</title>
        <!-- Pico CSS from CDN -->
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css"/>
        <!-- Custom overrides -->
        <link rel="stylesheet" href="/static/css/custom.css"/>
    </head>
    <body>
        @Nav()
        <main class="container">
            { children... }
        </main>
        @Footer()
        <!-- HTMX from CDN -->
        <script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js"></script>
    </body>
    </html>
}
```

### Page Using Layout

```templ
// templates/pages/home.templ
package pages

import "myapp/templates/layouts"
import "myapp/templates/components"

templ HomePage(user User, items []Item) {
    @layouts.Base("Home") {
        <hgroup>
            <h1>Welcome, { user.Name }</h1>
            <p>Your dashboard</p>
        </hgroup>

        <div class="grid">
            for _, item := range items {
                @components.ItemCard(item)
            }
        </div>
    }
}
```

---

## HTMX Fragments

Create components that work as HTMX response fragments.

```templ
// Full page with layout
templ ItemsPage(items []Item) {
    @layouts.Base("Items") {
        <div id="items-container">
            @ItemsList(items)
        </div>
    }
}

// Fragment for HTMX (no layout wrapper)
templ ItemsList(items []Item) {
    for _, item := range items {
        @ItemCard(item)
    }
}

// Single item fragment using Pico's article styling
templ ItemCard(item Item) {
    <article id={ "item-" + item.ID }>
        <header>
            <h3>{ item.Name }</h3>
        </header>
        <p>{ item.Description }</p>
        <footer>
            <button
                class="secondary"
                hx-delete={ "/items/" + item.ID }
                hx-target={ "#item-" + item.ID }
                hx-swap="outerHTML"
                hx-confirm="Delete this item?"
            >
                Delete
            </button>
        </footer>
    </article>
}
```

### Handler Pattern

```go
func handleItems(w http.ResponseWriter, r *http.Request) {
    items := getItems()

    if r.Header.Get("HX-Request") == "true" {
        // HTMX request: return fragment only
        templates.ItemsList(items).Render(r.Context(), w)
    } else {
        // Browser request: return full page
        templates.ItemsPage(items).Render(r.Context(), w)
    }
}
```

---

## Attributes

### Boolean Attributes

```templ
templ Checkbox(label string, checked bool, disabled bool) {
    <label>
        <input
            type="checkbox"
            checked?={ checked }
            disabled?={ disabled }
        />
        { label }
    </label>
}
```

### Spread Attributes

```templ
templ Input(attrs templ.Attributes) {
    <input class="input" { attrs... }/>
}

// Usage
@Input(templ.Attributes{
    "type": "email",
    "name": "email",
    "placeholder": "Enter email",
    "required": true,
})
```

### Dynamic Attributes

```templ
templ Link(href string, external bool) {
    <a
        href={ templ.SafeURL(href) }
        if external {
            target="_blank"
            rel="noopener noreferrer"
        }
    >
        { children... }
    </a>
}
```

---

## Scripts and Styles

### Inline Script

```templ
templ WithScript() {
    <div id="counter">0</div>
    <script>
        document.getElementById('counter').onclick = function() {
            this.textContent = parseInt(this.textContent) + 1;
        }
    </script>
}
```

### Script Component (Deduplicated)

```templ
// Only included once even if component used multiple times
script chartInit() {
    window.initChart = function(el) {
        // Chart initialization
    }
}

templ Chart(data []Point) {
    @chartInit()
    <canvas id="chart" onload="initChart(this)"></canvas>
}
```

### CSS Component

```templ
css cardStyles() {
    .custom-card {
        border-radius: 8px;
        box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }
}

templ StyledCard() {
    @cardStyles()
    <div class="custom-card">
        { children... }
    </div>
}
```

---

## Raw HTML

When you need to render trusted HTML:

```templ
templ MarkdownContent(html string) {
    <div class="prose">
        @templ.Raw(html)
    </div>
}
```

**Warning**: Only use with trusted/sanitized content. Never use with user input.

---

## Error Handling in Templates

### Showing Errors

```templ
// Pico CSS uses aria-invalid for form validation styling
templ FormField(name string, value string, err string) {
    <label>
        { name }
        <input
            name={ strings.ToLower(name) }
            value={ value }
            aria-invalid={ err != "" }
        />
        if err != "" {
            <small>{ err }</small>
        }
    </label>
}
```

### Error Page

```templ
templ ErrorPage(code int, message string) {
    @layouts.Base(fmt.Sprintf("Error %d", code)) {
        <article>
            <header>
                <hgroup>
                    <h1>{ fmt.Sprintf("%d", code) }</h1>
                    <p>{ message }</p>
                </hgroup>
            </header>
            <a href="/" role="button">Go Home</a>
        </article>
    }
}
```

---

## Build Workflow

### Generate Go Files

```bash
templ generate
```

This creates `*_templ.go` files next to your `.templ` files.

### Watch Mode

```bash
templ generate --watch
```

### With Air (Hot Reload)

`air.toml`:
```toml
[build]
  cmd = "templ generate && go build -o ./tmp/main ."
  bin = "tmp/main"
  include_ext = ["go", "templ"]
  exclude_dir = ["tmp", "node_modules"]
```

---

## Type Safety Examples

### Struct Props

```templ
type CardProps struct {
    Title       string
    Description string
    ImageURL    string
    Link        string
    Tags        []string
}

templ Card(props CardProps) {
    <article>
        if props.ImageURL != "" {
            <img src={ props.ImageURL } alt={ props.Title }/>
        }
        <header>
            <h3>{ props.Title }</h3>
        </header>
        <p>{ props.Description }</p>
        if len(props.Tags) > 0 {
            <footer>
                for _, tag := range props.Tags {
                    <mark>{ tag }</mark>
                }
            </footer>
        }
        <a href={ templ.SafeURL(props.Link) } role="button" class="outline">Read more</a>
    </article>
}
```

### Enum-like Types

```templ
// Pico CSS button variants
type ButtonVariant string

const (
    ButtonPrimary   ButtonVariant = ""          // Default Pico styling
    ButtonSecondary ButtonVariant = "secondary"
    ButtonContrast  ButtonVariant = "contrast"
    ButtonOutline   ButtonVariant = "outline"
)

templ Button(text string, variant ButtonVariant) {
    if variant == "" {
        <button>{ text }</button>
    } else {
        <button class={ string(variant) }>{ text }</button>
    }
}

// Usage:
// @Button("Submit", ButtonPrimary)
// @Button("Cancel", ButtonSecondary)
// @Button("Delete", ButtonContrast)
// @Button("Edit", ButtonOutline)
```

---

## Common Patterns

### Icon Component

```templ
templ Icon(name string, size string) {
    <svg class={ "icon", "icon-" + size }>
        <use href={ "/static/icons.svg#" + name }></use>
    </svg>
}

// Usage
@Icon("trash", "sm")
@Icon("edit", "md")
```

### Empty State

```templ
templ EmptyState(title string, description string) {
    <article>
        <header>
            <h3>{ title }</h3>
        </header>
        <p>{ description }</p>
        { children... }
    </article>
}

// Usage
@EmptyState("No items", "Get started by creating a new item.") {
    <button>Create Item</button>
}
```

### Loading Skeleton

```templ
// Use Pico's aria-busy for loading states
templ LoadingCard() {
    <article aria-busy="true">
        Loading...
    </article>
}

templ ItemsListWithLoading(items []Item, loading bool) {
    if loading {
        for i := 0; i < 3; i++ {
            @LoadingCard()
        }
    } else if len(items) == 0 {
        @EmptyState("No items", "Create your first item to get started.")
    } else {
        for _, item := range items {
            @ItemCard(item)
        }
    }
}
```

---

## Integration

This skill works with:
- **go-project-bootstrap**: Initial project setup
- **go-htmx-core**: HTMX attributes and patterns
- **go-htmx-sse**: Real-time streaming components
- **go-htmx-forms**: Form components with validation
- **go-pico-embed**: Asset embedding and deployment

Reference this skill when writing any `.templ` file.
