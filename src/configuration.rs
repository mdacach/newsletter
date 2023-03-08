use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub smtp: SMTPSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct SMTPSettings {
    pub username: String,
    pub password: Secret<String>,
    pub from: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub hmac_secret: Secret<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    // Secret does not let we expose this by mistake (e.g. Debug display)
    // and will also make sure it gets zeroed out in memory due to Zeroize trait
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }
    // We want to connect with postgres itself directly in order to create new logical databases.
    // This is used for testing.
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
                Use either `local` or `production`.",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Merge variables in .env file to OS environment variables.
    // This makes the variables accessible for `config` below.
    dotenv::dotenv().ok();

    let bash_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = bash_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into()) // By default, we use Local environment.
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    // Depending on the environment (local or production), we load the corresponding configuration
    // file.
    let environment_filename = format!("{}.yaml", environment.as_str());

    let base = config::File::from(configuration_directory.join("base.yaml"));
    let environment = config::File::from(configuration_directory.join(environment_filename));

    let settings = config::Config::builder()
        .add_source(base)
        .add_source(environment)
        // This determines the format of environment variables we must set.
        // APP_SMTP__USERNAME will map to Settings.smtp.username
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    // Serde will return to it strongly typed.
    settings.try_deserialize::<Settings>()
}
