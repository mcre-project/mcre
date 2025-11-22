fn main() {
    println!("cargo:rerun-if-changed=");
    if java_locator::locate_java_home().is_err() {
        println!("cargo:warning=Could not find java installation!");
    };
}
