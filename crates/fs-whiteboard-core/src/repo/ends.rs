//! `connector_ends`: where the two ends of a connector are.
//!
//! Pattern: Facade method — a reading rule made available beyond the contract (an inherent method,
//! not on the trait): an end on a deleted element stays at that element's last frame.

use fs_objects::ObjectId;

use super::handle::WhiteboardRepo;
use crate::error::{Error, Result};
use crate::model::Kind;
use crate::reading::End;

impl WhiteboardRepo {
    /// The resolved ends of connector `id` on `board`.
    pub async fn connector_ends(&self, board: ObjectId, id: ObjectId) -> Result<(End, End)> {
        let mut g = self.boards.lock().await;
        let st = self.state(&mut g, board).await?;
        let view = st.view();
        match st.elements.get(&id).map(|e| &e.kind) {
            Some(Kind::Connector(c)) => Ok((
                view.end(&st.elements, &c.source),
                view.end(&st.elements, &c.target),
            )),
            Some(_) => Err(Error::invalid("not a connector")),
            None => Err(Error::Foreign(id)),
        }
    }
}
