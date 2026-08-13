# Missed Issues Analysis

Analyze a crate for code quality violations that may have been missed during development.

**Usage:** Invoke with a crate name: `@missed analyze the ecs crate`

**Scope:** Focus on `crates/<crate_name>/src/` directory

---

## Phase 1: Context Gathering

### 1.1 Validate Target Crate

First, verify the crate exists:
```bash
ls crates/<crate_name>/
```

If not found, list available crates:
```bash
ls crates/
```

### 1.2 Read Existing Documentation

Read these files in order:
1. `crates/<crate_name>/ANALYSIS.md` - Previous analysis and plans
2. `crates/<crate_name>/TECH_DEBT.md` - Known existing debt
3. `PROJECT_ROADMAP.md` - Related roadmap items
4. `AGENTS.md` - High-level project guidance
5. `training.md` - Established patterns to check against

### 1.3 Assess Crate Size

Count files and lines to determine analysis approach:
```bash
find crates/<crate_name>/src -name "*.rs" | wc -l
```

**For large crates (>20 files):** Use targeted grep patterns instead of reading every file.

---

## Phase 2: Targeted Code Quality Audit

Use `Grep` and `Glob` tools for efficient pattern detection rather than reading every file.

### 2.1 DRY Violations (Don't Repeat Yourself)

**Detection patterns:**
```bash
# Find similar error handling patterns
grep -n "expect\|unwrap\|panic!" *.rs | head -20

# Find repeated code blocks (3+ lines)
# Look for identical patterns in different functions
```

**Check for:**
- [ ] Duplicated code blocks (3+ lines repeated across files)
- [ ] Similar functions that could be generalized
- [ ] Copy-pasted logic with minor variations
- [ ] Repeated error handling patterns (`.expect()` with same message)
- [ ] Redundant type conversions

**Tool recommendation:** Use `Grep` with patterns like:
- `pattern: "fn.*\{[^}]*\}"` to find small functions that might be duplicates
- `pattern: "\.expect\("` to find error handling patterns

### 2.2 SRP Violations (Single Responsibility Principle)

**Detection approach:**
```bash
# Find large structs and impl blocks
grep -n "^pub struct" *.rs
wc -l *.rs | sort -n
```

**Check for:**
- [ ] Structs with >3 distinct responsibilities (check fields and methods)
- [ ] Functions doing multiple unrelated things (>50 lines is a warning sign)
- [ ] Files mixing unrelated functionality
- [ ] Methods that should be split (complex control flow)
- [ ] God objects (>500 lines) or god functions (>100 lines in Rust)

**Context note:** In Rust, 100-line functions are very large. Typical good functions are 10-30 lines.

### 2.3 KISS Violations (Keep It Simple, Stupid)

**Detection patterns:**
```bash
# Find complex generics
grep -n "<.*,.*,.*>" *.rs  # 3+ generic parameters

# Find deeply nested code
grep -n "        " *.rs | wc -l  # 8+ space indents = deep nesting
```

**Check for:**
- [ ] Over-engineered abstractions (traits with single implementation)
- [ ] Unnecessary generics or complex trait bounds
- [ ] Complex nested logic (>3 levels of nesting)
- [ ] Premature optimization (unsafe code, complex algorithms)
- [ ] Unused flexibility (config options never changed)

### 2.4 Additional Quality Checks

Use targeted searches:
- **Unused imports:** `cargo clippy` (if available) or check for `#[allow(unused_imports)]`
- **Missing docs:** `grep -L "^///" *.rs` to find undocumented public items
- **Error handling:** `grep -n "unwrap\|expect" *.rs | wc -l` (count potential panic points)
- **Test coverage:** Compare `grep -c "^fn " *.rs` vs `grep -c "#\[test\]" *.rs`

---

## Phase 3: Architecture Review

### 3.1 File Placement Audit

For each major file, verify:
- [ ] File location matches its responsibility
- [ ] Public API is intentionally exposed (not `pub` by default)
- [ ] Internal helpers are private or `pub(crate)`
- [ ] Module structure follows Rust conventions (`mod.rs` or flat structure consistently)

### 3.2 Cross-Crate Dependencies

