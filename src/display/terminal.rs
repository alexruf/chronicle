//! TTY detection and color support logic

use std::io::IsTerminal;

/// Determine if colors should be used based on environment and TTY status
pub fn should_use_colors() -> bool {
    // Priority order:
    // 1. NO_COLOR takes precedence (https://no-color.org/)
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // 2. CLICOLOR_FORCE enables colors even when piped
    if let Ok(val) = std::env::var("CLICOLOR_FORCE") {
        if val != "0" {
            return true;
        }
    }

    // 3. CLICOLOR=0 disables colors
    if let Ok(val) = std::env::var("CLICOLOR") {
        if val == "0" {
            return false;
        }
    }

    // 4. Check if stdout is a TTY
    std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_color_disables() {
        // Clean environment first
        std::env::remove_var("CLICOLOR_FORCE");
        std::env::remove_var("CLICOLOR");

        std::env::set_var("NO_COLOR", "1");
        assert_eq!(should_use_colors(), false);
        std::env::remove_var("NO_COLOR");
    }

    #[test]
    fn test_clicolor_force_enables() {
        // Clean environment first
        std::env::remove_var("NO_COLOR");
        std::env::remove_var("CLICOLOR");

        std::env::set_var("CLICOLOR_FORCE", "1");
        assert_eq!(should_use_colors(), true);
        std::env::remove_var("CLICOLOR_FORCE");
    }

    #[test]
    fn test_no_color_overrides_force() {
        // Clean environment first
        std::env::remove_var("CLICOLOR");

        std::env::set_var("NO_COLOR", "1");
        std::env::set_var("CLICOLOR_FORCE", "1");
        assert_eq!(should_use_colors(), false);
        std::env::remove_var("NO_COLOR");
        std::env::remove_var("CLICOLOR_FORCE");
    }

    #[test]
    fn test_clicolor_zero_disables() {
        // Clean environment first
        std::env::remove_var("NO_COLOR");
        std::env::remove_var("CLICOLOR_FORCE");

        std::env::set_var("CLICOLOR", "0");
        assert_eq!(should_use_colors(), false);
        std::env::remove_var("CLICOLOR");
    }
}
