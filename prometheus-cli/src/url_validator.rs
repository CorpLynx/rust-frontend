use anyhow::Result;
use std::fmt;
use url::Url;

/// URL validation module for HTTPS-only enforcement
/// 
/// This module provides validation functions to ensure all remote backend URLs
/// use HTTPS protocol while allowing HTTP for localhost development environments.
pub struct UrlValidator;

impl UrlValidator {
    /// Validate a backend URL for HTTPS compliance
    /// 
    /// # Arguments
    /// * `url` - The URL string to validate
    /// 
    /// # Returns
    /// * `Ok(())` if the URL is valid (HTTPS for remote, HTTP/HTTPS for localhost)
    /// * `Err(UrlValidationError)` if the URL violates HTTPS requirements
    /// 
    /// # Requirements
    /// * 1.1: Reject remote HTTP URLs with error message
    /// * 1.2: Accept remote HTTPS URLs
    /// * 2.1, 2.2, 2.3: Allow HTTP for localhost URLs
    pub fn validate_backend_url(url: &str) -> Result<(), UrlValidationError> {
        // Handle empty or whitespace-only URLs
        if url.trim().is_empty() {
            return Err(UrlValidationError::EmptyUrl);
        }

        // Parse the URL to extract components
        let parsed_url = match Url::parse(url) {
            Ok(url) => url,
            Err(_) => return Err(UrlValidationError::InvalidFormat { 
                url: url.to_string() 
            }),
        };

        // Check if this is a localhost URL (exempt from HTTPS requirement)
        if Self::is_localhost_url(url) {
            // Localhost URLs can use HTTP or HTTPS
            return Ok(());
        }

        // For remote URLs, enforce HTTPS
        match parsed_url.scheme() {
            "https" => Ok(()),
            "http" => {
                let suggested = Self::suggest_https_url(url);
                Err(UrlValidationError::InvalidProtocol { 
                    url: url.to_string(), 
                    suggested 
                })
            }
            _ => Err(UrlValidationError::InvalidFormat { 
                url: url.to_string() 
            }),
        }
    }

