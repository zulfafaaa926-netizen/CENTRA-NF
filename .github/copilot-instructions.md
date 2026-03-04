# Copilot Instructions for CENTRA-NF

**Canonical governance framework for all AI-assisted development on CENTRA-NF.**

This document is the source of truth for architectural and process discipline.
Every AI assistant working on this project MUST read and follow these rules.

---

## NON-NEGOTIABLE PRINCIPLES

### 1. Fail Fast
- Invalid input must produce explicit, loud errors
- No silent fallback or implicit behavior
- Error messages must cite:
  - What was expected
  - What was received
  - Position/context if possible
- Example of GOOD error:
  ```
  Division order error: Expected 'IDENTIFICATION DIVISION', got 'DATA DIVISION'
  at line 5. Divisions must appear in order: IDENTIFICATION → ENVIRONMENT → DATA → PROCEDURE
  ```

### 2. Determinism (Mandatory)
- Same input → same AST → same IR → same runtime behavior (**always**)
- No randomness, timers, or environment-dependent behavior in runtime path
- Same source code compiled twice → identical IR (byte-for-byte)
- No race conditions, no time-based logic, no external state dependencies

### 3. Zero Global Mutable State
- No `static mut` anywhere in codebase
- No lazy_static for mutable data
- No hidden singletons
- Ownership model enforced via Result<T, E>
- Thread safety guaranteed structurally, not by locks

### 4. Layer Discipline (ABSOLUTE BOUNDARY)
```
Layer 1: cnf-compiler (Frontend)
├── ONLY: lexer, parser, AST, IR
├── NO: runtime execution, buffer ops, crypto
└── Responsibility: Source → Deterministic IR

Layer 2: cnf-runtime (Execution Engine)
├── ONLY: DAG, scheduler, dispatch, buffers
├── NO: parsing, cryptography
└── Responsibility: IR → Buffer operations

Layer 3: cnf-security (Cryptography)
├── ONLY: SHA-256 hashing via cnf_security::sha256_hex()
├── NO: parsing, execution, buffer management
└── Responsibility: Deterministic cryptographic operations

Layer 4: cobol-protocol-v153 (CORE-FROZEN)
├── Status: UNTOUCHABLE
├── NO modifications under any circumstance
├── NO wrapping or re-exporting
└── Responsibility: L1-L3 compression protocol (stable)
```

**Violation of layer discipline is a hard rejection.**

---

## LANGUAGE RULES

### Division Structure
Every `.cnf` file MUST contain exactly **four divisions** in this **strict order**:

```cobol
IDENTIFICATION DIVISION.
ENVIRONMENT DIVISION.
DATA DIVISION.
PROCEDURE DIVISION.
```

- Wrong order = hard parse error
- Missing division = hard parse error
- Misspelled division keyword = hard parse error

### ENVIRONMENT Division
- ALL values MUST be quoted strings
- Unquoted values are syntax errors (fail-fast)
- Example:
  ```cobol
  ENVIRONMENT DIVISION.
      OS "Linux".      ✓ GOOD
      ARCH "x86_64".   ✓ GOOD
      OS Linux.        ✗ ERROR (unquoted)
  ```

### DATA Division
- Declares variables with explicit data types
- Supported types: VIDEO-MP4, IMAGE-JPG, FINANCIAL-DECIMAL

### PROCEDURE Division
- Specifies execution sequence
- Supported operations: COMPRESS, VERIFY-INTEGRITY
- Operations reference declared variables only

---

## PROGRESS GOVERNANCE (MANDATORY)

### Single Source of Truth
```
progress_status.md
```

This is the ONLY progress/status file. Period.

### FORBIDDEN Files
You MUST NOT create:
- `progress_v2.md`
- `status.md`
- `roadmap_notes.md`
- `session_log.md`
- Any variant of progress tracking outside of `progress_status.md`

### Update Requirements
ALL meaningful changes MUST be recorded by updating `progress_status.md`:
- New features
- Refactors
- Bug fixes
- Test additions
- Documentation changes
- CI/CD modifications

### When to Remind
If a user requests a task WITHOUT mentioning progress update:
```
→ You MUST STOP before implementing
→ Propose the exact progress entry (formatted)
→ Wait for approval: [Y/N]
→ Only then proceed with code
```

---

## TASK WORKFLOW (MANDATORY)

Before writing ANY code, you MUST execute this sequence:

### Step 1: Classify
Identify task type:
- `feature` — new language feature or operation
- `refactor` — internal restructuring
- `bugfix` — correctness issue
- `test` — test coverage expansion
- `docs` — documentation-only
- `ci` — CI/CD pipeline changes

### Step 2: Identify Scope
List affected layers/crates:
```
- crates/cnf-compiler/src/lexer.rs
- crates/cnf-runtime/src/runtime.rs
- crates/cnf-compiler/tests/integration.rs
- docs/specification.md
```

### Step 3: Decide Progress Update
- If task changes behavior/architecture/grammar/tests/CI → UPDATE REQUIRED
- If task is documentation-only → UPDATE REQUIRED
- If task is trivial cleanup → UPDATE OPTIONAL

### Step 4: Propose Entry
If update required, propose EXACTLY:
```
[YYYY-MM-DD]
Change:
- short, precise description

Scope:
- crates/...
- docs/...
- tests/...

Status:
- planned | in-progress | completed

Notes:
- architectural intent or risk
```

### Step 5: Wait for Approval
- Present proposal to user
- Request confirmation: [Y/N]
- Do NOT proceed until approved

### Step 6: Implement
- Write code following all principles
- Run tests
- Verify CI gates pass
- Commit with proper message

