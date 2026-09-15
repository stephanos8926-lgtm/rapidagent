# Security Policy

## Supported Versions

Use this section to tell people about which versions of your project are currently being supported with security updates.

| Version | Supported |
|---------|-----------|
| 5.1.x   | Yes       |
| 5.0.x   | No        |
| 4.0.x   | Yes       |
| < 4.0   | No        |

## Reporting a Vulnerability

We take the security of [PROJECT_NAME] seriously. If you believe you've found a security vulnerability, please report it to us as described below.

### Reporting Process

1. **DO NOT** open a public issue
2. Email security findings to [SECURITY_EMAIL]
3. Include a description of the vulnerability and steps to reproduce
4. Allow reasonable time for response (typically 72 hours)

### What to Expect

- You will receive an acknowledgment within 48 hours
- We will investigate and determine the impact
- You will be kept informed of progress
- Credit will be given for valid reports (unless anonymity is requested)

### Vulnerability Disclosure Policy

- We request that you allow reasonable time for remediation before disclosing publicly
- We will coordinate with you on timing of public disclosure
- We will not take legal action against good-faith researchers

## Security Best Practices

When contributing to this project:

- Keep dependencies up to date
- Run security audits: `pip-audit` or `trivy`
- Never commit secrets or credentials
- Use environment variables for sensitive configuration
- Enable two-factor authentication on your accounts

## References

- [OpenSSF Best Practices Badge](https://bestpractices.coreinfrastructure.org/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Python Security Guidelines](https://docs.python.org/3/security/)
