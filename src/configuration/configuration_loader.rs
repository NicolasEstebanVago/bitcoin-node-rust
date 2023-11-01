use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::SplitN;

#[derive(Debug, PartialEq)]
/// #ENUM ConfigurationError
/// Represents an enum whit possible values of errors
pub enum ConfigurationError {
    ValueNotExist,
    ReadFileFail,
    ReadLineFail,
    FormatFileLineFail,
}

/// #TDA Configuration
/// Contains the keys and values ​​obtained from the configuration file
pub struct Configuration {
    dictionary_keys_values: HashMap<String, String>,
}

impl Configuration {
    ///Constructor
    /// Returns a struct Configuration without keys and values
    pub fn new() -> Configuration {
        Configuration {
            dictionary_keys_values: HashMap::new(),
        }
    }

    ///Read the keys and values of a specific file
    pub fn load_configuration(&mut self, file_path: &str) -> Result<(), ConfigurationError> {
        let file = self.read_config_file(file_path)?;
        let buffer_file = BufReader::new(file);

        for line in buffer_file.lines() {
            let line = self.validate_line(line)?;

            let mut parts = line.splitn(2, '=');
            let key = self.get_data_from_parts(&mut parts, &line)?;
            let value = self.get_data_from_parts(&mut parts, &line)?;

            self.insert_key_value(key, value);
        }

        Ok(())
    }

    /// Get a key or value from a splitN that contains one of it
    fn get_data_from_parts(
        &self,
        parts: &mut SplitN<char>,
        line: &String,
    ) -> Result<String, ConfigurationError> {
        match parts.next() {
            None => {
                println!("Error: Verify the next line format in the config file");
                println!("{}", line);
                Err(ConfigurationError::FormatFileLineFail)
            }
            Some(k) => Ok(k.trim().to_string()),
        }
    }

    /// Validate if a line is correct
    fn validate_line(
        &self,
        line_result: Result<String, std::io::Error>,
    ) -> Result<String, ConfigurationError> {
        match line_result {
            Ok(line_string_ok) => Ok(line_string_ok),
            Err(_) => {
                println!(
                    "Error: Error ocurred while formatting one of the lines, verify config file"
                );
                Err(ConfigurationError::ReadLineFail)
            }
        }
    }

    /// Open a file from a specific path
    fn read_config_file(&self, file_path: &str) -> Result<File, ConfigurationError> {
        let file = File::open(file_path);

        match file {
            Ok(file_ok) => Ok(file_ok),
            Err(_) => {
                println!("Error: Failed to read file located in  {}", file_path);
                Err(ConfigurationError::ReadFileFail)
            }
        }
    }

    /// Insert a key value in the dictionary
    fn insert_key_value(&mut self, key: String, value: String) {
        self.dictionary_keys_values.insert(key, value);
    }

    /// Get the value from a key
    pub fn get_value_from_key(&mut self, key: String) -> Result<String, ConfigurationError> {
        let value = self.dictionary_keys_values.get(&key);
        match value {
            None => {
                println!("Error: : The key {} has no associated value", key);
                Err(ConfigurationError::ValueNotExist)
            }
            Some(v) => Ok(v.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_value_from_key_return_correct_value() {
        let mut configuration = Configuration::new();
        configuration.insert_key_value(String::from("protocol"), String::from("18333"));
        let value_result: String = match configuration.get_value_from_key(String::from("protocol"))
        {
            Ok(value) => value,
            Err(_) => {
                return;
            }
        };
        assert_eq!(value_result, String::from("18333"));
    }

    #[test]
    pub fn test_get_value_from_key_return_error_because_not_exists() {
        let mut configuration = Configuration::new();
        configuration.insert_key_value(String::from("protocol"), String::from("18333"));
        let value_result = configuration.get_value_from_key(String::from("direction"));
        assert!(value_result.is_err());
    }

    #[test]
    pub fn test_initialize_configuration_with_file_save_correctly_the_keys_and_values() {
        let mut configuration = Configuration::new();
        let load_file = configuration.load_configuration("nodo.conf");
        assert!(load_file.is_ok());
        let value_result = configuration.get_value_from_key(String::from("direction"));
        assert!(value_result.is_ok());
        let value_result = configuration.get_value_from_key(String::from("protocol_version"));
        assert!(value_result.is_ok());
    }

    #[test]
    pub fn test_initialize_configuration_with_file_return_error_because_file_not_exists() {
        let mut configuration = Configuration::new();
        let load_file = configuration.load_configuration("prueba.conf");
        assert!(load_file.is_err());
    }
}
