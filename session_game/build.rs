use session_io::ProxyMetadata;

fn main() {
    gear_wasm_builder::build_with_metadata::<ProxyMetadata>();
}