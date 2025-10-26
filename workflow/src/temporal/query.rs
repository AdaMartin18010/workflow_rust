//! Query definitions and handling

use serde::{Serialize, de::DeserializeOwned};

/// Query trait - defines the query interface
pub trait Query: Send + 'static {
    /// Query name
    fn name() -> &'static str;
    
    /// Result type
    type Result: Serialize + DeserializeOwned + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    struct TestQuery;

    impl Query for TestQuery {
        fn name() -> &'static str {
            "test_query"
        }
        
        type Result = TestQueryResult;
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestQueryResult {
        value: i32,
    }

    #[test]
    fn test_query_name() {
        assert_eq!(TestQuery::name(), "test_query");
    }
}

