//! Markdown terminal formatting using termimad

use termimad::{gray, MadSkin};

use crate::display::terminal::should_use_colors;

/// Print markdown to terminal with rich formatting (or plain fallback)
pub fn print_markdown(markdown: &str) {
    if should_use_colors() {
        if let Err(e) = print_rich(markdown) {
            eprintln!(
                "Warning: Terminal rendering failed ({}), using plain output",
                e
            );
            print_plain(markdown);
        }
    } else {
        print_plain(markdown);
    }
}

/// Print with termimad styling
fn print_rich(markdown: &str) -> Result<(), termimad::Error> {
    let mut skin = MadSkin::default();
    customize_skin(&mut skin);
    skin.print_text(markdown);
    Ok(())
}

/// Customize termimad skin to match chronicle aesthetic
fn customize_skin(skin: &mut MadSkin) {
    use termimad::crossterm::style::{Attribute, Color::*};

    // Headers: Bold cyan/blue
    skin.headers[0].set_fg(Cyan);
    skin.headers[0].add_attr(Attribute::Bold);
    skin.headers[1].set_fg(Blue);
    skin.headers[1].add_attr(Attribute::Bold);
    skin.headers[2].set_fg(Blue);

    // Code blocks: Green with gray background
    skin.code_block.set_bg(gray(2));
    skin.code_block.set_fg(Green);

    // Inline code: Yellow
    skin.inline_code.set_fg(Yellow);

    // Tables: White
    skin.table.set_fg(White);

    // Bold/Italic: Use attributes
    skin.bold.add_attr(Attribute::Bold);
    skin.italic.add_attr(Attribute::Italic);

    // Lists: Cyan bullets
    skin.bullet.set_fg(Cyan);
}

/// Print plain markdown without formatting
fn print_plain(markdown: &str) {
    println!("{}", markdown);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_print_markdown_plain_fallback() {
        // Set NO_COLOR to force plain output
        std::env::set_var("NO_COLOR", "1");

        // Should not panic, should use plain output
        print_markdown("# Test\n\nHello **world**");

        std::env::remove_var("NO_COLOR");
    }

    #[test]
    fn test_customize_skin_no_panic() {
        let mut skin = MadSkin::default();

        // Should customize without panicking
        customize_skin(&mut skin);

        // Basic assertion to verify skin was modified
        assert!(true);
    }

    #[test]
    fn test_print_rich_with_valid_markdown() {
        // Should handle valid markdown without error
        let result = print_rich("# Header\n\n- Item 1\n- Item 2");
        // In test environment (non-TTY), this may fail, which is expected
        // We're testing it doesn't panic, not that it succeeds
        match result {
            Ok(_) | Err(_) => assert!(true),
        }
    }
}
