# Permissions And Security

## Threat Model

Assume the provider can be wrong or malicious, project files can contain hostile instructions, tool arguments can target secrets, plugins can be compromised, and a local API client may be untrusted if the port is exposed.

## Capability Layers

1. Process policy: whether the feature is enabled.
2. Project policy: approved roots, commands, and network targets.
3. Agent policy: plan/build capabilities.
4. Request policy: exact operation and scope.
5. User decision: approve, deny, or bounded approval.

Every layer must be able to deny. A lower layer cannot grant what a higher layer forbids.

## Approval Scopes

- Once: one exact operation.
- Session: matching operation class within the current session.
- Project: persisted only with explicit user action and visible configuration.

Default to once for mutating and external operations.

The current runtime implementation creates durable pending permission requests and suspends tool execution until `POST /api/v1/permissions/{permission_id}/resolve` answers them. Approval scopes and timeout expiry remain future work.

## Filesystem Security

- Canonicalize and validate project-relative paths.
- Reject traversal and unexpected absolute paths.
- Handle symlink escapes explicitly.
- Protect credential files and configurable sensitive patterns. The built-in default denies `.env`, credential/secret names, private-key names, and common private-key extensions before broker approval.
- Do not follow arbitrary links during recursive search by default.

## Command Security

- Show the full command and working directory before approval.
- Restrict environment inheritance.
- Limit duration, CPU/memory where supported, output, and process tree.
- Make network access a distinct risk class.
- Kill descendants on cancellation when possible.
- Read-only Git queries use fixed command strings, disable Git optional locks, and authorize status, diff, log, and worktree metadata separately.

The current shell boundary kills and reaps its direct child on timeout or cancellation. Process-group descendant cleanup and restricted environment inheritance remain follow-up work.

## API Security

Loopback only by default. If remote binding is enabled, require authentication, origin policy, explicit configuration, and warnings. Do not treat an obscure port as authentication.

## Secrets

Use environment variables, OS keychain integration later, or explicit config references. Redact known secret patterns in logs and tool previews. Never send secrets to the model unless the user explicitly supplies them as task data.

## Plugin Security

Plugins cannot access storage internals or bypass the permission broker. Native dynamic loading is not MVP. External plugin processes need a versioned protocol, scoped capabilities, resource limits, and explicit trust configuration.

## Security Gates

Run path-policy tests, secret-redaction tests, malicious instruction fixtures, permission race tests, remote-bind checks, dependency audits, and fuzzers before release.
