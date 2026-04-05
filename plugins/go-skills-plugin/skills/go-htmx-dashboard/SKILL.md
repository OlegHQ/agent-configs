---
name: go-htmx-dashboard
description: Dashboard and admin UI patterns with Go + HTMX + clawde. Kanban boards, data tables, real-time agent control, and status indicators.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go + HTMX Dashboard Patterns

Build admin dashboards and control panels with Go, HTMX, and clawde. Real-time updates, data visualization, and agent streaming.

## When to Use

- Building admin panels and dashboards
- Agent/AI control interfaces
- Real-time monitoring systems
- Data management UIs

---

## Dashboard Layout

### Base Layout

```templ
templ DashboardLayout(title string, activeNav string) {
    <!DOCTYPE html>
    <html lang="en" data-theme="light">
    <head>
        <meta charset="UTF-8"/>
        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <title>{ title } | Dashboard</title>
        <!-- Pico CSS from CDN -->
        <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css"/>
        <link rel="stylesheet" href="/static/css/dashboard.css"/>
    </head>
    <body>
        <div class="dashboard-layout">
            @Sidebar(activeNav)
            <main class="container">
                @TopBar(title)
                { children... }
            </main>
        </div>
        <!-- HTMX, SSE extension, and Hyperscript from CDN -->
        <script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js"></script>
        <script src="https://cdn.jsdelivr.net/npm/htmx-ext-sse@2.2.2/sse.js"></script>
        <script src="https://unpkg.com/hyperscript.org@0.9.13"></script>
    </body>
    </html>
}
```

**Dashboard CSS** (static/css/dashboard.css):
```css
/* Dashboard layout */
.dashboard-layout {
    display: grid;
    grid-template-columns: 16rem 1fr;
    min-height: 100vh;
}

/* Sidebar styling */
.dashboard-sidebar {
    background: var(--pico-contrast-background);
    color: var(--pico-contrast);
    padding: 1rem;
}

.dashboard-sidebar nav ul {
    list-style: none;
    padding: 0;
}

.dashboard-sidebar nav a {
    display: block;
    padding: 0.75rem 1rem;
    color: inherit;
    text-decoration: none;
    border-radius: var(--pico-border-radius);
}

.dashboard-sidebar nav a:hover,
.dashboard-sidebar nav a.active {
    background: rgba(255,255,255,0.1);
}

/* Grid utilities */
.grid-4 { grid-template-columns: repeat(4, 1fr); }
.grid-3 { grid-template-columns: repeat(3, 1fr); }
.grid-2 { grid-template-columns: repeat(2, 1fr); }
```

### Sidebar Navigation

```templ
type NavItem struct {
    Name   string
    Href   string
    Icon   string
}

var navItems = []NavItem{
    {Name: "Dashboard", Href: "/", Icon: "home"},
    {Name: "Pipeline", Href: "/pipeline", Icon: "kanban"},
    {Name: "Control", Href: "/control", Icon: "terminal"},
    {Name: "Runs", Href: "/runs", Icon: "history"},
    {Name: "Settings", Href: "/settings", Icon: "settings"},
}

templ Sidebar(activeNav string) {
    <aside class="dashboard-sidebar">
        <hgroup>
            <h1>Agent Dashboard</h1>
        </hgroup>
        <nav>
            <ul>
                for _, item := range navItems {
                    <li>
                        <a
                            href={ templ.SafeURL(item.Href) }
                            hx-boost="true"
                            class={ templ.KV("active", activeNav == item.Name) }
                        >
                            { item.Name }
                        </a>
                    </li>
                }
            </ul>
        </nav>
    </aside>
}
```

---

## Kanban Board

### Board Container

```templ
type Stage struct {
    ID    string
    Name  string
    Items []Item
}

templ KanbanBoard(stages []Stage) {
    <div class="flex gap-4 overflow-x-auto pb-4">
        for _, stage := range stages {
            @StageColumn(stage)
        }
    </div>
}

templ StageColumn(stage Stage) {
    <div class="w-80 flex-shrink-0 bg-gray-100 rounded-lg">
        <div class="p-3 font-medium text-gray-700 border-b border-gray-200 flex justify-between">
            <span>{ stage.Name }</span>
            <span class="bg-gray-200 px-2 py-0.5 rounded-full text-sm">
                { fmt.Sprint(len(stage.Items)) }
            </span>
        </div>
        <div
            id={ "stage-" + stage.ID }
            class="p-2 space-y-2 min-h-[200px]"
            hx-get={ "/api/stage/" + stage.ID }
            hx-trigger="every 30s"
            hx-swap="innerHTML"
        >
            for _, item := range stage.Items {
                @KanbanCard(item)
            }
            if len(stage.Items) == 0 {
                <div class="text-center py-8 text-gray-400 text-sm">
                    No items
                </div>
            }
        </div>
    </div>
}
```

