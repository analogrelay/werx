## ADDED Requirements

### Requirement: Shared trash utility
Werx SHALL provide a shared `branch_trash()` function in `src/trash.rs` that any werx subsystem MUST use when automatically removing a branch. Direct branch deletion (without trashing) is prohibited for any automated operation.

#### Scenario: Utility is the single point of automated branch removal
- **WHEN** any werx command automatically removes a branch
- **THEN** it uses `branch_trash()` rather than deleting the branch ref directly

---

### Requirement: Trash branch naming
The `branch_trash()` utility SHALL move a branch to a new ref named `werx/trash/<original>/<YYYYMMDD>`, where `<original>` is the full original branch name (including any slashes) and `<YYYYMMDD>` is the date the trash operation occurred.

#### Scenario: Simple branch name
- **WHEN** trashing a branch named `my-feature`
- **THEN** the new branch name is `werx/trash/my-feature/20260302`

#### Scenario: Branch name with slashes
- **WHEN** trashing a branch named `feature/my-feature`
- **THEN** the new branch name is `werx/trash/feature/my-feature/20260302`

#### Scenario: Date is caller-supplied
- **WHEN** `branch_trash()` is called with a specific date string
- **THEN** that exact date string is used in the trash branch name (enabling deterministic tests without mocking system time)

---

### Requirement: Collision handling
If a trash branch with the computed name already exists (i.e., the same branch was trashed more than once on the same date), the `branch_trash()` utility SHALL append a monotonically increasing numeric suffix to make the name unique (`werx/trash/<original>/<YYYYMMDD>-2`, `-3`, etc.).

#### Scenario: No collision — no suffix
- **WHEN** no trash branch already exists for the given name and date
- **THEN** the trash branch is created with no suffix

#### Scenario: Collision on first attempt
- **WHEN** `werx/trash/<original>/<YYYYMMDD>` already exists
- **THEN** the branch is created as `werx/trash/<original>/<YYYYMMDD>-2`

#### Scenario: Multiple collisions
- **WHEN** `werx/trash/<original>/<YYYYMMDD>` and `werx/trash/<original>/<YYYYMMDD>-2` already exist
- **THEN** the branch is created as `werx/trash/<original>/<YYYYMMDD>-3`

---

### Requirement: Trash branch is a valid git ref
After `branch_trash()` completes, the trashed branch SHALL exist as a regular git branch ref and SHALL be recoverable by the user via standard git commands (e.g., `git checkout werx/trash/<original>/<YYYYMMDD>`).

#### Scenario: Trashed branch is accessible
- **WHEN** a branch has been trashed
- **THEN** the trash branch ref exists and points to the same commit the original branch pointed to before trashing

#### Scenario: Original branch name is gone
- **WHEN** a branch has been trashed
- **THEN** the original branch name no longer exists as a ref

---

### Requirement: Trash utility return value
The `branch_trash()` function SHALL return the final trash branch name that was created (including any collision suffix), so callers can report it to the user.

#### Scenario: Return value reflects actual name used
- **WHEN** `branch_trash()` creates `werx/trash/feature/my-feature/20260302-2` due to a collision
- **THEN** the function returns the string `"werx/trash/feature/my-feature/20260302-2"`

---

### Requirement: Error on missing branch
If the branch to be trashed does not exist, `branch_trash()` SHALL return an error. It SHALL NOT silently succeed.

#### Scenario: Branch not found
- **WHEN** `branch_trash()` is called with a branch name that does not exist
- **THEN** an error is returned describing the missing branch
