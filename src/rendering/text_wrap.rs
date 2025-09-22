use crate::{
    common::{END_SEQ, FLAG_SEQ, START_SEQ},
    rendering::text_content_length,
};

/// A word with its visual representation and any formatting codes
#[derive(Debug, Clone)]
pub struct FormattedWord {
    pub text: String,               // The actual word text (may include formatting)
    pub visual_length: usize,       // Length without formatting codes
    pub formatting: Option<String>, // Active formatting codes like "R-G-B-y repeat"
}

/// A complete line with formatting preserved
#[derive(Debug, Clone)]
pub struct FormattedLine {
    pub text: String,         // Complete line with formatting
    pub visual_length: usize, // Visual length without formatting
    pub words: Vec<FormattedWord>,
}

/// Wraps text to fit within specified width, preserving formatting
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![];
    }

    let words = split_formatted_text_into_words(text);
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_visual_length = 0;

    for word in words {
        let word_visual_length = word.visual_length;
        let needs_space = !current_line.is_empty() && current_visual_length > 0;
        let space_cost = if needs_space { 1 } else { 0 };

        // Check if adding this word would exceed max width
        if current_visual_length + space_cost + word_visual_length > max_width
            && !current_line.is_empty()
        {
            lines.push(current_line);
            current_line = String::new();
            current_visual_length = 0;
        }

        // Add space if needed (but not at the start of a new line)
        if needs_space && current_visual_length > 0 {
            current_line.push(' ');
            current_visual_length += 1;
        }

        // Handle words that are too long for a single line
        if word_visual_length > max_width {
            // For now, just add the word anyway - we can implement proper breaking later
            current_line.push_str(&word.text);
            current_visual_length += word_visual_length;
        } else {
            current_line.push_str(&word.text);
            current_visual_length += word_visual_length;
        }
    }

    // Add final line if not empty
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Handle empty input
    if lines.is_empty() && !text.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Splits a formatted text string into words while preserving formatting
pub fn split_formatted_text_into_words(text: &str) -> Vec<FormattedWord> {
    let mut words = Vec::new();
    let mut current_word = String::new();
    let mut current_word_visual = String::new();
    let mut in_seq = false;
    let mut in_flags = false;
    let mut seq_setting = String::new();
    let mut seq_value = String::new();

    for c in text.chars() {
        if c == START_SEQ {
            in_seq = true;
            in_flags = true;
            seq_setting.clear();
            seq_value.clear();
            current_word.push(c);
            continue;
        }

        if in_seq && c == END_SEQ {
            in_seq = false;
            in_flags = false;
            current_word.push(c);
            current_word_visual.push_str(&seq_value);
            seq_setting.clear();
            seq_value.clear();
            continue;
        }

        if in_seq && c == FLAG_SEQ {
            in_flags = false;
            current_word.push(c);
            continue;
        }

        if in_flags {
            seq_setting.push(c);
            current_word.push(c);
            continue;
        }

        if in_seq {
            seq_value.push(c);
            current_word.push(c);
            continue;
        }

        // Regular character processing
        if c.is_whitespace() {
            if !current_word.is_empty() {
                let visual_len = current_word_visual.len();
                words.push(FormattedWord {
                    text: current_word.clone(),
                    visual_length: visual_len,
                    formatting: None, // We'll let wrap_text handle formatting continuation
                });
                current_word.clear();
                current_word_visual.clear();
            }
        } else {
            current_word.push(c);
            current_word_visual.push(c);
        }
    }

    // Handle any remaining word
    if !current_word.is_empty() {
        let visual_len = current_word_visual.len();
        words.push(FormattedWord {
            text: current_word,
            visual_length: visual_len,
            formatting: None, // We'll let wrap_text handle formatting continuation
        });
    }

    words
}

/// Breaks a long word into smaller chunks that fit within max_width
fn break_long_word(
    word: &FormattedWord,
    max_width: usize,
    active_formatting: &Option<String>,
) -> Vec<FormattedWord> {
    let mut result = Vec::new();

    // For now, we'll do a simple character-by-character break
    // This could be enhanced to respect formatting boundaries better
    let chars: Vec<char> = word.text.chars().collect();
    let mut current_chunk = String::new();
    let mut current_visual_length = 0;

    for c in chars {
        // Skip formatting characters when counting length
        if c == START_SEQ || c == END_SEQ || c == FLAG_SEQ {
            current_chunk.push(c);
            continue;
        }

        if current_visual_length + 1 > max_width && !current_chunk.is_empty() {
            result.push(FormattedWord {
                text: current_chunk.clone(),
                visual_length: current_visual_length,
                formatting: active_formatting.clone(),
            });
            current_chunk.clear();
            current_visual_length = 0;
        }

        current_chunk.push(c);
        current_visual_length += 1;
    }

    if !current_chunk.is_empty() {
        result.push(FormattedWord {
            text: current_chunk,
            visual_length: current_visual_length,
            formatting: active_formatting.clone(),
        });
    }

    if result.is_empty() {
        // Fallback: return original word even if too long
        result.push(word.clone());
    }

    result
}

