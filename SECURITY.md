# Security Policy

## Supported Versions

Security updates are applied to the latest released version.

## Reporting a Vulnerability

If you discover a security vulnerability, please do not open a public issue.

Instead, send details to:
- Email: security@caret.dev (placeholder)

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if known)

We will:
- Acknowledge receipt within 48 hours
- Provide a detailed response within 7 days
- Issue a fix according to severity

## Threat Model

See [docs/threat-model.md](docs/threat-model.md) for our documented threat model.

## Security Best Practices

- Always use bounded queues (default)
- Validate all plugin manifests
- Never run untrusted pipelines without sandboxing
- Keep dependencies updated

## Responsible Disclosure

We follow coordinated disclosure:
- We will notify users of critical vulnerabilities
- We will provide upgrade paths
- We will credit reporters
