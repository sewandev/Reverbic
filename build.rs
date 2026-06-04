fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/reverbic.rc", embed_resource::NONE);
    }
    load_dotenv();
}

fn load_dotenv() {
    let Ok(content) = std::fs::read_to_string(".env") else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let key = key.trim();
            let val = val.trim().trim_matches('"');
            println!("cargo:rustc-env={key}={val}");
            println!("cargo:rerun-if-env-changed={key}");
        }
    }
    println!("cargo:rerun-if-changed=.env");
}
