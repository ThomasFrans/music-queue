use rand::seq::SliceRandom;

use crate::item::QueueItem;
use crate::item::QueueableCollection;
use crate::util::shuffled_vec;

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
#[derive(Debug)]
pub struct Queue<I, C: QueueableCollection> {
    /// Indices showing previously played songs. The history before the
    /// current_item can never change.
    history: Vec<usize>,
    /// If the user went backwards, they are now in the history, and this index
    /// shows where in the history. Can move forwards and backwards!
    history_index: Option<usize>,
    pub repeat_status: Option<RepeatMode>,
    unshuffle_strat: UnshuffleStrategy,
    /// If the queue is shuffled, this contains the playback order.
    shuffle_order: Option<Vec<usize>>,
    /// The index of the currently playing item, if any. Can only move forwards!
    current_item: Option<usize>,
    /// Items is a collection of items that this queue can play.
    items: Vec<QueueItem<I, C>>,
}

impl<I, C: QueueableCollection> From<Vec<QueueItem<I, C>>> for Queue<I, C> {
    fn from(items: Vec<QueueItem<I, C>>) -> Self {
        Queue {
            history: Vec::new(),
            history_index: None,
            repeat_status: None,
            unshuffle_strat: UnshuffleStrategy::PlayUnplayed,
            shuffle_order: None,
            current_item: if items.is_empty() { None } else { Some(0) },
            items,
        }
    }
}

impl <I, C: QueueableCollection> Default for Queue<I, C> {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            history_index: None,
            repeat_status: None,
            shuffle_order: None,
            unshuffle_strat: UnshuffleStrategy::PlayUnplayed,
            current_item: None,
            items: Vec::new(),
        }
    }
}

