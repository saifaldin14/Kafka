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

    use zeroize::Zeroize;

    use super::Secret;

    /// A value that counts how many times it was zeroized.
    #[derive(Clone)]
    struct Tracked(Arc<AtomicUsize>);

    impl Zeroize for Tracked {
        fn zeroize(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn debug_and_display_hide_the_value() {
        #[derive(Debug)]
        #[expect(dead_code, reason = "only printed with Debug")]
        struct Credentials {
            user: String,
            password: Secret<String>,
        }
        let secret = Secret::new("hunter2".to_owned());
        assert_eq!(format!("{secret:?}"), "Secret(***)");
        assert_eq!(secret.to_string(), "Secret(***)");
        let printed = format!(
            "{:?}",
            Credentials {
                user: "alice".to_owned(),
                password: secret,
            }
        );
        assert_eq!(
            printed,
            "Credentials { user: \"alice\", password: Secret(***) }"
        );
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
