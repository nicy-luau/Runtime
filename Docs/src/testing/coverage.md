# Code Coverage

Code coverage tracking for NicyRuntime's Rust codebase.

## Current Status

**Target**: 80% line coverage for `Runtime/src/`

> 📊 Coverage reports are not yet automated. This is a planned feature.

## Manual Coverage with `cargo-tarpaulin`

Install tarpaulin:

```bash
cargo install cargo-tarpaulin
```

Run coverage:

```bash
cd Runtime
cargo tarpaulin --out Html --output-dir tarpaulin-report
```

This generates `tarpaulin-report/tarpaulin-report.html` with interactive coverage data.

## Manual Coverage with `grcov`

For coverage of tests:

```bash
# Install grcov
cargo install grcov

# Build with coverage instrumentation
cd Runtime
RUSTFLAGS="-C instrument-coverage" cargo build

# Run tests
nicy run tests/run_all.luau

# Generate report
grcov target/ -s . --binary-path target/debug/ -t html --branch --ignore-not-existing -o grcov-report/
```

## Coverage Goals

| Module | Current | Target |
|--------|---------|--------|
| `lib.rs` | TBD | 80% |
| `ffi_exports.rs` | TBD | 90% |
| `require_resolver.rs` | TBD | 85% |
| `task_scheduler.rs` | TBD | 85% |
| `error.rs` | TBD | 90% |

## Future Plans

- [ ] GitHub Actions coverage check on every PR
- [ ] Coverage badge in README
- [ ] Coverage regression prevention (fail PR if coverage drops > 2%)
- [ ] Per-module coverage thresholds in CI

## Contributing

When adding new features:

1. Write tests that cover the new code paths
2. Run `cargo tarpaulin` locally to verify coverage
3. Ensure new code is at least 80% covered
