use std::process::Command;

struct Template {
    path: String,
}

fn template_cmd(template: &str, out: &str) {
    let command = format!("oto -template {} \
        -pkg paradise \
        -out {} \
        ./definitions.go && \
        rustfmt {} && \
        mv {} ../src", template, out, out, out);
    let output = Command::new("bash")
        .current_dir("definitions")
        .args(&["-c", &command])
        .output()
        .expect("failed to execute process");
    assert!(output.status.success(), format!("{}", std::str::from_utf8(&output.stderr[..]).unwrap()));
}

fn main() {
    template_cmd("$GOPATH/src/github.com/pacedotdev/oto/otohttp/templates/rust/types.rs.plush", "types.rs");
    template_cmd("$GOPATH/src/github.com/pacedotdev/oto/otohttp/templates/rust/async_client.rs.plush", "client.rs");
    template_cmd("$GOPATH/src/github.com/pacedotdev/oto/otohttp/templates/rust/server_actixweb.rs.plush", "server.rs");
}