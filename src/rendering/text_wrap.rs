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
    let mut active_formatting: Option<String> = None;

    for word in words {
        let word_visual_length = word.visual_length;
        let needs_space = !current_line.is_empty() && current_visual_length > 0;
        let space_cost = if needs_space { 1 } else { 0 };

        // Check if adding this word would exceed max width
        if current_visual_length + space_cost + word_visual_length > max_width
            && !current_line.is_empty()
        {
            // Close any active formatting at end of line
            if active_formatting.is_some() {
                current_line.push(END_SEQ);
            }

            lines.push(current_line);
            current_line = String::new();
            current_visual_length = 0;

            // If we have active formatting, start the new line with it
            if let Some(ref fmt) = active_formatting {
                current_line.push(START_SEQ);
                current_line.push_str(fmt);
                current_line.push(FLAG_SEQ);
            }
        }

        // Add space if needed
        if needs_space && current_visual_length > 0 {
            current_line.push(' ');
            current_visual_length += 1;
        }

        // Handle words that are too long for a single line
        if word_visual_length > max_width {
            let broken_words = break_long_word(&word, max_width, &active_formatting);
            for (i, broken_word) in broken_words.iter().enumerate() {
                if i > 0 {
                    // Close formatting and start new line
                    if active_formatting.is_some() {
                        current_line.push(END_SEQ);
                    }
                    lines.push(current_line);
                    current_line = String::new();
                    current_visual_length = 0;

                    // Start new line with formatting if needed
                    if let Some(ref fmt) = active_formatting {
                        current_line.push(START_SEQ);
                        current_line.push_str(fmt);
                        current_line.push(FLAG_SEQ);
                    }
                }

                current_line.push_str(&broken_word.text);
                current_visual_length += broken_word.visual_length;
            }
        } else {
            current_line.push_str(&word.text);
            current_visual_length += word_visual_length;
        }

        // Update active formatting based on this word
        if let Some(ref fmt) = word.formatting {
            active_formatting = Some(fmt.clone());
        }
    }

    // Close any active formatting at end of final line
    if !current_line.is_empty() {
        if active_formatting.is_some() {
            current_line.push(END_SEQ);
        }
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
    let mut active_formatting: Option<String> = None;
    let mut in_seq = false;
    let mut in_flags = false;
    let mut seq_setting = String::new();
    let mut seq_value = String::new();

    for c in text.chars() {
        if c == START_SEQ {
            // If we were building a word, finish it first
            if !current_word.is_empty() {
                let visual_len = text_content_length(&current_word);
                words.push(FormattedWord {
                    text: current_word.clone(),
                    visual_length: visual_len,
                    formatting: active_formatting.clone(),
                });
                current_word.clear();
            }

            in_seq = true;
            in_flags = true;
            seq_setting.clear();
            seq_value.clear();
            continue;
        }

        if in_seq && c == END_SEQ {
            in_seq = false;
            in_flags = false;

            // Update active formatting
            if !seq_setting.is_empty() {
                active_formatting = Some(seq_setting.clone());
            }

            // If there's content in the sequence, treat it as a formatted word
            if !seq_value.is_empty() {
                let formatted_text = format!(
                    "{}{}{}{}{}",
                    START_SEQ, seq_setting, FLAG_SEQ, seq_value, END_SEQ
                );
                let visual_len = text_content_length(&seq_value);
                words.push(FormattedWord {
                    text: formatted_text,
                    visual_length: visual_len,
                    formatting: Some(seq_setting.clone()),
                });
            }

            seq_setting.clear();
            seq_value.clear();
            continue;
        }

        if in_seq && c == FLAG_SEQ {
            in_flags = false;
            continue;
        }

        if in_flags {
            seq_setting.push(c);
            continue;
        }

        if in_seq {
            seq_value.push(c);
            continue;
        }

        // Regular character processing
        if c.is_whitespace() {
            if !current_word.is_empty() {
                let visual_len = text_content_length(&current_word);
                words.push(FormattedWord {
                    text: current_word.clone(),
                    visual_length: visual_len,
                    formatting: active_formatting.clone(),
                });
                current_word.clear();
            }
        } else {
            current_word.push(c);
        }
    }

    // Handle any remaining word
    if !current_word.is_empty() {
        let visual_len = text_content_length(&current_word);
        words.push(FormattedWord {
            text: current_word,
            visual_length: visual_len,
            formatting: active_formatting,
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
    if let Some(formatting) = active_formatting {
        if !current_line.starts_with(START_SEQ) {
            return format!(
                "{}{}{}{}{}",
                START_SEQ, formatting, FLAG_SEQ, current_line, END_SEQ
            );
        }
    }

    current_line.to_string()
}
