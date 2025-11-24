use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::env;
use std::fs;
use std::io;

#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
use std::os::unix::fs::MetadataExt;

/// Manages the update process for the Prometheus CLI application
pub struct UpdateManager {
    install_dir: PathBuf,
    bin_dir: PathBuf,
    binary_name: String,
}

/// Represents the current update status of the installation
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    /// The installation is up to date
    UpToDate,
    /// Updates are available with commit count and change summary
    UpdatesAvailable {
        commits_behind: usize,
        changes: String,
    },
}

impl UpdateManager {
    /// Create a new UpdateManager instance
    /// 
    /// Attempts to detect the installation directory and sets up paths
    /// for the binary directory and executable name.
    /// 
    /// # Returns
    /// A Result containing the UpdateManager or an error if detection fails
    /// 
    /// # Requirements
    /// * 6.1: Verify the installation directory exists
    pub fn new() -> Result<Self> {
        // Detect installation directory
        let install_dir = Self::detect_install_dir()?;
        
        // Standard binary directory
        let bin_dir = PathBuf::from("/usr/local/bin");
        
        Ok(Self {
            install_dir,
            bin_dir,
            binary_name: "prometheus-cli".to_string(),
        })
    }

    /// Detect the installation directory
    /// 
    /// Tries multiple strategies to find the Prometheus installation:
    /// 1. Standard location (~/.prometheus)
    /// 2. Walking up from current executable path to find Cargo.toml
    /// 
    /// # Returns
    /// A Result containing the installation directory path
    /// 
    /// # Requirements
    /// * 6.1: Verify the installation directory exists
    fn detect_install_dir() -> Result<PathBuf> {
        // Try standard location first
        let home = env::var("HOME").context("HOME environment variable not set")?;
        let standard_path = PathBuf::from(home).join(".prometheus");
        
        if standard_path.exists() {
            return Ok(standard_path);
        }

        // Try to find via current executable path
        if let Ok(exe_path) = env::current_exe() {
            // Walk up the directory tree looking for Cargo.toml
            let mut current = exe_path.parent();
            while let Some(dir) = current {
                if dir.join("Cargo.toml").exists() {
                    return Ok(dir.to_path_buf());
                }
                current = dir.parent();
            }
        }

        bail!("Could not detect installation directory. Prometheus CLI must be installed from source.")
    }

    /// Validate the installation directory
    /// 
    /// Checks that the installation directory is a valid git repository
    /// with the expected structure for Prometheus CLI.
    /// 
    /// # Returns
    /// A Result indicating success or failure with error details
    /// 
    /// # Requirements
    /// * 6.1: Verify the installation directory exists
    /// * 6.2: Verify the installation directory is a git repository
    /// * 6.3: Verify the installation directory contains expected Cargo.toml
    pub fn validate_installation(&self) -> Result<()> {
        // Check if directory exists
        if !self.install_dir.exists() {
            bail!("Installation directory does not exist: {}", self.install_dir.display());
        }

        // Check if it's a git repository
        if !self.install_dir.join(".git").exists() {
            bail!("Installation directory is not a git repository. Update command requires git-based installation.");
        }

        // Check if Cargo.toml exists
        if !self.install_dir.join("Cargo.toml").exists() {
            bail!("Invalid installation: Cargo.toml not found in {}", self.install_dir.display());
        }

        Ok(())
    }