    /// Check if a URL is localhost (exempt from HTTPS requirement)
    /// 
    /// # Arguments
    /// * `url` - The URL string to check
    /// 
    /// # Returns
    /// * `true` if the URL is localhost (localhost, 127.0.0.1, with any port)
    /// * `false` if the URL is a remote endpoint
    /// 
    /// # Requirements
    /// * 2.1: Allow http://localhost
    /// * 2.2: Allow http://127.0.0.1
    /// * 2.3: Allow localhost URLs with port numbers
    pub fn is_localhost_url(url: &str) -> bool {
        match Url::parse(url) {
            Ok(parsed_url) => {
                match parsed_url.host() {
                    Some(url::Host::Domain(host)) => {
                        host == "localhost"
                    }
                    Some(url::Host::Ipv4(addr)) => {
                        addr.is_loopback()
                    }
                    Some(url::Host::Ipv6(addr)) => {
                        addr.is_loopback()
                    }
                    None => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Suggest HTTPS equivalent for HTTP URLs
    /// 
    /// # Arguments
    /// * `url` - The HTTP URL to convert
    /// 
    /// # Returns
    /// * The suggested HTTPS URL string
    /// 
    /// # Requirements
    /// * 3.2: Suggest HTTPS equivalent in error messages
    pub fn suggest_https_url(url: &str) -> String {
        if url.starts_with("http://") {
            url.replacen("http://", "https://", 1)
        } else {
            // If it doesn't start with http://, just prepend https://
            format!("https://{}", url.trim_start_matches("https://"))
        }
    }

    /// Validate and filter a list of saved URLs
    /// 
    /// # Arguments
    /// * `urls` - Vector of URL strings to validate
    /// 
    /// # Returns
    /// * Tuple of (valid_urls, invalid_urls) where invalid URLs are removed
    /// 
    /// # Requirements
    /// * 5.2: Validate each URL's protocol during configuration loading
    /// * 5.3: Remove HTTP URLs and log warnings
    pub fn filter_valid_urls(urls: Vec<String>) -> (Vec<String>, Vec<String>) {
        let mut valid_urls = Vec::new();
        let mut invalid_urls = Vec::new();

        for url in urls {
            match Self::validate_backend_url(&url) {
                Ok(()) => valid_urls.push(url),
                Err(_) => invalid_urls.push(url),
            }
        }

        (valid_urls, invalid_urls)
    }
}

/// Comprehensive error types for URL validation failures
/// 
/// # Requirements
/// * 3.1: Display rejected URL in error message
/// * 3.2: Suggest HTTPS equivalent
/// * 3.3: Explain security requirement
/// * 3.5: Provide appropriate error codes
#[derive(Debug, Clone, PartialEq)]
pub enum UrlValidationError {
    /// HTTP protocol used for remote URL (HTTPS required)
    InvalidProtocol { 
        url: String, 
        suggested: String 
    },
    /// Malformed URL structure
    InvalidFormat { 
        url: String 
    },
    /// Empty or whitespace-only URL
    EmptyUrl,
}

impl fmt::Display for UrlValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UrlValidationError::InvalidProtocol { url, suggested } => {
                write!(f, "Invalid backend URL protocol\n")?;
                write!(f, "  Provided: {}\n", url)?;
                write!(f, "  Required: HTTPS protocol for remote endpoints\n")?;
                write!(f, "  Suggested: {}\n", suggested)?;
                write!(f, "\n")?;
                write!(f, "Security: Remote connections must use HTTPS to encrypt your prompts and responses.\n")?;
                write!(f, "\n")?;
                write!(f, "Examples of valid URLs:\n")?;
                write!(f, "  - https://api.example.com:8080\n")?;
                write!(f, "  - https://my-ollama-server.com\n")?;
                write!(f, "  - http://localhost:11434 (localhost only)")
            }
            UrlValidationError::InvalidFormat { url } => {
                write!(f, "Invalid URL format: {}\n", url)?;
                write!(f, "\n")?;
                write!(f, "Examples of valid URLs:\n")?;
                write!(f, "  - https://api.example.com:8080\n")?;
                write!(f, "  - https://my-ollama-server.com\n")?;
                write!(f, "  - http://localhost:11434")
            }
            UrlValidationError::EmptyUrl => {
                write!(f, "Backend URL cannot be empty\n")?;
                write!(f, "\n")?;
                write!(f, "Examples of valid URLs:\n")?;
                write!(f, "  - https://api.example.com:8080\n")?;
                write!(f, "  - https://my-ollama-server.com\n")?;
                write!(f, "  - http://localhost:11434")
            }
        }
    }
}

impl std::error::Error for UrlValidationError {}

/// URL validation result with detailed context
/// 
/// This struct provides comprehensive information about URL validation
/// results for use in error handling and user feedback.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the URL is valid
    pub is_valid: bool,
    /// Validation error if invalid
    pub error: Option<UrlValidationError>,
    /// Whether the URL is localhost
    pub is_localhost: bool,
    /// Suggested URL if invalid
    pub suggested_url: Option<String>,
}

