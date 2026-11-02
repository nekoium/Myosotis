fn main() {
    println!("cargo:rerun-if-changed=assets/myosotis.rc");
    println!("cargo:rerun-if-changed=assets/myosotis.ico");
    println!("cargo:rerun-if-changed=assets/myosotis.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/myosotis.rc", embed_resource::NONE)
            .manifest_required()
            .expect("Windows SDK resource compiler is required");
    }
}
