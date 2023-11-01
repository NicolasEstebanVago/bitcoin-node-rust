use super::configuration_loader::{Configuration, ConfigurationError};
use std::env;

/// #ENUM ParametersEnterError
/// Represents the possible errors in the parameters input
pub enum ParametersEnterError {
    FileConfigurationPathParameter,
}

/// Gets the configuration lodaded
pub fn get_configuration() -> Result<Configuration, ConfigurationError> {
    let file_path =
        read_configuration_file_path_parameter().map_err(|_| ConfigurationError::ReadFileFail)?;

    let mut configuration = Configuration::new();
    configuration
        .load_configuration(&file_path)
        .map_err(|_| ConfigurationError::ReadFileFail)?;

    Ok(configuration)
}

///Read the parameters and return de ubication of file configuration
pub fn read_configuration_file_path_parameter() -> Result<String, ParametersEnterError> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => Ok(args[1].clone()),
        _ => {
            println!("Error: you should enter the location of the configuration file");
            Err(ParametersEnterError::FileConfigurationPathParameter)
        }
    }
}
