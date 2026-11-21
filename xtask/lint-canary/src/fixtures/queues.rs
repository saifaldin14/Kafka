//! Queues without a capacity.

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// Creating an unbounded channel.
pub fn unbounded_channel() -> (UnboundedSender<u8>, UnboundedReceiver<u8>) {
    tokio::sync::mpsc::unbounded_channel() // expect-lint: clippy::disallowed_methods
}
