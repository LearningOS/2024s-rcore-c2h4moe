static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

fn main() {
    println!("cargo:rerun-if-changed=../ci-user/user/src/");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
}
