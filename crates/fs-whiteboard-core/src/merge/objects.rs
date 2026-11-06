//! Merge of two stored objects (`apply_remote` writes the result).
//!
//! Pattern: Adapter — reads both contents as documents, merges them, and builds the stored form
//! again. Envelope times play no part; the envelope is rebuilt from the merged document.

use fs_store::StoredObject;

use super::{merge_board, merge_element};
use crate::consts::{BOARD_TYPE, ELEMENT_TYPE};
use crate::doc::{board_to_stored, element_to_stored, stored_to_board, stored_to_element};
use crate::error::{Error, Result};

/// The merge of a local and a remote version of the same object. `instance` and `actor` fill the
/// envelope, as when the object was first written.
pub fn merge_objects(
    local: &StoredObject,
    remote: &StoredObject,
    instance: &str,
    actor: &str,
) -> Result<StoredObject> {
    if local.envelope.id != remote.envelope.id
        || local.envelope.object_type != remote.envelope.object_type
    {
        return Err(Error::invalid("not versions of the same object"));
    }
    match local.envelope.object_type.as_str() {
        ELEMENT_TYPE => {
            let m = merge_element(&stored_to_element(local)?, &stored_to_element(remote)?);
            element_to_stored(&m, instance, Some(local))
        }
        BOARD_TYPE => {
            let m = merge_board(&stored_to_board(local)?, &stored_to_board(remote)?);
            board_to_stored(&m, local.envelope.id, instance, actor, Some(local))
        }
        other => Err(Error::invalid(format!(
            "{other} is not an object of this program"
        ))),
    }
}
