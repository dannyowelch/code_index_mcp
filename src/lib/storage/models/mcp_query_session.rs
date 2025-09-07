use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tracks MCP client sessions and query history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpQuerySession {
    /// Unique session identifier (UUID)
    pub session_id: Uuid,
    /// Name of AI assistant client
    pub client_name: String,
    /// Currently active Code Index
    pub active_index_id: Option<Uuid>,
    /// Session start time
    pub created_at: DateTime<Utc>,
    /// Last query timestamp
    pub last_activity: DateTime<Utc>,
    /// Number of queries in session
    pub query_count: u32,
    /// Session status
    pub status: SessionStatus,
    /// Optional metadata about the client
    pub client_metadata: Option<String>,
}

/// Represents the status of an MCP session
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session is active and accepting queries
    Active,
    /// Session is temporarily inactive but can be resumed
    Inactive,
    /// Session has been terminated
    Terminated,
    /// Session ended due to an error
    Error,
}

/// Query statistics for a session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionStats {
    pub total_queries: u32,
    pub successful_queries: u32,
    pub failed_queries: u32,
    pub avg_response_time_ms: Option<f64>,
    pub most_used_tool: Option<String>,
}

impl McpQuerySession {
    /// Creates a new MCP query session
    pub fn new(client_name: String) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            client_name,
            active_index_id: None,
            created_at: now,
            last_activity: now,
            query_count: 0,
            status: SessionStatus::Active,
            client_metadata: None,
        }
    }

    /// Creates a session with a specific session ID (for restoration)
    pub fn with_session_id(session_id: Uuid, client_name: String) -> Self {
        let now = Utc::now();
        Self {
            session_id,
            client_name,
            active_index_id: None,
            created_at: now,
            last_activity: now,
            query_count: 0,
            status: SessionStatus::Active,
            client_metadata: None,
        }
    }

    /// Sets the active index for this session
    pub fn set_active_index(&mut self, index_id: Uuid) {
        self.active_index_id = Some(index_id);
        self.update_activity();
    }

    /// Clears the active index
    pub fn clear_active_index(&mut self) {
        self.active_index_id = None;
        self.update_activity();
    }

    /// Records a query execution
    pub fn record_query(&mut self) {
        self.query_count += 1;
        self.update_activity();
    }

    /// Updates the last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Sets client metadata
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.client_metadata = Some(metadata);
        self
    }

    /// Terminates the session
    pub fn terminate(&mut self) {
        self.status = SessionStatus::Terminated;
        self.update_activity();
    }

    /// Sets session as inactive
    pub fn set_inactive(&mut self) {
        self.status = SessionStatus::Inactive;
        self.update_activity();
    }

    /// Reactivates an inactive session
    pub fn reactivate(&mut self) {
        if self.status == SessionStatus::Inactive {
            self.status = SessionStatus::Active;
            self.update_activity();
        }
    }

    /// Sets session status to error
    pub fn set_error(&mut self) {
        self.status = SessionStatus::Error;
        self.update_activity();
    }

    /// Validates the MCP query session fields
    pub fn validate(&self) -> Result<(), String> {
        if self.client_name.trim().is_empty() {
            return Err("Client name cannot be empty".to_string());
        }

        if self.created_at > Utc::now() {
            return Err("Created timestamp cannot be in the future".to_string());
        }

        if self.last_activity < self.created_at {
            return Err("Last activity cannot be before creation time".to_string());
        }

        Ok(())
    }

    /// Returns the session duration
    pub fn duration(&self) -> chrono::Duration {
        self.last_activity - self.created_at
    }

    /// Returns true if the session is active and can accept queries
    pub fn can_query(&self) -> bool {
        self.status == SessionStatus::Active && self.active_index_id.is_some()
    }

    /// Returns true if the session has been idle for the given duration
    pub fn is_idle_for(&self, duration: chrono::Duration) -> bool {
        Utc::now() - self.last_activity > duration
    }

    /// Returns the queries per minute rate
    pub fn queries_per_minute(&self) -> f64 {
        let duration_minutes = self.duration().num_minutes() as f64;
        if duration_minutes > 0.0 {
            self.query_count as f64 / duration_minutes
        } else {
            0.0
        }
    }

    /// Returns session statistics
    pub fn basic_stats(&self) -> SessionStats {
        SessionStats {
            total_queries: self.query_count,
            successful_queries: 0, // Would need query log to calculate
            failed_queries: 0,     // Would need query log to calculate
            avg_response_time_ms: None, // Would need timing data
            most_used_tool: None,  // Would need query log to calculate
        }
    }
}