impl<I, C: QueueableCollection> Queue<I, C> {
    /// Change the current song to the next one in the queue and return whether
    /// the current song was changed.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<(), QueueError> {
        if let Some(ref mut index) = self.current_item {
            // Playing
            if let Some(ref mut history_index) = self.history_index {
                // Going forward through history
                if *history_index + 1 == *index {
                    // Caught back up to the present
                    self.history_index = None;
                    Ok(())
                } else {
                    // Still inside history
                    *history_index += 1;
                    Ok(())
                }
            } else {
                // Not in history, playing normally
                if *index < self.items.len() - 1 {
                    // Not at end of queue
                    if let Some(ref shuffle_indices) = self.shuffle_order {
                        self.history.push(shuffle_indices[*index]);
                    } else {
                        self.history.push(*index);
                    }
                    *index += 1;
                    Ok(())
                } else {
                    // At end of queue
                    Err(QueueError::ReachedEnd)
                }
            }
        } else {
            // Stopped
            Err(QueueError::NotPlaying)
        }
    }

    /// Change the current song to the previous one in the queue and return
    /// whether the current song was changed.
    pub fn previous(&mut self) -> Result<(), QueueError> {
        if let Some(index) = self.current_item {
            if let Some(ref mut history_index) = self.history_index {
                // User already listening to history.
                if *history_index > 0 {
                    *history_index -= 1;
                    Ok(())
                } else {
                    Err(QueueError::ReachedBeginning)
                }
            } else {
                // User went back for the first time.
                if index > 0 {
                    self.history_index = Some(index - 1);
                    Ok(())
                } else {
                    Err(QueueError::ReachedBeginning)
                }
            }
        } else {
            Err(QueueError::NotPlaying)
        }
    }

    /// Gets the currently playing item.
    pub fn get_current_item(&self) -> Result<&QueueItem<I, C>, QueueError> {
        if let Some(index) = self.current_item {
            // Playing
            if let Some(history_index) = self.history_index {
                Ok(&self.items[self.history[history_index]])
            } else {
                if let Some(ref shuffle_indices) = self.shuffle_order {
                    // Shuffled
                    Ok(&self.items[shuffle_indices[index]])
                } else {
                    // Not shuffled
                    Ok(&self.items[index])
                }
            }
        } else {
            // Stopped
            Err(QueueError::NotPlaying)
        }
    }

    pub fn get_items(&self) -> Vec<&QueueItem<I, C>> {
        let mut items: Vec<&QueueItem<I, C>> = Vec::with_capacity(self.items.len());
        if let Some(index) = self.current_item {
            // Playing
            for i in &self.history {
                items.push(&self.items[*i]);
            }
            if let Some(ref shuffle_indices) = self.shuffle_order {
                // Shuffled
                for i in index..shuffle_indices.len() {
                    items.push(&self.items[shuffle_indices[i]]);
                }
            } else {
                // Not shuffled
                for i in index..self.items.len() {
                    items.push(&self.items[i]);
                }
            }
        } else {
            // Not playing
            if let Some(ref shuffle_indices) = self.shuffle_order {
                // Shuffled
                for index in 0..self.items.len() {
                    items[index] = &self.items[shuffle_indices[index]];
                }
            } else {
                // Not shuffled
                for item in &self.items {
                    items.push(item);
                }
            }
        }
        items
    }

    /// Clear the queue.
    pub fn clear(&mut self) {
        self.items.clear();
        self.current_item = None;
    }

    /// Return whether the queue is shuffled.
    #[inline]
    pub fn is_shuffled(&self) -> bool {
        self.shuffle_order.is_some()
    }

    /// (Re)shuffle the queue.
    ///
    /// `shuffle_order`:
    /// \[0, 1, 2, 3, 4, 5]
    /// -------^
    /// can become
    /// \[0, 1, 2, 5, 3, 4]
    /// -------^
    ///
    /// `shuffle_order`:
    /// \[0, 1, 2, 3, 4, 5]
    /// -^
    /// can become
    /// \[0, 4, 2, 5, 3, 1]
    /// -^
    ///
    /// `shuffle_order`:
    /// \[0, 1, 2, 3, 4, 5]
    /// ----------------^
    /// becomes
    /// \[0, 1, 2, 3, 4, 5]
    /// ----------------^
    pub fn shuffle(&mut self) {
        if let Some(index) = self.current_item {
            // Playing
            if index < self.items.len() - 1 {
                // We should shuffle
                if let Some(ref mut shuffle_indices) = self.shuffle_order {
                    // Shuffled
                    shuffle_indices[index+1..].shuffle(&mut rand::thread_rng());
                } else {
                    // Not shuffled
                    let mut shuffle_indices: Vec<usize> = (0..self.items.len()).collect();
                    shuffle_indices[index+1..].shuffle(&mut rand::thread_rng());
                    self.shuffle_order = Some(shuffle_indices);
                }
            }
        } else {
            // Not playing
            self.shuffle_order = Some(shuffled_vec(self.items.len()));
        }
    }

    /// Unshuffle the queue.
    /// See [UnshuffleStrategy] for all the options.
    pub fn unshuffle(&mut self) {
        if let Some(index) = self.current_item {
            // Playing
            if let Some(ref mut shuffle_indices) = self.shuffle_order {
                // Shuffled
                match self.unshuffle_strat {
                    UnshuffleStrategy::PlayUnplayed => {
                        if index != self.items.len() - 1 {
                            // If not at the last item, otherwise shuffling
                            // isn't needed!
                            shuffle_indices[index+1..].sort();
                        }
                    }
                    UnshuffleStrategy::KeepIndex => {
                        todo!()
                    }
                    UnshuffleStrategy::KeepRawIndex => {
                        for i in index+1..self.items.len() {
                            shuffle_indices[i] = i;
                        }
                    }
                    UnshuffleStrategy::FromBeginning => {
                        todo!()
                    }
                }
            }
        } else {
            // Not playing
            self.shuffle_order = None;
        }
    }

    /// Toggle shuffle.
    pub fn toggle_shuffle(&mut self) {
        if self.shuffle_order.is_some() {
            self.unshuffle();
        } else {
            self.shuffle();
        }
    }

    #[inline]
    pub fn is_playing(&self) -> bool {
        self.current_item.is_some()
    }
}

