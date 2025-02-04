use anyhow::Context;
use dotenv;

use crate::error::app_error::AppError;

pub fn init() {
    dotenv::dotenv().ok();
}

pub fn get(parameter: &str) -> Result<String, AppError> {
    std::env::var(parameter)
        .with_context(|| format!("{} is not defined in the environment.", parameter))
        .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::config::parameter;

    #[test]
    fn test_get_existing_parameter() {
        env::set_var("TEST_PARAM", "test_value");
        let result = parameter::get("TEST_PARAM");
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_value");
        
        env::remove_var("TEST_PARAM");
    }

    #[test]
    fn test_get_missing_parameter() {
        let result = parameter::get("MISSING_PARAM");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MISSING_PARAM is not defined"));
    }
}
