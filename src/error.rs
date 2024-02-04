#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Docker API error: {0}")]
    Docker(#[from] bollard::errors::Error),

    #[error("No sudo program: {0}")]
    Sudo(Box<dyn std::error::Error>),

    #[error("Pass network name as argument. Select one of:\n{0:#?}")]
    NetworkNotSelect(Vec<String>),

    #[error("Wrong network: {selected_network}.\nSelect one of:\n{networks:#?}")]
    NetworkInvalidChoice {
        selected_network: String,
        networks: Vec<String>,
    },

    #[error("Error when writing file: {0}")]
    WriteFile(std::io::Error),
}