/// The mode that is used to repeat the queue playback.
#[derive(Debug)]
pub enum RepeatMode {
    /// Repeat all the items in the queue when the queue reaches the end.
    All,
    /// Repeat the currently playing container when it ends.
    /// When the currently playing item is a song, this will behave like
    /// RepeatMode::Item.
    Container,
    /// Repeat the currently playing item when it ends.
    Item,
}

#[derive(Debug)]
pub enum UnshuffleStrategy {
    /// Order all the unplayed songs in order. This doesn't preserve the
    /// original order, so songs might play out of order from how they were
    /// added, depending on if songs between them already played before.
    ///
    /// \[7, 3, 5, 1, 2, 0, 4, 6]
    /// -------^
    /// becomes
    /// \[7, 3, 5, 0, 1, 2, 4, 6]
    /// -------^
    PlayUnplayed,
    /// Keep playing from the current interpreted index. This may skip a lot of
    /// songs if the current song happens to be at the end of the Queue.
    ///
    /// \[7, 3, 5, 1, 2, 0, 4, 6]
    /// -------^
    /// becomes
    /// \[7, 3, 5, 6, 7]
    /// -------^
    KeepIndex,
    /// Keep playing from the current raw index (the amount of songs played).
    /// This may replay songs that already played, and may skip songs, if items
    /// before this index happened to be past the current index in the shuffled
    /// queue.
    ///
    /// \[7, 3, 5, 1, 2, 0, 4, 6]
    /// -------^
    /// becomes
    /// \[7, 3, 5, 3, 4, 5, 6, 7]
    /// -------^
    KeepRawIndex,
    /// Restart the Queue and pretend nothing happend. 
    /// "Sometimes the smartest move is to start from the beginning ;)"
    ///
    /// \[7, 3, 5, 1, 2, 0, 4, 6]
    /// -------^
    /// becomes
    /// \[7, 3, 5, 0, 1, 2, 3, 4, 5, 6, 7]
    /// -------^
    FromBeginning,
}

