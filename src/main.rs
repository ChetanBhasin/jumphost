use std::net::TcpListener;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_available_port() -> Option<u16> {
    (1025..65535).find(|port| port_is_available(*port))
}

fn execute_process(port: u16, cmd: &mut Command, retries: u16, backoff: Duration) {
    let mut retries = retries;
    let mut proxy = Command::new("ssh")
        .arg("-TND")
        .arg(port.to_string())
        .arg("-o")
        .arg("ExitOnForwardFailure=yes")
        .arg("ChetanBhasin@bastion-stage.jimdo-platform-eks.net")
        .spawn()
        .expect("Spawning this process should not fail");

    while retries > 0 {
        if port_is_available(8080) {
            retries -= 1;
            sleep(backoff);
        } else {
            break;
        }
    }

    cmd.env("HTTPS_PROXY", format!("socks5://127.0.0.1:{}", port))
        .spawn()
        .expect("The spawned process failed")
        .wait()
        .expect("Waiting for process failed");

    proxy.kill().expect("Failed to kill the proxy after use");
}

fn main() {
    let mut cmd = Command::new("kubectl");
    cmd.arg("-n").arg("kube-system").arg("get").arg("all");
    let retries = 5;
    let backoff = Duration::new(1, 0);
    match get_available_port() {
        Some(port) => execute_process(port, &mut cmd, retries, backoff),
        None => println!("No available port to run proxy on."),
    }
}
