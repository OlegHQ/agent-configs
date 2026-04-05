---
name: go-htmx-core
description: Core HTMX + Go patterns for building hypermedia-driven web applications with stdlib http.ServeMux.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go + HTMX Core Patterns

Build modern web applications using Go's stdlib and HTMX. No JavaScript frameworks, just HTML attributes that make your pages interactive.

## When to Use

- Building web UIs with Go backend
- Replacing React/Vue/Angular with server-rendered HTML
- Need real-time updates without WebSocket complexity
- Want single binary deployment

---

## Project Structure

```
project/
├── main.go              # Entry point, router setup
├── handlers/
│   ├── pages.go         # Full page handlers
│   └── fragments.go     # HTMX fragment handlers
├── templates/
│   ├── layouts/
│   │   └── base.templ
│   ├── pages/
│   │   └── home.templ
│   └── components/
│       └── card.templ
├── static/
│   └── css/
│       └── custom.css    # HTMX indicators, custom styles
└── go.mod
```

---

## Router Setup (Go 1.22+)

Go 1.22 introduced pattern matching in `http.ServeMux`. No external router needed.

```go
package main

import (
    "embed"
    "net/http"
)

//go:embed static/*
var staticFS embed.FS

func main() {
    mux := http.NewServeMux()

    // Static files
    mux.Handle("GET /static/", http.FileServer(http.FS(staticFS)))

    // Pages (full HTML)
    mux.HandleFunc("GET /", handleHome)
    mux.HandleFunc("GET /items", handleItemsPage)
    mux.HandleFunc("GET /items/{id}", handleItemDetail)

    // Fragments (HTMX partials)
    mux.HandleFunc("GET /fragments/items", handleItemsList)
    mux.HandleFunc("POST /items", handleCreateItem)
    mux.HandleFunc("DELETE /items/{id}", handleDeleteItem)

    http.ListenAndServe(":8080", mux)
}
```

### Path Parameters

```go
func handleItemDetail(w http.ResponseWriter, r *http.Request) {
    id := r.PathValue("id")  // Go 1.22+
    // ...
}
```

---

## Handler Patterns

### Detecting HTMX Requests

HTMX sends `HX-Request: true` header. Return fragment for HTMX, full page otherwise.

```go
func handleItems(w http.ResponseWriter, r *http.Request) {
    items := fetchItems()

    if isHTMX(r) {
        // Return just the list fragment
        renderFragment(w, "items-list", items)
    } else {
        // Return full page with layout
        renderPage(w, "items-page", items)
    }
}

func isHTMX(r *http.Request) bool {
    return r.Header.Get("HX-Request") == "true"
}
```

### Fragment Response

```go
func handleSearch(w http.ResponseWriter, r *http.Request) {
    query := r.URL.Query().Get("q")
    results := search(query)

    // Just return the results HTML, HTMX will swap it in
    templates.SearchResults(results).Render(r.Context(), w)
}
```

### POST with Redirect

```go
func handleCreateItem(w http.ResponseWriter, r *http.Request) {
    r.ParseForm()
    item := createItem(r.Form)

    if isHTMX(r) {
        // Tell HTMX to redirect
        w.Header().Set("HX-Redirect", "/items/"+item.ID)
        w.WriteHeader(http.StatusOK)
    } else {
        http.Redirect(w, r, "/items/"+item.ID, http.StatusSeeOther)
    }
}
```

---

## HTMX Attributes Reference

### Request Triggers

| Attribute | Description | Example |
|-----------|-------------|---------|
| `hx-get` | GET request to URL | `hx-get="/items"` |
| `hx-post` | POST request | `hx-post="/items"` |
| `hx-put` | PUT request | `hx-put="/items/1"` |
| `hx-patch` | PATCH request | `hx-patch="/items/1"` |
| `hx-delete` | DELETE request | `hx-delete="/items/1"` |

### Targeting

| Attribute | Description | Example |
|-----------|-------------|---------|
| `hx-target` | Where to put response | `hx-target="#results"` |
| `hx-target="this"` | Replace trigger element | `hx-target="this"` |
| `hx-target="closest tr"` | Find closest ancestor | `hx-target="closest .card"` |
| `hx-target="next .error"` | Next sibling matching | `hx-target="next .error"` |

### Swap Strategies

| Attribute | Description |
|-----------|-------------|
| `hx-swap="innerHTML"` | Replace inner content (default) |
| `hx-swap="outerHTML"` | Replace entire element |
| `hx-swap="beforeend"` | Append inside target |
| `hx-swap="afterend"` | Insert after target |
| `hx-swap="beforebegin"` | Insert before target |
| `hx-swap="delete"` | Delete target element |
| `hx-swap="none"` | Don't swap, just trigger |

