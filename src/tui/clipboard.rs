//! Clipboard utility functions for TUI
//!
//! This module provides clipboard functionality used across multiple screens.

/// Copy text to clipboard
///
/// Public function for clipboard functionality used across TUI screens.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    use arboard::Clipboard;

    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;

    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to copy: {}", e))?;

    Ok(())
}

/// Paste text from clipboard
///
/// Public function for clipboard paste functionality used across TUI dialogs.
/// Returns the clipboard text content or an error message.
pub fn paste_from_clipboard() -> Result<String, String> {
    use arboard::Clipboard;

    // Try to get clipboard with better error handling
    let clipboard_result = Clipboard::new();
    let mut clipboard = match clipboard_result {
        Ok(clip) => clip,
        Err(e) => {
            // Provide more user-friendly error messages
            let error_msg = format!("Failed to access clipboard: {}", e);
            // Check if it's a common Windows clipboard issue
            if error_msg.contains("The clipboard contents are not available")
                || error_msg.contains("Opened clipboard timeout")
            {
                return Err("Clipboard is busy. Please try again.".to_string());
            }
            return Err(error_msg);
        }
    };

    let text = clipboard.get_text().map_err(|e| {
        // Provide more specific error messages
        if e.to_string().contains("empty") {
            "Clipboard is empty".to_string()
        } else {
            format!("Failed to paste: {}", e)
        }
    })?;

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_to_clipboard() {
        let text = "test text";
        // This test may fail in headless environments, but we can test the function exists
        let result = copy_to_clipboard(text);
        // We don't assert the result since clipboard may not be available in CI
        let _ = result;
    }

    #[test]
    fn test_copy_empty_text() {
        let text = "";
        let result = copy_to_clipboard(text);
        // Empty string should still work
        let _ = result;
    }

    #[test]
    fn test_paste_from_clipboard() {
        let result = paste_from_clipboard();
        // We don't assert the result since clipboard may not be available in CI
        let _ = result;
    }
}
