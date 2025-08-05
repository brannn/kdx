//! Progress tracking for long-running operations

use indicatif::{ProgressBar, ProgressStyle};

/// Progress tracker for resource discovery operations
pub struct ProgressTracker {
    bar: Option<ProgressBar>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(show_progress: bool, total: Option<u64>) -> Self {
        let bar = if show_progress {
            let pb = match total {
                Some(t) => ProgressBar::new(t),
                None => ProgressBar::new_spinner(),
            };
            
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
            );
            
            Some(pb)
        } else {
            None
        };
        
        Self { bar }
    }

    /// Create a spinner for indeterminate progress
    pub fn new_spinner(show_progress: bool, message: &str) -> Self {
        let bar = if show_progress {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap_or_else(|_| ProgressStyle::default_spinner())
            );
            pb.set_message(message.to_string());
            Some(pb)
        } else {
            None
        };
        
        Self { bar }
    }

    /// Set the progress message
    pub fn set_message(&self, msg: &str) {
        if let Some(bar) = &self.bar {
            bar.set_message(msg.to_string());
        }
    }

    /// Increment progress by delta
    pub fn inc(&self, delta: u64) {
        if let Some(bar) = &self.bar {
            bar.inc(delta);
        }
    }

    /// Set the current position
    pub fn set_position(&self, pos: u64) {
        if let Some(bar) = &self.bar {
            bar.set_position(pos);
        }
    }

    /// Finish the progress bar with a message
    pub fn finish(&self) {
        if let Some(bar) = &self.bar {
            bar.finish_with_message("Complete");
        }
    }

    /// Finish the progress bar and clear it
    pub fn finish_and_clear(&self) {
        if let Some(bar) = &self.bar {
            bar.finish_and_clear();
        }
    }
}

impl Drop for ProgressTracker {
    fn drop(&mut self) {
        if let Some(bar) = &self.bar {
            bar.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(false, Some(100));
        assert!(tracker.bar.is_none());

        let tracker = ProgressTracker::new(true, Some(100));
        assert!(tracker.bar.is_some());
    }

    #[test]
    fn test_spinner_creation() {
        let tracker = ProgressTracker::new_spinner(false, "Loading...");
        assert!(tracker.bar.is_none());

        let tracker = ProgressTracker::new_spinner(true, "Loading...");
        assert!(tracker.bar.is_some());
    }

    #[test]
    fn test_progress_operations() {
        let tracker = ProgressTracker::new(false, Some(100));
        
        // These should not panic even with no progress bar
        tracker.set_message("Test");
        tracker.inc(10);
        tracker.set_position(50);
        tracker.finish();
    }
}
