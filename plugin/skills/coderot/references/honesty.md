# Test-honesty checks

Read-only, on the main tree (no edits). Test files are those the test runner picks up for the target: co-located `*.test.*`/`*.spec.*`, `tests/`, Rust `#[cfg(test)]` modules and `tests/*.rs`.

| Kind | How to find it |
|---|---|
| `only` | `.only(`, `fit(`, `fdescribe(`: the rest of the suite silently skips |
| `skipped` | `.skip(`, `xit(`, `xdescribe(`, `.todo(`, `#[ignore]`, `@pytest.mark.skip`, `@unittest.skip` |
| `commented-out` | a commented line that opens a test: `// it(`, `// test(`, `/* describe(`, `// #[test]`, `# def test_` |
| `hollow` | a test body with no assertion (`expect`, `assert*`, `should`, `panic`-on-fail helpers), or whose only assertions are mock-was-called (`toHaveBeenCalled*`, `.called`) |
| `flaky` | failed in some but not all of the three baseline runs |
| `untested change` | a source file in the target changed in the last 10 commits (`git log -10 --format= --name-only -- <prefixes>`) that no test file imports or names, and whose module no test touched in those commits |

One row per finding. Quote the test name or the file, never the whole test.
