# Agent Instructions

## Markdown Formatting

When writing project documentation or feedback markdown, keep lines short enough
to read comfortably in a split editor on a 13-inch laptop.

Use these rules:

- Wrap prose at about 80 characters.
- Treat 88 characters as the practical hard limit for normal prose lines.
- Prefer short paragraphs over dense blocks of text.
- Keep code blocks readable; do not wrap code in a way that changes meaning.
- Use headings and flat bullet lists when they improve scanning.
- Avoid deeply nested bullets unless the hierarchy is genuinely necessary.
- Put commands, file names, constants, types, and function names in backticks.
- Keep feedback concrete and tied to the actual code.

Good markdown should be readable both in a rendered view and in a plain text
editor.

## Creating Exercise Sets

This repository uses small, standalone Rust exercises. Each exercise file should
compile independently with `rustc --test`.

Use this process when creating a new exercise set:

1. Create a `README.md` for the step.
2. Explain the overall goal of the step.
3. List each exercise file in order.
4. For each exercise, describe the specific learning goal.
5. Include a single-file run command.
6. Include a command for running all exercises in the directory.
7. Create one `.rs` file per exercise.
8. Keep each `.rs` file independent; do not require importing another exercise.
9. Put the implementation skeleton in normal code.
10. Mark the intended work with `todo!()`.
11. Put the expected behavior in `#[cfg(test)]` tests.
12. Keep tests focused on the concept introduced by that exercise.

The preferred shape is:

```rust
// Step N-M: Short exercise title.
//
// Run:
// rustc --edition=2021 --test NN_exercise_name.rs && ./NN_exercise_name

#![allow(unused)]

// Constants, types, and skeleton implementation here.

fn main() {
    println!("short exercise label");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_expected_behavior() {
        todo!("assert the behavior students should implement");
    }
}
```

Tests should define the contract. The skeleton should be small enough that the
student can infer the next implementation step from the tests, README, and local
type signatures.

## Exercise Design Notes

- Prefer fixed-size, local problems before introducing cross-file complexity.
- Introduce one new idea per exercise when possible.
- Reuse naming and layout patterns from earlier exercises in the same step.
- Keep constants explicit when they are part of the learning goal.
- If an exercise is independent, duplicate the small required definitions.
- Avoid hidden framework code; the student should see the important mechanics.
- Use `Result` in skeleton APIs when failure handling is part of the lesson.
- Make missing-key or empty-result cases explicit in tests.
- Add duplicate/update tests when mutation semantics matter.
- Add reopen or persistence tests when file state matters.

When reviewing a completed exercise, comment on:

- whether the tests pass
- readability and naming
- structure and helper boundaries
- `Result` and error propagation
- method receiver choices such as `self`, `&self`, and `&mut self`
- whether helper return types match the operation's meaning
