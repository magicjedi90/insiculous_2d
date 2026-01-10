# Continue Development Workflow

Please proceed with the next task in the project roadmap by following this workflow:

## 1. Identify the Next Task
Read `PROJECT_ROADMAP.md` and identify the next incomplete task or milestone to work on.

## 2. Plan and Document Changes
Before modifying any crate:
- Read the crate's `ANALYSIS.md` file (if it exists)
- Update `ANALYSIS.md` with the planned changes and implementation approach
- If no `ANALYSIS.md` exists, create one documenting the current state and planned modifications

## 3. Implement the Feature
Complete the implementation for the identified task, following the project's coding guidelines and patterns.

## 4. Validate with Example
Once the feature work is complete:
- Add a demonstration of the new functionality to `examples/hello_world.rs`
- Ensure the example compiles and runs correctly with `cargo run --example hello_world`

## 5. Update Documentation
After successful implementation:
- Update relevant `ANALYSIS.md` files to reflect what was implemented
- Update any affected `README.md` files
- Mark the task as complete in `PROJECT_ROADMAP.md` and note any follow-up work identified

## 6. Verify
Run `cargo test` and `cargo build` to ensure everything compiles and tests pass.
