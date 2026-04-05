---
name: go-hyperscript-patterns
description: Hyperscript patterns for enhancing HTMX interactions. Declarative scripting for modals, scrolling, toggles, and event handling without writing JavaScript.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Hyperscript Patterns for Go + HTMX

Hyperscript is HTMX's companion library for small UI interactions that need scripting. It provides a readable, English-like syntax for DOM manipulation and event handling.

## When to Use Hyperscript

Use hyperscript when you need behavior that HTMX can't handle alone:

- Opening native `<dialog>` elements (requires `showModal()`)
- Auto-scrolling containers on content updates
- Toggle states and visibility
- Custom event messaging between components
- Conditional logic based on DOM state
- Animations and transitions

**Philosophy:** HTMX handles server communication, hyperscript handles client-side behavior.

## Setup

Add hyperscript after HTMX in your base layout:

```html
<!-- HTMX + hyperscript -->
<script src="https://unpkg.com/htmx.org@2.0.4"></script>
<script src="https://unpkg.com/htmx-ext-sse@2.2.2/sse.js"></script>
<script src="https://unpkg.com/hyperscript.org@0.9.13"></script>
```

In Templ:
```templ
templ Base(title string) {
    <!DOCTYPE html>
    <html>
        <head>
            <script src="https://unpkg.com/htmx.org@2.0.4"></script>
            <script src="https://unpkg.com/hyperscript.org@0.9.13"></script>
        </head>
        <body>
            { children... }
        </body>
    </html>
}
```

## Fundamentals

### The `_` Attribute

Hyperscript uses the `_` attribute (underscore) for inline scripting:

```html
<button _="on click toggle .active">Toggle</button>
```

### Core Keywords

| Keyword | Purpose | Example |
|---------|---------|---------|
| `on` | Listen for events | `on click` |
| `me` | Reference current element | `add .active to me` |
| `it` | Reference target of event | `remove it` |
| `send` | Dispatch custom event | `send openModal to #dialog` |
| `call` | Call a method | `call me.showModal()` |
| `set` | Set property/attribute | `set my.innerHTML to ''` |
| `add` | Add class | `add .visible` |
| `remove` | Remove class/element | `remove .hidden` |
| `toggle` | Toggle class | `toggle .active` |
| `scroll` | Scroll element | `scroll me to bottom` |
| `then` | Chain commands | `on load ... then on click ...` |
| `wait` | Delay execution | `wait 500ms` |
| `if` | Conditional | `if I match .active` |

### Selectors

```html
<!-- Target by ID -->
_="on click add .active to #my-element"

<!-- Target by class -->
_="on click remove .hidden from .all-items"

<!-- Target closest ancestor -->
_="on click remove closest .card"

<!-- Target next sibling -->
_="on click toggle .visible on next .dropdown"
```

## Event Handling

### HTMX Events

Hyperscript can listen to HTMX lifecycle events:

```html
<!-- After HTMX swaps content -->
_="on htmx:afterSwap scroll me to bottom"

<!-- After HTMX settles (animations done) -->
_="on htmx:afterSettle add .loaded"

<!-- Before HTMX sends request -->
_="on htmx:beforeRequest add .loading"

<!-- After HTMX request completes -->
_="on htmx:afterRequest remove .loading"
```

### Custom Events (Component Messaging)

Components can communicate via custom events, keeping them decoupled:

```html
<!-- Sender: dispatch event to specific target -->
<article id="modal-content"
         _="on htmx:afterSwap send openModal to #my-dialog">
</article>

<!-- Receiver: listen and act -->
<dialog id="my-dialog" _="on openModal call me.showModal()">
    ...
</dialog>
```

### Multiple Event Handlers

Chain handlers with `then`:

```html
<!-- On load AND on updates -->
_="on load scroll me to bottom then on htmx:afterSwap scroll me to bottom"

<!-- Multiple actions on same event -->
_="on click add .active to me then remove .active from .siblings"
```

### Event Sources

Listen to events from specific elements:

```html
<!-- Only when event comes from #source -->
_="on htmx:afterSwap from #modal-content call me.showModal()"

<!-- From any element matching selector -->
_="on customEvent from .buttons log 'button clicked'"
```

## Common Patterns

### Native Dialog Modal

The cleanest HTMX modal pattern using native `<dialog>`:

```templ
// Button to open modal
<button
    hx-get="/modal-content"
    hx-target="#modal-content"
    hx-swap="innerHTML"
>
    Open Modal
</button>

// Dialog with hyperscript
<dialog id="my-dialog" _="on openModal call me.showModal()">
    <article id="modal-content" _="on htmx:afterSwap send openModal to #my-dialog">
        // Content loaded here
    </article>
</dialog>
```

Modal content with close button (pure HTML, no JS):
```templ
templ ModalContent(title string) {
    <header>
        <h3>{ title }</h3>
        <form method="dialog">
            <button type="submit" class="outline">&times;</button>
        </form>
    </header>
    <div>
        // Modal body
    </div>
    <footer>
        <form method="dialog">
            <button type="submit" class="outline">Cancel</button>
        </form>
        <button type="submit" class="primary">Save</button>
    </footer>
}
```

