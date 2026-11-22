//! `Secret<T>`: a value that must never be printed, logged or left in memory.

use std::fmt;

use zeroize::Zeroize;

/// A secret, such as a password, a SCRAM salted password or a token key.
///
/// - `Debug` and `Display` print `Secret(***)`, so a secret inside a struct that is
///   logged or printed in a panic message stays hidden.
/// - The value is reachable only through [`Secret::expose_secret`], which makes every
///   use easy to find in review.
/// - The value is zeroized when the `Secret` is dropped. `Clone` exists only when the
///   value can be cloned, and the copy is a `Secret` too, so both copies are zeroized.
///
/// It deliberately has no `Serialize` impl, so a secret can't end up in serialized
/// output, and no `PartialEq` or `Hash`: comparing a secret with `==` takes longer the
/// more leading bytes match, which leaks the value through timing. Code that must
/// compare secrets uses a constant-time comparison.
///
/// Kafka's counterpart is `org.apache.kafka.common.config.types.Password` at 4.4.0,
/// whose `toString()` prints `[hidden]`. Unlike `Password`, a `Secret` wipes its value
/// on drop, can't be compared or hashed, and wraps any zeroizable type, not just
/// strings.
///
/// Zeroizing is best effort: it wipes the value's current buffer, not copies made
/// before the value was wrapped (for example when a `String` grew and reallocated).
/// Build the value, then wrap it at once. Secrets come from files or environment
/// variables, never from command-line arguments, which other users can read in `ps`
/// ([security standard](../../../docs/standards/security.md)).
pub struct Secret<T: Zeroize>(T);

impl<T: Zeroize> Secret<T> {
    /// Wraps `value`.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// The secret value. Every call site is a place where the secret is used, so keep
    /// the borrow short and never pass the value to logging or formatting.
    pub fn expose_secret(&self) -> &T {
        &self.0
    }
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl<T: Zeroize + Clone> Clone for Secret<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

impl<T: Zeroize> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use std::fmt::{self, Write};
    use std::sync::Mutex;

    use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};
    use tracing::field::{Field, Visit};
    use tracing::span::{Attributes, Id, Record};
    use tracing::{Event, Metadata, Subscriber};
    use zeroize::Zeroize;

    use super::Secret;

    /// A `tracing` subscriber that writes every event and span field it sees, formatted
    /// as a log line would format it, into one string.
    #[derive(Clone, Default)]
    struct Capture(Arc<Mutex<String>>);

    struct Fields<'a>(&'a mut String);

    impl Visit for Fields<'_> {
        fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
            write!(self.0, "{}={value:?} ", field.name()).unwrap();
        }
    }

    impl Subscriber for Capture {
        fn enabled(&self, _: &Metadata<'_>) -> bool {
            true
        }
        fn new_span(&self, span: &Attributes<'_>) -> Id {
            span.record(&mut Fields(&mut self.0.lock().unwrap()));
            Id::from_u64(1)
        }
        fn record(&self, _: &Id, values: &Record<'_>) {
            values.record(&mut Fields(&mut self.0.lock().unwrap()));
        }
        fn record_follows_from(&self, _: &Id, _: &Id) {}
        fn event(&self, event: &Event<'_>) {
            event.record(&mut Fields(&mut self.0.lock().unwrap()));
        }
        fn enter(&self, _: &Id) {}
        fn exit(&self, _: &Id) {}
    }

    /// Every way a secret could be printed: `{:?}`, `{:#?}`, `{}`, inside a derived
    /// `Debug`, and as `?` and `%` fields of a `tracing` event and span.
    fn every_printing_of(value: &str) -> Vec<String> {
        #[derive(Debug)]
        #[expect(dead_code, reason = "only printed with Debug")]
        struct Login {
            user: &'static str,
            password: Secret<String>,
        }
        let secret = Secret::new(value.to_owned());
        let login = Login {
            user: "alice",
            password: secret.clone(),
        };
        let capture = Capture::default();
        tracing::subscriber::with_default(capture.clone(), || {
            tracing::info!(debug = ?secret, display = %secret, "login");
            let _span = tracing::info_span!("auth", password = ?secret, shown = %secret);
        });
        let captured = capture.0.lock().unwrap().clone();
        vec![
            format!("{secret:?}"),
            format!("{secret:#?}"),
            format!("{secret}"),
            format!("{login:?}"),
            format!("{login:#?}"),
            captured,
        ]
    }

    #[test]
    fn every_printing_is_redacted() {
        let printed = every_printing_of("hunter2");
        assert_eq!(printed[..3], ["Secret(***)", "Secret(***)", "Secret(***)"]);
        assert_eq!(
            printed[3],
            r#"Login { user: "alice", password: Secret(***) }"#
        );
        assert_eq!(
            printed[5],
            "message=login debug=Secret(***) display=Secret(***) password=Secret(***) \
             shown=Secret(***) "
        );
    }

    proptest! {
        // The printed text is the same whatever the secret, so it can't reveal it. The
        // window check states the requirement directly: no run of 4 or more of the
        // secret's characters appears, except runs that are part of the fixed text
        // (a secret containing "cret" matches "Secret(***)" by coincidence).
        #[test]
        fn no_printing_reveals_any_part_of_the_secret(value in any::<String>()) {
            let printed = every_printing_of(&value);
            let fixed = every_printing_of("x");
            prop_assert_eq!(&printed, &fixed);
            let characters: Vec<char> = value.chars().collect();
            for window in characters.windows(4) {
                let run: String = window.iter().collect();
                for (output, template) in printed.iter().zip(&fixed) {
                    prop_assert!(
                        !output.contains(&run) || template.contains(&run),
                        "{:?} appears in {:?}", run, output
                    );
                }
            }
        }
    }

    /// A value that counts how many times it was zeroized.
    #[derive(Clone)]
    struct Tracked(Arc<AtomicUsize>);

    impl Zeroize for Tracked {
        fn zeroize(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn the_value_is_reachable_through_expose_secret() {
        let secret = Secret::new(vec![1_u8, 2, 3]);
        assert_eq!(secret.expose_secret(), &[1, 2, 3]);
    }

    #[test]
    fn dropping_zeroizes_the_value() {
        let count = Arc::new(AtomicUsize::new(0));
        drop(Secret::new(Tracked(Arc::clone(&count))));
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn secrets_in_dropped_containers_are_zeroized() {
        struct Config {
            _password: Secret<Tracked>,
        }
        let count = Arc::new(AtomicUsize::new(0));
        let secret = || Secret::new(Tracked(Arc::clone(&count)));
        drop(vec![secret(), secret(), secret()]);
        assert_eq!(count.load(Ordering::SeqCst), 3);
        drop(Some(Box::new(secret())));
        assert_eq!(count.load(Ordering::SeqCst), 4);
        drop(Config {
            _password: secret(),
        });
        assert_eq!(count.load(Ordering::SeqCst), 5);
    }

    #[test]
    fn a_clone_is_zeroized_too() {
        let count = Arc::new(AtomicUsize::new(0));
        let secret = Secret::new(Tracked(Arc::clone(&count)));
        let copy = secret.clone();
        drop(secret);
        assert_eq!(count.load(Ordering::SeqCst), 1);
        drop(copy);
        assert_eq!(count.load(Ordering::SeqCst), 2);
    }
}
