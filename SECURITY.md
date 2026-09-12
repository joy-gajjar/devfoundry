# Security Policy

## Reporting A Vulnerability

Do not report an unpatched vulnerability in a public issue, pull request, chat,
or release discussion. Use GitHub's **Report a vulnerability** button on the
repository Security tab when private vulnerability reporting is enabled. If
that option is unavailable, contact the repository owner privately and mark the
message `security-sensitive`; do not attach secrets or private project data.

Include:

- affected release, platform, and commit if known;
- impact and the security boundary involved;
- minimal reproduction steps or a safe proof of concept;
- whether the issue is already public or being actively exploited; and
- a safe contact method and preferred disclosure credit.

Remove provider tokens, authorization headers, signing material, private keys,
project contents, and personal data before sending a report. If a log or
fixture is necessary, replace sensitive values with stable placeholders.

Maintainers should acknowledge a report within five business days, triage its
severity, coordinate a fix and disclosure date with the reporter, and credit
the reporter unless anonymity is requested. Do not promise a severity or
disclosure date before triage.

The repository's fuller scope, secret-handling rules, and release gates are in
[`docs/security.md`](docs/security.md).

## Supported Versions

Security fixes target the current release line. Upgrade to the latest release
and use the documented rollback procedure if an upgrade causes an operational
regression.
