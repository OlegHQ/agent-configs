---
name: go-project-bootstrap
description: Bootstrap a complete Go web project with HTMX, Pico CSS, Templ, and Air hot reload. Zero Node.js dependencies, single binary deployment ready.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go Project Bootstrap

Create production-ready Go web applications with zero Node.js dependencies. Uses HTMX for interactivity, Pico CSS for styling, Templ for type-safe templates, and Air for hot reload.

## When to Use

- Starting a new Go web project
- Need HTMX + Pico CSS setup without Node.js
- Want hot reload development with Air
- Building for single binary deployment

---

## Prerequisites

Install Go tools (one-time setup):

```bash
# Templ - type-safe HTML templates
go install github.com/a-h/templ/cmd/templ@latest

# Air - hot reload for Go
go install github.com/air-verse/air@latest
```

---

## Project Structure

```
myproject/
├── cmd/
│   └── server/
│       └── main.go           # Entry point, router, static files
├── internal/
│   ├── handlers/
│   │   ├── pages.go          # Full page handlers
│   │   └── api.go            # API/fragment handlers
│   └── middleware/
│       └── logging.go        # Request logging (optional)
├── views/
│   ├── layouts/
│   │   └── base.templ        # Base HTML with CDN links
│   ├── pages/
│   │   ├── home.templ        # Home page
│   │   └── about.templ       # About page
│   └── components/
│       ├── nav.templ         # Navigation
│       ├── card.templ        # Card component
│       └── button.templ      # Button variants
├── static/
│   └── css/
│       └── custom.css        # HTMX indicators, custom styles
├── .air.toml                 # Hot reload configuration
├── .gitignore
├── go.mod
└── Makefile
```

---

## Complete Project Files

### go.mod

```go
module myproject

go 1.22

require github.com/a-h/templ v0.2.793
```

Run `go mod tidy` after creating to fetch dependencies.

---

### cmd/server/main.go

```go
package main

import (
	"embed"
	"io/fs"
	"log"
	"net/http"
	"os"

	"myproject/internal/handlers"
)

//go:embed all:static
var staticFS embed.FS

func main() {
	mux := http.NewServeMux()

	// Static files (custom CSS, images)
	sub, _ := fs.Sub(staticFS, "static")
	mux.Handle("GET /static/", http.StripPrefix("/static/",
		http.FileServer(http.FS(sub))))

	// Pages
	mux.HandleFunc("GET /", handlers.HandleHome)
	mux.HandleFunc("GET /about", handlers.HandleAbout)

	// API endpoints (HTMX fragments)
	mux.HandleFunc("GET /api/greeting", handlers.HandleGreeting)
	mux.HandleFunc("POST /api/contact", handlers.HandleContact)

	// Server
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	log.Printf("Server starting on http://localhost:%s", port)
	log.Fatal(http.ListenAndServe("localhost:"+port, mux))
}
```

---

### internal/handlers/pages.go

```go
package handlers

import (
	"net/http"

	"myproject/views/pages"
)

func HandleHome(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != "/" {
		http.NotFound(w, r)
		return
	}
	pages.HomePage().Render(r.Context(), w)
}

func HandleAbout(w http.ResponseWriter, r *http.Request) {
	pages.AboutPage().Render(r.Context(), w)
}
```

---

### internal/handlers/api.go

```go
package handlers

import (
	"fmt"
	"net/http"
	"time"
)

func HandleGreeting(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html")
	greeting := fmt.Sprintf("Hello! The time is %s", time.Now().Format("15:04:05"))
	fmt.Fprintf(w, "<strong>%s</strong>", greeting)
}

func HandleContact(w http.ResponseWriter, r *http.Request) {
	r.ParseForm()
	name := r.FormValue("name")
	email := r.FormValue("email")

	// Validate
	if name == "" || email == "" {
		w.WriteHeader(http.StatusBadRequest)
		w.Header().Set("Content-Type", "text/html")
		fmt.Fprint(w, `<p role="alert">Please fill in all fields.</p>`)
		return
	}

	// Success response
	w.Header().Set("Content-Type", "text/html")
	fmt.Fprintf(w, `<p><ins>Thanks %s! We'll contact you at %s.</ins></p>`, name, email)
}
```

---

### views/layouts/base.templ

```templ
package layouts