/// Gets the visual length of text, ignoring formatting codes (wrapper around existing function)
pub fn get_visual_length(text: &str) -> usize {
    text_content_length(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pine_tree_formatting() {
        let input = "{c|P}ine {c|T}ree";
        let wrapped = wrap_text(input, 20);

        println!("Pine Tree input: '{}'", input);
        println!("Pine Tree wrapped: {:?}", wrapped);

        // Should be one line since it's short
        assert_eq!(wrapped.len(), 1);

        // Check that the wrapped text preserves formatting
        let result = &wrapped[0];
        println!("Pine Tree result: '{}'", result);

        // The visual length should be 9 characters ("Pine Tree")
        assert_eq!(text_content_length(result), 9);
    }

    #[test]
    fn test_rattlesnake_formatting() {
        let input = "{Y-y-X-y repeat|Rattlesnake}";
        let wrapped = wrap_text(input, 20);

        println!("Rattlesnake input: '{}'", input);
        println!("Rattlesnake wrapped: {:?}", wrapped);

        // Should be one line since it's short
        assert_eq!(wrapped.len(), 1);

        // Check that the wrapped text preserves formatting
        let result = &wrapped[0];
        println!("Rattlesnake result: '{}'", result);

        // The visual length should be 11 characters ("Rattlesnake")
        assert_eq!(text_content_length(result), 11);
    }

    #[test]
    fn test_brown_bear_formatting() {
        let input = "{X|Brown Bear}";
        let wrapped = wrap_text(input, 20);

        println!("Brown Bear input: '{}'", input);
        println!("Brown Bear wrapped: {:?}", wrapped);

        // Should be one line since it's short
        assert_eq!(wrapped.len(), 1);

        // Check that the wrapped text preserves formatting
        let result = &wrapped[0];
        println!("Brown Bear result: '{}'", result);

        // The visual length should be 10 characters ("Brown Bear")
        assert_eq!(text_content_length(result), 10);
    }

    #[test]
    fn test_split_formatted_text_into_words() {
        let input = "{c|P}ine {c|T}ree";
        let words = split_formatted_text_into_words(input);

        println!("Pine Tree words: {:?}", words);

        // Should have words for each part
        for (i, word) in words.iter().enumerate() {
            println!(
                "Word {}: text='{}', visual_length={}, formatting={:?}",
                i, word.text, word.visual_length, word.formatting
            );
        }
    }

    #[test]
    fn test_split_rattlesnake_into_words() {
        let input = "{Y-y-X-y repeat|Rattlesnake}";
        let words = split_formatted_text_into_words(input);

        println!("Rattlesnake words: {:?}", words);

        // Should be one word
        assert_eq!(words.len(), 1);

        let word = &words[0];
        println!(
            "Rattlesnake word: text='{}', visual_length={}, formatting={:?}",
            word.text, word.visual_length, word.formatting
        );

        // Should have visual length of 11 for "Rattlesnake"
        assert_eq!(word.visual_length, 11);
    }

    #[test]
    fn test_text_content_length_simple() {
        assert_eq!(text_content_length("Hello"), 5);
        assert_eq!(text_content_length(""), 0);
    }

    #[test]
    fn test_text_content_length_formatted() {
        assert_eq!(text_content_length("{R|Hello}"), 5);
        assert_eq!(text_content_length("{Y-y-X-y repeat|Rattlesnake}"), 11);
        assert_eq!(text_content_length("{c|P}ine {c|T}ree"), 9);
    }
}

/// Applies formatting from previous line to current line (helper for manual line building)
pub fn carry_formatting_to_next_line(previous_line: &str, current_line: &str) -> String {
    // Extract the last active formatting from previous line
    let mut active_formatting: Option<String> = None;
    let mut in_seq = false;
    let mut in_flags = false;
    let mut seq_setting = String::new();

    for c in previous_line.chars() {
        if c == START_SEQ {
            in_seq = true;
            in_flags = true;
            seq_setting.clear();
            continue;
        }

        if in_seq && c == END_SEQ {
            in_seq = false;
            in_flags = false;

            if !seq_setting.is_empty() {
                active_formatting = Some(seq_setting.clone());
            }
            continue;
        }

        if in_seq && c == FLAG_SEQ {
            in_flags = false;
            continue;
        }

        if in_flags {
            seq_setting.push(c);
        }
    }

    // If we have active formatting and current line doesn't start with formatting, prepend it
    if let Some(formatting) = active_formatting
        && !current_line.starts_with(START_SEQ)
    {
        return format!(
            "{}{}{}{}{}",
            START_SEQ, formatting, FLAG_SEQ, current_line, END_SEQ
        );
    }

    current_line.to_string()
}
