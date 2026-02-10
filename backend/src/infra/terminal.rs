use crate::core::input::{InputError, InputProvider};
use std::io::{self, Write};

/// Real terminal-based input provider for production use.
pub struct TerminalInput;

impl InputProvider for TerminalInput {
    fn read_line(&mut self, prompt: &str) -> Result<String, InputError> {
        print!("{}", prompt);
        io::stdout().flush()?;

        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        Ok(buf.trim_end_matches('\n').trim_end_matches('\r').to_string())
    }
}