### Kanban Card

```templ
type Item struct {
    ID          string
    Title       string
    Description string
    Status      string
    Priority    string
    CreatedAt   time.Time
}

templ KanbanCard(item Item) {
    <div
        id={ "item-" + item.ID }
        class="bg-white rounded-lg shadow-sm p-3 cursor-pointer hover:shadow-md transition-shadow"
        hx-get={ "/items/" + item.ID }
        hx-target="#detail-panel"
        hx-swap="innerHTML"
    >
        <div class="flex items-start justify-between">
            <h4 class="font-medium text-gray-900 text-sm">{ item.Title }</h4>
            @PriorityBadge(item.Priority)
        </div>
        <p class="text-gray-500 text-xs mt-1 line-clamp-2">{ item.Description }</p>
        <div class="flex items-center justify-between mt-3">
            @StatusBadge(item.Status)
            <span class="text-xs text-gray-400">
                { item.CreatedAt.Format("Jan 2") }
            </span>
        </div>
    </div>
}
```

### Status Badges

```templ
// Use Pico's semantic elements for status colors
templ StatusBadge(status string) {
    switch status {
        case "success", "merged", "completed":
            <ins>{ status }</ins>
        case "running", "in_progress":
            <span aria-busy="true">{ status }</span>
        case "failed", "error":
            <del>{ status }</del>
        case "pending", "waiting":
            <mark>{ status }</mark>
        default:
            <span>{ status }</span>
    }
}

templ PriorityBadge(priority string) {
    switch priority {
        case "high":
            <del title="High priority">!</del>
        case "medium":
            <mark title="Medium priority">-</mark>
        default:
            <span title="Low priority">o</span>
    }
}
```

---

## Data Tables

### Sortable Table

```templ
type Column struct {
    Key      string
    Label    string
    Sortable bool
}

// Pico CSS styles tables semantically
templ DataTable(columns []Column, rows []map[string]any, sortBy string, sortDir string) {
    <figure>
        <table>
            <thead>
                <tr>
                    for _, col := range columns {
                        <th scope="col">
                            if col.Sortable {
                                <a
                                    href="#"
                                    hx-get={ fmt.Sprintf("/api/table?sort=%s&dir=%s", col.Key, toggleDir(sortDir)) }
                                    hx-target="closest figure"
                                    hx-swap="outerHTML"
                                >
                                    { col.Label }
                                    if sortBy == col.Key {
                                        if sortDir == "asc" {
                                            ↑
                                        } else {
                                            ↓
                                        }
                                    }
                                </a>
                            } else {
                                { col.Label }
                            }
                        </th>
                    }
                </tr>
            </thead>
            <tbody>
                for _, row := range rows {
                    <tr>
                        for _, col := range columns {
                            <td>{ fmt.Sprint(row[col.Key]) }</td>
                        }
                    </tr>
                }
            </tbody>
        </table>
    </figure>
}

func toggleDir(dir string) string {
    if dir == "asc" {
        return "desc"
    }
    return "asc"
}
```

### With Search and Filter

```templ
templ TableWithControls(data TableData) {
    <article>
        <header>
            <div class="grid">
                <input
                    type="search"
                    name="q"
                    placeholder="Search..."
                    value={ data.Query }
                    hx-get="/api/table"
                    hx-trigger="keyup changed delay:300ms"
                    hx-target="#table-container"
                    hx-include="[name='status']"
                />
                <select
                    name="status"
                    hx-get="/api/table"
                    hx-trigger="change"
                    hx-target="#table-container"
                    hx-include="[name='q']"
                >
                    <option value="">All Status</option>
                    <option value="active" selected?={ data.Status == "active" }>Active</option>
                    <option value="pending" selected?={ data.Status == "pending" }>Pending</option>
                    <option value="completed" selected?={ data.Status == "completed" }>Completed</option>
                </select>
            </div>
        </header>
        <div id="table-container">
            @DataTable(data.Columns, data.Rows, data.SortBy, data.SortDir)
        </div>
    </article>
}
```

