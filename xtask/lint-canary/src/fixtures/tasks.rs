//! Starting threads and tasks outside the task supervisor.

/// Spawning an OS thread.
pub fn std_thread_spawn() -> std::thread::JoinHandle<()> {
    std::thread::spawn(|| {}) // expect-lint: clippy::disallowed_methods
}

/// Spawning a tokio task through the crate-root re-export.
pub fn tokio_spawn() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async {}) // expect-lint: clippy::disallowed_methods
}

/// Spawning a tokio task through `tokio::task`. It is the same function as
/// `tokio::spawn`, and clippy matches functions rather than paths, so the one
/// `tokio::spawn` entry in clippy.toml bans both spellings.
pub fn tokio_task_spawn() -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async {}) // expect-lint: clippy::disallowed_methods
}
