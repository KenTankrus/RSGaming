use serde::{Deserialize, Serialize};

use crate::models::investment::Investment;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Portfolio {
    pub investments: Vec<Investment>,
}
