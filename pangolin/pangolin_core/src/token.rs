// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Token management and revocation support

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a revoked token in the blacklist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokedToken {
    /// Unique identifier for the token (from JWT jti claim)
    pub token_id: Uuid,
    /// When the token was revoked
    pub revoked_at: DateTime<Utc>,
    /// When the token expires (for cleanup)
    pub expires_at: DateTime<Utc>,
    /// Optional reason for revocation
    pub reason: Option<String>,
}

impl RevokedToken {
    /// Create a new revoked token entry
    pub fn new(token_id: Uuid, expires_at: DateTime<Utc>, reason: Option<String>) -> Self {
        Self {
            token_id,
            revoked_at: Utc::now(),
            expires_at,
            reason,
        }
    }

    /// Check if this revoked token has expired and can be cleaned up
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_revoked_token_creation() {
        let token_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(24);
        let reason = Some("User requested logout".to_string());

        let revoked = RevokedToken::new(token_id, expires_at, reason.clone());

        assert_eq!(revoked.token_id, token_id);
        assert_eq!(revoked.expires_at, expires_at);
        assert_eq!(revoked.reason, reason);
        assert!(!revoked.is_expired());
    }

    #[test]
    fn test_revoked_token_expiration() {
        let token_id = Uuid::new_v4();
        let expires_at = Utc::now() - Duration::hours(1); // Already expired

        let revoked = RevokedToken::new(token_id, expires_at, None);

        assert!(revoked.is_expired());
    }
}
