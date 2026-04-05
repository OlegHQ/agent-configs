---
name: go-htmx-forms
description: Form handling with real-time validation, error display, and submission patterns using Go and HTMX.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# Go + HTMX Form Handling

Build forms with real-time validation and seamless submission using HTMX. No JavaScript form libraries needed.

## When to Use

- Building forms with instant validation feedback
- Need server-side validation (always required)
- Want progressive enhancement (forms work without JS)
- Implementing multi-step wizards

---

## Basic Form Submission

### HTML Form with HTMX

```html
<form hx-post="/items" hx-target="#result" hx-swap="innerHTML">
    <input type="text" name="title" required>
    <button type="submit">Create</button>
</form>
<div id="result"></div>
```

### Go Handler

```go
func handleCreateItem(w http.ResponseWriter, r *http.Request) {
    r.ParseForm()
    title := r.FormValue("title")

    if title == "" {
        w.WriteHeader(http.StatusUnprocessableEntity)
        ErrorMessage("Title is required").Render(r.Context(), w)
        return
    }

    item := createItem(title)
    ItemCard(item).Render(r.Context(), w)
}
```

---

## Real-Time Field Validation

Validate fields as user types or on blur.

### Input with Validation

```templ
// Pico CSS uses aria-invalid for validation styling
templ EmailField(email string, err string) {
    <label for="email">
        Email
        <input
            type="email"
            id="email"
            name="email"
            value={ email }
            hx-post="/validate/email"
            hx-trigger="blur changed"
            hx-target="#email-error"
            hx-swap="innerHTML"
            aria-invalid={ err != "" }
        />
        <small id="email-error">{ err }</small>
    </label>
}
```

### Validation Handler

```go
func handleValidateEmail(w http.ResponseWriter, r *http.Request) {
    r.ParseForm()
    email := r.FormValue("email")

    if email == "" {
        w.WriteHeader(http.StatusUnprocessableEntity)
        fmt.Fprint(w, "Email is required")
        return
    }

    if !isValidEmail(email) {
        w.WriteHeader(http.StatusUnprocessableEntity)
        fmt.Fprint(w, "Please enter a valid email address")
        return
    }

    // Check if email exists
    if emailExists(email) {
        w.WriteHeader(http.StatusUnprocessableEntity)
        fmt.Fprint(w, "Email already registered")
        return
    }

    // Valid - return empty response to clear error
    w.WriteHeader(http.StatusOK)
}

func isValidEmail(email string) bool {
    _, err := mail.ParseAddress(email)
    return err == nil
}
```

---

## Validation with go-playground/validator

### Setup

```go
import "github.com/go-playground/validator/v10"

var validate = validator.New()

type CreateUserInput struct {
    Name     string `form:"name" validate:"required,min=2,max=50"`
    Email    string `form:"email" validate:"required,email"`
    Password string `form:"password" validate:"required,min=8"`
    Age      int    `form:"age" validate:"omitempty,gte=18,lte=120"`
}
```

### Parse and Validate

```go
func handleCreateUser(w http.ResponseWriter, r *http.Request) {
    r.ParseForm()

    input := CreateUserInput{
        Name:     r.FormValue("name"),
        Email:    r.FormValue("email"),
        Password: r.FormValue("password"),
    }

    if age := r.FormValue("age"); age != "" {
        input.Age, _ = strconv.Atoi(age)
    }

    if err := validate.Struct(input); err != nil {
        errors := extractValidationErrors(err)
        w.WriteHeader(http.StatusUnprocessableEntity)
        FormWithErrors(input, errors).Render(r.Context(), w)
        return
    }

    user := createUser(input)
    w.Header().Set("HX-Redirect", "/users/"+user.ID)
}

func extractValidationErrors(err error) map[string]string {
    errors := make(map[string]string)

    for _, e := range err.(validator.ValidationErrors) {
        field := strings.ToLower(e.Field())
        switch e.Tag() {
        case "required":
            errors[field] = fmt.Sprintf("%s is required", e.Field())
        case "email":
            errors[field] = "Invalid email address"
        case "min":
            errors[field] = fmt.Sprintf("%s must be at least %s characters", e.Field(), e.Param())
        case "max":
            errors[field] = fmt.Sprintf("%s must be at most %s characters", e.Field(), e.Param())
        default:
            errors[field] = fmt.Sprintf("Invalid %s", e.Field())
        }
    }

    return errors
}
```