---

## Modal Dialogs

Use native HTML `<dialog>` element with hyperscript for modal interactions. No JavaScript required.

### Basic Modal Pattern

```templ
templ PageWithModal() {
    // Button triggers HTMX load into modal content
    <button
        hx-get="/modal-content"
        hx-target="#modal-content"
        hx-swap="innerHTML"
    >
        Open Modal
    </button>

    // Native dialog with hyperscript event listener
    <dialog id="my-dialog" _="on openModal call me.showModal()">
        <article
            id="modal-content"
            _="on htmx:afterSwap send openModal to #my-dialog"
        >
            // Content loads here, then triggers dialog open
        </article>
    </dialog>
}
```

**Key insight:** The modal content element sends a custom event to the dialog after HTMX swaps in new content. The dialog listens for this event and calls `showModal()`.

### Modal Content with Close Button

```templ
templ ModalContent(title string) {
    <header>
        <h3>{ title }</h3>
        // Pure HTML dialog close - no JavaScript needed
        <form method="dialog">
            <button type="submit" class="outline">&times;</button>
        </form>
    </header>
    <div>
        // Modal body content
    </div>
    <footer class="grid">
        <form method="dialog">
            <button type="submit" class="outline">Cancel</button>
        </form>
        <button type="submit" class="primary">Save</button>
    </footer>
}
```

**Key insight:** `<form method="dialog">` is native HTML for closing dialogs without any JavaScript.

### Form Modal with Redirect

For forms that should close modal and navigate after submit:

```templ
templ NewItemModal() {
    <header>
        <h3>New Item</h3>
        <form method="dialog"><button type="submit" class="outline">&times;</button></form>
    </header>
    <form hx-post="/api/items" hx-swap="none">
        <label>
            Name
            <input type="text" name="name" required autofocus/>
        </label>
        <footer class="grid">
            <form method="dialog"><button type="submit" class="outline">Cancel</button></form>
            <button type="submit">Create</button>
        </footer>
    </form>
}
```

Server handler returns redirect header:

```go
func handleCreateItem(w http.ResponseWriter, r *http.Request) {
    item := createItem(r)

    // Tell HTMX to redirect after successful creation
    w.Header().Set("HX-Redirect", "/items/"+item.ID)
    w.WriteHeader(http.StatusOK)
}
```

See **go-hyperscript-patterns** for more modal and event patterns.

---

## Agent Control Panel

### Command Interface

```templ
type Command struct {
    Name        string
    Description string
    HasArgs     bool
}

var commands = []Command{
    {Name: "/status", Description: "Show current state"},
    {Name: "/daily-run", Description: "Execute daily workflow"},
    {Name: "/scout", Description: "Find new opportunities"},
    {Name: "/contribute", Description: "Execute contribution", HasArgs: true},
}

templ AgentControlPanel() {
    <div class="grid grid-cols-3 gap-6">
        <div class="col-span-1">
            @CommandSelector()
        </div>
        <div class="col-span-2">
            @OutputPanel()
        </div>
    </div>
}

templ CommandSelector() {
    <div class="bg-white rounded-lg shadow p-4">
        <h3 class="font-medium text-gray-900 mb-4">Run Command</h3>
        <form hx-post="/api/agent/start" hx-target="#output" hx-swap="innerHTML">
            <div class="space-y-2">
                for _, cmd := range commands {
                    <label class="flex items-start p-3 border rounded-lg cursor-pointer hover:bg-gray-50">
                        <input type="radio" name="command" value={ cmd.Name } class="mt-1"/>
                        <div class="ml-3">
                            <span class="font-mono text-sm">{ cmd.Name }</span>
                            <p class="text-xs text-gray-500">{ cmd.Description }</p>
                        </div>
                    </label>
                }
            </div>

            <div id="args-container" class="mt-4 hidden">
                <label class="block text-sm font-medium text-gray-700 mb-1">Arguments</label>
                <input type="text" name="args" class="input w-full" placeholder="e.g., issue URL"/>
            </div>

            <button type="submit" class="btn btn-primary w-full mt-4">
                <span class="htmx-indicator">
                    @Spinner()
                </span>
                <span class="htmx-hide-on-request">Run Command</span>
            </button>
        </form>
    </div>
}

templ OutputPanel() {
    <div class="bg-gray-900 rounded-lg shadow h-[600px] flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 border-b border-gray-700">
            <span class="text-gray-400 text-sm">Output</span>
            <button
                hx-post="/api/agent/cancel"
                class="text-red-400 hover:text-red-300 text-sm"
            >
                Cancel
            </button>
        </div>
        <div
            id="output"
            class="flex-1 overflow-y-auto p-4 font-mono text-sm text-gray-300"
        >
            <span class="text-gray-500">Ready. Select a command to run.</span>
        </div>
    </div>
}
```