**Key insight:** `<form method="dialog">` closes the dialog without any JavaScript.

### Auto-Scroll Container

For chat, logs, or streaming content:

```html
<!-- Scroll to bottom on load AND when new content arrives -->
<div id="stream"
     class="message-stream"
     style="max-height: 500px; overflow-y: auto;"
     _="on load scroll me to bottom then on htmx:afterSwap scroll me to bottom">
    // Content here
</div>
```

For SSE streaming (no initial content):
```html
<div id="stream"
     sse-swap="message"
     hx-swap="beforeend"
     _="on htmx:afterSwap scroll me to bottom">
</div>
```

### SSE with Pre-populated History

Load existing content from database, then append new SSE events:

```templ
templ StreamWithHistory(runID string, history []Message) {
    <div hx-ext="sse" sse-connect={ "/events?id=" + runID }>
        <div
            id="agent-stream"
            class="message-stream"
            sse-swap="message,tool_use"
            hx-swap="beforeend"
            _="on load scroll me to bottom then on htmx:afterSwap scroll me to bottom"
        >
            // Pre-render existing history
            for _, msg := range history {
                @MessageCard(msg)
            }
        </div>
        // Hidden handler for OOB-only events
        <div sse-swap="tool_result,status" hx-swap="none" style="display:none;"></div>
    </div>
}
```

**Key insight:** SSE `beforeend` appends to existing content. History renders first, SSE adds new items.

### Toggle Visibility

```html
<!-- Simple toggle -->
<button _="on click toggle .hidden on #details">
    Show/Hide Details
</button>
<div id="details" class="hidden">...</div>

<!-- Toggle with aria -->
<button _="on click toggle @aria-expanded on me then toggle .visible on next .dropdown">
    Menu
</button>
```

### Loading States

```html
<!-- Add loading class during HTMX request -->
<form hx-post="/submit"
      _="on htmx:beforeRequest add .loading to me
         then on htmx:afterRequest remove .loading from me">
    <button type="submit">Submit</button>
</form>
```

### Tabs

```html
<div class="tabs">
    <button class="active"
            hx-get="/tab/1"
            hx-target="#content"
            _="on click remove .active from .tabs button then add .active to me">
        Tab 1
    </button>
    <button hx-get="/tab/2"
            hx-target="#content"
            _="on click remove .active from .tabs button then add .active to me">
        Tab 2
    </button>
</div>
<div id="content">...</div>
```

### Confirm and Proceed

```html
<button _="on click
           if window.confirm('Are you sure?')
             trigger confirmed on me">
    Delete
</button>
```

## DOM Manipulation

### Classes

```html
_="add .active"                    // Add to me
_="add .active to #target"         // Add to specific element
_="remove .hidden from .items"     // Remove from multiple
_="toggle .visible"                // Toggle on me
_="toggle .open on next .dropdown" // Toggle on sibling
```

### Attributes

```html
_="set @disabled to true"
_="remove @disabled"
_="toggle @aria-expanded"
```

### Content

```html
_="set my.innerHTML to ''"         // Clear content
_="set my.textContent to 'Done'"   // Set text
_="put 'Loading...' into me"       // Alternative syntax
```

### Scrolling

```html
_="scroll me to top"
_="scroll me to bottom"
_="scroll #container to top"
_="scroll to top of #element"      // Scroll element into view
```

## Timing and Delays

```html
// Wait before action
_="on click wait 500ms then add .fade-out"

// Debounce
_="on keyup debounced at 300ms send search"

// Throttle
_="on scroll throttled at 100ms call checkPosition()"
```

## Gotchas and Best Practices

### 1. Event Bubbling

HTMX events bubble up. Be specific with `from` if needed:

```html
// May catch events from children
_="on htmx:afterSwap ..."

// Only from specific source
_="on htmx:afterSwap from #my-target ..."
```

### 2. Timing with HTMX

Use `htmx:afterSettle` instead of `htmx:afterSwap` if you need animations to complete:

```html
_="on htmx:afterSettle add .visible"
```

### 3. Element References

`me` refers to the element with the `_` attribute, not the event target:

```html
// me = the div, even if child button clicked
<div _="on click add .active to me">
    <button>Click</button>
</div>
```

### 4. Method Calls

Use `call` for DOM methods:

```html
_="call me.focus()"
_="call me.showModal()"
_="call me.closest('form').reset()"
```

### 5. Keep It Simple

If hyperscript gets complex, consider:
- Moving logic to server (HTMX philosophy)
- Using a dedicated handler in Go
- Breaking into smaller components

## Integration

Related skills:
- **go-htmx-core**: Base HTMX patterns and attributes
- **go-htmx-sse**: SSE streaming with hyperscript scroll
- **go-htmx-forms**: Form validation with HTMX
- **go-htmx-dashboard**: Dashboard patterns with modals and streaming
- **go-templ-components**: Templ template patterns