---

## Form Components

### Reusable Input

```templ
type FieldProps struct {
    Name        string
    Label       string
    Type        string
    Value       string
    Error       string
    Placeholder string
    Required    bool
}

// Pico CSS styles form elements semantically
templ Field(props FieldProps) {
    <label for={ props.Name }>
        { props.Label }
        if props.Required {
            <abbr title="required">*</abbr>
        }
        <input
            type={ props.Type }
            id={ props.Name }
            name={ props.Name }
            value={ props.Value }
            placeholder={ props.Placeholder }
            required?={ props.Required }
            hx-post={ "/validate/" + props.Name }
            hx-trigger="blur changed delay:300ms"
            hx-target={ "#" + props.Name + "-error" }
            aria-invalid={ props.Error != "" }
            aria-describedby={ props.Name + "-error" }
        />
        <small id={ props.Name + "-error" }>{ props.Error }</small>
    </label>
}
```

### Select Field

```templ
templ SelectField(name string, label string, options []Option, selected string, err string) {
    <label for={ name }>
        { label }
        <select
            id={ name }
            name={ name }
            aria-invalid={ err != "" }
        >
            <option value="">Select...</option>
            for _, opt := range options {
                <option value={ opt.Value } selected?={ opt.Value == selected }>
                    { opt.Label }
                </option>
            }
        </select>
        if err != "" {
            <small>{ err }</small>
        }
    </label>
}
```

### Textarea

```templ
templ TextareaField(name string, label string, value string, err string, rows int) {
    <label for={ name }>
        { label }
        <textarea
            id={ name }
            name={ name }
            rows={ fmt.Sprint(rows) }
            aria-invalid={ err != "" }
        >{ value }</textarea>
        if err != "" {
            <small>{ err }</small>
        }
    </label>
}
```

---

## Complete Form Example

### Form Component

```templ
type UserFormData struct {
    Name     string
    Email    string
    Role     string
}

type UserFormErrors struct {
    Name  string
    Email string
    Role  string
}

templ UserForm(data UserFormData, errors UserFormErrors, isEdit bool) {
    <form
        if isEdit {
            hx-put="/users/update"
        } else {
            hx-post="/users"
        }
        hx-target="this"
        hx-swap="outerHTML"
    >
        @Field(FieldProps{
            Name:     "name",
            Label:    "Full Name",
            Type:     "text",
            Value:    data.Name,
            Error:    errors.Name,
            Required: true,
        })

        @Field(FieldProps{
            Name:     "email",
            Label:    "Email Address",
            Type:     "email",
            Value:    data.Email,
            Error:    errors.Email,
            Required: true,
        })

        @SelectField("role", "Role", []Option{
            {Value: "user", Label: "User"},
            {Value: "admin", Label: "Admin"},
            {Value: "mod", Label: "Moderator"},
        }, data.Role, errors.Role)

        <div class="grid">
            <button type="submit">
                if isEdit {
                    Update
                } else {
                    Create
                }
            </button>
            <a href="/users" role="button" class="secondary">Cancel</a>
        </div>
    </form>
}
```

### Handler

```go
func handleUserForm(w http.ResponseWriter, r *http.Request) {
    r.ParseForm()

    data := UserFormData{
        Name:  r.FormValue("name"),
        Email: r.FormValue("email"),
        Role:  r.FormValue("role"),
    }

    errors := UserFormErrors{}
    hasErrors := false

    if data.Name == "" {
        errors.Name = "Name is required"
        hasErrors = true
    }

    if data.Email == "" {
        errors.Email = "Email is required"
        hasErrors = true
    } else if !isValidEmail(data.Email) {
        errors.Email = "Invalid email address"
        hasErrors = true
    }

    if hasErrors {
        w.WriteHeader(http.StatusUnprocessableEntity)
        UserForm(data, errors, false).Render(r.Context(), w)
        return
    }

    user := createUser(data)
    w.Header().Set("HX-Redirect", "/users/"+user.ID)
}
```