templ Base(title string) {
	<!DOCTYPE html>
	<html lang="en" data-theme="light">
	<head>
		<meta charset="UTF-8"/>
		<meta name="viewport" content="width=device-width, initial-scale=1.0"/>
		<title>{ title }</title>

		<!-- Pico CSS - Semantic HTML styling, minimal classes -->
		<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css"/>

		<!-- Custom styles (HTMX indicators, overrides) -->
		<link rel="stylesheet" href="/static/css/custom.css"/>
	</head>
	<body>
		<header class="container">
			@Nav()
		</header>

		<main class="container">
			{ children... }
		</main>

		<footer class="container">
			<hr/>
			<p><small>Built with Go + HTMX + Pico CSS</small></p>
		</footer>

		<!-- HTMX - Dynamic HTML via attributes -->
		<script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js"></script>
	</body>
	</html>
}

templ Nav() {
	<nav>
		<ul>
			<li><strong>MyProject</strong></li>
		</ul>
		<ul>
			<li><a href="/">Home</a></li>
			<li><a href="/about">About</a></li>
			<li><a href="#contact" role="button" class="outline">Contact</a></li>
		</ul>
	</nav>
}
```

---

### views/pages/home.templ

```templ
package pages

import "myproject/views/layouts"
import "myproject/views/components"

templ HomePage() {
	@layouts.Base("Home - MyProject") {
		<article>
			<header>
				<hgroup>
					<h1>Welcome to MyProject</h1>
					<p>A Go web application with HTMX and Pico CSS.</p>
				</hgroup>
			</header>

			<section>
				<h2>Features</h2>
				<ul>
					<li>Zero Node.js dependencies</li>
					<li>Single binary deployment</li>
					<li>Hot reload development with Air</li>
					<li>Type-safe templates with Templ</li>
					<li>Semantic HTML styling with Pico CSS</li>
				</ul>
			</section>

			<section>
				<h2>HTMX Demo</h2>
				<p>Click the button to load dynamic content:</p>
				<button
					hx-get="/api/greeting"
					hx-target="#greeting"
					hx-swap="innerHTML"
					hx-indicator="#greeting-spinner"
				>
					Load Greeting
				</button>
				<span id="greeting-spinner" class="htmx-indicator" aria-busy="true"></span>
				<p id="greeting"></p>
			</section>

			<section id="contact">
				<h2>Contact Form</h2>
				<form hx-post="/api/contact" hx-target="#contact-result" hx-swap="innerHTML">
					<div class="grid">
						<label>
							Name
							<input type="text" name="name" placeholder="Your name" required/>
						</label>
						<label>
							Email
							<input type="email" name="email" placeholder="your@email.com" required/>
						</label>
					</div>
					<button type="submit">
						<span class="htmx-hide-on-request">Send Message</span>
						<span class="htmx-indicator" aria-busy="true">Sending...</span>
					</button>
				</form>
				<div id="contact-result"></div>
			</section>

			<section>
				<h2>Components</h2>
				<div class="grid">
					@components.Card("Card One") {
						<p>This card uses Pico's semantic article styling.</p>
						@components.Button("Primary", "")
					}
					@components.Card("Card Two") {
						<p>Cards automatically have shadows and padding.</p>
						@components.Button("Secondary", "secondary")
					}
				</div>
			</section>
		</article>
	}
}
```

---

### views/pages/about.templ

```templ
package pages

import "myproject/views/layouts"