---

## Clawde Agent Streaming

### Stream Handler

```go
func handleAgentStream(w http.ResponseWriter, r *http.Request) {
    sse, err := NewSSEWriter(w)
    if err != nil {
        http.Error(w, err.Error(), 500)
        return
    }

    command := r.FormValue("command")
    args := r.FormValue("args")

    prompt := command
    if args != "" {
        prompt += " " + args
    }

    client := clawde.NewClient(
        clawde.WithModel("claude-sonnet-4-20250514"),
        clawde.WithSystemPrompt(loadSystemPrompt()),
        clawde.WithMaxTurns(50),
    )

    ctx := r.Context()
    stream, err := client.Query(ctx, prompt)
    if err != nil {
        sse.SendHTML("error", renderError(err.Error()))
        return
    }

    for stream.Next() {
        msg := stream.Current()

        switch m := msg.(type) {
        case *clawde.AssistantMessage:
            for _, block := range m.Content {
                switch b := block.(type) {
                case *clawde.TextBlock:
                    sse.SendHTML("message", renderTextChunk(b.Text))
                case *clawde.ThinkingBlock:
                    sse.SendHTML("thinking", renderThinking(b.Text))
                }
            }

        case *clawde.ToolUseMessage:
            sse.SendHTML("tool", renderToolUse(m.Name, m.Input))

        case *clawde.ToolResultMessage:
            sse.SendHTML("result", renderToolResult(m.Name, m.Result))

        case *clawde.ResultMessage:
            sse.SendHTML("done", renderComplete(m))
        }
    }

    if err := stream.Err(); err != nil {
        sse.SendHTML("error", renderError(err.Error()))
    }
}
```

### Streaming Output Component

```templ
templ StreamingOutput(runID string) {
    <!-- SSE Connection Wrapper -->
    <div hx-ext="sse" sse-connect={ "/api/agent/stream?run_id=" + runID }>
        <!-- Stream events that append content -->
        <div
            id="messages"
            class="space-y-2"
            style="max-height: 600px; overflow-y: auto;"
            sse-swap="message,thinking,tool"
            hx-swap="beforeend"
            _="on htmx:afterSwap scroll me to bottom"
        >
            <!-- Text and tool cards appear here -->
        </div>

        <!-- Hidden handler for OOB-only events (REQUIRED for OOB to work!) -->
        <div
            sse-swap="tool_result,done,error"
            hx-swap="none"
            style="display:none;"
        ></div>
    </div>

    <!-- OOB targets (updated via hx-swap-oob in event HTML) -->
    <div id="status"><!-- Updated by done event --></div>
}

### Streaming with Pre-populated History

For viewing past runs with history + live updates:

```templ
templ RunDetailLive(run Run, messages []Message) {
    <div hx-ext="sse" sse-connect={ "/api/agent/stream?run_id=" + run.ID }>
        <div
            id="messages"
            class="space-y-2"
            style="max-height: 600px; overflow-y: auto;"
            sse-swap="message,tool"
            hx-swap="beforeend"
            _="on load scroll me to bottom then on htmx:afterSwap scroll me to bottom"
        >
            // Pre-render existing history from database
            for _, msg := range messages {
                @MessageCard(msg)
            }
            // SSE events append after existing content
        </div>

        <div sse-swap="tool_result,done" hx-swap="none" style="display:none;"></div>
    </div>
}
```

**Key pattern:** Use `on load ... then on htmx:afterSwap` to scroll on initial load AND new messages.

```templ
templ TextChunk(text string) {
    <span class="text-gray-300">{ text }</span>
}

