# Browser Workspace Boundaries

The browser companion is an untrusted presentation client. It does not accept,
store, display, log, or place provider/API tokens in URLs. Authentication and
CSRF protection belong to the host that serves the app. The client uses
same-origin credentials only and sends no bearer header.

The default Vite document includes a strict baseline CSP. A production host
must emit its own CSP header (headers take precedence over the document meta)
with `connect-src` narrowed to the configured local API origin. Do not add
`unsafe-eval`, `unsafe-inline`, broad wildcard origins, preview origins, or
token-bearing query parameters.

Bootstrap may provide only non-secret configuration such as `apiBaseUrl` and
project/session identifiers through a server-rendered document or a
same-origin endpoint. The bootstrap boundary must not contain credentials.
Until an authenticated host integration exists, the app remains fixture-backed
for UI development and does not claim to provide production authentication.

Markdown and attachments are untrusted. This slice renders document metadata
only; future content rendering must sanitize Markdown and keep preview content
on a separate origin. API errors shown to users are generic and do not expose
filesystem, token, or stack details.