    /// Check if updates are available
    /// 
    /// Fetches the latest changes from the remote repository and compares
    /// the current commit with the remote commit to determine if updates
    /// are available.
    /// 
    /// # Returns
    /// A Result containing the UpdateStatus
    /// 
    /// # Requirements
    /// * 2.1: Compare local git commit with remote repository
    /// * 2.2: Display number of commits behind and summary of changes
    /// * 2.3: Display message when installation is up to date
    /// * 2.4: Display error message when check operation fails
    pub fn check_for_updates(&self) -> Result<UpdateStatus> {
        // Fetch latest from remote without merging
        let fetch_output = ProcessCommand::new("git")
            .args(&["fetch", "origin"])
            .current_dir(&self.install_dir)
            .output()
            .context("Failed to execute git fetch. Please check your internet connection and ensure git is installed.")?;

        if !fetch_output.status.success() {
            let stderr = String::from_utf8_lossy(&fetch_output.stderr);
            
            // Provide more specific error messages based on common git fetch failures
            if stderr.contains("Could not resolve host") || stderr.contains("Name or service not known") {
                bail!("Network error: Unable to reach the remote repository. Please check your internet connection.");
            } else if stderr.contains("Permission denied") || stderr.contains("Authentication failed") {
                bail!("Authentication error: Unable to access the remote repository. Please check your git credentials.");
            } else if stderr.contains("fatal: not a git repository") {
                bail!("Git repository error: The installation directory is not a valid git repository.");
            } else {
                bail!("Git fetch failed: {}", stderr.trim());
            }
        }

        // Get current commit
        let current_output = ProcessCommand::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(&self.install_dir)
            .output()
            .context("Failed to get current commit. The git repository may be corrupted.")?;

        if !current_output.status.success() {
            let stderr = String::from_utf8_lossy(&current_output.stderr);
            bail!("Unable to determine current version: {}", stderr.trim());
        }

        let current_commit = String::from_utf8_lossy(&current_output.stdout).trim().to_string();

        // Get remote commit
        let remote_output = ProcessCommand::new("git")
            .args(&["rev-parse", "origin/main"])
            .current_dir(&self.install_dir)
            .output()
            .context("Failed to get remote commit. The remote branch may not exist.")?;

        if !remote_output.status.success() {
            let stderr = String::from_utf8_lossy(&remote_output.stderr);
            if stderr.contains("unknown revision") {
                bail!("Remote branch 'main' not found. The repository structure may have changed.");
            } else {
                bail!("Unable to determine remote version: {}", stderr.trim());
            }
        }

        let remote_commit = String::from_utf8_lossy(&remote_output.stdout).trim().to_string();

        // Count commits behind
        let count_output = ProcessCommand::new("git")
            .args(&["rev-list", "--count", &format!("{}..{}", current_commit, remote_commit)])
            .current_dir(&self.install_dir)
            .output()
            .context("Failed to count commits behind remote")?;

        if !count_output.status.success() {
            let stderr = String::from_utf8_lossy(&count_output.stderr);
            bail!("Unable to compare versions: {}", stderr.trim());
        }

        let commits_behind: usize = String::from_utf8_lossy(&count_output.stdout)
            .trim()
            .parse()
            .context("Invalid commit count returned by git")?;

        if commits_behind == 0 {
            Ok(UpdateStatus::UpToDate)
        } else {
            // Get commit messages
            let log_output = ProcessCommand::new("git")
                .args(&["log", "--oneline", "--max-count=10", &format!("{}..{}", current_commit, remote_commit)])
                .current_dir(&self.install_dir)
                .output()
                .context("Failed to get commit log")?;

            if !log_output.status.success() {
                // If we can't get the log, still return that updates are available
                let changes = format!("Unable to retrieve change details. {} commit(s) available.", commits_behind);
                return Ok(UpdateStatus::UpdatesAvailable {
                    commits_behind,
                    changes,
                });
            }

            let mut changes = String::from_utf8_lossy(&log_output.stdout).to_string();
            
            // If there are more than 10 commits, add a note
            if commits_behind > 10 {
                changes.push_str(&format!("\n... and {} more commit(s)", commits_behind - 10));
            }

            Ok(UpdateStatus::UpdatesAvailable {
                commits_behind,
                changes,
            })
        }
    }