templ AboutPage() {
	@layouts.Base("About - MyProject") {
		<article>
			<header>
				<h1>About</h1>
			</header>

			<section>
				<h2>Technology Stack</h2>
				<figure>
					<table>
						<thead>
							<tr>
								<th>Technology</th>
								<th>Purpose</th>
								<th>Source</th>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td>Go</td>
								<td>Backend server</td>
								<td>Compiled binary</td>
							</tr>
							<tr>
								<td>HTMX</td>
								<td>Dynamic HTML</td>
								<td>CDN</td>
							</tr>
							<tr>
								<td>Pico CSS</td>
								<td>Styling</td>
								<td>CDN</td>
							</tr>
							<tr>
								<td>Templ</td>
								<td>Templates</td>
								<td>Compiled</td>
							</tr>
						</tbody>
					</table>
				</figure>
			</section>

			<section>
				<h2>Accordion Example</h2>
				<details>
					<summary>How does HTMX work?</summary>
					<p>HTMX extends HTML with attributes like hx-get, hx-post, hx-target that make AJAX requests and swap content without writing JavaScript.</p>
				</details>
				<details>
					<summary>Why Pico CSS?</summary>
					<p>Pico CSS styles semantic HTML elements directly. No utility classes needed - just write proper HTML and it looks good.</p>
				</details>
				<details>
					<summary>What about JavaScript?</summary>
					<p>HTMX is the only JavaScript needed. Everything else is server-rendered HTML from Go templates.</p>
				</details>
			</section>

			<a href="/" role="button" class="outline">Back to Home</a>
		</article>
	}
}
```

---

### views/components/card.templ

```templ
package components

// Card uses Pico's semantic <article> styling
templ Card(title string) {
	<article>
		<header>
			<h3>{ title }</h3>
		</header>
		{ children... }
	</article>
}
```

---

### views/components/button.templ

```templ
package components

// Button with Pico CSS variants
// Variants: "" (primary), "secondary", "contrast", "outline"
templ Button(text string, variant string) {
	if variant == "" {
		<button>{ text }</button>
	} else {
		<button class={ variant }>{ text }</button>
	}
}

// ButtonLink renders as a link styled as button
templ ButtonLink(text string, href string, variant string) {
	if variant == "" {
		<a href={ templ.SafeURL(href) } role="button">{ text }</a>
	} else {
		<a href={ templ.SafeURL(href) } role="button" class={ variant }>{ text }</a>
	}
}
```

---

### views/components/nav.templ

```templ
package components

// Nav renders a responsive navigation bar
templ Nav(brand string, items []NavItem) {
	<nav>
		<ul>
			<li><strong>{ brand }</strong></li>
		</ul>
		<ul>
			for _, item := range items {
				<li>
					if item.IsButton {
						<a href={ templ.SafeURL(item.Href) } role="button" class="outline">{ item.Label }</a>
					} else {
						<a href={ templ.SafeURL(item.Href) }>{ item.Label }</a>
					}
				</li>
			}
		</ul>
	</nav>
}

type NavItem struct {
	Label    string
	Href     string
	IsButton bool
}
```

---

### static/css/custom.css

```css
/* HTMX Loading Indicators */
.htmx-indicator {
    display: none;
}

.htmx-request .htmx-indicator {
    display: inline-block;
}

.htmx-request.htmx-indicator {
    display: inline-block;
}

/* Hide content during HTMX request */
.htmx-request .htmx-hide-on-request {
    display: none;
}

/* Optional: Custom Pico CSS variable overrides */
:root {
    /* Uncomment to customize */
    /* --pico-font-size: 100%; */
    /* --pico-border-radius: 0.375rem; */
    /* --pico-primary: #0ea5e9; */
}

/* Optional: Dark mode preference */
/*
@media (prefers-color-scheme: dark) {
    :root:not([data-theme]) {
        --pico-background-color: #11191f;
    }
}
*/
```

---

### .air.toml

```toml
# Air configuration for Go + Templ hot reload

root = "."
tmp_dir = "tmp"

[build]
  # Generate Templ files, then build Go
  cmd = "templ generate && go build -o ./tmp/main ./cmd/server"
  bin = "./tmp/main"

  # Delay before rebuilding (ms)
  delay = 500

  # Watch these file extensions
  include_ext = ["go", "templ"]

  # Exclude directories
  exclude_dir = ["tmp", "bin", "static", "node_modules", ".git"]

  # Exclude generated Templ files from triggering rebuild
  exclude_regex = ["_templ\\.go$"]

