//! Signal definitions and handling

use serde::{Serialize, de::DeserializeOwned};

/// Signal trait - defines the signal interface
pub trait Signal: Serialize + DeserializeOwned + Send + 'static {
    /// Signal name
    fn name() -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestSignal {
        value: String,
    }

    impl Signal for TestSignal {
        fn name() -> &'static str {
            "test_signal"
        }
    }

    #[test]
    fn test_signal_name() {
        assert_eq!(TestSignal::name(), "test_signal");
    }
}

