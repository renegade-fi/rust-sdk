//! Task API types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A type alias for a task identifier
pub type TaskIdentifier = Uuid;

/// A relayer task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTask {
    /// The task ID
    pub id: TaskIdentifier,
    /// The state of the task
    pub state: String,
    /// The time the task was created, in milliseconds since the epoch
    pub created_at: u64,
    /// The axillary information that specifies the transformation the task took
    pub task_info: ApiTaskDescription,
}

/// A decription of a relayer task
// TODO: Replace w/ actual relayer task types once implemented
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiTaskDescription {
    CreateAccount,
    SyncAccount,
    Deposit,
    PayFee,
    Withdraw,
    CreateOrder,
    UpdateOrder,
    CancelOrder,
    SettleMatch,
}
