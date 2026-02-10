use std::str::FromStr;
use std::fmt::Display;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("I/O failure: {0}")]
    Io(#[from] io::Error),

    #[error("Parse failure: {0}")]
    Parse(String),
}

/// Abstraction for input sources, enabling easy Mocking for unit tests.
pub trait InputProvider {
    fn read_line(&mut self, prompt: &str) -> Result<String, InputError>;
}

/// Orchestrates input logic and type conversion.
pub struct InputHandler<I: InputProvider> {
    provider: I,
}

impl<I: InputProvider> InputHandler<I> {
    pub fn new(provider: I) -> Self {
        Self { provider }
    }

    pub fn get_string(&mut self, prompt: &str) -> Result<String, InputError> {
        self.provider.read_line(prompt)
    }

    pub fn get_string_trimmed(&mut self, prompt: &str) -> Result<String, InputError> {
        self.get_string(prompt).map(|s| s.trim().to_string())
    }

    /// Parses raw input. Use this when leading/trailing whitespace is significant.
    #[allow(dead_code)]
    pub fn parse<T>(&mut self, prompt: &str) -> Result<T, InputError> 
    where 
        T: FromStr,
        T::Err: Display 
    {
        let s = self.get_string(prompt)?;
        s.parse::<T>().map_err(|e| InputError::Parse(e.to_string()))
    }

    /// Parses trimmed input. Ideal for numeric inputs or clean identifiers.
    pub fn parse_trimmed<T>(&mut self, prompt: &str) -> Result<T, InputError> 
    where 
        T: FromStr,
        T::Err: Display 
    {
        let s = self.get_string_trimmed(prompt)?;
        s.parse::<T>().map_err(|e| InputError::Parse(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    /// A fast, O(1) provider for automated testing without actual terminal interaction.
    struct MockProvider {
        responses: VecDeque<String>,
    }

    impl InputProvider for MockProvider {
        fn read_line(&mut self, _p: &str) -> Result<String, InputError> {
            self.responses.pop_front()
                .ok_or_else(|| InputError::Io(io::Error::new(io::ErrorKind::UnexpectedEof, "No more responses")))
        }
    }

    #[test]
    fn test_parse_trimmed_success() {
        let responses = VecDeque::from(vec![" 42 ".to_string()]);
        let mock = MockProvider { responses };
        let mut handler = InputHandler::new(mock);
        assert_eq!(handler.parse_trimmed::<u32>("test").unwrap(), 42);
    }

    #[test]
    fn test_parse_raw_fails_with_spaces() {
        let responses = VecDeque::from(vec![" 42 ".to_string()]);
        let mock = MockProvider { responses };
        let mut handler = InputHandler::new(mock);
        let result = handler.parse::<u32>("test");
        // Verify that raw parse does NOT automatically trim, causing u32 to fail.
        assert!(matches!(result, Err(InputError::Parse(_))));
    }

    #[test]
    fn test_trim_empty_input() {
        let responses = VecDeque::from(vec!["   ".to_string()]);
        let mock = MockProvider { responses };
        let mut handler = InputHandler::new(mock);
        let result = handler.get_string_trimmed("test").unwrap();
        assert!(result.is_empty());
    }
}