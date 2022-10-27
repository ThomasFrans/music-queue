# Queue implementation

A queue is a type that manages the playback order of the items inside it. It can
manage two different types of items:

1. Single items:
    These items are items like tracks, podcast episodes, sound effects... They
    don't offer special functionality, they can just be managed by the queue.

2. Collection items:
    These items are collections like albums, playlists, podcasts... They offer
    special functionality, like the ability to individualy shuffle them.

The queue can receive new items, and the user of the queue can ask for the
currently playing item.