templ ToolUseCard(name string, input string) {
    <div class="bg-gray-800 rounded p-3 my-2 border-l-4 border-blue-500">
        <div class="flex items-center gap-2 text-blue-400 text-sm">
            @Icon("terminal", "w-4 h-4")
            <span class="font-mono">{ name }</span>
            <span class="animate-pulse">Running...</span>
        </div>
        if input != "" {
            <pre class="mt-2 text-xs text-gray-500 overflow-x-auto">{ input }</pre>
        }
    </div>
}

templ ToolResultCard(name string, result string, success bool) {
    <div class={
        "bg-gray-800 rounded p-3 my-2 border-l-4",
        templ.KV("border-green-500", success),
        templ.KV("border-red-500", !success),
    }>
        <div class="flex items-center gap-2 text-sm">
            if success {
                @Icon("check-circle", "w-4 h-4 text-green-400")
            } else {
                @Icon("x-circle", "w-4 h-4 text-red-400")
            }
            <span class="font-mono text-gray-400">{ name }</span>
        </div>
        <pre class="mt-2 text-xs text-gray-500 overflow-x-auto max-h-40">{ result }</pre>
    </div>
}
```

---

## Real-time Status Dashboard

### Stats Cards

```templ
templ StatsGrid() {
    <div
        class="grid"
        hx-get="/api/stats"
        hx-trigger="load, every 30s"
        hx-swap="innerHTML"
    >
        @StatCard("Total Runs", "128", "up", "+12%")
        @StatCard("Success Rate", "94%", "up", "+2%")
        @StatCard("Active Agents", "3", "neutral", "")
        @StatCard("Queue Size", "7", "down", "-3")
    </div>
}

// Use Pico's article for card-like stat display
templ StatCard(label string, value string, trend string, change string) {
    <article>
        <header>
            <small>{ label }</small>
        </header>
        <hgroup>
            <h2>{ value }</h2>
            if change != "" {
                switch trend {
                    case "up":
                        <p><ins>↑ { change }</ins></p>
                    case "down":
                        <p><del>↓ { change }</del></p>
                    default:
                        <p>{ change }</p>
                }
            }
        </hgroup>
    </article>
}
```

### Activity Feed

```templ
templ ActivityFeed() {
    <div class="bg-white rounded-lg shadow">
        <div class="px-4 py-3 border-b border-gray-200">
            <h3 class="font-medium text-gray-900">Recent Activity</h3>
        </div>
        <!-- SSE Connection Wrapper -->
        <div hx-ext="sse" sse-connect="/api/activity/stream">
            <!-- Activity stream with sse-swap on child element -->
            <div
                sse-swap="activity"
                hx-swap="afterbegin"
                class="divide-y divide-gray-100 max-h-96 overflow-y-auto"
            >
                <!-- Activity items appear here -->
            </div>
        </div>
    </div>
}

templ ActivityItem(activity Activity) {
    <div class="px-4 py-3 flex items-start gap-3">
        <div class={ "w-8 h-8 rounded-full flex items-center justify-center", activityColor(activity.Type) }>
            @Icon(activityIcon(activity.Type), "w-4 h-4")
        </div>
        <div class="flex-1 min-w-0">
            <p class="text-sm text-gray-900">{ activity.Message }</p>
            <p class="text-xs text-gray-500 mt-1">{ activity.Time.Format("3:04 PM") }</p>
        </div>
    </div>
}
```

---

## Detail Panel

### Lazy-loaded Sections

```templ
templ ItemDetailPage(item Item) {
    @DashboardLayout(item.Title, "Pipeline") {
        <div class="grid grid-cols-3 gap-6">
            <div class="col-span-2 space-y-6">
                @ItemHeader(item)
                @WorkflowSection(item.ID)
                @FilesSection(item.ID)
            </div>
            <div class="space-y-6">
                @MetadataCard(item)
                @ActionsCard(item)
            </div>
        </div>
    }
}

templ WorkflowSection(itemID string) {
    <div class="bg-white rounded-lg shadow">
        <div class="px-4 py-3 border-b border-gray-200">
            <h3 class="font-medium">Workflow Steps</h3>
        </div>
        <div
            hx-get={ "/api/items/" + itemID + "/workflow" }
            hx-trigger="revealed"
            hx-swap="innerHTML"
            class="p-4"
        >
            @LoadingPlaceholder()
        </div>
    </div>
}