### Triggers

| Attribute | Description |
|-----------|-------------|
| `hx-trigger="click"` | On click (default for buttons) |
| `hx-trigger="submit"` | On form submit |
| `hx-trigger="load"` | On element load |
| `hx-trigger="revealed"` | When scrolled into view |
| `hx-trigger="intersect"` | Intersection observer |
| `hx-trigger="every 5s"` | Polling interval |
| `hx-trigger="keyup delay:500ms"` | Debounced keyup |
| `hx-trigger="change"` | On input change |
| `hx-trigger="blur"` | On focus lost |

### Modifiers

```html
<!-- Only trigger once -->
<div hx-get="/stats" hx-trigger="load once">

<!-- Debounce input -->
<input hx-get="/search" hx-trigger="keyup changed delay:300ms">

<!-- Throttle -->
<div hx-get="/position" hx-trigger="mousemove throttle:100ms">

<!-- From another element -->
<button hx-get="/modal" hx-trigger="click from:#open-btn">
```

---

## URL and History

```html
<!-- Update URL after swap -->
<a hx-get="/page/2" hx-push-url="true">Page 2</a>

<!-- Replace URL (no history entry) -->
<div hx-get="/tab/settings" hx-replace-url="/settings">

<!-- Boost all links (SPA-like navigation) -->
<body hx-boost="true">
    <a href="/about">About</a>  <!-- Now uses HTMX -->
</body>
```

---

## Loading Indicators

```html
<!-- Show spinner during request (Pico CSS compatible) -->
<button hx-post="/submit" hx-indicator="#spinner">
    <span class="htmx-hide-on-request">Submit</span>
    <span id="spinner" class="htmx-indicator" aria-busy="true">Sending...</span>
</button>
```

CSS (add to `static/css/custom.css`):
```css
/* HTMX Loading Indicators */
.htmx-indicator { display: none; }
.htmx-request .htmx-indicator { display: inline-block; }
.htmx-request.htmx-indicator { display: inline-block; }

/* Hide content during request */
.htmx-request .htmx-hide-on-request { display: none; }
```

Pico CSS also supports `aria-busy="true"` for native loading spinners on buttons.

---

## Response Headers

Set these headers in your Go handlers to control HTMX behavior.

| Header | Purpose | Example |
|--------|---------|---------|
| `HX-Redirect` | Client-side redirect | `"/items/123"` |
| `HX-Refresh` | Full page refresh | `"true"` |
| `HX-Retarget` | Change swap target | `"#other-div"` |
| `HX-Reswap` | Change swap strategy | `"outerHTML"` |
| `HX-Trigger` | Trigger client event | `"itemCreated"` |
| `HX-Push-Url` | Update browser URL | `"/items/123"` |

```go
func handleCreate(w http.ResponseWriter, r *http.Request) {
    item := createItem(r)

    // Trigger event that other elements can listen for
    w.Header().Set("HX-Trigger", "itemCreated")

    // Return the new item fragment
    templates.ItemCard(item).Render(r.Context(), w)
}
```

### Trigger with Data

```go
// JSON in HX-Trigger for event data
w.Header().Set("HX-Trigger", `{"itemCreated": {"id": "123", "name": "New Item"}}`)
```

---

## Out-of-Band Swaps

Update multiple elements with one response.

```go
func handleToggleFavorite(w http.ResponseWriter, r *http.Request) {
    item := toggleFavorite(r.PathValue("id"))

    // Main response
    templates.ItemCard(item).Render(r.Context(), w)

    // Also update the favorites count in header (out of band)
    fmt.Fprintf(w, `<span id="fav-count" hx-swap-oob="true">%d</span>`, getFavCount())
}
```

```html
<!-- Template with OOB swap -->
<div id="item-123">Updated item content</div>
<span id="fav-count" hx-swap-oob="true">42</span>
```

> **SSE Note**: When using OOB swaps with Server-Sent Events, the event must first be captured via `sse-swap`. If no element has `sse-swap` for an event type, the event is ignored entirely - including any OOB attributes in the HTML. Use a hidden handler: `<div sse-swap="my-event" hx-swap="none" style="display:none;"></div>`

---

## Confirmation Dialogs

```html
<button
    hx-delete="/items/123"
    hx-confirm="Delete this item?"
    hx-target="closest .item"
    hx-swap="outerHTML"
>
    Delete
</button>
```

---

## Error Handling

### HTTP Status Codes

HTMX handles status codes:
- `2xx`: Swap content normally
- `4xx/5xx`: Don't swap (by default), trigger `htmx:responseError`

