//! `WhiteboardFeed`: the changes of one board.
//!
//! Pattern: Observer — hands out a `Subscription` over the store's change feed; the subscription
//! keeps only the activities of objects on that board.

use async_trait::async_trait;
use fs_store::ObjectId;

use super::handle::WhiteboardRepo;
use crate::error::Result;
use crate::traits::{Subscription, WhiteboardFeed};

#[async_trait]
impl WhiteboardFeed for WhiteboardRepo {
    async fn subscribe(&self, board: ObjectId) -> Result<Subscription> {
        let mut guard = self.boards.lock().await;
        self.state(&mut guard, board).await?;
        Ok(Subscription {
            rx: self.store.subscribe(),
            board,
        })
    }
}
