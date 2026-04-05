---
name: react-to-htmx-patterns
description: Translation guide from React/Next.js patterns to Go + Templ + HTMX. Component-by-component migration reference.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# React to HTMX Pattern Translation

Migrate React applications to Go + Templ + HTMX. This guide maps React patterns to their server-rendered equivalents.

## When to Use

- Migrating React/Next.js app to Go + HTMX
- Learning HTMX coming from React background
- Deciding if HTMX fits your use case
- Understanding architectural differences

---

## Architecture Shift

### React Model

```
User Action → JS Event → State Update → Virtual DOM Diff → DOM Update
              Client handles everything
```

### HTMX Model

```
User Action → HTTP Request → Server Logic → HTML Response → DOM Swap
              Server handles logic, client just swaps HTML
```

### Key Differences

| Aspect | React | HTMX |
|--------|-------|------|
| State location | Client (useState, Redux) | Server (DB, session) |
| Rendering | Client-side (hydration) | Server-side (HTML) |
| Data format | JSON API → JSX | HTML fragments |
| Bundle size | 100KB+ (React + deps) | 14KB (HTMX only) |
| SEO | Needs SSR setup | Built-in (HTML first) |
| Complexity | High (build, hydration) | Low (just HTML) |

---

## State Management

### React: useState

```tsx
function Counter() {
    const [count, setCount] = useState(0)
    return (
        <div>
            <span>{count}</span>
            <button onClick={() => setCount(count + 1)}>+</button>
        </div>
    )
}
```

### HTMX: Server State

```templ
templ Counter(count int) {
    <div>
        <span>{ fmt.Sprint(count) }</span>
        <button
            hx-post="/counter/increment"
            hx-target="closest div"
            hx-swap="outerHTML"
        >
            +
        </button>
    </div>
}
```

```go
var count int // or from DB/session

func handleIncrement(w http.ResponseWriter, r *http.Request) {
    count++
    Counter(count).Render(r.Context(), w)
}
```

---

## Data Fetching

### React: useEffect + fetch

```tsx
function UserList() {
    const [users, setUsers] = useState([])
    const [loading, setLoading] = useState(true)

    useEffect(() => {
        fetch('/api/users')
            .then(res => res.json())
            .then(data => {
                setUsers(data)
                setLoading(false)
            })
    }, [])

    if (loading) return <Spinner />
    return <ul>{users.map(u => <li key={u.id}>{u.name}</li>)}</ul>
}
```

### HTMX: Load on Render

```templ
templ UserListPage() {
    <div
        hx-get="/users/list"
        hx-trigger="load"
        hx-swap="innerHTML"
    >
        @Spinner()
    </div>
}

templ UserList(users []User) {
    <ul>
        for _, u := range users {
            <li>{ u.Name }</li>
        }
    </ul>
}
```

Or just render directly (simpler):
```go
func handleUsersPage(w http.ResponseWriter, r *http.Request) {
    users := getUsers()
    UserListPage(users).Render(r.Context(), w)
}
```

---

## Event Handlers

### React: onClick

```tsx
<button onClick={() => deleteItem(id)}>Delete</button>
```

### HTMX: hx-delete

```templ
<button
    hx-delete={ "/items/" + id }
    hx-target="closest .item"
    hx-swap="outerHTML"
    hx-confirm="Delete this item?"
>
    Delete
</button>
```

### React: onChange (Controlled Input)

```tsx
const [search, setSearch] = useState('')
<input value={search} onChange={e => setSearch(e.target.value)} />
```

### HTMX: hx-trigger on Change

```html
<input
    name="search"
    hx-get="/search"
    hx-trigger="keyup changed delay:300ms"
    hx-target="#results"
/>
```

---

## Conditional Rendering

### React

```tsx
{isLoggedIn ? <Dashboard /> : <LoginForm />}

{items.length > 0 && <ItemList items={items} />}

{error && <ErrorMessage error={error} />}
```

### Templ

```templ
if isLoggedIn {
    @Dashboard()
} else {
    @LoginForm()
}

if len(items) > 0 {
    @ItemList(items)
}

if err != "" {
    @ErrorMessage(err)
}
```

---

## List Rendering

### React: map

```tsx
{items.map(item => (
    <ItemCard key={item.id} item={item} />
))}
```

### Templ: range

```templ
for _, item := range items {
    @ItemCard(item)
}
```

---

## Component Props

### React

```tsx
interface CardProps {
    title: string
    description: string
    onClick: () => void
}

function Card({ title, description, onClick }: CardProps) {
    return (
        <div onClick={onClick}>
            <h3>{title}</h3>
            <p>{description}</p>
        </div>
    )
}

// Usage
<Card title="Hello" description="World" onClick={() => setSelected(id)} />
```

