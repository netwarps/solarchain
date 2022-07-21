fn main() {
	#[cfg(all(feature = "std", feature = "include-wasm"))]
	{
		use substrate_wasm_builder::WasmBuilder;
		WasmBuilder::new()
			.with_current_project()
			.export_heap_base()
			.import_memory()
			.build()
	}
}