Check `Cargo.toml`:
```toml
[dependencies]
# Are all dependencies necessary?
# Could any be dev-dependencies?
```

Verify:
- [ ] No circular dependencies between crates
- [ ] Dependencies follow intended architecture (lower crates don't depend on higher ones)
- [ ] No unused dependencies

---

## Phase 4: Generate TECH_DEBT.md

### 4.1 Check for Existing File

Read `crates/<crate_name>/TECH_DEBT.md` if it exists. Determine if you're:
- **Creating new:** Use template below
- **Updating:** Add new findings, mark resolved items with strikethrough

### 4.2 Template

```markdown
# Technical Debt: <crate_name>

Last audited: [DATE]

## Summary
- DRY violations: X (Y resolved)
- SRP violations: X (Y resolved)
- KISS violations: X (Y resolved)
- Architecture issues: X (Y resolved)
- Critical/High priority: X

## Recent Fixes
- ✅ **[CODE-XXX]** Brief description of fix
  - Resolution: What was done
  - Resolved: [DATE]

---

## DRY Violations

### [DRY-001] Description
- **File:** `path/to/file.rs`
- **Lines:** X-Y (or pattern location)
- **Issue:** What is duplicated
- **Suggested fix:** How to resolve
- **Priority:** High/Medium/Low
- **Estimated effort:** Small/Medium/Large

## SRP Violations

### [SRP-001] Description
- **File:** `path/to/file.rs`
- **Lines:** X-Y
- **Issue:** What responsibilities are mixed
- **Suggested fix:** How to separate concerns
- **Priority:** High/Medium/Low
- **Estimated effort:** Small/Medium/Large

## KISS Violations

### [KISS-001] Description
- **File:** `path/to/file.rs`
- **Lines:** X-Y
- **Issue:** What is over-complicated
- **Suggested fix:** How to simplify
- **Priority:** High/Medium/Low
- **Estimated effort:** Small/Medium/Large

## Architecture Issues

### [ARCH-001] Description
- **File:** `path/to/file.rs`
- **Issue:** Misplaced or incorrectly scoped
- **Suggested fix:** Where it should be / how to restructure
- **Priority:** High/Medium/Low
- **Estimated effort:** Small/Medium/Large
```

### 4.3 Prioritization Guidelines

**High Priority:**
- Data loss risks
- Stability issues
- API contract violations
- Security concerns

**Medium Priority:**
- Significant maintenance burden
- Clear refactoring path
- Pattern drift from established conventions

**Low Priority:**
- Style inconsistencies
- Minor duplication (<5 lines)
- Documentation gaps

---

## Phase 5: Cross-Reference and Update

### 5.1 Compare with PROJECT_ROADMAP.md

Read `PROJECT_ROADMAP.md` Technical Debt section:
- Are your findings already tracked? → Note existing tracking numbers
- Are your findings new? → Recommend adding to roadmap

### 5.2 Update PROJECT_ROADMAP.md (if needed)

If significant new debt was found (>3 medium+ priority items):
- Add items to appropriate priority section
- Update overall debt counts

---

## Phase 6: Report Summary

Provide a structured summary:

### Statistics
```
Total issues found: X
- DRY: X
- SRP: X
- KISS: X
- Architecture: X

By priority:
- High: X
- Medium: X
- Low: X
```

### Top 3 Priority Items
1. **[CODE-001]** Brief description (High priority)
2. **[CODE-002]** Brief description (Medium priority)
3. **[CODE-003]** Brief description (Medium priority)

### Recommendations
- Quick wins (low effort, high impact)
- Architectural improvements (medium effort, long-term benefit)
- Items to monitor (not urgent but watch for growth)

### Files Created/Updated
- `crates/<crate_name>/TECH_DEBT.md` (created/updated)
- `PROJECT_ROADMAP.md` (updated if new debt found)

---

## Output Format

When complete, provide:
1. **Path to TECH_DEBT.md:** `crates/<crate_name>/TECH_DEBT.md`
2. **Summary statistics** (as shown above)
3. **Top 3 priority issues** with brief context
4. **Recommendations** for next steps
5. **Items added to PROJECT_ROADMAP.md** (if any)