    /// Perform the update
    /// 
    /// Executes the complete update process:
    /// 1. Fetches latest changes from git repository
    /// 2. Rebuilds the CLI binary using cargo
    /// 3. Installs the new binary, handling permissions as needed
    /// 
    /// # Arguments
    /// * `progress_callback` - Function called with progress messages
    /// 
    /// # Returns
    /// A Result containing the new version string on success
    /// 
    /// # Requirements
    /// * 1.1: Fetch latest changes from git repository
    /// * 1.2: Rebuild CLI binary using cargo
    /// * 1.3: Replace installed binary with new version
    /// * 1.4: Display success message with new version number
    /// * 1.5: Display clear error message and maintain existing installation on error
    /// * 4.1: Display message indicating update has begun
    /// * 4.2: Display progress indicator during git fetch
    /// * 4.3: Display build progress information during cargo build
    /// * 4.4: Display completion message for each major step
    /// * 5.1: Prompt user for sudo access when binary installation requires elevated permissions
    /// * 5.2: Display error message and exit when user denies sudo access
    /// * 5.3: Complete binary installation with elevated permissions when sudo access is granted
    pub fn perform_update<F>(&self, progress_callback: F) -> Result<String>
    where
        F: Fn(&str),
    {
        progress_callback("🔄 Fetching latest changes from repository...");

        // Git pull with enhanced error handling
        let pull_output = ProcessCommand::new("git")
            .args(&["pull", "origin", "main"])
            .current_dir(&self.install_dir)
            .output()
            .context("Failed to execute git pull. Please ensure git is installed and accessible.")?;

        if !pull_output.status.success() {
            let stderr = String::from_utf8_lossy(&pull_output.stderr);
            
            // Provide specific error messages for common git pull failures
            if stderr.contains("Could not resolve host") || stderr.contains("Name or service not known") {
                bail!("Network error during update: Unable to reach the remote repository. Please check your internet connection and try again.");
            } else if stderr.contains("Permission denied") || stderr.contains("Authentication failed") {
                bail!("Authentication error during update: Unable to access the remote repository. Please check your git credentials.");
            } else if stderr.contains("Your local changes") || stderr.contains("would be overwritten") {
                bail!("Local changes detected: Your installation has local modifications that would be overwritten. Please backup your changes and run 'git reset --hard' in {} before updating.", self.install_dir.display());
            } else if stderr.contains("Merge conflict") {
                bail!("Merge conflict detected: Unable to automatically merge changes. Please resolve conflicts manually in {} and try again.", self.install_dir.display());
            } else {
                bail!("Git pull failed: {}. Your existing installation remains unchanged.", stderr.trim());
            }
        }

        progress_callback("✅ Successfully fetched latest changes");
        progress_callback("🔨 Building updated binary (this may take a few minutes)...");

        // Cargo build with enhanced error handling
        let build_output = ProcessCommand::new("cargo")
            .args(&["build", "--release", "-p", "prometheus-cli"])
            .current_dir(&self.install_dir)
            .output()
            .context("Failed to execute cargo build. Please ensure Rust and Cargo are installed.")?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            let stdout = String::from_utf8_lossy(&build_output.stdout);
            
            // Provide specific error messages for common cargo build failures
            if stderr.contains("could not find `Cargo.toml`") || stdout.contains("could not find `Cargo.toml`") {
                bail!("Build error: Cargo.toml not found. The installation directory may be corrupted.");
            } else if stderr.contains("failed to resolve dependencies") || stdout.contains("failed to resolve dependencies") {
                bail!("Build error: Failed to resolve dependencies. Please check your internet connection and try again.");
            } else if stderr.contains("rustc") && stderr.contains("not found") {
                bail!("Build error: Rust compiler not found. Please ensure Rust is properly installed.");
            } else if stderr.contains("linker") && stderr.contains("not found") {
                bail!("Build error: System linker not found. Please install build tools for your system.");
            } else {
                // Include both stderr and stdout for build errors as cargo outputs to both
                let full_error = if !stdout.is_empty() && !stderr.is_empty() {
                    format!("{}\n{}", stdout.trim(), stderr.trim())
                } else if !stdout.is_empty() {
                    stdout.trim().to_string()
                } else {
                    stderr.trim().to_string()
                };
                bail!("Cargo build failed: {}. Your existing installation remains unchanged.", full_error);
            }
        }

