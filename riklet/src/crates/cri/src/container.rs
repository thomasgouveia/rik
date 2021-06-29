use crate::*;
use shared::utils::find_binary;
use std::time::Duration;
use tokio::process::Command;
use std::process::Stdio;
use snafu::ensure;

#[derive(Debug, Clone, Default)]
pub struct RuncConfiguration {
    /// The path of the runc command. If None, we will try to search the package in your $PATH.
    pub command: Option<PathBuf>,
    /// Timeout for Runc commands.
    pub timeout: Option<Duration>,
    /// The root directory for storage of container state
    pub root: Option<PathBuf>,
    /// Ignore cgroup permission errors
    pub rootless: bool,
    /// Enable debug output for logging
    pub debug: bool
}

/// A basic implementation to interact with the Runc binary
#[derive(Debug)]
pub struct Runc {
    command: PathBuf,
    timeout: Duration,
    root: Option<PathBuf>,
    rootless: bool,
    debug: bool,
}

impl Runc {
    /// Create a Runc instance with the provided configuration.
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
            root: config.root,
            debug: config.debug,
            rootless: config.rootless
        })
    }

    /// List all containers
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

    /// Get the state of a container
    pub async fn state(&self, id: &str) -> Result<Container> {
        let args = vec![String::from("state"), String::from(id)];
        let output = self.exec(&args).await?;
        Ok(serde_json::from_str(&output).context(JsonDeserializationError {})?)
    }
}

impl Args for Runc {
    /// Implement arguments for Runc binary.
    fn args(&self) -> Result<Vec<String>> {
        let mut args: Vec<String> = Vec::new();

        if let Some(root) = self.root.clone() {
            args.push(String::from("--root"));
            args.push(root
                .canonicalize()
                .context(InvalidPathError {})?
                .to_string_lossy()
                .parse()
                .unwrap()
            )
        }

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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use std::env::{temp_dir, var_os, var, current_dir};
    use std::fs::{create_dir_all, File, copy};
    use std::path::{PathBuf, Path};

    use crate::container::{RuncConfiguration, Runc, CreateArgs};
    use crate::console::ConsoleSocket;
    use shared::utils::unpack;
    use tokio::time::sleep;
    use std::time::Duration;
    use tokio::runtime::Runtime;
    use log::error;

    const BUSYBOX_ARCHIVE: &str = "../../../fixtures/busybox.tar.gz";
    const RUNC_FIXTURE: &str = "../../../fixtures/runc.amd64";

    /// Install Runc in the temporary environment for the test & create all directories and files used by the test.
    fn setup_test_sequence() -> (PathBuf, PathBuf) {
        let sequence_id = format!("{}", Uuid::new_v4());
        let mut sequence_path = temp_dir().join(&sequence_id);
        let sequence_root = PathBuf::from(var_os("XDG_RUNTIME_DIR")
            .expect("Expected temporary path"))
            .join("runc")
            .join(&sequence_id);

        create_dir_all(&sequence_root).expect("Unable to create runc root");
        create_dir_all(&sequence_path).expect("Unable to create the temporary folder");

        sequence_path = sequence_path.join("runc.amd64");

        copy(PathBuf::from(RUNC_FIXTURE), &sequence_path)
            .expect("Unable to copy runc binary into the temporary folder.");

        (sequence_path, sequence_root)
    }

    #[test]
    fn test_it_run_a_container() {
        let (runc_path, runc_root) = setup_test_sequence();

        let mut config: RuncConfiguration = Default::default();
        config.command = Some(runc_path);
        config.root = Some(runc_root);

        let runc = Runc::new(config).expect("Unable to create runc instance");

        let task = async move {
            let id = format!("{}", Uuid::new_v4());

            let socket_path = temp_dir().join(&id).with_extension("console");
            let console_socket = ConsoleSocket::new(&socket_path)?;

            tokio::spawn(async move {
                match console_socket.get_listener().as_ref().unwrap().accept().await {
                    Ok((stream, _socket_addr)) => {
                        Box::leak(Box::new(stream));
                    },
                    Err(err) => {
                        error!("Receive PTY master error : {:?}", err)
                    }
                }
            });

            let bundle = temp_dir().join(&id);

            unpack(BUSYBOX_ARCHIVE, &bundle);

            runc.run(&id, &bundle, Some(&CreateArgs {
                pid_file: None,
                console_socket: Some(socket_path),
                no_pivot: false,
                no_new_keyring: false,
                detach: true
            })).await?;


            sleep(Duration::from_millis(500));

            runc.state(&id).await
        };

        let mut runtime = Runtime::new().expect("Unable to create runtime");
        let container = runtime.block_on(task).expect("test failed");

        assert_eq!(container.status, Some(String::from("running")))
    }
}