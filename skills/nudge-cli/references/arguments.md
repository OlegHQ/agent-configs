# Multiline text and structured arguments

Use this reference when text contains newlines, quotes, Unicode, shell metacharacters, or when a command edits nested records. The CLI accepts the original text as a string flag; JSON escaping is unnecessary.

## POSIX shell: literal multiline Markdown

A quoted heredoc prevents expansion of dollar signs, backticks, and backslashes. The sentinel below preserves trailing newlines that command substitution would otherwise remove:

```sh
content=$(cat <<'NUDGE_MARKDOWN'
# Recovery — 世界

Keep "double quotes", 'single quotes', `inline code`, and $variables literally.

NUDGE_MARKDOWN
printf x
)
content=${content%x}
nudge tool update_document --id DOCUMENT --content "$content" --expected-updated-at-ms 1700000000000
```

Choose a delimiter that does not occur as a line in the content. Replace the sample timestamp with the numeric `updatedAtMs` from the current document. Do not interpolate workspace text into a shell command string; pass the content as one quoted argument.

For existing UTF-8 Markdown, preserve every trailing newline with `content=$(cat ./runbook.md; printf x)` followed by `content=${content%x}` and the same quoted `--content "$content"` flag. If byte-for-byte content matters, fetch the document after writing and compare the returned string with the source.

## PowerShell 7.3+: read an existing Markdown file

```powershell
$PSNativeCommandArgumentPassing = 'Standard'
$content = [string](Get-Content -LiteralPath './runbook.md' -Raw -Encoding utf8)
nudge tool update_document --id DOCUMENT --content "$content" --expected-updated-at-ms 1700000000000
if ($LASTEXITCODE -ne 0) { throw 'Nudge update failed' }
```

Keep content in an argument variable; do not use `Invoke-Expression` to build a Nudge command from workspace text. The string cast preserves an empty file as an empty argument. Native process failures are signaled by the exit code. This example uses PowerShell 7.3+ native argument passing, which preserves embedded quotes and empty strings; Windows PowerShell 5.1 has different behavior. See [Microsoft's native argument documentation](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_parsing#passing-arguments-that-contain-quote-characters).

## Lists, nested objects, and batches

Repeat a scalar list flag: `--tags operations --tags 'release,urgent'`. The comma stays inside one tag. Use `--clear-tags` to submit an empty list; `--tags ''` submits one empty string and can be rejected by the domain. Use `--unset-parent-id` to send null rather than the literal string `null`.

For object-array flags, everything after the first equals sign is the value. `--issues-title '0=Check a=b, then retry'` sets item zero's title without splitting the comma or second equals sign. Nested arrays use one index per surrounding array; all indices are zero-based and contiguous. Supply each object's required fields. For example, a record with two typed cells:

```sh
nudge tool create_document --title Experiment --parent-id DATABASE \
  --values-property-id 0=notes --values-text '0=Check a=b, then retry' \
  --values-property-id 1=effort --values-number 1=0
```

Flags for one record's arrays replace that record's complete array; they are not an incremental cell update. Read the existing row and retain other cells when editing. Missing required fields, sparse indices, duplicate scalar assignments, and conflicting parent/child or clear/value arguments fail before a request is sent. Domain errors and version conflicts can still occur after successful argument parsing; inspect the result and current resource before deciding on a retry.

Existing generated JSON files can still use `--input FILE`, and pipelines can use `--input -`. This optional path accepts a complete input object and cannot mix with field flags or positional arguments. It is useful for existing machine-produced data; it is not required for ordinary CLI work.
