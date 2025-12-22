use serde::{Deserialize, Serialize};

/// Pagination parameters for list operations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    100
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: usize, limit: usize, offset: usize) -> Self {
        Self {
            items,
            total,
            limit,
            offset,
        }
    }
}
