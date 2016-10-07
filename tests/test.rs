extern crate vault_client;

use vault_client::client::VaultClient;
use std::process::{Command, Child};


macro_rules! vault_root {
    ($port:expr) => {
        Command::new("tests/vault")
            .env("VAULT_ADDR", &("http://127.0.0.1:".to_string() + $port))
            .env("VAULT_TOKEN", "vault-dev-root-token")
    }
}

fn spawn_vault(port: &str) -> Child {
    Command::new("tests/vault")
        .args(&["server", "-dev", "-log-level=trace", "-dev-root-token-id=vault-dev-root-token", &("-dev-listen-address=127.0.0.1:".to_string() + port)])
        .spawn()
        .unwrap()
}

fn create_client(port: &str) -> VaultClient {
    VaultClient::new(&("http://127.0.0.1:".to_string() + port),
                     "vault-dev-root-token".to_string())
        .unwrap()
}

#[test]
fn test_generic_raw() {
    let mut child = spawn_vault("8200");
    let output = vault_root!("8200")
        .args(&["write", "/secret/test-secret", "qwer=1234"])
        .output()
        .unwrap();
    println!("{:?}", output);

    let client = create_client("8200");
    let mut buf = client.get_secret_raw("/secret/test-secret").unwrap();
    child.kill();
    // let secret: Value = from_reader(buf.as_slice()).unwrap();
    println!("{:?}", String::from_utf8(buf).unwrap());
    assert_eq!(4, 4);
}


#[test]
fn test_generic_error() {
    let mut child = spawn_vault("8201");
    let client = create_client("8201");
    let mut buf = client.get_secret_raw("/qwerqwer");
    println!("{:?}", buf);
    child.kill();
    assert!(buf.is_err());
}
