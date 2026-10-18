//! A plain HTTP reader against the shipped binary and a fixture bundle.
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

pub const PAGE: &[u8] = b"<!doctype html><title>t</title>";
pub const POLICY: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";
pub fn fixture_bundle(directory: &Path) -> PathBuf {
    let bundle = directory.join("browser");
    let digest = bundle.join("0123456789abcdef");
    fs::create_dir_all(digest.join("files")).expect("bundle directory");
    fs::write(bundle.join("index.html"), PAGE).expect("bundle page");
    fs::write(digest.join("boot.js"), b"boot();").expect("bundle script");
    fs::write(digest.join("files/robots.txt"), b"User-agent: *\nDisallow:\n").expect("bundle root file");
    bundle
}
pub fn configuration(directory: &Path, bundle: &Path) {
    super::support::configuration(directory);
    let path = directory.join("config.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("config")).expect("JSON");
    value["workers"] = serde_json::json!([]);
    value["application"]["workers"] = serde_json::json!([]);
    value["service"]["components"] = serde_json::json!(["clients"]);
    value["engine"]["hosts"]["listener"] = serde_json::json!([]);
    value["application"]["server"]["bundle"] = bundle.to_str().expect("bundle path").into();
    fs::write(path, serde_json::to_vec(&value).expect("browser JSON")).expect("browser config");
}
pub struct Binary(Child);
impl Drop for Binary {
    fn drop(&mut self) {
        let _ = self.0.kill();
        self.0.wait().expect("binary reaped");
    }
}
pub fn start(directory: &Path) -> Binary {
    Binary(
        Command::new(env!("CARGO_BIN_EXE_jig-test"))
            .args(["serve", "config.json"])
            .current_dir(directory)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("jig-test starts"),
    )
}
pub fn printed_link_line(binary: &mut Binary) -> String {
    let output = binary.0.stdout.take().expect("printed link pipe");
    let mut line = String::new();
    BufReader::new(output).take(4097).read_line(&mut line).expect("printed link");
    assert!(line.len() <= 4096 && line.ends_with('\n'), "bounded printed link: {line}");
    line.trim_end().into()
}
pub fn get(url: &str, target: &str) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let host = url.strip_prefix("http://").expect("loopback URL").split('/').next().expect("host");
    let mut socket = TcpStream::connect(host).expect("server listener");
    socket.set_read_timeout(Some(std::time::Duration::from_secs(10))).expect("HTTP timeout");
    socket
        .write_all(format!("GET {target} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n").as_bytes())
        .expect("GET sent");
    let mut reader = BufReader::new(socket);
    let mut line = String::new();
    reader.read_line(&mut line).expect("HTTP status");
    let status = line.split_whitespace().nth(1).expect("status").parse().expect("status number");
    let mut headers = Vec::new();
    loop {
        line.clear();
        assert!(reader.read_line(&mut line).expect("HTTP field") > 0, "head complete");
        if line == "\r\n" {
            break;
        }
        assert!(headers.len() < 32 && line.len() <= 4096, "bounded head");
        let (name, value) = line.trim_end().split_once(':').expect("field");
        headers.push((name.into(), value.trim().into()));
    }
    let length = field(&headers, "content-length").expect("body length").parse::<usize>().expect("length number");
    assert!(length <= 65536, "bounded body");
    let mut body = vec![0; length];
    reader.read_exact(&mut body).expect("HTTP body");
    (status, headers, body)
}
pub fn field<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, value)| value.as_str())
}