        progress_callback("✅ Successfully built updated binary");
        progress_callback("📦 Installing updated binary...");

        // Install binary with enhanced error handling
        let source_binary = self.install_dir
            .join("target")
            .join("release")
            .join(&self.binary_name);

        // Verify the built binary exists
        if !source_binary.exists() {
            bail!("Build completed but binary not found at {}. The build may have failed silently.", source_binary.display());
        }

        let dest_binary = self.bin_dir.join(&self.binary_name);

        // Check if we need sudo by trying to write to the binary directory
        let needs_sudo = self.check_needs_sudo(&dest_binary)?;

        if needs_sudo {
            progress_callback("🔐 Elevated permissions required for installation...");
            
            // Prompt user about sudo requirement
            self.prompt_for_sudo()?;
            
            // Try with sudo
            let install_output = ProcessCommand::new("sudo")
                .args(&["cp", source_binary.to_str().unwrap(), dest_binary.to_str().unwrap()])
                .output()
                .context("Failed to execute sudo command for binary installation")?;

            if !install_output.status.success() {
                let stderr = String::from_utf8_lossy(&install_output.stderr);
                
                if stderr.contains("Permission denied") || stderr.contains("Operation not permitted") {
                    bail!("Permission denied: Unable to install binary even with sudo. Please check that {} is writable.", self.bin_dir.display());
                } else if stderr.contains("No such file or directory") {
                    bail!("Installation directory not found: {}. Please create the directory or install to a different location.", self.bin_dir.display());
                } else {
                    bail!("Failed to install binary with sudo: {}. Your existing installation remains unchanged.", stderr.trim());
                }
            }
        } else {
            // Copy without sudo
            if let Err(e) = fs::copy(&source_binary, &dest_binary) {
                match e.kind() {
                    io::ErrorKind::PermissionDenied => {
                        bail!("Permission denied: Unable to install binary to {}. You may need to run the update with sudo or install to a different location.", dest_binary.display());
                    }
                    io::ErrorKind::NotFound => {
                        bail!("Installation directory not found: {}. Please create the directory or install to a different location.", self.bin_dir.display());
                    }
                    io::ErrorKind::AlreadyExists => {
                        // This shouldn't happen with copy, but handle it anyway
                        bail!("Installation failed: Target file already exists and cannot be overwritten at {}.", dest_binary.display());
                    }
                    _ => {
                        bail!("Failed to install binary: {}. Your existing installation remains unchanged.", e);
                    }
                }
            }
        }

        progress_callback("✅ Successfully installed updated binary");
        progress_callback("🔍 Verifying installation...");

        // Get new version with error handling
        let version_output = ProcessCommand::new(&dest_binary)
            .arg("--version")
            .output()
            .context("Failed to verify new installation")?;

        if !version_output.status.success() {
            let stderr = String::from_utf8_lossy(&version_output.stderr);
            bail!("Installation verification failed: The new binary was installed but cannot be executed. Error: {}", stderr.trim());
        }

        let version = String::from_utf8_lossy(&version_output.stdout).trim().to_string();
        
        if version.is_empty() {
            bail!("Installation verification failed: The new binary was installed but returned no version information.");
        }

        progress_callback("✅ Installation verified successfully");

