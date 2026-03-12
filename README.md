# daml-lint

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

> [!WARNING]
> This software is experimental and not intended for production use. Use at your own risk.

Static analysis scanner for [DAML](https://www.digitalasset.com/developers) smart contracts. Catches security vulnerabilities and anti-patterns through AST pattern matching, similar to what [Slither](https://github.com/crytic/slither) does for Solidity.

## Detectors

| Detector | Severity | Description |
|----------|----------|-------------|
| `missing-ensure-decimal` | HIGH | Template has Decimal fields without an `ensure` clause bounding them to > 0 |
| `unguarded-division` | HIGH | Division operation without a prior guard checking the denominator is non-zero |
| `missing-positive-amount` | HIGH | Choice accepts amount/quantity/price parameter without asserting it is positive |
| `archive-before-execute` | HIGH | Contract archived before a `try/catch` block — contract is lost if execution fails |
| `head-of-list-query` | MEDIUM | Pattern match on head of `queryFilter` result — non-deterministic ordering risk |
| `unbounded-fields` | MEDIUM | Text, List, or TextMap fields without size bounds in the `ensure` clause |

## Installation

Requires [Rust](https://rustup.rs/) 1.70+.

```sh
git clone https://github.com/OpenZeppelin/daml-lint.git
cd daml-lint
cargo install --path .
```

## Usage

Scan a single file:

```sh
daml-lint src/MyContract.daml
```

Scan a directory recursively:

```sh
daml-lint ./daml/
```

Choose an output format:

```sh
daml-lint ./daml/ --format sarif    # SARIF JSON (GitHub / IDE integration)
daml-lint ./daml/ --format markdown # Human-readable (default)
daml-lint ./daml/ --format json     # Machine-readable JSON
```

Write results to a file:

```sh
daml-lint ./daml/ --format sarif --output report.sarif
```

### CI gating

Use `--fail-on` to control when the tool returns a non-zero exit code:

```sh
daml-lint ./daml/ --fail-on medium   # fail on medium or above
daml-lint ./daml/ --fail-on critical # fail only on critical
```

## Output Formats

- **SARIF** — Standard format for static analysis tools. Integrates with GitHub Code Scanning and IDEs.
- **Markdown** — Human-readable report grouped by severity. Good for pull request comments.
- **JSON** — Flat findings array with summary counts. Good for dashboards and aggregation.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No findings at or above the `--fail-on` threshold |
| 1 | One or more findings at or above the threshold |
| 2 | CLI error (invalid format, no files found, etc.) |

