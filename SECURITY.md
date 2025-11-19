# Security Policy

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you discover a security vulnerability in pickle-whip, please report it to:

**Email**: security@cisco.com

### What to Include

When reporting a vulnerability, please include:

- **Description**: Clear explanation of the vulnerability
- **Impact**: Potential security impact and affected versions
- **Steps to Reproduce**: Detailed steps to reproduce the issue
- **Proof of Concept**: Code or commands demonstrating the vulnerability (if applicable)
- **Suggested Fix**: Proposed solution or mitigation (if you have one)
- **Your Contact Information**: For follow-up questions

### Response Timeline

- **Initial Response**: Within 5 business days
- **Status Updates**: Every 7 days until resolved
- **Disclosure**: Coordinated with reporter after fix is available

### Security Update Process

1. Vulnerability is reported and acknowledged
2. Issue is validated and severity assessed
3. Fix is developed and tested
4. Security advisory is prepared
5. Fix is released with advisory
6. Public disclosure (coordinated with reporter)

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

We recommend always using the latest version for the most up-to-date security fixes.

## Security Considerations

### Tool Purpose

pickle-whip is a **security testing tool** that generates pickle bytecode for fuzzing and testing purposes. By design, it creates potentially malicious pickle data.

### Safe Usage Guidelines

- **DO NOT** use generated pickles in production systems
- **DO NOT** unpickle generated data without proper sandboxing
- **DO** use in isolated testing environments only
- **DO** follow responsible disclosure for vulnerabilities found using this tool
- **DO** ensure you have authorization before testing third-party systems

### Known Limitations

- Generated pickles may trigger security scanners (this is expected)
- Some generated pickles may cause resource exhaustion in parsers
- Mutation features can produce invalid or malformed pickles intentionally

## Responsible Disclosure

If you discover vulnerabilities in other projects using pickle-whip:

1. Report to the affected project's security team first
2. Allow reasonable time for fixes (typically 90 days)
3. Coordinate public disclosure with the affected project
4. Credit researchers appropriately

## Security Best Practices

When using pickle-whip for security research:

- **Isolate**: Run in containers or VMs
- **Monitor**: Watch for resource exhaustion
- **Document**: Keep records of findings
- **Coordinate**: Work with affected vendors
- **Respect**: Follow responsible disclosure practices

## Contact

For security-related questions or concerns:
- **Email**: security@cisco.com
- **PGP Key**: Available upon request

For general questions, use [GitHub Issues](https://github.com/cisco-ai-defense/pickle-whip/issues).

---

Thank you for helping keep pickle-whip and its users safe!