[screen]
  clear_on_rebuild = true

[log]
  time = false

[misc]
  clean_on_exit = true
```

---

### .gitignore

```
# Build outputs
bin/
tmp/

# Generated Templ files (regenerated on build)
*_templ.go

# Go
go.sum

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local
```

---

### Makefile

```makefile
.PHONY: dev build clean deps generate run

# Development with hot reload
dev:
	@echo "Starting development server with hot reload..."
	@air

# Generate Templ files
generate:
	@templ generate

# Build production binary
build: generate
	@echo "Building production binary..."
	@CGO_ENABLED=0 go build -ldflags="-s -w" -o bin/server ./cmd/server
	@echo "Binary created: bin/server"
	@ls -lh bin/server

# Run production binary
run: build
	@./bin/server

# Install dependencies (one-time)
deps:
	@echo "Installing Go tools..."
	@go install github.com/a-h/templ/cmd/templ@latest
	@go install github.com/air-verse/air@latest
	@echo "Installing Go module dependencies..."
	@go mod tidy
	@echo "Done! Run 'make dev' to start development."

# Clean build artifacts
clean:
	@rm -rf bin/ tmp/
	@find . -name '*_templ.go' -delete
	@echo "Cleaned build artifacts."
```

---

## Development Workflow

```bash
# 1. Create project directory
mkdir myproject && cd myproject

# 2. Initialize Go module
go mod init myproject

# 3. Create the file structure (copy files above)

# 4. Install tools and dependencies
make deps

# 5. Start development server
make dev

# 6. Open browser
open http://localhost:8080

# 7. Edit .templ or .go files - Air auto-rebuilds!
```

---

## Production Deployment

```bash
# Build optimized binary
make build

# Binary is at bin/server (~5-8 MB)
./bin/server

# Or with custom port
PORT=3000 ./bin/server
```

### Docker Deployment

```dockerfile
# Build stage
FROM golang:1.22-alpine AS builder

WORKDIR /app

# Install Templ
RUN go install github.com/a-h/templ/cmd/templ@latest

# Copy and download dependencies
COPY go.mod go.sum ./
RUN go mod download

# Copy source
COPY . .

# Build
RUN templ generate
RUN CGO_ENABLED=0 go build -ldflags="-s -w" -o server ./cmd/server

# Runtime stage (tiny!)
FROM alpine:latest

WORKDIR /app
COPY --from=builder /app/server .
COPY --from=builder /app/static ./static

EXPOSE 8080
CMD ["./server"]
```

Build: `docker build -t myproject .`
Run: `docker run -p 8080:8080 myproject`

---

## Pico CSS Quick Reference

### Semantic Elements (auto-styled)

| Element | Styling |
|---------|---------|
| `<article>` | Card with shadow |
| `<section>` | Grouped content |
| `<header>` | Page/article header |
| `<footer>` | Page/article footer |
| `<nav>` | Navigation bar |
| `<aside>` | Sidebar |
| `<details>` | Accordion |
| `<dialog>` | Modal |
| `<progress>` | Progress bar |
| `<hgroup>` | Heading group |

### Utility Classes

| Class | Purpose |
|-------|---------|
| `container` | Centered max-width |
| `grid` | Auto-responsive grid |
| `secondary` | Gray button |
| `contrast` | High contrast button |
| `outline` | Outline button |

### Form Validation

```html
<input aria-invalid="true"/>   <!-- Red border -->
<input aria-invalid="false"/>  <!-- Green border -->
<small>Error message</small>   <!-- Help text -->
```

### Dark Mode

```html
<html data-theme="dark">   <!-- Force dark -->
<html data-theme="light">  <!-- Force light -->
<html>                     <!-- System preference -->
```

---

## Integration

This skill works with:
- **go-pico-embed**: Asset embedding strategies
- **go-htmx-core**: HTMX handler patterns
- **go-templ-components**: Advanced template patterns
- **go-htmx-forms**: Form validation patterns
- **go-htmx-sse**: Real-time updates

Reference this skill when starting any new Go web project.