/// Errors specific to the Queue.
#[derive(Debug)]
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
    pub struct Track {
        pub id: u32,
    }

    #[derive(Debug)]
    pub struct Episode {
        pub id: u32,
    }

    #[derive(Debug)]
    pub enum CollectionItem {
        Album(Album),
        Playlist(Playlist),
    }

    impl QueueableCollection for CollectionItem {
        type Item = CollectionItem;

        fn get_at_index(&self, index: usize) -> &Self::Item {
            todo!()
        }

        fn get_at_index_raw(&self, index: usize) -> &Self::Item {
            todo!()
        }

        fn shuffle(&mut self) {
            todo!()
        }

        fn unshuffle(&mut self) {
            todo!()
        }

        fn toggle_shuffle(&mut self) {
            todo!()
        }
    }

    #[derive(Debug)]
    pub enum SingleItem {
        Track(Track),
        Episode(Episode),
    }

    /// Simple test with only single items, to test the most basic
    /// functionality.
    #[test]
    pub fn simple_queue_test() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Episode(Episode {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
        ]);

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Track(Track {id: 1})))
        ));

        assert!(queue.next().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Track(Track {id: 2})))
        ));

        assert!(queue.next().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Episode(Episode {id: 3})))
        ));

        assert!(queue.next().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Track(Track {id: 4})))
        ));

        assert!(queue.next().is_err());

        assert!(queue.previous().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Episode(Episode {id: 3})))
        ));

        assert!(queue.previous().is_ok());
        assert!(queue.previous().is_ok());
        assert!(queue.previous().is_err());

        queue.clear();

        assert!(matches!(
            queue.get_current_item(),
            Err(QueueError::NotPlaying)
        ))
    }

    /// Test with only single items if the shuffle works correctly.
    #[test]
    fn shuffled_queue_test() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Episode(Episode {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
        ]);
        queue.shuffle_order = Some(vec![2, 3, 0, 1]);

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Episode(Episode {id: 3})))
        ));

        assert!(queue.next().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Track(Track {id: 4})))
        ));

        assert!(queue.next().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Track(Track {id: 1})))
        ));


        assert!(queue.next().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Track(Track {id: 2})))
        ));

        assert!(queue.next().is_err());

        assert!(queue.previous().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Track(Track {id: 1})))
        ));

        assert!(queue.previous().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Track(Track {id: 4})))
        ));

        assert!(queue.previous().is_ok());

        assert!(matches!(
            queue.get_current_item(),
            Ok(QueueItem::Single(SingleItem::Episode(Episode {id: 3})))
        ));

        assert!(queue.previous().is_err());

        queue.clear();

        assert!(matches!(
            queue.get_current_item(),
            Err(QueueError::NotPlaying)
        ))
    }

    #[test]
    fn unshuffle_simple_queue() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 0})),
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Track(Track {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
            QueueItem::Single(SingleItem::Track(Track {id: 5})),
            QueueItem::Single(SingleItem::Track(Track {id: 6})),
            QueueItem::Single(SingleItem::Track(Track {id: 7})),
        ]);

        queue.shuffle_order = Some(vec![5, 2, 7, 1, 0, 3, 4, 6]);
        queue.unshuffle();
        assert_eq!(queue.shuffle_order, Some(vec![5, 0, 1, 2, 3, 4, 6, 7]));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 0})))));

        queue.previous().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 5})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 0})))));

        queue.previous().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 5})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 0})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 1})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 2})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 3})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 4})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 6})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 7})))));

        assert!(queue.next().is_err());

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 7})))));
    }

    #[test]
    fn shuffle_unshuffle() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 0})),
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Track(Track {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
            QueueItem::Single(SingleItem::Track(Track {id: 5})),
            QueueItem::Single(SingleItem::Track(Track {id: 6})),
            QueueItem::Single(SingleItem::Track(Track {id: 7})),
        ]);

        queue.shuffle_order = Some(vec![3, 1, 7, 2, 6, 4, 5, 0]);

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 3})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 1})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 7})))));

        queue.unshuffle();
        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 0})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 2})))));
    }

    #[test]
    fn unshuffle_strat_keep_raw_index() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 0})),
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Track(Track {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
            QueueItem::Single(SingleItem::Track(Track {id: 5})),
            QueueItem::Single(SingleItem::Track(Track {id: 6})),
            QueueItem::Single(SingleItem::Track(Track {id: 7})),
        ]);

        queue.shuffle_order = Some(vec![3, 1, 7, 2, 6, 4, 5, 0]);
        queue.unshuffle_strat = UnshuffleStrategy::KeepRawIndex;

        queue.unshuffle();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 3})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 1})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 2})))));
    }

    #[test]
    fn unshuffle_strat_keep_raw_index2() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 0})),
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Track(Track {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
            QueueItem::Single(SingleItem::Track(Track {id: 5})),
            QueueItem::Single(SingleItem::Track(Track {id: 6})),
            QueueItem::Single(SingleItem::Track(Track {id: 7})),
        ]);

        queue.shuffle_order = Some(vec![3, 1, 7, 2, 6, 4, 5, 0]);
        queue.unshuffle_strat = UnshuffleStrategy::KeepRawIndex;

        queue.next().unwrap();
        queue.next().unwrap();
        queue.next().unwrap();
        
        // Should be on index 3

        queue.unshuffle();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 2})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 4})))));

        queue.next().unwrap();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 5})))));
    }

    #[test]
    fn unshuffle_strat_keep_raw_index3() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 0})),
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Track(Track {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
            QueueItem::Single(SingleItem::Track(Track {id: 5})),
            QueueItem::Single(SingleItem::Track(Track {id: 6})),
            QueueItem::Single(SingleItem::Track(Track {id: 7})),
        ]);

        queue.shuffle_order = Some(vec![3, 1, 7, 2, 6, 4, 5, 0]);
        queue.unshuffle_strat = UnshuffleStrategy::KeepRawIndex;

        queue.next().unwrap();
        queue.next().unwrap();
        queue.next().unwrap();
        queue.next().unwrap();
        queue.next().unwrap();
        queue.next().unwrap();
        queue.next().unwrap();
        assert!(queue.next().is_err());
        
        // Should be on index 7

        queue.unshuffle();

        assert!(matches!(queue.get_current_item(), Ok(QueueItem::Single(SingleItem::Track(Track {id: 0})))));
    }

    #[test]
    fn get_items_playing_start() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 0})),
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Track(Track {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
            QueueItem::Single(SingleItem::Track(Track {id: 5})),
            QueueItem::Single(SingleItem::Track(Track {id: 6})),
            QueueItem::Single(SingleItem::Track(Track {id: 7})),
        ]);


        for i in 0..queue.items.len() {
            assert!(matches!(queue.get_items()[i], &QueueItem::Single(SingleItem::Track(Track {id: i}))));
            if i < queue.items.len() - 1 {
                queue.next().unwrap();
            }
        }
    }

    #[test]
    fn get_items_shuffled_playing_start() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 0})),
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Track(Track {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
            QueueItem::Single(SingleItem::Track(Track {id: 5})),
            QueueItem::Single(SingleItem::Track(Track {id: 6})),
            QueueItem::Single(SingleItem::Track(Track {id: 7})),
        ]);

        queue.shuffle_order = Some(vec![3, 1, 7, 2, 6, 4, 5, 0]);

        assert!(matches!(queue.get_items()[0], QueueItem::Single(SingleItem::Track(Track {id: 3}))));
        assert!(matches!(queue.get_items()[1], QueueItem::Single(SingleItem::Track(Track {id: 1}))));
        assert!(matches!(queue.get_items()[2], QueueItem::Single(SingleItem::Track(Track {id: 7}))));
        assert!(matches!(queue.get_items()[3], QueueItem::Single(SingleItem::Track(Track {id: 2}))));
        assert!(matches!(queue.get_items()[7], QueueItem::Single(SingleItem::Track(Track {id: 0}))));
    }

    #[test]
    fn get_items_shuffled_playing_middle() {
        let mut queue: Queue<SingleItem, CollectionItem> = Queue::from(vec![
            QueueItem::Single(SingleItem::Track(Track {id: 0})),
            QueueItem::Single(SingleItem::Track(Track {id: 1})),
            QueueItem::Single(SingleItem::Track(Track {id: 2})),
            QueueItem::Single(SingleItem::Track(Track {id: 3})),
            QueueItem::Single(SingleItem::Track(Track {id: 4})),
            QueueItem::Single(SingleItem::Track(Track {id: 5})),
            QueueItem::Single(SingleItem::Track(Track {id: 6})),
            QueueItem::Single(SingleItem::Track(Track {id: 7})),
        ]);

        queue.next().unwrap();
        queue.next().unwrap();
        queue.next().unwrap(); // 3

        queue.shuffle_order = Some(vec![0, 1, 2, 3, 6, 4, 7, 5]);

        assert!(matches!(queue.get_items()[0], QueueItem::Single(SingleItem::Track(Track {id: 0}))));
        assert!(matches!(queue.get_items()[1], QueueItem::Single(SingleItem::Track(Track {id: 1}))));
        assert!(matches!(queue.get_items()[2], QueueItem::Single(SingleItem::Track(Track {id: 2}))));
        assert!(matches!(queue.get_items()[3], QueueItem::Single(SingleItem::Track(Track {id: 3}))));
        assert!(matches!(queue.get_items()[4], QueueItem::Single(SingleItem::Track(Track {id: 6}))));
        assert!(matches!(queue.get_items()[7], QueueItem::Single(SingleItem::Track(Track {id: 5}))));
    }
}
