use std::collections::HashMap;
use crate::{flag::Flag, module_flag::ModuleFlags};
use alloc::{string::{String, ToString}, vec::Vec};

/// `FlagParser` acts as the central engine for interpreting and evaluating bit-based flags.
///
/// It maintains a registry of "Modules," where each module is identified by a unique name
/// and contains a bitmask representing various toggled states.
pub struct FlagParser {
    /// A map where keys are module names (e.g., "Core", "Network") and values 
    /// are the actual bitmask wrappers (`ModuleFlags`).
    pub module_flags: HashMap<String, ModuleFlags>,
}
impl Default for FlagParser {
    fn default() -> Self {
        Self { module_flags: Default::default() }
    }
}
impl FlagParser {
    /// Initializes a fresh parser with an empty registry.
    pub fn new() -> Self {
        Self {
            module_flags: HashMap::new(),
        }
    }

    /// Parses a raw configuration string and populates the module registry.
    ///
    /// # Expected Format
    /// `module_name:binary_bits` (e.g., "Network:1011 Core:001")
    ///
    /// # Logic
    /// 1. Splits the input string into individual tokens by whitespace.
    /// 2. For each token, splits at the `:` to separate the name from the bitmask.
    /// 3. Reverses the bits (right-to-left) so that index 0 corresponds to the least significant bit.
    pub fn parse_flag_string(&mut self, flag_str: &str) {
        for token in flag_str.split_whitespace() {
            if let Some((name, bits)) = token.split_once(':') {
                let mut module = ModuleFlags::new();
                
                // We iterate through bits in reverse because binary strings read 
                // most-significant to least-significant (e.g., '10' means index 1 is set).
                for (i, c) in bits.chars().rev().enumerate() {
                    if c == '1' {
                        module.set_flag(i as u64);
                    }
                }
                self.module_flags.insert(name.to_string(), module);
            }
        }
    }

    /// Entry point for the boolean expression engine.
    /// 
    /// Evaluates complex strings like `!Network.Flag1 && Core.Flag0`.
    pub fn check_complex_pattern(&self, pattern: &str) -> bool {
        let tokens = Self::tokenize(pattern);
        Self::eval_tokens(&tokens, &self.module_flags)
    }

    /// Pre-processor for the logic engine. 
    /// 
    /// Injects spaces around logical operators and negations to ensure the 
    /// split-by-whitespace method can capture them accurately.
    fn tokenize(input: &str) -> Vec<String> {
        input
            .replace("&&", " && ")
            .replace("||", " || ")
            .replace("!", " !") // Allows !Module.Flag to be handled as a prefix or separate token
            .split_whitespace()
            .map(String::from)
            .collect()
    }

    /// Core evaluation loop for tokenized boolean logic.
    ///
    /// This is a basic "Linear Evaluator" (it processes left-to-right).
    /// Note: It does not currently support operator precedence (Parentheses).
    fn eval_tokens(tokens: &[String], flags: &HashMap<String, ModuleFlags>) -> bool {
        let mut result = None;
        let mut current_op = None;

        for token in tokens {
            match token.as_str() {
                // Set the pending operation for the next value encountered
                "&&" => current_op = Some("&&"),
                "||" => current_op = Some("||"),
                _ => {
                    // Check for '!' negation prefix
                    let negated = token.starts_with('!');
                    let flag_expr = token.trim_start_matches('!');
                    
                    // Parse the "Module.FlagX" format
                    let value = match flag_expr.split_once('.') {
                        Some((module, index_str)) => {
                            // Extract numeric index by stripping the "Flag" prefix
                            if let Ok(index) = index_str.trim_start_matches("Flag").parse::<u64>() {
                                flags.get(module)
                                    .map(|m| m.check_flag(index))
                                    .unwrap_or(false) // Module not found = False
                            } else {
                                false // Invalid index format = False
                            }
                        }
                        None => false, // No '.' separator = False
                    };
                    
                    // Apply negation if '!' was present
                    let value = if negated { !value } else { value };

                    // Combine the parsed value with the existing running total
                    result = Some(match (result, current_op) {
                        (Some(lhs), Some("&&")) => lhs && value,
                        (Some(lhs), Some("||")) => lhs || value,
                        (None, _) => value, // First token in the expression
                        (Some(_), _) => value, // Safety fallback for malformed ops
                    });
                }
            }
        }

        result.unwrap_or(false)
    }
}