use std::net::TcpListener;
use std::process::{exit, Command};

fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn get_available_port() -> Option<u16> {
    (1025..65535).find(|port| port_is_available(*port))
}

fn establish_connection(id: &String, port: &u16, user: &String, host: &String) {
    match std::fs::create_dir_all("/tmp/com.cbopt") {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Could not create temporary directory: {}", err);
            exit(1);
        }
    }
    let mut cmd_handle = Command::new("ssh")
        .arg("-TNMD")
        .arg(port.to_string())
        .arg("-o")
        .arg("ExitOnForwardFailure=yes")
        .arg("-S")
        .arg(format!("/tmp/com.cbopt/{}", id))
        .arg("-f")
        .arg(format!("{}@{}", user, host))
        .spawn()
        .expect("Failed to spawn SSH process");
    cmd_handle
        .wait()
        .expect("Could not wait for proxy starting process");
}

fn close_connection(id: &String, user: &String, host: &String) {
    let mut cmd_handle = Command::new("ssh")
        .arg("-S")
        .arg(format!("/tmp/com.cbopt/{}", id))
        .arg("-O")
        .arg("exit")
        .arg(format!("{}@{}", user, host))
        .spawn()
        .expect("Failed to close the SSH process");
    cmd_handle
        .wait()
        .expect("Could not wait for the proxy closing process");
}

fn execute_process(port: u16, cmd: &mut Command) {
    for (key, value) in std::env::vars() {
        cmd.env(key, value);
    }

    let mut cmd_handle = cmd
        .env("HTTPS_PROXY", format!("socks5://127.0.0.1:{}", port))
        .spawn()
        .expect("The spawned process failed");

    match cmd_handle.wait() {
        Ok(_) => (),
        Err(err) => {
            eprintln!(
                "Could not properly finish executing the requested process: {}",
                err,
            );
        }
    }
}

fn main() {
    //let mut args: Vec<String> = std::env::args().collect();
    //let mut cmd = Command::new(args[1].as_str());
    //cmd.args(args.drain(2..args.len()));
    let mut cmd = Command::new("/bin/bash");
    cmd.args(std::env::args());
    let host: String = String::from(std::env::var_os("JH_HOST").unwrap().to_str().unwrap());
    let user: String = String::from(std::env::var_os("JH_USER").unwrap().to_str().unwrap());
    match get_available_port() {
        Some(port) => {
            establish_connection(&String::from("id"), &port, &user, &host);
            execute_process(port, &mut cmd);
            close_connection(&String::from("id"), &user, &host);
        }
        None => println!("No available port to run proxy on."),
    }
}
