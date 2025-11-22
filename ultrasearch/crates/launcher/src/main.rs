use anyhow::{Context, Result};
use console::style;
use dotenvy::dotenv;
use ipc::{StatusRequest, client::PipeClient};
use std::{
    env,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread::sleep,
    time::Duration,
};
use tokio::runtime::Runtime;
use uuid::Uuid;

/// Ensures a child process is terminated if this guard is dropped.
struct ChildGuard {
    name: &'static str,
    child: Option<Child>,
}

impl ChildGuard {
    fn spawn(name: &'static str, cmd: &mut Command) -> Result<Self> {
        let child = cmd.spawn().with_context(|| format!("spawn {}", name))?;
        Ok(Self {
            name,
            child: Some(child),
        })
    }

    fn wait(mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            let status = child.wait()?;
            if !status.success() {
                anyhow::bail!("{} exited with status {:?}", self.name, status.code());
            }
        }
        Ok(())
    }

    fn kill_if_running(&mut self) {
        if let Some(child) = &mut self.child
            && child.try_wait().ok().flatten().is_none()
        {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill_if_running();
    }
}

fn main() -> Result<()> {
    dotenv().ok();
    let exe_dir = current_exe_dir()?;

    let service_path = resolve_binary(&exe_dir, "service")?;
    let ui_path = resolve_binary(&exe_dir, "ui")?;

    println!(
        "{} {}",
        style("Launching UltraSearch").bold().green(),
        style("one-click mode").cyan()
    );

    let mut service_cmd = Command::new(&service_path);
    service_cmd
        .arg("--console")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let mut service = ChildGuard::spawn("service", &mut service_cmd)?;
    println!("{}", style("service started, waiting for IPC...").dim());

    wait_for_ipc_ready()?;

    let mut ui_cmd = Command::new(&ui_path);
    ui_cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let ui = ChildGuard::spawn("ui", &mut ui_cmd)?;
    println!("{}", style("ui launched; press Ctrl+C to exit").dim());

    // When UI exits, tear down service.
    ui.wait()?;
    service.kill_if_running();
    println!("{}", style("UltraSearch closed").green());
    Ok(())
}

fn current_exe_dir() -> Result<PathBuf> {
    let exe = env::current_exe().context("current_exe")?;
    let dir = exe
        .parent()
        .map(Path::to_path_buf)
        .context("executable has no parent dir")?;
    Ok(dir)
}

fn resolve_binary(dir: &Path, stem: &str) -> Result<PathBuf> {
    let name = if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        stem.to_string()
    };
    let path = dir.join(name);
    if !path.exists() {
        anyhow::bail!(
            "{} not found at {}. Build release binaries first with `cargo build --release`.",
            stem,
            path.display()
        );
    }
    Ok(path)
}

fn wait_for_ipc_ready() -> Result<()> {
    #[cfg(not(windows))]
    {
        return Ok(()); // IPC pipe only on Windows; nothing to probe elsewhere.
    }

    #[cfg(windows)]
    {
        let rt = Runtime::new().context("build tokio runtime")?;
        let client = PipeClient::default().with_request_timeout(Duration::from_millis(400));
        let mut attempts: u32 = 0;
        loop {
            let res = rt.block_on(client.status(StatusRequest { id: Uuid::new_v4() }));
            match res {
                Ok(_) => {
                    println!("{}", style("IPC ready").green());
                    return Ok(());
                }
                Err(err) if attempts < 40 => {
                    attempts += 1;
                    print!(".");
                    let _ = std::io::stdout().flush();
                    sleep(Duration::from_millis(250));
                    continue;
                }
                Err(err) => {
                    println!();
                    anyhow::bail!("IPC not ready after retries: {err}");
                }
            }
        }
    }
}
