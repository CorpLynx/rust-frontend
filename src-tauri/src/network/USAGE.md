# Error Handling and Logging Usage Guide

This document demonstrates how to use the error handling and logging features implemented for the remote Ollama integration.

## Overview

The error handling system provides:
- User-friendly error messages for all error types
- Comprehensive error logging with context
- Retry logic with exponential backoff
- API key redaction in logs

## Basic Usage

### 1. Error Logging

```rust
use crate::network::{ErrorLogger, ErrorContext, get_error_category};
use crate::config::ConnectionMode;

// Create a logger
let logger = ErrorLogger::new();

// Create error context
let context = ErrorContext::new(
    &ConnectionMode::Remote,
    Some("https://example.com:11434".to_string()),
    get_error_category(&error),
);

// Log the error
if let Err(e) = logger.log_error(&error, context) {
    eprintln!("Failed to log error: {}", e);
}
```

### 2. Retry with Exponential Backoff

```rust
use crate::network::{RetryConfig, retry_with_backoff, OllamaClient};

let client = OllamaClient::new();
let retry_config = RetryConfig::default(); // 3 attempts, exponential backoff

// Retry a network operation
let result = retry_with_backoff(&retry_config, || async {
    client.test_connection("http://localhost:11434").await
}).await;

match result {
    Ok(test_result) => {
        if test_result.success {
            println!("Connection successful! Response time: {}ms", test_result.response_time_ms);
        } else {
            println!("Connection failed: {:?}", test_result.error_message);
        }
    }
    Err(e) => {
        eprintln!("All retry attempts failed: {}", e);
    }
}
```

### 3. Custom Retry Configuration

```rust
use crate::network::RetryConfig;

// Custom retry configuration
let retry_config = RetryConfig {
    max_attempts: 5,           // Try 5 times
    initial_delay_ms: 200,     // Start with 200ms delay
    max_delay_ms: 10000,       // Cap at 10 seconds
    backoff_multiplier: 2.0,   // Double the delay each time
};
```

### 4. User-Friendly Error Messages

```rust
use crate::network::{get_user_friendly_message, NetworkError};

let error = NetworkError::Timeout;
let message = get_user_friendly_message(&error);
// Returns: "Server unreachable - connection timed out"

// Display to user
println!("Error: {}", message);
```

### 5. API Key Redaction

```rust
use crate::network::redact_api_key;

let url = "https://api.example.com?api_key=secret123&other=value";
let safe_url = redact_api_key(url);
// Returns: "https://api.example.com?api_key=***REDACTED***&other=value"

// Safe to log
println!("Connecting to: {}", safe_url);
```

## Complete Example: Connection Test with Error Handling

```rust
use crate::network::{
    OllamaClient, ErrorLogger, ErrorContext, RetryConfig,
    retry_with_backoff, get_error_category, get_user_friendly_message
};
use crate::config::ConnectionMode;

async fn test_connection_with_retry(
    endpoint: &str,
    mode: &ConnectionMode,
) -> Result<String, String> {
    let client = OllamaClient::new();
    let logger = ErrorLogger::new();
    let retry_config = RetryConfig::default();
    
    // Attempt connection with retry
    let result = retry_with_backoff(&retry_config, || async {
        client.test_connection(endpoint).await
    }).await;
    
    match result {
        Ok(test_result) => {
            if test_result.success {
                Ok(format!(
                    "Connection successful! Response time: {}ms",
                    test_result.response_time_ms
                ))
            } else {
                Err(test_result.error_message.unwrap_or_else(|| "Unknown error".to_string()))
            }
        }
        Err(error) => {
            // Log the error with context
            let context = ErrorContext::new(
                mode,
                Some(endpoint.to_string()),
                get_error_category(&error),
            );
            
            if let Err(log_err) = logger.log_error(&error, context) {
                eprintln!("Failed to log error: {}", log_err);
            }
            
            // Return user-friendly message
            Err(get_user_friendly_message(&error))
        }
    }
}
```

## Error Categories

The system recognizes the following error categories:

- **TIMEOUT**: Connection timed out
- **CONNECTION_REFUSED**: Server not accepting connections
- **INVALID_RESPONSE**: Protocol mismatch or invalid server response
- **TLS_ERROR**: Certificate or SSL/TLS issues
- **NETWORK_ERROR**: Generic network errors

## Retryable vs Non-Retryable Errors

**Retryable** (will be retried with exponential backoff):
- Timeout
- Connection refused
- Generic network errors

**Non-Retryable** (fail immediately):
- Invalid response (protocol issue)
- TLS/SSL errors (require configuration changes)

## Log Format

Error logs are written to `logs/error.log` with the following format:

```
[2024-01-15T10:30:45.123Z] ERROR: TIMEOUT - Server unreachable - connection timed out | Mode: Remote | Endpoint: https://example.com:11434 | Technical: Timeout
```

Each log entry includes:
- ISO 8601 timestamp
- Error category
- User-friendly message
- Connection mode
- Endpoint (with API keys redacted)
- Technical details for debugging

## Best Practices

1. **Always log errors**: Use `ErrorLogger` to log all network errors for debugging
2. **Use retry for transient errors**: Apply `retry_with_backoff` for operations that might fail temporarily
3. **Redact sensitive data**: Always use `redact_api_key` before logging URLs or headers
4. **Provide user-friendly messages**: Use `get_user_friendly_message` for UI display
5. **Add context**: Include connection mode and endpoint in error context for better debugging
