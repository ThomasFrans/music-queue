use std::fmt::Debug;

/// A collection that can be queued as a QueueItem.
pub trait QueueableCollection {
    type Item;

    /// Get the item at the given index, taking into account the shuffle status.
    fn get_at_index(&self, index: usize) -> &Self::Item;

    /// Get the item at the given index, not taking into account the shuffle status.
    fn get_at_index_raw(&self, index: usize) -> &Self::Item;

    /// Shuffle the collection.
    fn shuffle(&mut self);

    /// Unshuffle the collection.
    fn unshuffle(&mut self);

    /// Toggle the shuffle status of the collection.
    fn toggle_shuffle(&mut self);
}

pub struct SimpleCollection<T> {
    items: Vec<T>,
    shuffled: bool,
}

impl<T> From<Vec<T>> for SimpleCollection<T> {
    fn from(items: Vec<T>) -> Self {
        Self {
            items,
            shuffled: false,
        }
    }
}

impl<T> QueueableCollection for SimpleCollection<T> {
    type Item = T;

    fn get_at_index(&self, index: usize) -> &Self::Item {
        &self.items[index]
    }

    fn get_at_index_raw(&self, index: usize) -> &Self::Item {
        &self.items[index]
    }

    fn shuffle(&mut self) {
        self.shuffled = true;
    }

    fn unshuffle(&mut self) {
        self.shuffled = false;
    }

    fn toggle_shuffle(&mut self) {
        self.shuffled = !self.shuffled;
    }
}

/// A type that can directly be queued.
#[derive(Debug)]
pub enum QueueItem<I, C: QueueableCollection> {
    /// A single item that can be queued, like a track or episode.
    Single(I),
    /// A collection of items that can be queued, and offers some extra
    /// functionality.
    Collection(C),
}
