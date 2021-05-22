const ENV_VAR: &str = "SKYLINE_ADD_NRO_HEADER";

fn main() {
    println!("cargo:rerun-if-env-changed={}", ENV_VAR);
    if std::env::var_os(ENV_VAR).is_some() {
        println!("cargo:rustc-cfg=nro_header");
    }
}
