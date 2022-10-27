use crate::item::QueueItem;
use crate::item::QueueableCollection;

/// An advanced, configurable music queue.
///
/// Features:
/// - Basic queue features
/// - Shuffle
/// - Container shuffle
/// - Unshuffle
/// - Repeat:
///     - Track
///     - Container
///     - All
///     - Off
pub struct Queue<I, C: QueueableCollection> {
    /// Items is a collection of items that this queue can play.
    items: Vec<QueueItem<I, C>>,
    current_item: Option<usize>,
}

impl<I, C: QueueableCollection> From<Vec<QueueItem<I, C>>> for Queue<I, C> {
    fn from(items: Vec<QueueItem<I, C>>) -> Self {
        Queue {
            items,
            current_item: Some(0),
        }
    }
}

impl<I, C: QueueableCollection> Queue<I, C> {
    /// Change the current song to the next one in the queue and return whether
    /// the current song was changed.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<(), QueueError> {
        if let Some(index) = self.current_item {
            if index < self.items.len() - 1{
                self.next_unchecked();
                Ok(())
            } else {
                Err(QueueError::ReachedEnd)
            }
        } else {
            Err(QueueError::NotPlaying)
        }
    }

    /// Change the current song to the next one in the queue.
    pub fn next_unchecked(&mut self) {
        if let Some(ref mut index) = self.current_item {
            *index += 1;
        }
    }

    /// Change the current song to the previous one in the queue and return
    /// whether the current song was changed.
    pub fn previous(&mut self) -> Result<(), QueueError> {
        if let Some(index) = self.current_item {
            if index > 0 {
                self.previous_unchecked();
                Ok(())
            } else {
                Err(QueueError::ReachedBeginning)
            }
        } else {
            Err(QueueError::NotPlaying)
        }
    }

    /// Change the current song to the previous one in the queue.
    pub fn previous_unchecked(&mut self) {
        if let Some(ref mut index) = self.current_item {
            *index -= 1;
        }
    }

    /// Gets the currently playing item.
    pub fn get_current_item(&self) -> Result<&QueueItem<I, C>, QueueError> {
        if let Some(index) = self.current_item {
            Ok(&self.items[index])
        } else {
            Err(QueueError::NotPlaying)
        }
    }

    /// Clear the queue.
    pub fn clear(&mut self) {
        self.items.clear();
        self.current_item = None;
    }

    /// Shuffle the queue.
    pub fn shuffle() {
        todo!()
    }

    /// Unshuffle the queue.
    pub fn unshuffle() {
        todo!()
    }

    /// Toggle shuffle.
    pub fn toggle_shuffle() {
        todo!()
    }

    /// Sets the repeat mode of the queue.
    pub fn set_repeat() {
        todo!()
    }
}

/// Errors specific to the Queue.
pub enum QueueError {
    /// Reached the beginning of the queue, can't go to the previous item.
    ReachedBeginning,
    /// Reached the end of the queue, can't go to the next item.
    ReachedEnd,
    /// The queue isn't playing; the current_item isn't set.
    NotPlaying,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    pub struct Album {}

    #[derive(Debug)]
    pub struct Playlist {}

    #[derive(Debug)]
    pub struct Track {}

    #[derive(Debug)]
    pub struct Episode {}

    #[derive(Debug)]
    pub enum CollectionItem {
        Album(Album),
        Playlist(Playlist),
    }

    impl QueueableCollection for CollectionItem {}

    #[derive(Debug)]
    pub enum SingleItem {
        Track(Track),
        Episode(Episode),
    }

    #[test]
    pub fn simple_queue_test() {
        let mut queue = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {})),
            QueueItem::Collection(CollectionItem::Album(Album {})),
            QueueItem::Single(SingleItem::Episode(Episode {})),
            QueueItem::Collection(CollectionItem::Playlist(Playlist {})),
        ]);

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Track(Track {})))
        ));

        assert!(queue.next().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Collection { .. })
        ));

        assert!(queue.next().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Episode(Episode {})))
        ));

        assert!(queue.next().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Collection { .. })
        ));

        assert!(queue.next().is_err());

        assert!(queue.previous().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Episode(Episode {})))
        ));

        assert!(queue.previous().is_ok());
        assert!(queue.previous().is_ok());
        assert!(queue.previous().is_err());

        queue.clear();

        assert!(matches!(queue.get_current_item(), Err(QueueError::NotPlaying)))
    }
}
