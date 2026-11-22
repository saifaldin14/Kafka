//! Testing a sans-I/O state machine against a simple reference model: apply the same
//! generated operations to both and compare what each observes after every one
//! ([testing standard](../../../../docs/standards/testing.md#state-machine-property-tests)).
//!
//! Generate the operations as a `Vec` with weighted choices (`vec(prop_oneof![...], ..)`)
//! and call [`compare`] in a `proptest!` test. proptest shrinks a failure to the shortest
//! sequence of the simplest operations that still fails, and records its seed in the
//! test file's `proptest-regressions/` entry. Its message lists the operations as `Debug`
//! prints them, which for plain enums is Rust: paste them into a unit test that calls
//! [`compare`] on that slice to keep the failure as a regression test.
//!
//! Operations aren't filtered by the state: one that the current state makes invalid is
//! applied anyway, and the system must reject it as the model does.

use std::fmt::Debug;

use proptest::test_runner::TestCaseError;

/// Applies each operation to the system and to the model, in order, and fails at the
/// first one where what they observe differs. The failure names the step, both
/// observations and the operations up to it.
///
/// # Errors
///
/// A [`TestCaseError`] describing the first difference.
pub fn compare<Operation: Debug, Observation: Debug + PartialEq>(
    operations: &[Operation],
    mut system: impl FnMut(&Operation) -> Observation,
    mut model: impl FnMut(&Operation) -> Observation,
) -> Result<(), TestCaseError> {
    for (index, operation) in operations.iter().enumerate() {
        let (observed, expected) = (system(operation), model(operation));
        if observed != expected {
            let steps = operations.get(..=index).unwrap_or(operations);
            return Err(TestCaseError::fail(format!(
                "step {} of {}, {operation:?}: the system observed {observed:?}, the model {expected:?}\n\
                 operations: {steps:?}",
                index + 1,
                operations.len(),
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use proptest::collection::vec;
    use proptest::prelude::{Just, Strategy, any, prop_oneof, proptest};
    use proptest::test_runner::{Config, TestError, TestRunner};

    use super::compare;

    /// The operations of a toy stack.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Operation {
        Push(u8),
        Pop,
    }

    /// What an operation returns: the length after a push, the value a pop removed.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Observation {
        Length(usize),
        Popped(Option<u8>),
    }

    /// The stack under test, with a bug planted when `planted_bug` is set: its length
    /// is reported one short once it holds three values.
    struct Stack {
        values: Vec<u8>,
        planted_bug: bool,
    }

    impl Stack {
        fn apply(&mut self, operation: Operation) -> Observation {
            match operation {
                Operation::Push(value) => {
                    self.values.push(value);
                    let length = self.values.len();
                    Observation::Length(if self.planted_bug && length == 3 {
                        2
                    } else {
                        length
                    })
                }
                Operation::Pop => Observation::Popped(self.values.pop()),
            }
        }
    }

    /// The model: a plain `Vec` used as a stack.
    fn model(values: &mut Vec<u8>, operation: Operation) -> Observation {
        match operation {
            Operation::Push(value) => {
                values.push(value);
                Observation::Length(values.len())
            }
            Operation::Pop => Observation::Popped(values.pop()),
        }
    }

    fn operations() -> impl Strategy<Value = Vec<Operation>> {
        vec(
            prop_oneof![2 => any::<u8>().prop_map(Operation::Push), 1 => Just(Operation::Pop)],
            0..40,
        )
    }

    fn check(planted_bug: bool) -> Result<(), TestError<Vec<Operation>>> {
        let mut runner = TestRunner::new(Config {
            failure_persistence: None,
            ..Config::default()
        });
        runner.run(&operations(), |operations| {
            let mut stack = Stack {
                values: Vec::new(),
                planted_bug,
            };
            let mut values = Vec::new();
            compare(
                &operations,
                |&operation| stack.apply(operation),
                |&operation| model(&mut values, operation),
            )
        })
    }

    #[test]
    fn a_planted_bug_shrinks_to_the_shortest_failing_sequence() {
        assert_eq!(check(false), Ok(()));
        match check(true) {
            Err(TestError::Fail(reason, minimal)) => {
                assert_eq!(minimal, [Operation::Push(0); 3]);
                assert!(
                    reason
                        .message()
                        .contains("operations: [Push(0), Push(0), Push(0)]"),
                    "{reason}"
                );
            }
            other => panic!("expected the planted bug to be found, got {other:?}"),
        }
    }

    /// The failure above, pasted as a regression test.
    #[test]
    fn the_printed_sequence_replays_as_a_unit_test() {
        let mut stack = Stack {
            values: Vec::new(),
            planted_bug: true,
        };
        let mut values = Vec::new();
        let replay = compare(
            &[Operation::Push(0), Operation::Push(0), Operation::Push(0)],
            |&operation| stack.apply(operation),
            |&operation| model(&mut values, operation),
        );
        assert!(replay.is_err());
    }

    proptest! {
        #[test]
        fn the_correct_stack_matches_the_model(operations in operations()) {
            let mut stack = Stack { values: Vec::new(), planted_bug: false };
            let mut values = Vec::new();
            compare(
                &operations,
                |&operation| stack.apply(operation),
                |&operation| model(&mut values, operation),
            )?;
        }
    }
}
