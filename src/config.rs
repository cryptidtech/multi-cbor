//! Configuration for CBOR deserialization limits and security settings.

/// Configuration for CBOR deserializer to prevent `DoS` attacks.
///
/// This struct provides configurable limits for deserialization to protect against
/// malicious inputs that could consume excessive memory or CPU resources.
///
/// # Examples
///
/// ```
/// use multi_cbor::config::DeserializerConfig;
///
/// let config = DeserializerConfig::default()
///     .max_array_size(1000)
///     .max_map_size(500)
///     .max_recursion_depth(64);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DeserializerConfig {
    /// Maximum number of elements allowed in an array
    max_array_size: Option<usize>,
    /// Maximum number of key-value pairs allowed in a map
    max_map_size: Option<usize>,
    /// Maximum recursion depth for nested structures
    max_recursion_depth: u8,
    /// Maximum number of iterations for indefinite-length structures
    max_indefinite_iterations: Option<usize>,
}

impl Default for DeserializerConfig {
    /// Creates a default configuration with reasonable limits.
    ///
    /// Default limits:
    /// - Max array size: 100,000 elements
    /// - Max map size: 100,000 key-value pairs
    /// - Max recursion depth: 128 levels
    /// - Max indefinite iterations: 100,000
    fn default() -> Self {
        Self {
            max_array_size: Some(100_000),
            max_map_size: Some(100_000),
            max_recursion_depth: 128,
            max_indefinite_iterations: Some(100_000),
        }
    }
}

impl DeserializerConfig {
    /// Creates a new configuration with no limits (use with caution).
    ///
    /// This is potentially dangerous as it allows unbounded resource consumption.
    /// Only use this when you have full control over the input data.
    #[must_use]
    pub const fn unlimited() -> Self {
        Self {
            max_array_size: None,
            max_map_size: None,
            max_recursion_depth: 255,
            max_indefinite_iterations: None,
        }
    }

    /// Creates a strict configuration with conservative limits for untrusted input.
    ///
    /// Strict limits:
    /// - Max array size: 1,000 elements
    /// - Max map size: 1,000 key-value pairs
    /// - Max recursion depth: 32 levels
    /// - Max indefinite iterations: 1,000
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            max_array_size: Some(1_000),
            max_map_size: Some(1_000),
            max_recursion_depth: 32,
            max_indefinite_iterations: Some(1_000),
        }
    }

    /// Sets the maximum number of elements allowed in an array.
    ///
    /// Pass `None` to disable this limit (not recommended for untrusted input).
    #[must_use]
    pub const fn max_array_size(mut self, limit: usize) -> Self {
        self.max_array_size = Some(limit);
        self
    }

    /// Removes the limit on array size (use with caution).
    #[must_use]
    pub const fn unlimited_array_size(mut self) -> Self {
        self.max_array_size = None;
        self
    }

    /// Sets the maximum number of key-value pairs allowed in a map.
    ///
    /// Pass `None` to disable this limit (not recommended for untrusted input).
    #[must_use]
    pub const fn max_map_size(mut self, limit: usize) -> Self {
        self.max_map_size = Some(limit);
        self
    }

    /// Removes the limit on map size (use with caution).
    #[must_use]
    pub const fn unlimited_map_size(mut self) -> Self {
        self.max_map_size = None;
        self
    }

    /// Sets the maximum recursion depth for nested structures.
    ///
    /// Lower values provide better `DoS` protection but may reject legitimate deeply nested data.
    #[must_use]
    pub const fn max_recursion_depth(mut self, depth: u8) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Sets the maximum number of iterations for indefinite-length structures.
    ///
    /// This prevents attackers from sending indefinite-length arrays or maps with
    /// extremely large numbers of elements.
    #[must_use]
    pub const fn max_indefinite_iterations(mut self, limit: usize) -> Self {
        self.max_indefinite_iterations = Some(limit);
        self
    }

    /// Removes the limit on indefinite-length iterations (use with caution).
    #[must_use]
    pub const fn unlimited_indefinite_iterations(mut self) -> Self {
        self.max_indefinite_iterations = None;
        self
    }

    /// Returns the maximum array size limit.
    #[must_use]
    pub const fn get_max_array_size(&self) -> Option<usize> {
        self.max_array_size
    }

    /// Returns the maximum map size limit.
    #[must_use]
    pub const fn get_max_map_size(&self) -> Option<usize> {
        self.max_map_size
    }

    /// Returns the maximum recursion depth.
    #[must_use]
    pub const fn get_max_recursion_depth(&self) -> u8 {
        self.max_recursion_depth
    }

    /// Returns the maximum indefinite iterations limit.
    #[must_use]
    pub const fn get_max_indefinite_iterations(&self) -> Option<usize> {
        self.max_indefinite_iterations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DeserializerConfig::default();
        assert_eq!(config.get_max_array_size(), Some(100_000));
        assert_eq!(config.get_max_map_size(), Some(100_000));
        assert_eq!(config.get_max_recursion_depth(), 128);
        assert_eq!(config.get_max_indefinite_iterations(), Some(100_000));
    }

    #[test]
    fn test_unlimited_config() {
        let config = DeserializerConfig::unlimited();
        assert_eq!(config.get_max_array_size(), None);
        assert_eq!(config.get_max_map_size(), None);
        assert_eq!(config.get_max_recursion_depth(), 255);
        assert_eq!(config.get_max_indefinite_iterations(), None);
    }

    #[test]
    fn test_strict_config() {
        let config = DeserializerConfig::strict();
        assert_eq!(config.get_max_array_size(), Some(1_000));
        assert_eq!(config.get_max_map_size(), Some(1_000));
        assert_eq!(config.get_max_recursion_depth(), 32);
        assert_eq!(config.get_max_indefinite_iterations(), Some(1_000));
    }

    #[test]
    fn test_custom_config() {
        let config = DeserializerConfig::default()
            .max_array_size(500)
            .max_map_size(250)
            .max_recursion_depth(16)
            .max_indefinite_iterations(1000);

        assert_eq!(config.get_max_array_size(), Some(500));
        assert_eq!(config.get_max_map_size(), Some(250));
        assert_eq!(config.get_max_recursion_depth(), 16);
        assert_eq!(config.get_max_indefinite_iterations(), Some(1000));
    }

    #[test]
    fn test_unlimited_modifications() {
        let config = DeserializerConfig::default()
            .unlimited_array_size()
            .unlimited_map_size()
            .unlimited_indefinite_iterations();

        assert_eq!(config.get_max_array_size(), None);
        assert_eq!(config.get_max_map_size(), None);
        assert_eq!(config.get_max_indefinite_iterations(), None);
    }
}