templ FilesSection(itemID string) {
    <div class="bg-white rounded-lg shadow">
        <div class="px-4 py-3 border-b border-gray-200">
            <h3 class="font-medium">Changed Files</h3>
        </div>
        <div
            hx-get={ "/api/items/" + itemID + "/files" }
            hx-trigger="revealed"
            hx-swap="innerHTML"
            class="p-4"
        >
            @LoadingPlaceholder()
        </div>
    </div>
}

templ LoadingPlaceholder() {
    <div class="animate-pulse space-y-3">
        <div class="h-4 bg-gray-200 rounded w-3/4"></div>
        <div class="h-4 bg-gray-200 rounded w-1/2"></div>
        <div class="h-4 bg-gray-200 rounded w-2/3"></div>
    </div>
}
```

---

## Notification Toast

### Toast Container

```templ
templ ToastContainer() {
    <!-- SSE Connection Wrapper -->
    <div hx-ext="sse" sse-connect="/api/notifications">
        <!-- Toast container with sse-swap on child element -->
        <div
            id="toast-container"
            class="fixed bottom-4 right-4 space-y-2 z-50"
            sse-swap="toast"
            hx-swap="beforeend"
        >
            <!-- Toasts appear here -->
        </div>
    </div>
}

templ Toast(message string, variant string) {
    <div
        class={
            "p-4 rounded-lg shadow-lg max-w-sm transform transition-all duration-300",
            templ.KV("bg-green-500 text-white", variant == "success"),
            templ.KV("bg-red-500 text-white", variant == "error"),
            templ.KV("bg-blue-500 text-white", variant == "info"),
        }
        x-data="{ show: true }"
        x-show="show"
        x-init="setTimeout(() => show = false, 5000)"
        x-transition
    >
        <div class="flex items-center gap-3">
            @toastIcon(variant)
            <p class="text-sm">{ message }</p>
            <button @click="show = false" class="ml-auto">
                @Icon("x", "w-4 h-4")
            </button>
        </div>
    </div>
}
```

---

## Progress Indicators

### Step Progress

```templ
type Step struct {
    Name   string
    Status string // pending, running, completed, failed
}

templ StepProgress(steps []Step, currentStep int) {
    <div class="space-y-3">
        for i, step := range steps {
            <div class="flex items-center gap-3">
                <div class={
                    "w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium",
                    templ.KV("bg-green-500 text-white", step.Status == "completed"),
                    templ.KV("bg-blue-500 text-white animate-pulse", step.Status == "running"),
                    templ.KV("bg-red-500 text-white", step.Status == "failed"),
                    templ.KV("bg-gray-200 text-gray-500", step.Status == "pending"),
                }>
                    if step.Status == "completed" {
                        @Icon("check", "w-4 h-4")
                    } else if step.Status == "running" {
                        @Spinner()
                    } else if step.Status == "failed" {
                        @Icon("x", "w-4 h-4")
                    } else {
                        { fmt.Sprint(i + 1) }
                    }
                </div>
                <span class={
                    "text-sm",
                    templ.KV("text-gray-900 font-medium", step.Status == "running"),
                    templ.KV("text-gray-500", step.Status == "pending"),
                    templ.KV("text-green-600", step.Status == "completed"),
                    templ.KV("text-red-600", step.Status == "failed"),
                }>
                    { step.Name }
                </span>
            </div>
        }
    </div>
}
```

---

## Icon Component

```templ
templ Icon(name string, class string) {
    <svg class={ class }>
        <use href={ "/static/icons.svg#" + name }></use>
    </svg>
}
```

Using sprite sheet for icons. Create with:
```bash
npx @pqina/svg-sprite-generator --input ./icons/*.svg --output ./static/icons.svg
```

---

## Integration

This skill works with:
- **go-project-bootstrap**: Initial project setup
- **go-htmx-core**: Handler patterns
- **go-templ-components**: Component building
- **go-htmx-sse**: Real-time streaming
- **go-htmx-forms**: Form components
- **go-hyperscript-patterns**: Client-side behavior (modals, scroll, toggles)
- **go-pico-embed**: Asset embedding and deployment

Reference this skill when building admin UIs and agent control panels.