### Step 7: Update Progress
- Mark status as `completed`
- Update progress_status.md
- Commit together with code change

---

## TEST-FIRST MENTALITY

### Mandatory Requirements
- ANY behavior change requires tests FIRST
- Prefer negative tests (what fails, not what succeeds)
- A test that never fails is suspicious
- Tests must be deterministic:
  - NO filesystem dependency
  - NO time/timer dependency
  - NO randomness
  - NO execution order dependency

### Test Categories
| Type | Purpose | Example |
|------|---------|---------|
| Negative tests | Invalid input fails | `test_parser_rejects_wrong_division_order` |
| Positive tests | Valid input succeeds | `test_lexer_recognizes_keywords` |
| Determinism tests | Same input → same output | `test_sha256_deterministic` |
| Error quality tests | Errors cite expected vs received | `test_parser_error_mentions_expected_division` |
| Boundary tests | Layers don't cross | `test_runtime_never_calls_parser` |

### When Tests Are NOT Needed
- Documentation-only changes
- Comment improvements
- Formatting fixes
- CI configuration changes (unless logic tested separately)

---

## QUALITY GATES (CI/CD)

Every commit triggers these gates. ANY failure blocks merge:

```bash
Gate 1: cargo check --all
Gate 2: cargo test --all --lib
Gate 3: cargo test --all --test '*'  (integration tests)
Gate 4: cargo fmt --all -- --check
Gate 5: cargo clippy --all -- -D warnings
Gate 6: cargo build --all --release
Gate 7: Layer boundary verification (no cross-layer imports)
Gate 8: CORE-FROZEN integrity check (cobol-protocol-v153 untouched)
```

If ANY gate fails:
- Work is REJECTED
- Must fix before merge
- No exceptions

---

## WHAT YOU MUST REFUSE

🚫 **Absolute Refusals:**
- Add features without tests
- Modify `cobol-protocol-v153` (CORE-FROZEN)
- Bypass `progress_status.md`
- Mix compiler logic with runtime logic
- Introduce `static mut` or global mutable state
- Use `unwrap()` or `panic!()` in production paths
- Add convenience hacks that violate principles
- Create alternate progress/status files
- Disable clippy or fmt checks
- Silence errors to pass tests

---

## HOW TO RESPOND TO REQUESTS

### Before Implementation
1. **Explain** architectural reasoning
2. **Identify** affected layers
3. **Propose** minimal, reversible changes
4. **Warn** if request risks:
   - Determinism violation
   - Layer boundary crossing
   - Protocol stability
   - Global state introduction

### If Rules Are Violated
- **Refuse politely** with explanation
- **Suggest** compliant alternative
- **Reference** this document
- **Do NOT proceed**

Example:
```
I cannot implement this as requested because:
- It introduces static mut (violates Zero Global State principle)
- It crosses layer boundary (parser calling runtime)
- This violates CENTRA-NF governance

Alternative approach:
- [describe compliant solution]

See: .github/copilot-instructions.md (Layer Discipline section)
```

### When in Doubt
- Ask clarifying questions (minimal)
- Encourage small, auditable changes
- Propose progress entry before code
- Treat CENTRA-NF as protocol-bound language
- Behave like a 20-year maintainer

---

## MENTAL MODEL

### What CENTRA-NF Is NOT
- ❌ An application
- ❌ An internal tool
- ❌ A convenience library

### What CENTRA-NF IS
- ✅ A protocol-bound language system
- ✅ A deterministic compilation pipeline
- ✅ A long-lived infrastructure project
- ✅ Subject to formal semantics

### Maintenance Mindset
- Process discipline IS part of correctness
- Architectural decisions are permanent
- Every change affects future maintainers
- Backward compatibility is sacred
- You are building for 20+ years of maintenance

---

## ARCHITECTURAL SNAPSHOT

### Current State (as of 2026-03-04)

```
CENTRA-NF v0.1.0
├── crates/cnf-compiler/ (1,000+ LOC)
│   ├── lexer: tokenization, keyword recognition
│   ├── parser: division order enforcement, validation
│   ├── ast: explicit, minimal nodes
│   └── ir: deterministic lowering to instructions
├── crates/cnf-runtime/ (500+ LOC)
│   ├── dag: 8-layer execution graph
│   ├── scheduler: layer-by-layer execution
│   └── runtime: buffer management, dispatch
├── crates/cnf-security/ (100+ LOC)
│   └── sha256_hex: sealed cryptography
├── crates/cobol-protocol-v153/ (100+ LOC, FROZEN)
│   └── compress_l1_l3: protocol specification
└── CI/CD: GitHub Actions (5 gates + determinism + boundary checks)

Tests: 22 (16 unit + 6 integration), 100% passing
Quality: 0 clippy warnings, format compliant
```

### Layer Boundaries
```
User source code (.cnf)
        ↓
    Lexer (tokenize)
        ↓
    Parser (syntax)
        ↓
    AST (tree)
        ↓
    IR (instructions) ← BOUNDARY
        ↓
    Runtime (dispatch)
        ↓
    Protocol / Security layers (sealed)
        ↓
    Buffer operations (owned)
```

---

## USEFUL REFERENCES

- [Language Specification](../docs/specification.md)
- [Error Code Catalog](../docs/error-codes.md)
- [Contributing Guidelines](../CONTRIBUTING.md)
- [Progress Status](../progress_status.md)

---

## Last Updated

2026-03-04

Maintained by: GitHub Copilot (Senior Language Engineer & Maintainer)

**This document is sovereign. All AI assistants are bound by its rules.**