impl SessionStatus {
    /// Returns true if the session can accept new queries
    pub fn can_accept_queries(&self) -> bool {
        matches!(self, SessionStatus::Active)
    }

    /// Returns true if the session is in a final state
    pub fn is_final(&self) -> bool {
        matches!(self, SessionStatus::Terminated | SessionStatus::Error)
    }

    /// Returns string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Active => "active",
            SessionStatus::Inactive => "inactive",
            SessionStatus::Terminated => "terminated",
            SessionStatus::Error => "error",
        }
    }

    /// Returns a description of the status
    pub fn description(&self) -> &'static str {
        match self {
            SessionStatus::Active => "Session is active and accepting queries",
            SessionStatus::Inactive => "Session is temporarily inactive",
            SessionStatus::Terminated => "Session has been terminated",
            SessionStatus::Error => "Session ended due to an error",
        }
    }
}

impl SessionStats {
    /// Returns the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_queries > 0 {
            (self.successful_queries as f64 / self.total_queries as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Returns the error rate as a percentage
    pub fn error_rate(&self) -> f64 {
        if self.total_queries > 0 {
            (self.failed_queries as f64 / self.total_queries as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Builder for querying sessions
#[derive(Debug, Clone)]
pub struct SessionQuery {
    pub client_name_pattern: Option<String>,
    pub status_filter: Option<SessionStatus>,
    pub active_index_id: Option<Uuid>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub min_queries: Option<u32>,
    pub idle_longer_than: Option<chrono::Duration>,
}

impl SessionQuery {
    pub fn new() -> Self {
        Self {
            client_name_pattern: None,
            status_filter: None,
            active_index_id: None,
            created_after: None,
            created_before: None,
            min_queries: None,
            idle_longer_than: None,
        }
    }

    pub fn with_client(mut self, pattern: String) -> Self {
        self.client_name_pattern = Some(pattern);
        self
    }

    pub fn with_status(mut self, status: SessionStatus) -> Self {
        self.status_filter = Some(status);
        self
    }

    pub fn for_index(mut self, index_id: Uuid) -> Self {
        self.active_index_id = Some(index_id);
        self
    }

    pub fn created_after(mut self, timestamp: DateTime<Utc>) -> Self {
        self.created_after = Some(timestamp);
        self
    }

    pub fn with_min_queries(mut self, min: u32) -> Self {
        self.min_queries = Some(min);
        self
    }

    pub fn idle_longer_than(mut self, duration: chrono::Duration) -> Self {
        self.idle_longer_than = Some(duration);
        self
    }
}

impl Default for SessionQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_mcp_query_session_new() {
        let client_name = "Claude".to_string();
        let session = McpQuerySession::new(client_name.clone());

        assert_eq!(session.client_name, client_name);
        assert!(session.active_index_id.is_none());
        assert_eq!(session.query_count, 0);
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.created_at <= Utc::now());
        assert!(session.last_activity <= Utc::now());
        assert!(session.client_metadata.is_none());
    }

    #[test]
    fn test_session_with_session_id() {
        let session_id = Uuid::new_v4();
        let client_name = "GPT-4".to_string();
        let session = McpQuerySession::with_session_id(session_id, client_name.clone());

        assert_eq!(session.session_id, session_id);
        assert_eq!(session.client_name, client_name);
    }

    #[test]
    fn test_active_index_management() {
        let mut session = McpQuerySession::new("Test Client".to_string());
        let index_id = Uuid::new_v4();
        let original_activity = session.last_activity;

        // Sleep a tiny bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        session.set_active_index(index_id);
        assert_eq!(session.active_index_id, Some(index_id));
        assert!(session.last_activity > original_activity);

        session.clear_active_index();
        assert!(session.active_index_id.is_none());
    }

    #[test]
    fn test_query_recording() {
        let mut session = McpQuerySession::new("Test Client".to_string());
        let original_count = session.query_count;
        let original_activity = session.last_activity;

        // Sleep a tiny bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        session.record_query();
        assert_eq!(session.query_count, original_count + 1);
        assert!(session.last_activity > original_activity);
    }

    #[test]
    fn test_status_transitions() {
        let mut session = McpQuerySession::new("Test Client".to_string());

        // Test setting inactive
        session.set_inactive();
        assert_eq!(session.status, SessionStatus::Inactive);

        // Test reactivation
        session.reactivate();
        assert_eq!(session.status, SessionStatus::Active);

        // Test termination
        session.terminate();
        assert_eq!(session.status, SessionStatus::Terminated);

        // Test that terminated session cannot be reactivated
        session.reactivate();
        assert_eq!(session.status, SessionStatus::Terminated);

        // Test error status
        let mut session2 = McpQuerySession::new("Test Client 2".to_string());
        session2.set_error();
        assert_eq!(session2.status, SessionStatus::Error);
    }

    #[test]
    fn test_validation() {
        let mut session = McpQuerySession::new("Valid Client".to_string());
        assert!(session.validate().is_ok());

        // Test empty client name
        session.client_name = "".to_string();
        assert!(session.validate().is_err());

        // Test future created timestamp
        session.client_name = "Valid Client".to_string();
        session.created_at = Utc::now() + chrono::Duration::hours(1);
        assert!(session.validate().is_err());

        // Test last activity before creation
        session.created_at = Utc::now();
        session.last_activity = Utc::now() - chrono::Duration::hours(1);
        assert!(session.validate().is_err());
    }

    #[test]
    fn test_can_query() {
        let mut session = McpQuerySession::new("Test Client".to_string());
        
        // Cannot query without active index
        assert!(!session.can_query());

        // Can query with active index and active status
        session.set_active_index(Uuid::new_v4());
        assert!(session.can_query());

        // Cannot query when inactive
        session.set_inactive();
        assert!(!session.can_query());

        // Cannot query when terminated
        session.terminate();
        assert!(!session.can_query());
    }

    #[test]
    fn test_idle_detection() {
        let mut session = McpQuerySession::new("Test Client".to_string());
        
        // Fresh session is not idle
        assert!(!session.is_idle_for(chrono::Duration::minutes(1)));

        // Artificially set old last activity
        session.last_activity = Utc::now() - chrono::Duration::hours(2);
        assert!(session.is_idle_for(chrono::Duration::minutes(30)));
        assert!(!session.is_idle_for(chrono::Duration::hours(3)));
    }

    #[test]
    fn test_queries_per_minute() {
        let mut session = McpQuerySession::new("Test Client".to_string());
        
        // Set created time to 2 minutes ago
        session.created_at = Utc::now() - chrono::Duration::minutes(2);
        session.last_activity = Utc::now();
        session.query_count = 10;

        let qpm = session.queries_per_minute();
        assert!(qpm >= 4.0 && qpm <= 6.0); // Approximately 5 queries per minute
    }

    #[test]
    fn test_session_status_properties() {
        assert!(SessionStatus::Active.can_accept_queries());
        assert!(!SessionStatus::Inactive.can_accept_queries());
        assert!(!SessionStatus::Terminated.can_accept_queries());
        assert!(!SessionStatus::Error.can_accept_queries());

        assert!(!SessionStatus::Active.is_final());
        assert!(!SessionStatus::Inactive.is_final());
        assert!(SessionStatus::Terminated.is_final());
        assert!(SessionStatus::Error.is_final());
    }

    #[test]
    fn test_session_stats() {
        let stats = SessionStats {
            total_queries: 100,
            successful_queries: 85,
            failed_queries: 15,
            avg_response_time_ms: Some(150.0),
            most_used_tool: Some("search_symbols".to_string()),
        };

        assert_eq!(stats.success_rate(), 85.0);
        assert_eq!(stats.error_rate(), 15.0);

        let empty_stats = SessionStats {
            total_queries: 0,
            successful_queries: 0,
            failed_queries: 0,
            avg_response_time_ms: None,
            most_used_tool: None,
        };

        assert_eq!(empty_stats.success_rate(), 0.0);
        assert_eq!(empty_stats.error_rate(), 0.0);
    }

    #[test]
    fn test_session_query_builder() {
        let query = SessionQuery::new()
            .with_client("Claude*".to_string())
            .with_status(SessionStatus::Active)
            .for_index(Uuid::new_v4())
            .created_after(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap())
            .with_min_queries(10)
            .idle_longer_than(chrono::Duration::minutes(30));

        assert!(query.client_name_pattern.is_some());
        assert_eq!(query.status_filter, Some(SessionStatus::Active));
        assert!(query.active_index_id.is_some());
        assert!(query.created_after.is_some());
        assert_eq!(query.min_queries, Some(10));
        assert!(query.idle_longer_than.is_some());
    }
}