To swap on errors:
```html
<div hx-get="/data" hx-target="#result" hx-target-4xx="#error" hx-target-5xx="#error">
```

### Error Fragment Pattern

```go
func handleCreate(w http.ResponseWriter, r *http.Request) {
    if err := validate(r); err != nil {
        w.WriteHeader(http.StatusUnprocessableEntity)
        templates.ErrorMessage(err.Error()).Render(r.Context(), w)
        return
    }
    // ...
}
```

---

## Common Patterns

### Delete and Remove

```html
<button
    hx-delete="/items/123"
    hx-target="closest tr"
    hx-swap="outerHTML swap:500ms"
>
    Delete
</button>
```

The `swap:500ms` gives time for fade-out animation.

### Infinite Scroll

```html
<div id="items">
    <!-- existing items -->

    <div
        hx-get="/items?page=2"
        hx-trigger="revealed"
        hx-swap="outerHTML"
    >
        Loading more...
    </div>
</div>
```

### Active Search

```html
<input
    type="search"
    name="q"
    hx-get="/search"
    hx-trigger="keyup changed delay:300ms, search"
    hx-target="#results"
    hx-indicator="#search-spinner"
>
```

### Click to Edit

```html
<!-- View mode -->
<div hx-get="/items/123/edit" hx-trigger="click" hx-swap="outerHTML">
    Item Name
</div>

<!-- Edit mode (returned by /items/123/edit) -->
<form hx-put="/items/123" hx-swap="outerHTML">
    <input name="name" value="Item Name">
    <button type="submit">Save</button>
    <button hx-get="/items/123" hx-swap="outerHTML">Cancel</button>
</form>
```

### Tabs

```html
<div class="tabs">
    <button hx-get="/tab/info" hx-target="#tab-content" class="active">Info</button>
    <button hx-get="/tab/settings" hx-target="#tab-content">Settings</button>
</div>
<div id="tab-content">
    <!-- Tab content loads here -->
</div>
```

---

## HTMX Events

Listen for HTMX events in JavaScript when needed.

```javascript
document.body.addEventListener('htmx:afterSwap', function(event) {
    // Do something after content swapped
    console.log('Swapped:', event.detail.target);
});

document.body.addEventListener('htmx:responseError', function(event) {
    console.error('Request failed:', event.detail.xhr.status);
});
```

Common events:
- `htmx:beforeRequest` - Before request sent
- `htmx:afterRequest` - After request completes
- `htmx:beforeSwap` - Before DOM swap
- `htmx:afterSwap` - After DOM swap
- `htmx:responseError` - On 4xx/5xx response

---

## Including Data

### From Form

```html
<form hx-post="/items">
    <input name="title">
    <button type="submit">Create</button>
</form>
```

### From Multiple Elements

```html
<input id="search" name="q">
<select id="filter" name="category">
<button hx-get="/items" hx-include="#search, #filter">Search</button>
```

### With hx-vals

```html
<button hx-post="/vote" hx-vals='{"itemId": "123", "direction": "up"}'>
    Upvote
</button>
```

Dynamic values:
```html
<button hx-post="/vote" hx-vals="js:{timestamp: Date.now()}">
    Vote
</button>
```

---

## Hyperscript Integration

For client-side behavior that HTMX can't handle alone (modals, scrolling, toggles), use hyperscript - HTMX's companion library.

Add after HTMX in your base layout:
```html
<script src="https://unpkg.com/htmx.org@2.0.4"></script>
<script src="https://unpkg.com/hyperscript.org@0.9.13"></script>
```

Common patterns:

```html
<!-- Open native dialog modal -->
<button hx-get="/modal" hx-target="#modal-content" hx-swap="innerHTML">Open</button>
<dialog id="my-dialog" _="on openModal call me.showModal()">
    <article id="modal-content" _="on htmx:afterSwap send openModal to #my-dialog"></article>
</dialog>

<!-- Auto-scroll container on new content -->
<div id="stream"
     sse-swap="message"
     hx-swap="beforeend"
     _="on htmx:afterSwap scroll me to bottom">
</div>

<!-- Toggle class -->
<button _="on click toggle .active on #menu">Menu</button>
```

See **go-hyperscript-patterns** for complete hyperscript documentation.

---

## Integration

This skill works with:
- **go-project-bootstrap**: Initial project setup
- **go-templ-components**: Template rendering
- **go-htmx-sse**: Real-time updates
- **go-htmx-forms**: Form validation
- **go-hyperscript-patterns**: Client-side behavior (modals, scrolling, toggles)
- **go-pico-embed**: Asset embedding

Reference this skill when building any Go + HTMX handler.