---

## Loading States

### Button with Loading State

```templ
// Pico CSS supports aria-busy for native loading spinners
templ SubmitButton(text string) {
    <button type="submit" hx-indicator="this" hx-disabled-elt="this">
        <span class="htmx-hide-on-request">{ text }</span>
        <span class="htmx-indicator" aria-busy="true">Submitting...</span>
    </button>
}
```

CSS (add to custom.css):
```css
.htmx-indicator { display: none; }
.htmx-request .htmx-indicator { display: inline-block; }
.htmx-request .htmx-hide-on-request { display: none; }
```

Alternative using Pico's built-in busy state:
```html
<!-- Button shows spinner automatically when aria-busy is true -->
<button type="submit" aria-busy="false" hx-on::htmx:before-request="this.ariaBusy='true'" hx-on::htmx:after-request="this.ariaBusy='false'">
    Submit
</button>
```

### Disabled During Request

```html
<form hx-post="/submit" hx-disabled-elt="find button">
    <button type="submit">Submit</button>
</form>
```

---

## Out-of-Band Error Summary

Show errors at top of form while keeping inline errors.

### Form with Summary

```templ
templ FormWithSummary(data FormData, errors map[string]string) {
    <form hx-post="/submit" hx-target="this" hx-swap="outerHTML">
        <div id="error-summary">
            if len(errors) > 0 {
                @ErrorSummary(errors)
            }
        </div>

        <!-- Fields... -->
    </form>
}

templ ErrorSummary(errors map[string]string) {
    <article aria-invalid="true">
        <header>
            <strong>Please fix the following errors:</strong>
        </header>
        <ul>
            for field, msg := range errors {
                <li><del>{ msg }</del></li>
            }
        </ul>
    </article>
}
```

### OOB Response

```go
func handleSubmit(w http.ResponseWriter, r *http.Request) {
    // ... validation

    if hasErrors {
        w.WriteHeader(http.StatusUnprocessableEntity)

        // Main form response
        FormWithSummary(data, errors).Render(r.Context(), w)

        // OOB update for error summary (scrolls to top)
        fmt.Fprint(w, `<div id="error-summary" hx-swap-oob="true">`)
        ErrorSummary(errors).Render(r.Context(), w)
        fmt.Fprint(w, `</div>`)
        return
    }
}
```

---

## File Uploads

### Upload Form

```templ
templ FileUploadForm() {
    <form
        hx-post="/upload"
        hx-encoding="multipart/form-data"
        hx-target="#upload-result"
    >
        <label>
            Choose file
            <input type="file" name="file" accept=".pdf,.doc,.docx" required/>
        </label>
        <button type="submit">Upload</button>
    </form>
    <div id="upload-result"></div>
}
```

### Handler

```go
func handleUpload(w http.ResponseWriter, r *http.Request) {
    r.ParseMultipartForm(10 << 20) // 10MB max

    file, header, err := r.FormFile("file")
    if err != nil {
        w.WriteHeader(http.StatusBadRequest)
        ErrorMessage("No file provided").Render(r.Context(), w)
        return
    }
    defer file.Close()

    // Validate file type
    contentType := header.Header.Get("Content-Type")
    if !isAllowedType(contentType) {
        w.WriteHeader(http.StatusUnprocessableEntity)
        ErrorMessage("File type not allowed").Render(r.Context(), w)
        return
    }

    // Save file
    savedPath, err := saveFile(file, header.Filename)
    if err != nil {
        w.WriteHeader(http.StatusInternalServerError)
        ErrorMessage("Failed to save file").Render(r.Context(), w)
        return
    }

    UploadSuccess(savedPath).Render(r.Context(), w)
}
```

---

## Multi-Step Forms

### Wizard Component

