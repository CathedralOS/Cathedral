fn main() {
    // A linker-script edit must also invalidate downstream executable linking.
    println!("cargo:rerun-if-changed=user.ld");
}