### Templ

```templ
type CardProps struct {
    Title       string
    Description string
    ID          string
}

templ Card(props CardProps) {
    <div
        hx-post={ "/select/" + props.ID }
        hx-swap="none"
    >
        <h3>{ props.Title }</h3>
        <p>{ props.Description }</p>
    </div>
}

// Usage
@Card(CardProps{Title: "Hello", Description: "World", ID: "123"})
```

---

## Children (Slots)

### React

```tsx
function Card({ children }) {
    return <div className="card">{children}</div>
}

<Card>
    <h3>Title</h3>
    <p>Content</p>
</Card>
```

### Templ

```templ
templ Card() {
    <div class="card">
        { children... }
    </div>
}

// Usage
@Card() {
    <h3>Title</h3>
    <p>Content</p>
}
```

---

## Navigation

### React: Link (React Router / Next.js)

```tsx
import Link from 'next/link'
<Link href="/about">About</Link>
```

### HTMX: Boosted Links

```html
<!-- Enable for all links in body -->
<body hx-boost="true">
    <a href="/about">About</a>  <!-- Now uses HTMX -->
</body>

<!-- Or per-link -->
<a href="/about" hx-boost="true">About</a>
```

Boost makes regular links use HTMX (swap body, update URL, no full reload).

---

## Modals

### React

```tsx
const [isOpen, setIsOpen] = useState(false)

<button onClick={() => setIsOpen(true)}>Open</button>

{isOpen && (
    <Modal onClose={() => setIsOpen(false)}>
        <ModalContent />
    </Modal>
)}
```

### HTMX: Server-Rendered Modal

```templ
// Trigger
<button hx-get="/modal/confirm" hx-target="#modal-container">
    Open
</button>

// Container (in layout)
<div id="modal-container"></div>

// Modal content (returned by /modal/confirm)
templ ConfirmModal() {
    <div class="modal-backdrop" onclick="closeModal()">
        <div class="modal" onclick="event.stopPropagation()">
            <h2>Confirm Action</h2>
            <button hx-post="/action" hx-target="#modal-container" hx-swap="innerHTML">
                Confirm
            </button>
            <button onclick="closeModal()">Cancel</button>
        </div>
    </div>
}
```

Or use `<dialog>` element:
```templ
templ Modal() {
    <dialog id="modal" class="modal">
        <form method="dialog">
            { children... }
            <button>Close</button>
        </form>
    </dialog>
}
```

---

## Tabs

### React

```tsx
const [activeTab, setActiveTab] = useState('info')

<div className="tabs">
    <button onClick={() => setActiveTab('info')} className={activeTab === 'info' ? 'active' : ''}>
        Info
    </button>
    <button onClick={() => setActiveTab('settings')} className={activeTab === 'settings' ? 'active' : ''}>
        Settings
    </button>
</div>

{activeTab === 'info' && <InfoPanel />}
{activeTab === 'settings' && <SettingsPanel />}
```

### HTMX

```templ
templ Tabs(activeTab string) {
    <div class="tabs">
        <button
            hx-get="/tabs/info"
            hx-target="#tab-content"
            class={ templ.KV("active", activeTab == "info") }
        >
            Info
        </button>
        <button
            hx-get="/tabs/settings"
            hx-target="#tab-content"
            class={ templ.KV("active", activeTab == "settings") }
        >
            Settings
        </button>
    </div>
    <div id="tab-content">
        switch activeTab {
            case "info":
                @InfoPanel()
            case "settings":
                @SettingsPanel()
        }
    </div>
}
```

---

## Real-time Updates

### React: EventSource

```tsx
useEffect(() => {
    const source = new EventSource('/events')
    source.onmessage = (e) => {
        setMessages(prev => [...prev, JSON.parse(e.data)])
    }
    return () => source.close()
}, [])
```

### HTMX: SSE Extension

```templ
<div
    hx-ext="sse"
    sse-connect="/events"
    sse-swap="message"
    hx-swap="beforeend"
>
    <!-- Messages appear here -->
</div>
```

Server returns HTML fragments instead of JSON.

---

## Form Handling

### React (with useState)

```tsx
const [formData, setFormData] = useState({ name: '', email: '' })
const [errors, setErrors] = useState({})

const handleSubmit = async (e) => {
    e.preventDefault()
    const res = await fetch('/api/users', {
        method: 'POST',
        body: JSON.stringify(formData)
    })
    if (!res.ok) {
        setErrors(await res.json())
    } else {
        // redirect or success
    }
}

<form onSubmit={handleSubmit}>
    <input
        value={formData.name}
        onChange={e => setFormData({...formData, name: e.target.value})}
    />
    {errors.name && <span>{errors.name}</span>}
</form>
```