impl ValidationResult {
    /// Create a validation result for a URL
    /// 
    /// # Arguments
    /// * `url` - The URL to validate
    /// 
    /// # Returns
    /// * ValidationResult with all relevant information
    pub fn for_url(url: &str) -> Self {
        let is_localhost = UrlValidator::is_localhost_url(url);
        
        match UrlValidator::validate_backend_url(url) {
            Ok(()) => ValidationResult {
                is_valid: true,
                error: None,
                is_localhost,
                suggested_url: None,
            },
            Err(error) => {
                let suggested_url = match &error {
                    UrlValidationError::InvalidProtocol { .. } => {
                        Some(UrlValidator::suggest_https_url(url))
                    }
                    _ => None,
                };
                
                ValidationResult {
                    is_valid: false,
                    error: Some(error),
                    is_localhost,
                    suggested_url,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{QuickCheck, TestResult};

    #[test]
    fn test_validate_https_urls() {
        // Valid HTTPS URLs should pass
        assert!(UrlValidator::validate_backend_url("https://api.example.com").is_ok());
        assert!(UrlValidator::validate_backend_url("https://api.example.com:8080").is_ok());
        assert!(UrlValidator::validate_backend_url("https://my-server.com/api").is_ok());
    }

    #[test]
    fn test_reject_http_remote_urls() {
        // Remote HTTP URLs should be rejected
        let result = UrlValidator::validate_backend_url("http://api.example.com");
        assert!(result.is_err());
        
        if let Err(UrlValidationError::InvalidProtocol { url, suggested }) = result {
            assert_eq!(url, "http://api.example.com");
            assert_eq!(suggested, "https://api.example.com");
        } else {
            panic!("Expected InvalidProtocol error");
        }
    }

    #[test]
    fn test_allow_localhost_http() {
        // Localhost HTTP URLs should be allowed
        assert!(UrlValidator::validate_backend_url("http://localhost:11434").is_ok());
        assert!(UrlValidator::validate_backend_url("http://127.0.0.1:8080").is_ok());
        assert!(UrlValidator::validate_backend_url("http://localhost").is_ok());
        assert!(UrlValidator::validate_backend_url("http://127.0.0.1").is_ok());
    }

    #[test]
    fn test_allow_localhost_https() {
        // Localhost HTTPS URLs should also be allowed
        assert!(UrlValidator::validate_backend_url("https://localhost:11434").is_ok());
        assert!(UrlValidator::validate_backend_url("https://127.0.0.1:8080").is_ok());
    }

    #[test]
    fn test_is_localhost_url() {
        // Test localhost detection
        assert!(UrlValidator::is_localhost_url("http://localhost:11434"));
        assert!(UrlValidator::is_localhost_url("https://localhost:8080"));
        assert!(UrlValidator::is_localhost_url("http://127.0.0.1:11434"));
        assert!(UrlValidator::is_localhost_url("https://127.0.0.1"));
        assert!(UrlValidator::is_localhost_url("http://localhost"));
        
        // Remote URLs should not be localhost
        assert!(!UrlValidator::is_localhost_url("http://api.example.com"));
        assert!(!UrlValidator::is_localhost_url("https://remote-server.com"));
        assert!(!UrlValidator::is_localhost_url("http://192.168.1.100"));
    }

    #[test]
    fn test_suggest_https_url() {
        assert_eq!(
            UrlValidator::suggest_https_url("http://api.example.com"),
            "https://api.example.com"
        );
        assert_eq!(
            UrlValidator::suggest_https_url("http://server.com:8080"),
            "https://server.com:8080"
        );
        assert_eq!(
            UrlValidator::suggest_https_url("http://example.com/path"),
            "https://example.com/path"
        );
    }

    #[test]
    fn test_empty_url_validation() {
        assert!(matches!(
            UrlValidator::validate_backend_url(""),
            Err(UrlValidationError::EmptyUrl)
        ));
        assert!(matches!(
            UrlValidator::validate_backend_url("   "),
            Err(UrlValidationError::EmptyUrl)
        ));
        assert!(matches!(
            UrlValidator::validate_backend_url("\t\n"),
            Err(UrlValidationError::EmptyUrl)
        ));
    }

    #[test]
    fn test_invalid_format_urls() {
        // Malformed URLs should return InvalidFormat error
        assert!(matches!(
            UrlValidator::validate_backend_url("not-a-url"),
            Err(UrlValidationError::InvalidFormat { .. })
        ));
        assert!(matches!(
            UrlValidator::validate_backend_url("ftp://example.com"),
            Err(UrlValidationError::InvalidFormat { .. })
        ));
        assert!(matches!(
            UrlValidator::validate_backend_url("://missing-scheme"),
            Err(UrlValidationError::InvalidFormat { .. })
        ));
    }

    #[test]
    fn test_filter_valid_urls() {
        let urls = vec![
            "https://api.example.com".to_string(),
            "http://api.example.com".to_string(),  // Invalid - remote HTTP
            "http://localhost:11434".to_string(),   // Valid - localhost HTTP
            "https://localhost:8080".to_string(),   // Valid - localhost HTTPS
            "invalid-url".to_string(),              // Invalid - malformed
            "https://secure-server.com".to_string(), // Valid - remote HTTPS
        ];

        let (valid, invalid) = UrlValidator::filter_valid_urls(urls);

        assert_eq!(valid.len(), 4);
        assert!(valid.contains(&"https://api.example.com".to_string()));
        assert!(valid.contains(&"http://localhost:11434".to_string()));
        assert!(valid.contains(&"https://localhost:8080".to_string()));
        assert!(valid.contains(&"https://secure-server.com".to_string()));

        assert_eq!(invalid.len(), 2);
        assert!(invalid.contains(&"http://api.example.com".to_string()));
        assert!(invalid.contains(&"invalid-url".to_string()));
    }

    #[test]
    fn test_validation_result() {
        // Test valid URL
        let result = ValidationResult::for_url("https://api.example.com");
        assert!(result.is_valid);
        assert!(result.error.is_none());
        assert!(!result.is_localhost);
        assert!(result.suggested_url.is_none());

        // Test invalid remote HTTP URL
        let result = ValidationResult::for_url("http://api.example.com");
        assert!(!result.is_valid);
        assert!(result.error.is_some());
        assert!(!result.is_localhost);
        assert_eq!(result.suggested_url, Some("https://api.example.com".to_string()));

        // Test valid localhost URL
        let result = ValidationResult::for_url("http://localhost:11434");
        assert!(result.is_valid);
        assert!(result.error.is_none());
        assert!(result.is_localhost);
        assert!(result.suggested_url.is_none());
    }

    #[test]
    fn test_error_display_formatting() {
        let error = UrlValidationError::InvalidProtocol {
            url: "http://api.example.com".to_string(),
            suggested: "https://api.example.com".to_string(),
        };
        
        let error_str = error.to_string();
        assert!(error_str.contains("Invalid backend URL protocol"));
        assert!(error_str.contains("http://api.example.com"));
        assert!(error_str.contains("https://api.example.com"));
        assert!(error_str.contains("HTTPS protocol for remote endpoints"));
        assert!(error_str.contains("encrypt your prompts and responses"));
        assert!(error_str.contains("Examples of valid URLs"));
    }

    #[test]
    fn test_ipv6_localhost() {
        // Test IPv6 localhost support
        assert!(UrlValidator::is_localhost_url("http://[::1]:11434"));
        assert!(UrlValidator::validate_backend_url("http://[::1]:11434").is_ok());
    }

    #[test]
    fn test_url_with_paths_and_queries() {
        // Test URLs with paths and query parameters
        assert!(UrlValidator::validate_backend_url("https://api.example.com/v1/chat?model=llama").is_ok());
        
        let result = UrlValidator::validate_backend_url("http://api.example.com/v1/chat?model=llama");
        assert!(result.is_err());
        
        if let Err(UrlValidationError::InvalidProtocol { suggested, .. }) = result {
            assert_eq!(suggested, "https://api.example.com/v1/chat?model=llama");
        }
    }

    #[test]
    fn test_case_sensitivity() {
        // Test that localhost detection is case-insensitive for the scheme but case-sensitive for host
        assert!(UrlValidator::validate_backend_url("HTTP://localhost:11434").is_ok());
        assert!(UrlValidator::validate_backend_url("HTTPS://localhost:11434").is_ok());
        
        // Host should be case-sensitive (LOCALHOST != localhost in some contexts)
        // But our implementation should handle this gracefully
        assert!(UrlValidator::is_localhost_url("http://LOCALHOST:11434"));
    }

    #[test]
    fn test_port_variations() {
        // Test various port configurations
        assert!(UrlValidator::validate_backend_url("https://api.example.com:443").is_ok());
        assert!(UrlValidator::validate_backend_url("https://api.example.com:8080").is_ok());
        assert!(UrlValidator::validate_backend_url("http://localhost:11434").is_ok());
        assert!(UrlValidator::validate_backend_url("http://127.0.0.1:8080").is_ok());
        
        // No port should also work
        assert!(UrlValidator::validate_backend_url("https://api.example.com").is_ok());
        assert!(UrlValidator::validate_backend_url("http://localhost").is_ok());
    }

    // Property-based tests
    
    /// **Feature: https-only-enforcement, Property 1: Remote URL protocol enforcement**
    /// 
    /// For any remote backend URL (non-localhost), the system should accept the URL 
    /// if and only if it uses HTTPS protocol, rejecting all HTTP remote URLs with 
    /// appropriate error messages.
    /// **Validates: Requirements 1.1, 1.2**
    #[test]
    fn prop_remote_url_protocol_enforcement() {
        fn property(domain: String, port: Option<u16>, path: String) -> TestResult {
            // Filter out empty domains or domains that might be localhost
            if domain.trim().is_empty() 
                || domain.contains("localhost") 
                || domain.contains("127.0.0.1")
                || domain.contains("::1")
                || domain.starts_with("192.168.")
                || domain.starts_with("10.")
                || domain.starts_with("172.")
                || !domain.contains('.') // Simple domain validation
            {
                return TestResult::discard();
            }

            // Clean the domain to make it valid
            let clean_domain = domain
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
                .collect::<String>()
                .trim_matches('.')
                .to_lowercase();

            // Skip if domain becomes empty after cleaning
            if clean_domain.is_empty() || clean_domain == "." {
                return TestResult::discard();
            }

            // Clean the path
            let clean_path = path
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '/' || *c == '-' || *c == '_' || *c == '.')
                .collect::<String>();

            // Construct URLs
            let port_str = match port {
                Some(p) if p > 0 => format!(":{}", p),
                _ => String::new(),
            };

            let path_str = if clean_path.is_empty() || !clean_path.starts_with('/') {
                String::new()
            } else {
                clean_path
            };

            let http_url = format!("http://{}{}{}", clean_domain, port_str, path_str);
            let https_url = format!("https://{}{}{}", clean_domain, port_str, path_str);

            // Test HTTP URL (should be rejected for remote URLs)
            let http_result = UrlValidator::validate_backend_url(&http_url);
            let http_rejected = http_result.is_err();
            
            // If HTTP is rejected, it should be due to InvalidProtocol
            if let Err(error) = &http_result {
                match error {
                    UrlValidationError::InvalidProtocol { url, suggested } => {
                        // Verify the error contains the correct URL and suggestion
                        if url != &http_url || suggested != &https_url {
                            return TestResult::failed();
                        }
                    }
                    _ => {
                        // If it's not InvalidProtocol, it might be InvalidFormat
                        // which is acceptable for malformed URLs
                        return TestResult::discard();
                    }
                }
            }

            // Test HTTPS URL (should be accepted for remote URLs)
            let https_result = UrlValidator::validate_backend_url(&https_url);
            let https_accepted = https_result.is_ok();

            // If HTTPS is rejected, it should only be due to InvalidFormat (malformed URL)
            if let Err(error) = &https_result {
                match error {
                    UrlValidationError::InvalidFormat { .. } => {
                        // If both HTTP and HTTPS are malformed, discard this test case
                        return TestResult::discard();
                    }
                    _ => {
                        // HTTPS should not be rejected for protocol reasons
                        return TestResult::failed();
                    }
                }
            }

            // Verify that neither URL is detected as localhost
            let http_is_localhost = UrlValidator::is_localhost_url(&http_url);
            let https_is_localhost = UrlValidator::is_localhost_url(&https_url);

            // Property: Remote HTTP URLs should be rejected, remote HTTPS URLs should be accepted
            // and neither should be detected as localhost
            TestResult::from_bool(
                http_rejected && https_accepted && !http_is_localhost && !https_is_localhost
            )
        }

        QuickCheck::new()
            .tests(100)
            .quickcheck(property as fn(String, Option<u16>, String) -> TestResult);
    }
}