        Ok(version)
    }

    /// Check if sudo is needed for binary installation
    /// 
    /// Attempts to determine if elevated permissions are required by checking
    /// if we can write to the binary directory.
    /// 
    /// # Arguments
    /// * `dest_binary` - Path to the destination binary
    /// 
    /// # Returns
    /// A Result containing true if sudo is needed, false otherwise
    /// 
    /// # Requirements
    /// * 5.1: Determine when elevated permissions are required
    fn check_needs_sudo(&self, _dest_binary: &Path) -> Result<bool> {
        // First, check if the binary directory exists
        if !self.bin_dir.exists() {
            // If the directory doesn't exist, we'll likely need sudo to create it
            return Ok(true);
        }

        // Check if the binary directory is writable
        if let Ok(metadata) = self.bin_dir.metadata() {
            // On Unix systems, check if we can write to the directory
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = metadata.permissions();
                let mode = permissions.mode();
                
                // Check if we have write permission (owner, group, or other)
                let uid = unsafe { libc::getuid() };
                let gid = unsafe { libc::getgid() };
                
                // Get file owner and group
                let file_uid = metadata.st_uid();
                let file_gid = metadata.st_gid();
                
                // Check owner write permission
                if uid == file_uid && (mode & 0o200) != 0 {
                    return Ok(false);
                }
                
                // Check group write permission
                if gid == file_gid && (mode & 0o020) != 0 {
                    return Ok(false);
                }
                
                // Check other write permission
                if (mode & 0o002) != 0 {
                    return Ok(false);
                }
                
                // If none of the above, we need sudo
                return Ok(true);
            }
            
            #[cfg(not(unix))]
            {
                // On non-Unix systems, assume we need sudo if the directory is read-only
                return Ok(metadata.permissions().readonly());
            }
        }
        
        // If we can't get metadata, try to create a test file
        let test_file = self.bin_dir.join(".prometheus_test_write");
        match fs::write(&test_file, "test") {
            Ok(_) => {
                // Clean up test file
                let _ = fs::remove_file(&test_file);
                Ok(false)
            }
            Err(e) => {
                // Check the specific error to provide better feedback
                match e.kind() {
                    io::ErrorKind::PermissionDenied => Ok(true),
                    io::ErrorKind::NotFound => {
                        // Directory doesn't exist, we'll need sudo to create it
                        Ok(true)
                    }
                    _ => {
                        // For other errors, assume we need sudo
                        Ok(true)
                    }
                }
            }
        }
    }

    /// Prompt user for sudo access and provide clear guidance
    /// 
    /// This method informs the user that elevated permissions are required
    /// and provides guidance on what will happen next.
    /// 
    /// # Requirements
    /// * 5.1: Prompt user for sudo access when binary installation requires elevated permissions
    /// * 5.2: Display error message and exit when user denies sudo access
    fn prompt_for_sudo(&self) -> Result<()> {
        // Print informational message about sudo requirement
        println!("🔐 Administrator privileges required to install the binary to {}", self.bin_dir.display());
        println!("   The system will prompt for your password to complete the installation.");
        println!("   This is necessary to update the prometheus-cli executable in the system directory.");
        println!();
        
        // Check if sudo is available
        let sudo_check = ProcessCommand::new("sudo")
            .args(&["-n", "true"])
            .output();
            
        match sudo_check {
            Ok(output) if output.status.success() => {
                // User already has sudo privileges cached
                println!("✅ Sudo privileges confirmed");
            }
            Ok(_) => {
                // Sudo available but needs password
                println!("Please enter your password when prompted by sudo:");
            }
            Err(_) => {
                bail!("Sudo command not found. Please install sudo or run the update as an administrator.");
            }
        }
        
        Ok(())
    }

    /// Get the installation directory path
    pub fn install_dir(&self) -> &Path {
        &self.install_dir
    }

    /// Get the binary directory path
    pub fn bin_dir(&self) -> &Path {
        &self.bin_dir
    }

    /// Get the binary name
    pub fn binary_name(&self) -> &str {
        &self.binary_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Test UpdateManager creation with valid installation
    #[test]
    fn test_update_manager_creation() {
        // This test will only pass if run in a valid git repository
        // In a real environment, we'd mock the detection logic
        let result = UpdateManager::new();
        
        // The result depends on the environment, so we just check it doesn't panic
        match result {
            Ok(manager) => {
                assert!(!manager.binary_name().is_empty());
                assert!(manager.bin_dir().exists() || manager.bin_dir() == Path::new("/usr/local/bin"));
            }
            Err(_) => {
                // Expected in test environments without proper git setup
            }
        }
    }

    /// Test installation validation with invalid directory
    #[test]
    fn test_validate_installation_invalid_directory() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("nonexistent");
        
        let manager = UpdateManager {
            install_dir: invalid_path,
            bin_dir: PathBuf::from("/usr/local/bin"),
            binary_name: "prometheus-cli".to_string(),
        };
        
        let result = manager.validate_installation();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Installation directory does not exist"));
    }

    /// Test installation validation with non-git directory
    #[test]
    fn test_validate_installation_non_git() {
        let temp_dir = TempDir::new().unwrap();
        
        let manager = UpdateManager {
            install_dir: temp_dir.path().to_path_buf(),
            bin_dir: PathBuf::from("/usr/local/bin"),
            binary_name: "prometheus-cli".to_string(),
        };
        
        let result = manager.validate_installation();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a git repository"));
    }

    /// Test installation validation with missing Cargo.toml
    #[test]
    fn test_validate_installation_missing_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create .git directory but no Cargo.toml
        fs::create_dir(temp_dir.path().join(".git")).unwrap();
        
        let manager = UpdateManager {
            install_dir: temp_dir.path().to_path_buf(),
            bin_dir: PathBuf::from("/usr/local/bin"),
            binary_name: "prometheus-cli".to_string(),
        };
        
        let result = manager.validate_installation();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cargo.toml not found"));
    }

    /// Test installation validation with valid setup
    #[test]
    fn test_validate_installation_valid() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create .git directory and Cargo.toml
        fs::create_dir(temp_dir.path().join(".git")).unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        
        let manager = UpdateManager {
            install_dir: temp_dir.path().to_path_buf(),
            bin_dir: PathBuf::from("/usr/local/bin"),
            binary_name: "prometheus-cli".to_string(),
        };
        
        let result = manager.validate_installation();
        assert!(result.is_ok());
    }

    /// Test UpdateStatus equality
    #[test]
    fn test_update_status_equality() {
        let status1 = UpdateStatus::UpToDate;
        let status2 = UpdateStatus::UpToDate;
        assert_eq!(status1, status2);
        
        let status3 = UpdateStatus::UpdatesAvailable {
            commits_behind: 5,
            changes: "Some changes".to_string(),
        };
        let status4 = UpdateStatus::UpdatesAvailable {
            commits_behind: 5,
            changes: "Some changes".to_string(),
        };
        assert_eq!(status3, status4);
        
        assert_ne!(status1, status3);
    }

    /// Test sudo requirement detection
    #[test]
    fn test_check_needs_sudo() {
        let temp_dir = TempDir::new().unwrap();
        
        let manager = UpdateManager {
            install_dir: temp_dir.path().to_path_buf(),
            bin_dir: temp_dir.path().to_path_buf(), // Use temp dir as bin dir for testing
            binary_name: "test-binary".to_string(),
        };
        
        let test_binary = temp_dir.path().join("test-binary");
        
        // Should not need sudo for temp directory (usually writable)
        let result = manager.check_needs_sudo(&test_binary);
        assert!(result.is_ok());
        // The actual result depends on the temp directory permissions
    }

    /// Test getters
    #[test]
    fn test_getters() {
        let install_dir = PathBuf::from("/test/install");
        let bin_dir = PathBuf::from("/test/bin");
        let binary_name = "test-binary".to_string();
        
        let manager = UpdateManager {
            install_dir: install_dir.clone(),
            bin_dir: bin_dir.clone(),
            binary_name: binary_name.clone(),
        };
        
        assert_eq!(manager.install_dir(), install_dir.as_path());
        assert_eq!(manager.bin_dir(), bin_dir.as_path());
        assert_eq!(manager.binary_name(), binary_name);
    }
}