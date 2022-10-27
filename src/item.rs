use std::fmt::Debug;

/// A collection that can be queued as a QueueItem.
pub trait QueueableCollection {}

/// A type that can directly be queued.
#[derive(Debug)]
pub enum QueueItem<I, C: QueueableCollection> {
    /// A single item that can be queued, like a track or episode.
    Single(I),
    /// A collection of items that can be queued, and offers some extra
    /// functionality.
    Collection(C),
}
