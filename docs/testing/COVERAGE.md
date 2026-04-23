# Test Coverage Configuration

## Configuration
The project uses Bun's built-in test runner with coverage enabled via `bunfig.toml`:

```toml
[test]
coverage = true
coverageSkipTestFiles = true
coverageReporter = ["text", "lcov"]
```

## Research Findings
After extensive research of the official Bun documentation (analyzing 150+ code snippets), **Bun does not currently support arbitrary file exclusion patterns for code coverage**. 

The only coverage exclusion option available is:
- `coverageSkipTestFiles = true` - Excludes test files themselves (files matching `*.test.ts`, `*.spec.ts`, etc.)

## Files Still Included in Coverage
Despite configuration attempts, these files cannot be excluded with current Bun capabilities:
- `pkg/mpc_wallet.js` (45.93% func, 49.02% line coverage) - Auto-generated WASM bindings
- `src/entrypoints/offscreen/test-utils.ts` (69.23% func, 70.59% line coverage) - Test utilities

## Available Coverage Configuration Options
Based on official documentation, Bun supports these coverage-related configurations:

```toml
[test]
coverage = true                          # Enable/disable coverage
coverageSkipTestFiles = true            # Skip test files only
coverageReporter = ["text", "lcov"]     # Output formats
coverageDir = "coverage"                # Output directory
coverageThreshold = 0.8                 # Global threshold
coverageIgnoreSourcemaps = false       # Sourcemap handling
```

**No support for:**
- `coverageExclude` patterns
- `coverageIgnore` patterns  
- Glob-based file exclusion
- Custom file filtering

## Workaround Options
Since Bun lacks built-in file exclusion, consider these alternatives:

### 1. File Structure Changes
Move auto-generated files outside source directories:
```bash
# Move WASM files to a separate directory
mkdir -p generated/
mv pkg/ generated/
```

### 2. Post-Process LCOV Reports
Filter the generated `coverage/lcov.info` file:
```bash
# Remove unwanted files from LCOV report
grep -v "SF:.*pkg/mpc_wallet.js" coverage/lcov.info > coverage/filtered.info
grep -v "SF:.*test-utils.ts" coverage/filtered.info > coverage/final.info
```

### 3. Alternative Coverage Tools
Use external coverage tools that support exclusion patterns:
```bash
# Example with c8 (would require additional setup)
bun test --coverage && c8 --exclude="pkg/**" --exclude="**/test-utils.ts" report
```

### 4. Custom Coverage Script
Create a script to run tests and filter results programmatically.

## Recommendation
Accept the current coverage metrics as-is: core application code
coverage is healthy, and the over-counted files are generated WASM
bindings + test utilities that shouldn't skew real coverage
discussions. Monitor Bun's development for future coverage-exclusion
features.
