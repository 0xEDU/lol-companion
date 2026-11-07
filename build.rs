fn main() {
    println!("cargo:rerun-if-changed=windows.rc");
    println!("cargo:rerun-if-changed=windows.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && let Err(error) =
            embed_resource::compile("windows.rc", embed_resource::NONE).manifest_required()
    {
        eprintln!("Failed to embed the Windows application manifest: {error}");
        std::process::exit(1);
    }
}
