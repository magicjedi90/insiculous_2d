# Missed Issues Analysis

Analyze a crate for code quality violations that may have been missed during development.

**Usage:** `/missed <crate_name>` (e.g., `/missed renderer`, `/missed engine_core`)

**Argument:** `$ARGUMENTS` - The crate name to analyze (e.g., `renderer`, `ecs`, `engine_core`)

---

## Phase 1: Context Gathering

### 1.1 Validate Target Crate
```
Target crate: crates/$ARGUMENTS/
```

- Verify the crate exists at `crates/$ARGUMENTS/`
- If crate not found, list available crates and ask user to specify

### 1.2 Read Existing Documentation
- Read `crates/$ARGUMENTS/ANALYSIS.md` if it exists
- Read `PROJECT_ROADMAP.md` for known tech debt related to this crate
- Read `training.md` for established patterns this crate should follow

---

## Phase 2: Code Quality Audit

For EVERY `.rs` file in `crates/$ARGUMENTS/src/`:

### 2.1 DRY Violations (Don't Repeat Yourself)
Check for:
- [ ] Duplicated code blocks (3+ lines repeated)
- [ ] Similar functions that could be generalized
- [ ] Copy-pasted logic with minor variations
- [ ] Repeated error handling patterns
- [ ] Redundant type conversions

### 2.2 SRP Violations (Single Responsibility Principle)
Check for:
- [ ] Structs with too many responsibilities (>3 distinct concerns)
- [ ] Functions doing multiple unrelated things
- [ ] Files mixing unrelated functionality
- [ ] Methods that should be split into smaller units
- [ ] God objects or god functions (>100 lines)

### 2.3 KISS Violations (Keep It Simple, Stupid)
Check for:
- [ ] Over-engineered abstractions
- [ ] Unnecessary generics or traits
- [ ] Complex nested logic that could be flattened
- [ ] Premature optimization
- [ ] Unused flexibility/configurability

### 2.4 Additional Quality Checks
- [ ] Unused imports or dead code
- [ ] Missing or outdated documentation
- [ ] Inconsistent naming conventions
- [ ] Error handling gaps
- [ ] Test coverage gaps

---

## Phase 3: Architecture Review

### 3.1 File Placement Audit
For each file in the crate, verify:
- [ ] File is in the correct module based on its responsibility
- [ ] Public API matches what should be exposed
- [ ] Internal helpers are properly scoped (not public when they shouldn't be)
- [ ] Module structure follows project conventions

### 3.2 Cross-Crate Dependencies
- [ ] Check if this crate depends on things it shouldn't
- [ ] Check if other crates should depend on this one differently
- [ ] Identify circular dependency risks

---

## Phase 4: Generate TECH_DEBT.md

Create or update `crates/$ARGUMENTS/TECH_DEBT.md` with this structure:

```markdown
# Technical Debt: $ARGUMENTS

Last audited: [DATE]

## Summary
- DRY violations: X
- SRP violations: X
- KISS violations: X
- Architecture issues: X

## DRY Violations

### [DRY-001] Description
- **File:** `path/to/file.rs`
- **Lines:** X-Y
- **Issue:** [What is duplicated]
- **Suggested fix:** [How to resolve]
- **Priority:** High/Medium/Low

## SRP Violations

### [SRP-001] Description
- **File:** `path/to/file.rs`
- **Lines:** X-Y
- **Issue:** [What responsibilities are mixed]
- **Suggested fix:** [How to separate concerns]
- **Priority:** High/Medium/Low

## KISS Violations

### [KISS-001] Description
- **File:** `path/to/file.rs`
- **Lines:** X-Y
- **Issue:** [What is over-complicated]
- **Suggested fix:** [How to simplify]
- **Priority:** High/Medium/Low

## Architecture Issues

### [ARCH-001] Description
- **File:** `path/to/file.rs`
- **Issue:** [Misplaced or incorrectly scoped]
- **Suggested fix:** [Where it should be / how to restructure]
- **Priority:** High/Medium/Low
```

---

## Phase 5: Final Verification

### 5.1 Cross-Reference
- Compare findings against `PROJECT_ROADMAP.md` - are these new issues or already tracked?
- Update `PROJECT_ROADMAP.md` if significant new debt was found

### 5.2 Report Summary
Provide a summary:
- Total issues found by category
- Top 3 highest priority items
- Estimated effort to resolve (if applicable)
- Recommendations for next steps

---

## Output Format

When complete, provide:
1. Path to generated `TECH_DEBT.md`
2. Summary statistics
3. Top priority issues highlighted
4. Any items that should be added to `PROJECT_ROADMAP.md`
