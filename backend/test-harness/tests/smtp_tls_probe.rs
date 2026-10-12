// SMTP-over-TLS probe for the Mailpit test double.
//
// ADDITIVE-ONLY: verifies your new "custom port + STARTTLS on 1025" setup
// without touching backend/src/* or backend/Cargo.toml. Uses only the Rust
// stdlib (raw TCP + manual EHLO/STARTTLS bytes) plus openssl s_client from
// PATH for the negative/plaintext check, so no new dependencies are needed.
//
// Run: cargo test --test smtp_tls_probe
// Env:
//   KUNJIKA_TEST_SMTP_HOST (default 127.0.0.1)
//   KUNJIKA_TEST_SMTP_PORT (default 1025)
//   KUNJIKA_TEST_SMTP_REQUIRE_STARTTLS=1 (default 1): assert plaintext MAIL
//     FROM is rejected when Mailpit runs with --smtp-require-starttls.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn smtp_host() -> String {
    env_or("KUNJIKA_TEST_SMTP_HOST", "127.0.0.1")
}

fn smtp_port() -> u16 {
    env_or("KUNJIKA_TEST_SMTP_PORT", "1025")
        .parse()
        .expect("KUNJIKA_TEST_SMTP_PORT must be numeric")
}

fn read_reply(reader: &mut BufReader<TcpStream>) -> String {
    // Read a (possibly multi-line) SMTP reply, e.g. "250-STARTTLS\r\n250 X\r\n".
    let mut out = String::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read smtp line");
        out.push_str(&line);
        // Final line has "NNN " prefix; continuation lines "NNN-".
        if line.len() >= 4 && line.as_bytes()[3] == b' ' {
            break;
        }
        if out.len() > 8192 {
            break;
        }
    }
    out
}

fn send_cmd(stream: &mut TcpStream, cmd: &str) {
    stream
        .write_all(format!("{cmd}\r\n").as_bytes())
        .expect("write smtp cmd");
    stream.flush().expect("flush smtp cmd");
}

#[test]
fn smtp_banner_and_ehlo_advertise_starttls() {
    let addr = format!("{}:{}", smtp_host(), smtp_port());
    let stream = TcpStream::connect(&addr).expect("connect to test SMTP port");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("set timeout");
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut stream = stream;

    let banner = read_reply(&mut reader);
    assert!(
        banner.starts_with("220"),
        "expected 220 banner, got: {banner}"
    );

    send_cmd(&mut stream, "EHLO kunjika-test");
    let ehlo = read_reply(&mut reader);
    assert!(ehlo.contains("250"), "expected 250 to EHLO, got: {ehlo}");
    assert!(
        ehlo.to_uppercase().contains("STARTTLS"),
        "custom SMTP port must advertise STARTTLS (encrypted 1025), got: {ehlo}"
    );

    send_cmd(&mut stream, "QUIT");
}

#[test]
fn starttls_handshake_succeeds_with_test_ca() {
    // Full TLS handshake via openssl s_client -starttls smtp against the
    // self-signed cert in test-harness/certs/mailpit-cert.pem. This proves
    // "1025 will be encrypted" end-to-end (Mailpit --smtp-tls-cert/key).
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ca = manifest.join("certs").join("mailpit-cert.pem");
    assert!(ca.exists(), "missing test CA at {}", ca.display());

    let addr = format!("{}:{}", smtp_host(), smtp_port());
    let probe = format!("QUIT\r\n");
    let out = std::process::Command::new("openssl")
        .args([
            "s_client",
            "-connect",
            &addr,
            "-starttls",
            "smtp",
            "-CAfile",
            ca.to_str().unwrap(),
            "-verify_return_error",
            "-brief",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(probe.as_bytes())
                .ok();
            child.wait_with_output()
        })
        .expect("run openssl s_client");

    let combined =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(
        combined.contains("Verification: OK") || combined.contains("Verify return code: 0"),
        "TLS verify against test CA failed:\n{combined}"
    );
}

#[test]
fn plaintext_mail_rejected_when_starttls_required() {
    if env_or("KUNJIKA_TEST_SMTP_REQUIRE_STARTTLS", "1") != "1" {
        eprintln!("skipping: server allows plaintext");
        return;
    }
    let addr = format!("{}:{}", smtp_host(), smtp_port());
    let stream = TcpStream::connect(&addr).expect("connect to test SMTP port");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("set timeout");
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut stream = stream;

    let _ = read_reply(&mut reader); // banner
    send_cmd(&mut stream, "EHLO kunjika-test");
    let _ = read_reply(&mut reader);
    // No STARTTLS issued: Mailpit with --smtp-require-starttls must refuse.
    send_cmd(&mut stream, "MAIL FROM:<probe@example.com>");
    let mut raw = [0u8; 256];
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set timeout");
    // Read via the buffered reader to stay in sync.
    let mut line = String::new();
    reader.read_line(&mut line).expect("read mail-from reply");
    let _ = reader.read(&mut raw).ok();
    assert!(
        line.starts_with('5'),
        "expected 5xx to plaintext MAIL FROM when STARTTLS required, got: {line}"
    );
    send_cmd(&mut stream, "QUIT");
}
