// use async_trait::async_trait;
// use std::time::Duration;
// use tokio::process::Command;
// use std::path::PathBuf;
//
// /// A simple trait to handle args for an executable.
// pub trait Args {
//     fn args(&self) -> Result<Vec<String>>;
// }
//
// /// A trait to implement executable
// #[async_trait]
// pub trait Executable: Args {
//     async fn exec(&self, args: &[String]) -> Result<String>;
//
//     fn concat_args(&self, args: &[String]) -> Result<Vec<String>> {
//         let mut combined = self.args()?;
//         combined.append(&mut Vec::from_iter(args.iter().cloned().map(String::from)));
//         Ok(combined)
//     }
//
//     fn append_opts(args: &mut Vec<String>, opts: Option<&dyn Args>) -> Result<()> where Self: Sized {
//         if let Some(opts) = opts {
//             args.append(&mut opts.args()?);
//         }
//         Ok(())
//     }
// }
//
//     {
//         let args = self.concat_args(args)?;
//         let process = Command::new(command)
//             .args(&args.clone())
//             .stdin(Stdio::null())
//             .stdout(Stdio::piped())
//             .stderr(Stdio::piped())
//             .spawn()
//             .context(ProcessSpawnError {})?;
//
//         debug!("{} {}", command.to_str().unwrap(), &args.clone().join(" "));
//
//         let result = tokio::time::timeout(timeout.clone(), process.wait_with_output())
//             .await
//             .context(ExecCommandTimeoutError {})?
//             .context(ExecCommandError {})?;
//
//         let stdout = String::from_utf8(result.stdout.clone()).unwrap();
//         let stderr = String::from_utf8(result.stderr.clone()).unwrap();
//
//         if stderr != "" {
//             error!("Executable error : {}", stderr);
//         }
//
//         ensure!(
//             result.status.success(),
//             ExecCommandFailedError {
//                 stdout: stdout,
//                 stderr: stderr
//             }
//         );
//
//         Ok(stdout)
//     }
//
//
