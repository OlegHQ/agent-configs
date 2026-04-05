---
name: go-pico-embed
description: TE Pico Theme - Teenage Engineering inspired CSS theme for Pico CSS. Industrial minimalism with dense layouts and monospace typography.
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# TE Pico Theme - Teenage Engineering Inspired CSS

A light-mode, dense, industrial CSS theme for Pico CSS inspired by Teenage Engineering's design philosophy.

## When to Use

- Starting a new Go + HTMX project with Pico CSS
- Building dashboards, admin panels, or data-dense UIs
- Creating developer tools with a professional, technical aesthetic
- Wanting a clean, industrial look with monospace typography

## Theme Philosophy

- **Industrial minimalism**: Clean light backgrounds, exposed components, no decoration
- **Dense layouts**: 4px grid instead of 8px for data-rich displays
- **Monospace-heavy**: Technical UI with code-like typography
- **Warm neutrals**: Off-white backgrounds (#f5f4f2), not stark white

## Installation

### 1. HTML Head Setup

```html
<html lang="en" data-theme="light">
  <head>
    <!-- Google Fonts -->
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
    <!-- Pico CSS base -->
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css">
    <!-- TE theme (inline or static) -->
    <style>/* te-pico.css content */</style>
  </head>
```

### 2. For Go with static files

```go
mux.Handle("GET /static/", http.StripPrefix("/static/", http.FileServer(http.Dir("static"))))
```

## Color System

| Token | Value | Usage |
|-------|-------|-------|
| `--te-bg-deep` | #f5f4f2 | Primary background |
| `--te-bg-base` | #ebeae8 | Elevated surfaces |
| `--te-bg-elevated` | #ffffff | Cards, modals |
| `--te-bg-surface` | #e0dfdd | Inputs, hover states |
| `--te-text-primary` | #0f0e12 | Primary text |
| `--te-text-secondary` | #4a4a4a | Secondary text |
| `--te-text-tertiary` | #7a7a7a | Muted text |
| `--te-blue` | #0d5a91 | Primary actions, links |
| `--te-green` | #158855 | Success states |
| `--te-red` | #b81c1d | Error states |
| `--te-yellow` | #d9a003 | Warning states |
| `--te-purple` | #7a5699 | Accent color |

## Components

### Badges

```html
<span class="badge badge-blue">Blue</span>
<span class="badge badge-green">Success</span>
<span class="badge badge-red">Error</span>
<span class="badge badge-yellow">Warning</span>
<span class="badge badge-purple">Purple</span>
<span class="badge badge-gray">Neutral</span>

<!-- Semantic -->
<span class="badge badge-success">Success</span>
<span class="badge badge-warning">Warning</span>
<span class="badge badge-error">Error</span>
<span class="badge badge-info">Info</span>
```

### Navigation

```html
<nav class="container nav-terminal">
  <div class="nav-container">
    <a href="/" class="nav-logo contrast"><strong>App Name</strong></a>
    <ul class="nav-links">
      <li><a href="/">Home</a></li>
      <li><a href="/docs">Docs</a></li>
    </ul>
  </div>
</nav>
```

### Stats Grid

```html
<div class="stats-grid">
  <div class="stat-card">
    <div class="stat-value">42</div>
    <div class="stat-label">Total</div>
  </div>
</div>
```

### Pipeline Board

```html
<div class="pipeline-board">
  <div class="stage-column stage-in-progress">
    <div class="stage-header">
      <h3>In Progress</h3>
      <span class="stage-count">3</span>
    </div>
    <div class="stage-items">
      <!-- cards here -->
    </div>
  </div>
</div>
```

### Interactive Cards

```html
<article class="card-interactive">
  <header>Card Title</header>
  <p>Card content</p>
</article>
```

### Message Stream

```html
<div class="message-stream">
  <div class="message message-assistant">
    <div class="message-role">Assistant</div>
    <div class="message-content">Response text</div>
  </div>
  <div class="message message-user">User message</div>
  <div class="message message-tool">Tool output</div>
</div>
```

## Utility Classes

### Typography
- `.mono`, `.font-mono` - Monospace font
- `.text-xs` to `.text-2xl` - Font sizes
- `.text-primary`, `.text-secondary`, `.text-muted` - Text colors
- `.label` - Uppercase label style
- `.truncate`, `.line-clamp-2`, `.line-clamp-3` - Text truncation

### Spacing (4px grid)
- `.gap-1` to `.gap-6` - Gap utilities
- `.p-0` to `.p-6` - Padding
- `.m-0` to `.m-4` - Margin
- `.px-*`, `.py-*` - Directional padding
- `.stack`, `.stack-tight`, `.stack-loose` - Vertical spacing

### Layout
- `.flex`, `.flex-col`, `.flex-wrap`
- `.items-center`, `.items-start`, `.items-end`
- `.justify-between`, `.justify-center`, `.justify-end`
- `.grid`

### Animation
- `.hover-scale` - Scale 1.02x on hover
- `.hover-border` - Blue border on hover
- `.hover-lift` - Lift up on hover
- `.animate-pulse` - Pulsing animation
- `.animate-fade-in` - Fade in animation

## Font Stack

- **Body**: Inter (with system fallbacks)
- **Monospace**: Pragmata Pro > IBM Plex Mono > JetBrains Mono > system

---

## Full CSS

Copy this CSS to your project (inline in `<style>` or as `static/css/te-pico.css`):

```css
/*! te-pico.css v1.0.0 | Teenage Engineering inspired Pico CSS theme */
:root {
  --te-bg-deep: #f5f4f2;
  --te-bg-base: #ebeae8;
  --te-bg-elevated: #ffffff;
  --te-bg-surface: #e0dfdd;
  --te-bg-muted: #d5d4d2;
  --te-text-primary: #0f0e12;
  --te-text-secondary: #4a4a4a;
  --te-text-tertiary: #7a7a7a;
  --te-blue: #0d5a91;
  --te-green: #158855;
  --te-red: #b81c1d;
  --te-yellow: #d9a003;
  --te-purple: #7a5699;
  --te-gray: #6b6b6b;
  --te-border-color: #d0cfcd;
  --te-border-color-subtle: #e0dfdd;
  --te-border-color-strong: #b0afad;
  --pico-font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif;
  --pico-font-family-monospace: "Pragmata Pro", "IBM Plex Mono", "JetBrains Mono", "SF Mono", "Consolas", monospace;
  --pico-font-size: 13px;
  --te-font-size-xs: 11px;
  --te-font-size-sm: 12px;
  --te-font-size-base: 13px;
  --te-font-size-md: 14px;
  --te-font-size-lg: 16px;
  --te-font-size-xl: 18px;
  --te-font-size-2xl: 24px;
  --pico-line-height: 1.4;
  --te-font-weight-light: 300;
  --te-font-weight-regular: 400;
  --te-font-weight-medium: 500;
  --te-font-weight-bold: 600;
  --te-space-1: 4px;
  --te-space-2: 8px;
  --te-space-3: 12px;
  --te-space-4: 16px;
  --te-space-5: 20px;
  --te-space-6: 24px;
  --te-space-8: 32px;
  --pico-border-radius: 2px;
  --te-border-radius-sm: 2px;
  --te-border-radius-md: 4px;
  --te-border-radius-lg: 6px;
  --te-border-radius-full: 9999px;
  --pico-transition: 0.15s ease-out;
  --te-transition-fast: 0.1s ease-out;
  --te-transition-normal: 0.15s ease-out;
  --te-shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
  --te-shadow-md: 0 2px 4px rgba(0, 0, 0, 0.08);
  --te-shadow-lg: 0 4px 8px rgba(0, 0, 0, 0.1);
}
:root, [data-theme="light"] {
  color-scheme: light;
  --pico-background-color: var(--te-bg-deep);
  --pico-card-background-color: var(--te-bg-elevated);
  --pico-form-element-background-color: var(--te-bg-surface);
  --pico-dropdown-background-color: var(--te-bg-elevated);
  --pico-code-background-color: var(--te-bg-base);
  --pico-color: var(--te-text-primary);
  --pico-h1-color: var(--te-text-primary);
  --pico-h2-color: var(--te-text-primary);
  --pico-h3-color: var(--te-text-primary);
  --pico-h4-color: var(--te-text-primary);
  --pico-h5-color: var(--te-text-primary);
  --pico-h6-color: var(--te-text-primary);
  --pico-muted-color: var(--te-text-secondary);
  --pico-muted-border-color: var(--te-border-color);
  --pico-primary: var(--te-blue);
  --pico-primary-hover: #0a4a78;
  --pico-primary-focus: rgba(13, 90, 145, 0.2);
  --pico-primary-inverse: #ffffff;
  --pico-primary-background: var(--te-blue);
  --pico-secondary: var(--te-gray);
  --pico-secondary-hover: #5a5a5a;
  --pico-secondary-focus: rgba(107, 107, 107, 0.2);
  --pico-secondary-inverse: #ffffff;
  --pico-contrast: var(--te-text-primary);
  --pico-contrast-hover: #000000;
  --pico-contrast-focus: rgba(15, 14, 18, 0.2);
  --pico-contrast-inverse: var(--te-bg-elevated);
  --pico-form-element-border-color: var(--te-border-color);
  --pico-form-element-active-border-color: var(--te-blue);
  --pico-form-element-focus-color: var(--pico-primary-focus);
  --pico-form-element-placeholder-color: var(--te-text-tertiary);
  --pico-form-element-valid-border-color: var(--te-green);
  --pico-form-element-invalid-border-color: var(--te-red);
  --pico-spacing: var(--te-space-3);
  --pico-typography-spacing-vertical: var(--te-space-3);
  --pico-block-spacing-vertical: var(--te-space-4);
  --pico-block-spacing-horizontal: var(--te-space-4);
  --pico-form-element-spacing-vertical: var(--te-space-2);
  --pico-form-element-spacing-horizontal: var(--te-space-3);
  --pico-nav-element-spacing-vertical: var(--te-space-2);
  --pico-nav-element-spacing-horizontal: var(--te-space-3);
  --pico-button-box-shadow: none;
  --pico-button-hover-box-shadow: none;
  --pico-table-border-color: var(--te-border-color);
  --pico-table-row-stripped-background-color: var(--te-bg-base);
  --pico-code-color: var(--te-text-primary);
  --pico-code-kbd-background-color: var(--te-bg-surface);
  --pico-code-tag-color: var(--te-purple);
  --pico-code-property-color: var(--te-blue);
  --pico-code-value-color: var(--te-green);
  --pico-code-comment-color: var(--te-text-tertiary);
  --pico-mark-background-color: rgba(217, 160, 3, 0.3);
  --pico-mark-color: var(--te-text-primary);
  --pico-ins-color: var(--te-green);
  --pico-del-color: var(--te-red);
  --pico-blockquote-border-color: var(--te-blue);
  --pico-blockquote-footer-color: var(--te-text-secondary);
  --pico-hr-border-color: var(--te-border-color);
  --pico-text-selection-color: rgba(13, 90, 145, 0.2);
  --pico-accordion-border-color: var(--te-border-color);
  --pico-accordion-active-summary-color: var(--te-blue);
  --pico-modal-overlay-background-color: rgba(245, 244, 242, 0.9);
  --pico-progress-background-color: var(--te-bg-surface);
  --pico-progress-color: var(--te-blue);
  --pico-tooltip-background-color: var(--te-text-primary);
  --pico-tooltip-color: var(--te-bg-elevated);
}
body { font-family: var(--pico-font-family); font-size: var(--pico-font-size); line-height: var(--pico-line-height); background-color: var(--pico-background-color); color: var(--te-text-primary); }
h1, h2, h3, h4, h5, h6 { font-weight: var(--te-font-weight-medium); line-height: 1.2; }
h1 { font-size: var(--te-font-size-2xl); } h2 { font-size: var(--te-font-size-xl); } h3 { font-size: var(--te-font-size-lg); } h4 { font-size: var(--te-font-size-md); } h5, h6 { font-size: var(--te-font-size-base); }
a { color: var(--te-blue); text-decoration: none; transition: color var(--te-transition-fast); } a:hover { color: var(--pico-primary-hover); }
code, kbd, pre, samp { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-sm); }
:not(pre) > code { padding: 0.125em 0.25em; background: var(--te-bg-base); border: 1px solid var(--te-border-color-subtle); border-radius: var(--te-border-radius-sm); }
pre { padding: var(--te-space-3); background: var(--te-bg-base); border: 1px solid var(--te-border-color); border-radius: var(--te-border-radius-sm); overflow-x: auto; }
nav { background: var(--te-bg-base); border-bottom: 1px solid var(--te-border-color); }
.nav-container { display: flex; justify-content: space-between; align-items: center; gap: var(--te-space-6); }
.nav-logo { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-md); letter-spacing: -0.02em; }
.nav-links { display: flex; gap: var(--te-space-4); list-style: none; margin: 0; padding: 0; }
.nav-links a { text-decoration: none; font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-sm); text-transform: uppercase; letter-spacing: 0.05em; color: var(--te-text-secondary); transition: color var(--te-transition-fast); }
.nav-links a:hover { color: var(--te-text-primary); }
.nav-links a[aria-current="page"], .nav-links a.active { color: var(--te-blue); }
.nav-terminal .nav-links a::before { content: ">"; margin-right: var(--te-space-1); color: var(--te-text-tertiary); }
article { background: var(--te-bg-elevated); border: 1px solid var(--te-border-color); border-radius: var(--te-border-radius-sm); padding: var(--te-space-3); margin-bottom: var(--te-space-3); }
article:hover { border-color: var(--te-border-color-strong); }
article > header { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-xs); color: var(--te-text-tertiary); text-transform: uppercase; letter-spacing: 0.1em; margin-bottom: var(--te-space-2); padding-bottom: var(--te-space-2); border-bottom: 1px solid var(--te-border-color-subtle); }
.card-interactive { cursor: pointer; transition: transform var(--te-transition-normal), border-color var(--te-transition-normal); }
.card-interactive:hover { transform: scale(1.02); border-color: var(--te-blue); }
button, [type="submit"], [type="button"], [type="reset"], [role="button"] { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-sm); font-weight: var(--te-font-weight-medium); text-transform: uppercase; letter-spacing: 0.05em; padding: var(--te-space-2) var(--te-space-4); border-radius: var(--te-border-radius-sm); box-shadow: none; transition: background-color var(--te-transition-fast), transform var(--te-transition-fast); }
button:hover, [type="submit"]:hover, [type="button"]:hover { transform: translateY(-1px); }
button:active, [type="submit"]:active, [type="button"]:active { transform: translateY(0); }
button.outline, button.secondary, .btn-outline { background: transparent; border: 1px solid var(--te-border-color); color: var(--te-text-primary); }
button.outline:hover, button.secondary:hover, .btn-outline:hover { background: var(--te-bg-surface); border-color: var(--te-border-color-strong); }
.btn-success { background: var(--te-green); border-color: var(--te-green); }
.btn-warning { background: var(--te-yellow); border-color: var(--te-yellow); color: var(--te-text-primary); }
.btn-danger { background: var(--te-red); border-color: var(--te-red); }
input, select, textarea { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-sm); background: var(--te-bg-surface); border-color: var(--te-border-color); padding: var(--te-space-2) var(--te-space-3); }
input:focus, select:focus, textarea:focus { border-color: var(--te-blue); box-shadow: 0 0 0 2px var(--pico-primary-focus); }
label { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-xs); text-transform: uppercase; letter-spacing: 0.05em; color: var(--te-text-secondary); margin-bottom: var(--te-space-1); }
table { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-sm); }
th { font-size: var(--te-font-size-xs); font-weight: var(--te-font-weight-medium); text-transform: uppercase; letter-spacing: 0.05em; color: var(--te-text-secondary); background: var(--te-bg-base); padding: var(--te-space-2) var(--te-space-3); }
td { padding: var(--te-space-2) var(--te-space-3); border-bottom: 1px solid var(--te-border-color-subtle); }
tr:hover td { background: var(--te-bg-surface); }
.badge { display: inline-flex; align-items: center; padding: var(--te-space-1) var(--te-space-2); font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-xs); font-weight: var(--te-font-weight-medium); text-transform: uppercase; letter-spacing: 0.05em; border-radius: var(--te-border-radius-sm); border: 1px solid transparent; line-height: 1; }
.badge-blue { background: #dbeafe; color: #0d5a91; border-color: #93c5fd; }
.badge-green { background: #dcfce7; color: #158855; border-color: #86efac; }
.badge-red { background: #fee2e2; color: #b81c1d; border-color: #fca5a5; }
.badge-yellow { background: #fef3c7; color: #92400e; border-color: #fcd34d; }
.badge-purple { background: #f3e8ff; color: #7a5699; border-color: #d8b4fe; }
.badge-gray { background: var(--te-bg-surface); color: var(--te-text-secondary); border-color: var(--te-border-color); }
.badge-success { background: #dcfce7; color: #158855; border-color: #86efac; }
.badge-warning { background: #fef3c7; color: #92400e; border-color: #fcd34d; }
.badge-error { background: #fee2e2; color: #b81c1d; border-color: #fca5a5; }
.badge-info { background: #dbeafe; color: #0d5a91; border-color: #93c5fd; }
.badge-opportunity { background: #f3e8ff; color: #7a5699; border-color: #d8b4fe; }
.badge-in_progress, .badge-in-progress { background: #dbeafe; color: #0d5a91; border-color: #93c5fd; }
.badge-fork_pr, .badge-fork-pr { background: #fef3c7; color: #92400e; border-color: #fcd34d; }
.badge-upstream { background: #f3e8ff; color: #7a5699; border-color: #d8b4fe; }
.badge-merged { background: #dcfce7; color: #158855; border-color: #86efac; }
.badge-closed { background: var(--te-bg-surface); color: var(--te-text-secondary); border-color: var(--te-border-color); }
.badge-tier1 { background: #dcfce7; color: #158855; border-color: #86efac; }
.badge-tier2 { background: #dbeafe; color: #0d5a91; border-color: #93c5fd; }
.badge-aws { background: #fef3c7; color: #92400e; border-color: #fcd34d; }
.badge-cncf { background: #f3e8ff; color: #7a5699; border-color: #d8b4fe; }
.badge-easy { background: #dcfce7; color: #158855; border-color: #86efac; }
.badge-medium { background: #fef3c7; color: #92400e; border-color: #fcd34d; }
.badge-hard { background: #fee2e2; color: #b81c1d; border-color: #fca5a5; }
.badge-pill { border-radius: var(--te-border-radius-full); padding: var(--te-space-1) var(--te-space-3); }
.mono, .font-mono { font-family: var(--pico-font-family-monospace); }
.text-xs { font-size: var(--te-font-size-xs); } .text-sm { font-size: var(--te-font-size-sm); } .text-base { font-size: var(--te-font-size-base); } .text-md { font-size: var(--te-font-size-md); } .text-lg { font-size: var(--te-font-size-lg); } .text-xl { font-size: var(--te-font-size-xl); } .text-2xl { font-size: var(--te-font-size-2xl); }
.text-primary { color: var(--te-text-primary); } .text-secondary { color: var(--te-text-secondary); } .text-tertiary { color: var(--te-text-tertiary); } .text-muted { color: var(--te-text-tertiary); }
.text-success { color: var(--te-green); } .text-warning { color: var(--te-yellow); } .text-error { color: var(--te-red); } .text-info { color: var(--te-blue); }
.label { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-xs); text-transform: uppercase; letter-spacing: 0.1em; color: var(--te-text-tertiary); }
.section-number { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-lg); font-weight: var(--te-font-weight-light); color: var(--te-text-tertiary); margin-right: var(--te-space-2); }
.section-number::before { content: "0"; }
.truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.line-clamp-2 { display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.line-clamp-3 { display: -webkit-box; -webkit-line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; }
.gap-1 { gap: var(--te-space-1); } .gap-2 { gap: var(--te-space-2); } .gap-3 { gap: var(--te-space-3); } .gap-4 { gap: var(--te-space-4); } .gap-6 { gap: var(--te-space-6); }
.p-0 { padding: 0; } .p-1 { padding: var(--te-space-1); } .p-2 { padding: var(--te-space-2); } .p-3 { padding: var(--te-space-3); } .p-4 { padding: var(--te-space-4); } .p-6 { padding: var(--te-space-6); }
.px-1 { padding-left: var(--te-space-1); padding-right: var(--te-space-1); } .px-2 { padding-left: var(--te-space-2); padding-right: var(--te-space-2); } .px-3 { padding-left: var(--te-space-3); padding-right: var(--te-space-3); } .px-4 { padding-left: var(--te-space-4); padding-right: var(--te-space-4); }
.py-1 { padding-top: var(--te-space-1); padding-bottom: var(--te-space-1); } .py-2 { padding-top: var(--te-space-2); padding-bottom: var(--te-space-2); } .py-3 { padding-top: var(--te-space-3); padding-bottom: var(--te-space-3); } .py-4 { padding-top: var(--te-space-4); padding-bottom: var(--te-space-4); }
.m-0 { margin: 0; } .m-1 { margin: var(--te-space-1); } .m-2 { margin: var(--te-space-2); } .m-3 { margin: var(--te-space-3); } .m-4 { margin: var(--te-space-4); }
.mb-1 { margin-bottom: var(--te-space-1); } .mb-2 { margin-bottom: var(--te-space-2); } .mb-3 { margin-bottom: var(--te-space-3); } .mb-4 { margin-bottom: var(--te-space-4); }
.mt-1 { margin-top: var(--te-space-1); } .mt-2 { margin-top: var(--te-space-2); } .mt-3 { margin-top: var(--te-space-3); } .mt-4 { margin-top: var(--te-space-4); }
.stack-tight > * + * { margin-top: var(--te-space-2); } .stack > * + * { margin-top: var(--te-space-3); } .stack-loose > * + * { margin-top: var(--te-space-4); }
.flex { display: flex; } .flex-wrap { flex-wrap: wrap; } .flex-col { flex-direction: column; }
.items-center { align-items: center; } .items-start { align-items: flex-start; } .items-end { align-items: flex-end; }
.justify-between { justify-content: space-between; } .justify-center { justify-content: center; } .justify-end { justify-content: flex-end; }
.grid { display: grid; }
.container-dense { max-width: 1200px; padding: var(--te-space-3); }
@keyframes te-pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.5; } }
.animate-pulse { animation: te-pulse 2s ease-in-out infinite; }
@keyframes te-fade-in { from { opacity: 0; } to { opacity: 1; } }
.animate-fade-in { animation: te-fade-in 0.2s ease-out; }
.hover-scale { transition: transform var(--te-transition-normal); } .hover-scale:hover { transform: scale(1.02); }
.hover-border { transition: border-color var(--te-transition-normal); } .hover-border:hover { border-color: var(--te-blue); }
.hover-lift { transition: transform var(--te-transition-fast); } .hover-lift:hover { transform: translateY(-1px); }
.pipeline-board { display: grid; grid-template-columns: repeat(5, 1fr); gap: var(--te-space-3); margin-top: var(--te-space-3); }
.stage-column { background: var(--te-bg-base); border: 1px solid var(--te-border-color); border-radius: var(--te-border-radius-sm); min-height: 400px; }
.stage-header { display: flex; justify-content: space-between; align-items: center; padding: var(--te-space-2) var(--te-space-3); border-bottom: 1px solid var(--te-border-color); font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-xs); text-transform: uppercase; letter-spacing: 0.1em; }
.stage-header h3 { margin: 0; font-size: inherit; font-weight: var(--te-font-weight-medium); }
.stage-count { background: var(--te-bg-surface); padding: var(--te-space-1) var(--te-space-2); border-radius: var(--te-border-radius-sm); font-size: var(--te-font-size-xs); }
.stage-items { padding: var(--te-space-2); }
.stage-opportunity .stage-header { border-left: 3px solid var(--te-purple); }
.stage-in_progress .stage-header, .stage-in-progress .stage-header { border-left: 3px solid var(--te-blue); }
.stage-fork_pr .stage-header, .stage-fork-pr .stage-header { border-left: 3px solid var(--te-yellow); }
.stage-upstream .stage-header { border-left: 3px solid var(--te-purple); }
.stage-merged .stage-header { border-left: 3px solid var(--te-green); }
.stage-closed .stage-header { border-left: 3px solid var(--te-gray); }
.stats-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: var(--te-space-3); margin-bottom: var(--te-space-3); }
.stat-card { background: var(--te-bg-elevated); border: 1px solid var(--te-border-color); border-radius: var(--te-border-radius-sm); padding: var(--te-space-3); text-align: center; }
.stat-value { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-2xl); font-weight: var(--te-font-weight-light); color: var(--te-text-primary); line-height: 1; margin-bottom: var(--te-space-1); }
.stat-label { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-xs); text-transform: uppercase; letter-spacing: 0.1em; color: var(--te-text-tertiary); }
.stat-value-blue { color: var(--te-blue); } .stat-value-green { color: var(--te-green); } .stat-value-yellow { color: var(--te-yellow); } .stat-value-red { color: var(--te-red); }
.work-item-card { background: var(--te-bg-elevated); border: 1px solid var(--te-border-color); border-radius: var(--te-border-radius-sm); padding: var(--te-space-3); margin-bottom: var(--te-space-2); cursor: pointer; transition: border-color var(--te-transition-fast), transform var(--te-transition-fast); }
.work-item-card:hover { border-color: var(--te-border-color-strong); transform: translateY(-1px); }
.work-item-title { font-size: var(--te-font-size-sm); font-weight: var(--te-font-weight-medium); margin-bottom: var(--te-space-2); display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }
.work-item-title a { font-family: var(--pico-font-family-monospace); color: var(--te-blue); text-decoration: none; }
.work-item-meta { display: flex; flex-wrap: wrap; gap: var(--te-space-1); font-size: var(--te-font-size-xs); color: var(--te-text-secondary); }
.message-stream, .run-feed { font-family: var(--pico-font-family-monospace); font-size: var(--te-font-size-sm); background: var(--te-bg-elevated); border: 1px solid var(--te-border-color); border-radius: var(--te-border-radius-sm); padding: var(--te-space-3); overflow-y: auto; max-height: 500px; }
.message { padding: var(--te-space-2) var(--te-space-3); margin-bottom: var(--te-space-2); border-radius: var(--te-border-radius-sm); border-left: 2px solid; }
.message-assistant { background: #eff6ff; border-left-color: var(--te-blue); }
.message-user { background: #f0fdf4; border-left-color: var(--te-green); }
.message-tool { background: #fefce8; border-left-color: var(--te-yellow); }
.message-error { background: #fef2f2; border-left-color: var(--te-red); }
.message-role { font-size: var(--te-font-size-xs); text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: var(--te-space-1); color: var(--te-text-secondary); }
.message-content { white-space: pre-wrap; word-break: break-word; }
.thinking { background: #fefce8; font-style: italic; color: var(--te-yellow); border-left-color: var(--te-yellow); }
@media (max-width: 1200px) { .stats-grid { grid-template-columns: repeat(3, 1fr); } }
@media (max-width: 1024px) { .pipeline-board { grid-template-columns: repeat(3, 1fr); } }
@media (max-width: 768px) { .pipeline-board { grid-template-columns: 1fr; } .stats-grid { grid-template-columns: repeat(2, 1fr); } }
@media (max-width: 480px) { .stats-grid { grid-template-columns: 1fr; } }
```

---

## Integration

This skill works with:
- **go-project-bootstrap**: CSS setup in new projects
- **go-templ-components**: Component styling patterns
- **go-htmx-dashboard**: Dashboard UI styling
- **go-htmx-forms**: Form element styling

Reference this skill when styling Go + HTMX applications.
