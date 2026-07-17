use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

const CONFIG_FILE_NAME: &str = "chunky-monkey.toml";
const CONFIG_ENV_VAR: &str = "CHUNKY_MONKEY_CONFIG";
const CONFIG_ARG: &str = "--config";

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub optimizer: OptimizerConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct OptimizerConfig {
    pub model: String,
    pub max_sample_tokens: usize,
    pub tramway_url: String,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            model: Config::default_model().to_string(),
            max_sample_tokens: Config::default_max_sample_tokens(),
            tramway_url: Config::default_tramway_url().to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let args = env::args().collect::<Vec<_>>();
        let env_path = env::var_os(CONFIG_ENV_VAR).map(PathBuf::from);
        let cwd = env::current_dir()?;
        Self::load_from_sources(&args, env_path.as_deref(), &cwd)
    }

    pub fn default_model() -> &'static str {
        "claude-sonnet-4-6"
    }

    pub fn default_max_sample_tokens() -> usize {
        500
    }

    pub fn default_tramway_url() -> &'static str {
        "http://localhost:8080"
    }

    fn load_from_sources(args: &[String], env_path: Option<&Path>, cwd: &Path) -> Result<Self> {
        let cli_path = cli_config_path(args);
        let cwd_path = cwd.join(CONFIG_FILE_NAME);

        let selected_path = [cli_path.as_deref(), env_path, Some(cwd_path.as_path())]
            .into_iter()
            .flatten()
            .find(|path| path.is_file());

        match selected_path {
            Some(path) => {
                let contents = fs::read_to_string(path)?;
                Ok(toml::from_str(&contents)?)
            }
            None => Ok(Self::default()),
        }
    }
}

fn cli_config_path(args: &[String]) -> Option<PathBuf> {
    let mut iter = args.iter().skip(1);

    while let Some(arg) = iter.next() {
        if arg == CONFIG_ARG {
            return iter.next().map(PathBuf::from);
        }

        if let Some(path) = arg.strip_prefix("--config=") {
            return Some(PathBuf::from(path));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::Config;

    #[test]
    fn returns_defaults_when_no_config_exists() {
        let cwd = unique_test_dir("defaults");

        let config = Config::load_from_sources(&["chunky-monkey".into()], None, &cwd).unwrap();

        assert_eq!(config, Config::default());
    }

    #[test]
    fn prefers_cli_config_path() {
        let cwd = unique_test_dir("cli");
        let cli_path = write_config(&cwd.join("cli.toml"), "claude-cli", 600, "http://cli");
        let env_path = write_config(&cwd.join("env.toml"), "claude-env", 700, "http://env");
        write_config(&cwd.join("chunky-monkey.toml"), "claude-cwd", 800, "http://cwd");

        let config = Config::load_from_sources(
            &[
                "chunky-monkey".into(),
                "--config".into(),
                cli_path.display().to_string(),
            ],
            Some(&env_path),
            &cwd,
        )
        .unwrap();

        assert_eq!(config.optimizer.model, "claude-cli");
        assert_eq!(config.optimizer.max_sample_tokens, 600);
        assert_eq!(config.optimizer.tramway_url, "http://cli");
    }

    #[test]
    fn falls_back_from_missing_cli_to_env_then_cwd() {
        let cwd = unique_test_dir("fallback");
        let env_path = write_config(&cwd.join("env.toml"), "claude-env", 700, "http://env");
        write_config(&cwd.join("chunky-monkey.toml"), "claude-cwd", 800, "http://cwd");

        let config = Config::load_from_sources(
            &[
                "chunky-monkey".into(),
                "--config".into(),
                cwd.join("missing.toml").display().to_string(),
            ],
            Some(&env_path),
            &cwd,
        )
        .unwrap();

        assert_eq!(config.optimizer.model, "claude-env");
        assert_eq!(config.optimizer.max_sample_tokens, 700);
        assert_eq!(config.optimizer.tramway_url, "http://env");
    }

    #[test]
    fn parses_config_equals_cli_argument() {
        let cwd = unique_test_dir("cli-equals");
        let cli_path = write_config(&cwd.join("cli.toml"), "claude-cli", 600, "http://cli");

        let config = Config::load_from_sources(
            &["chunky-monkey".into(), format!("--config={}", cli_path.display())],
            None,
            &cwd,
        )
        .unwrap();

        assert_eq!(config.optimizer.model, "claude-cli");
    }

    fn write_config(path: &Path, model: &str, max_sample_tokens: usize, tramway_url: &str) -> PathBuf {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                "[optimizer]\nmodel = \"{model}\"\nmax_sample_tokens = {max_sample_tokens}\ntramway_url = \"{tramway_url}\"\n"
            ),
        )
        .unwrap();
        path.to_path_buf()
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = env::temp_dir().join(format!("chunky-monkey-{name}-{suffix}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