```templ
templ WizardStep1(data WizardData) {
    <article id="wizard">
        <header>
            <nav>
                <ul>
                    <li><strong>1. Personal</strong></li>
                    <li>2. Address</li>
                    <li>3. Review</li>
                </ul>
            </nav>
        </header>

        <form hx-post="/wizard/step2" hx-target="#wizard" hx-swap="outerHTML">
            <input type="hidden" name="step" value="1"/>

            @Field(FieldProps{Name: "name", Label: "Name", Value: data.Name, Required: true})
            @Field(FieldProps{Name: "email", Label: "Email", Type: "email", Value: data.Email, Required: true})

            <button type="submit">Next</button>
        </form>
    </article>
}

templ WizardStep2(data WizardData) {
    <article id="wizard">
        <header>
            <nav>
                <ul>
                    <li><ins>1. Personal</ins></li>
                    <li><strong>2. Address</strong></li>
                    <li>3. Review</li>
                </ul>
            </nav>
        </header>

        <form hx-post="/wizard/step3" hx-target="#wizard" hx-swap="outerHTML">
            <!-- Preserve previous data -->
            <input type="hidden" name="name" value={ data.Name }/>
            <input type="hidden" name="email" value={ data.Email }/>

            @Field(FieldProps{Name: "address", Label: "Address", Value: data.Address})
            @Field(FieldProps{Name: "city", Label: "City", Value: data.City})

            <div class="grid">
                <button type="button" hx-get="/wizard/step1" hx-target="#wizard" class="secondary">
                    Back
                </button>
                <button type="submit">Next</button>
            </div>
        </form>
    </article>
}
```

---

## CSRF Protection

### Generate Token

```go
import "crypto/rand"

func generateCSRFToken() string {
    b := make([]byte, 32)
    rand.Read(b)
    return base64.StdEncoding.EncodeToString(b)
}

func setCSRFCookie(w http.ResponseWriter, token string) {
    http.SetCookie(w, &http.Cookie{
        Name:     "csrf_token",
        Value:    token,
        HttpOnly: true,
        Secure:   true,
        SameSite: http.SameSiteStrictMode,
    })
}
```

### Hidden Field

```templ
templ CSRFField(token string) {
    <input type="hidden" name="csrf_token" value={ token }/>
}

templ Form(csrfToken string) {
    <form hx-post="/submit">
        @CSRFField(csrfToken)
        <!-- other fields -->
    </form>
}
```

### Middleware Validation

```go
func csrfMiddleware(next http.Handler) http.Handler {
    return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        if r.Method == "POST" || r.Method == "PUT" || r.Method == "DELETE" {
            cookie, err := r.Cookie("csrf_token")
            if err != nil {
                http.Error(w, "Missing CSRF token", http.StatusForbidden)
                return
            }

            r.ParseForm()
            formToken := r.FormValue("csrf_token")

            if formToken != cookie.Value {
                http.Error(w, "Invalid CSRF token", http.StatusForbidden)
                return
            }
        }
        next.ServeHTTP(w, r)
    })
}
```

---

## Form Reset After Success

### Using HX-Trigger

```go
func handleSubmit(w http.ResponseWriter, r *http.Request) {
    // ... process form

    // Tell client to reset form
    w.Header().Set("HX-Trigger", "formSubmitted")
    SuccessMessage("Item created!").Render(r.Context(), w)
}
```

```html
<form
    hx-post="/items"
    hx-target="#result"
    hx-on::formSubmitted="this.reset()"
>
```

### Return Fresh Form

```go
func handleSubmit(w http.ResponseWriter, r *http.Request) {
    // ... process

    // Return empty form + success message
    FormWithSuccess().Render(r.Context(), w)
}
```

---

## Confirm Before Submit

```html
<form
    hx-post="/items/delete"
    hx-confirm="Are you sure you want to delete this item?"
>
    <button type="submit">Delete</button>
</form>
```

---

## Integration

This skill works with:
- **go-project-bootstrap**: Initial project setup
- **go-htmx-core**: Form submission patterns
- **go-templ-components**: Reusable form components
- **go-pico-embed**: Asset embedding and deployment

Reference this skill when building any form in Go + HTMX.
