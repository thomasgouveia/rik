use crate::*;
use shared::utils::find_binary;
use std::time::Duration;
use tokio::process::Command;
use std::process::Stdio;
use snafu::ensure;

#[derive(Debug, Clone, Default)]
pub struct RuncConfiguration {
    pub command: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub rootless: bool,
    pub debug: bool
}

/// A basic implementation to interact with the Runc binary
#[derive(Debug)]
pub struct Runc {
    command: PathBuf,
    timeout: Duration,
    rootless: bool,
    debug: bool,
}

impl Runc {
    pub fn new(config: RuncConfiguration) -> Result<Self> {
        let command = config
            .command
            .or_else(|| find_binary("runc"))
            .context(RuncNotFoundError {})?;

        let timeout = config.timeout.or(Some(Duration::from_millis(5000))).unwrap();

        debug!("Runc initialized.");

        Ok(Self {
            command,
            timeout,
            debug: config.debug,
            rootless: config.rootless
        })
    }

    /// List containers
    pub async fn list(&self) -> Result<Vec<Container>> {
        let args = vec![String::from("list"), String::from("--format=json")];
        let mut output = self.exec(&args).await?;
        output = output.trim().to_string();

        Ok(
            if output == "null" {
                Vec::new()
            } else {
                serde_json::from_str(&output).unwrap()
            }
        )
    }

    /// Run a container.
    pub async fn run(&self, id: &str, bundle: &PathBuf, opts: Option<&CreateArgs>) -> Result<()> {
        let mut args = vec![String::from("run")];
        Self::append_opts(&mut args, opts.map(|opts| opts as &dyn Args))?;

        let bundle: String = bundle
            .canonicalize()
            .context(InvalidPathError {})?
            .to_string_lossy()
            .parse()
            .unwrap();

        args.push(String::from("--bundle"));
        args.push(bundle);
        args.push(String::from(id));
        self.exec(&args).await.map(|_|())
    }
}

impl Args for Runc {
    /// Implement arguments for Runc binary.
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if self.rootless {
            args.push(format!("--rootless={}", self.rootless))
        }

        if self.debug {
            args.push(String::from("--debug"));
        }

        Ok(args)
    }
}

#[async_trait]
impl Executable for Runc {

    async fn exec(&self, args: &[String]) -> Result<String> {
        let args = self.concat_args(args)?;
        let process = Command::new(&self.command)
            .args(&args.clone())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(ProcessSpawnError {})?;

        debug!("{} {}", self.command.to_str().unwrap(), &args.clone().join(" "));

        let result = tokio::time::timeout(self.timeout, process.wait_with_output())
            .await
            .context(RuncCommandTimeoutError {})?
            .context(RuncCommandError {})?;

        let stdout = String::from_utf8(result.stdout.clone()).unwrap();
        let stderr = String::from_utf8(result.stderr.clone()).unwrap();

        if stderr != "" {
            error!("Runc error : {}", stderr);
        }

        ensure!(
            result.status.success(),
            RuncCommandFailedError {
                stdout: stdout,
                stderr: stderr
            }
        );

        Ok(stdout)
    }
}

/// runc create arguments
pub struct CreateArgs {
    pub pid_file: Option<PathBuf>,
    pub console_socket: Option<PathBuf>,
    pub no_pivot: bool,
    pub no_new_keyring: bool,
    pub detach: bool
}

impl Args for CreateArgs {
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if let Some(pid_file) = self.pid_file.clone() {
            args.push(String::from("--pid-file"));
            args.push(pid_file.to_string_lossy().parse().unwrap())
        }

        if let Some(console_socket) = self.console_socket.clone() {
            args.push(String::from("--console-socket"));
            args.push(
                console_socket
                    .canonicalize()
                    .context(InvalidPathError {})?
                    .to_string_lossy()
                    .parse()
                    .unwrap()
            )
        }

        if self.no_pivot {
            args.push(String::from("--no-pivot"))
        }

        if self.no_new_keyring {
            args.push(String::from("--no-new-keyring"))
        }

        if self.detach {
            args.push(String::from("--detach"))
        }

        Ok(args)
    }
}