### HTMX

```templ
templ UserForm(data FormData, errors FormErrors) {
    <form hx-post="/users" hx-target="this" hx-swap="outerHTML">
        <input name="name" value={ data.Name }/>
        if errors.Name != "" {
            <span class="error">{ errors.Name }</span>
        }
        <button type="submit">Submit</button>
    </form>
}
```

```go
func handleUserForm(w http.ResponseWriter, r *http.Request) {
    data := parseForm(r)
    errors := validate(data)

    if len(errors) > 0 {
        w.WriteHeader(422)
        UserForm(data, errors).Render(r.Context(), w)
        return
    }

    createUser(data)
    w.Header().Set("HX-Redirect", "/users")
}
```

---

## Context (Global State)

### React: Context API

```tsx
const ThemeContext = createContext('light')

function App() {
    const [theme, setTheme] = useState('light')
    return (
        <ThemeContext.Provider value={theme}>
            <Header />
            <Main />
        </ThemeContext.Provider>
    )
}
```

### HTMX: Server Session or Pass Data

```go
// Store in session
func setTheme(w http.ResponseWriter, r *http.Request, theme string) {
    session := getSession(r)
    session.Theme = theme
    saveSession(w, session)
}

// Pass to all templates
func handlePage(w http.ResponseWriter, r *http.Request) {
    session := getSession(r)
    PageTemplate(PageData{
        Theme: session.Theme,
        // ...
    }).Render(r.Context(), w)
}
```

---

## Infinite Scroll

### React

```tsx
const observer = useRef()
const lastItemRef = useCallback(node => {
    if (observer.current) observer.current.disconnect()
    observer.current = new IntersectionObserver(entries => {
        if (entries[0].isIntersecting && hasMore) {
            loadMore()
        }
    })
    if (node) observer.current.observe(node)
}, [hasMore])

{items.map((item, i) => (
    <div key={item.id} ref={i === items.length - 1 ? lastItemRef : null}>
        {item.name}
    </div>
))}
```

### HTMX

```templ
templ ItemList(items []Item, page int, hasMore bool) {
    for _, item := range items {
        <div>{ item.Name }</div>
    }

    if hasMore {
        <div
            hx-get={ fmt.Sprintf("/items?page=%d", page+1) }
            hx-trigger="revealed"
            hx-swap="outerHTML"
        >
            Loading more...
        </div>
    }
}
```

---

## When HTMX Isn't Enough

Use **Alpine.js** for client-side interactivity that doesn't need server:

### Dropdown

```html
<div x-data="{ open: false }">
    <button @click="open = !open">Menu</button>
    <div x-show="open" @click.away="open = false">
        <a href="/profile">Profile</a>
        <a href="/settings">Settings</a>
    </div>
</div>
```

### Accordion

```html
<div x-data="{ expanded: null }">
    <div>
        <button @click="expanded = expanded === 1 ? null : 1">
            Section 1
        </button>
        <div x-show="expanded === 1">Content 1</div>
    </div>
</div>
```

### When to Use Alpine.js vs HTMX

| Use Alpine.js | Use HTMX |
|--------------|----------|
| Toggle visibility | Load data from server |
| Dropdowns, accordions | Form submission |
| Client-side filtering | Search with server query |
| Animations | Real-time updates |
| No server interaction | Any server interaction |

---

## Performance Comparison

### React App

```
Initial load:
- index.html (1KB)
- react.min.js (140KB)
- app.bundle.js (300KB+)
- API calls for data (JSON)
Total: 400KB+ JS, multiple round trips
```

### HTMX App

```
Initial load:
- index.html (complete page, 20KB)
- htmx.min.js (14KB)
Total: 34KB, single request
```

### When React Still Makes Sense

- Highly interactive apps (Figma, Google Docs)
- Offline-first applications
- Complex client-side state machines
- Heavy animations/canvas work
- Mobile apps (React Native)

---

## Migration Strategy

### Incremental Approach

1. **Start with new features** - Build new pages in HTMX
2. **Replace simple pages** - Static pages, forms, lists
3. **Add HTMX to existing React** - Use `hx-boost` for links
4. **Replace complex components last** - Modals, wizards

### Parallel Run

```html
<!-- Keep React for complex parts -->
<div id="react-dashboard"></div>

<!-- Use HTMX for new sections -->
<div hx-get="/notifications" hx-trigger="load">
    Loading...
</div>
```

---

## Integration

This skill works with:
- **go-htmx-core**: HTMX patterns
- **go-templ-components**: Component syntax
- **go-htmx-forms**: Form migration
- **go-htmx-dashboard**: Dashboard patterns

Reference this skill when migrating React code to HTMX.
