//! Integration tests for data graph embedding and GAI functions
//!
//! Tests the full pipeline: parse data snippets -> build graph -> compile WASM -> execute GAI

use wasmtime::{AsContextMut, Engine, Instance, Linker, Module, Store};

/// Helper to compile source with data snippets and instantiate it
fn compile_data_module(source: &str) -> (Store<()>, Instance) {
    // Parse
    let program = covenant_parser::parse(source)
        .expect("Failed to parse");

    // Type check
    let check_result = covenant_checker::check(&program)
        .expect("Type checking failed");

    // Compile to WASM
    let wasm_bytes = covenant_codegen::compile(&program, &check_result.symbols)
        .expect("WASM compilation failed");

    // Instantiate with wasmtime
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes)
        .expect("Failed to create WASM module");

    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);

    // Provide stub imports for all extern-abstract modules registered by codegen
    // mem.alloc is the only one actually needed for data graph tests
    linker.func_wrap("mem", "alloc", |_size: i32| -> i32 { 0x10000 }).unwrap();

    // Provide no-op stubs for all other imported functions
    // Rather than listing all functions, use the module's imports to define fallbacks
    let module_ref = Module::new(&engine, &wasm_bytes).unwrap();
    for import in module_ref.imports() {
        let module_name = import.module();
        let name = import.name();
        if module_name == "mem" && name == "alloc" {
            continue; // Already defined
        }
        match import.ty() {
            wasmtime::ExternType::Func(func_ty) => {
                let results_len = func_ty.results().len();
                if results_len == 0 {
                    let _ = linker.func_new(module_name, name, func_ty.clone(), |_caller, _params, _results| Ok(()));
                } else {
                    let _ = linker.func_new(module_name, name, func_ty.clone(), |_caller, _params, results| {
                        for r in results.iter_mut() {
                            *r = wasmtime::Val::I64(0);
                        }
                        Ok(())
                    });
                }
            }
            _ => {}
        }
    }

    let instance = linker.instantiate(&mut store, &module)
        .expect("Failed to instantiate module");

    (store, instance)
}

/// Read a string from WASM memory given a fat pointer (i64: offset << 32 | len)
fn read_fat_ptr(store: &mut Store<()>, instance: &Instance, fat_ptr: i64) -> String {
    let memory = instance.get_memory(store.as_context_mut(), "memory")
        .expect("memory export");
    let offset = (fat_ptr >> 32) as usize;
    let len = (fat_ptr & 0xFFFFFFFF) as usize;
    let data = memory.data(&store);
    String::from_utf8_lossy(&data[offset..offset + len]).to_string()
}

#[test]
fn test_data_snippets_produce_gai_exports() {
    let source = r#"
snippet id="kb.root" kind="data"

content
  """
  Root knowledge node
  """
end

relations
  rel to="kb.child" type=contains
end

end

snippet id="kb.child" kind="data"

content
  """
  Child knowledge node
  """
end

end
"#;

    let (mut store, instance) = compile_data_module(source);

    // Check that cov_node_count is exported and returns correct count
    let node_count = instance
        .get_typed_func::<(), i32>(&mut store, "cov_node_count")
        .expect("cov_node_count should be exported");

    let count = node_count.call(&mut store, ()).unwrap();
    assert_eq!(count, 2, "Should have 2 data nodes");
}

#[test]
fn test_gai_get_node_id() {
    let source = r#"
snippet id="doc.intro" kind="data"

content
  """
  Introduction section
  """
end

end

snippet id="doc.body" kind="data"

content
  """
  Body section
  """
end

end
"#;

    let (mut store, instance) = compile_data_module(source);

    // Get node IDs via GAI
    let get_node_id = instance
        .get_typed_func::<i32, i64>(&mut store, "cov_get_node_id")
        .expect("cov_get_node_id should be exported");

    let id0_ptr = get_node_id.call(&mut store, 0).unwrap();
    let id0 = read_fat_ptr(&mut store, &instance, id0_ptr);
    assert_eq!(id0, "doc.intro");

    let id1_ptr = get_node_id.call(&mut store, 1).unwrap();
    let id1 = read_fat_ptr(&mut store, &instance, id1_ptr);
    assert_eq!(id1, "doc.body");
}

#[test]
fn test_gai_get_node_content() {
    let source = r#"
snippet id="kb.hello" kind="data"

content
  """
  Hello from the knowledge base
  """
end

end
"#;

    let (mut store, instance) = compile_data_module(source);

    let get_content = instance
        .get_typed_func::<i32, i64>(&mut store, "cov_get_node_content")
        .expect("cov_get_node_content should be exported");

    let content_ptr = get_content.call(&mut store, 0).unwrap();
    let content = read_fat_ptr(&mut store, &instance, content_ptr);
    // Triple-quoted strings preserve internal whitespace/newlines
    assert!(content.contains("Hello from the knowledge base"),
        "Content should contain the text, got: {:?}", content);
}

#[test]
fn test_gai_find_by_id() {
    let source = r#"
snippet id="alpha" kind="data"
content
  """
  Alpha content
  """
end
end

snippet id="beta" kind="data"
content
  """
  Beta content
  """
end
end

snippet id="gamma" kind="data"
content
  """
  Gamma content
  """
end
end
"#;

    let (mut store, instance) = compile_data_module(source);

    let find_by_id = instance
        .get_typed_func::<(i32, i32), i32>(&mut store, "cov_find_by_id")
        .expect("cov_find_by_id should be exported");

    // Write search string "beta" into memory
    // Write search string "beta" into memory at a high offset to avoid data segment
    let search_offset = 0x80000u32; // 512KB offset, well past data segment but within 1MB memory
    let search_str = b"beta";
    {
        let memory = instance.get_memory(store.as_context_mut(), "memory")
            .expect("memory export");
        memory.data_mut(&mut store)[search_offset as usize..search_offset as usize + search_str.len()]
            .copy_from_slice(search_str);
    }

    let idx = find_by_id.call(&mut store, (search_offset as i32, search_str.len() as i32)).unwrap();
    assert_eq!(idx, 1, "beta should be at index 1");

    // Search for non-existent node
    let not_found_str = b"nonexistent";
    let nf_offset = 0x80100u32;
    {
        let memory = instance.get_memory(store.as_context_mut(), "memory")
            .expect("memory export");
        memory.data_mut(&mut store)[nf_offset as usize..nf_offset as usize + not_found_str.len()]
            .copy_from_slice(not_found_str);
    }

    let not_found = find_by_id.call(&mut store, (nf_offset as i32, not_found_str.len() as i32)).unwrap();
    assert_eq!(not_found, -1, "Should return -1 for non-existent ID");
}

#[test]
fn test_gai_outgoing_relations() {
    let source = r#"
snippet id="parent" kind="data"
content
  """
  Parent node
  """
end
relations
  rel to="child1" type=contains
  rel to="child2" type=contains
end
end

snippet id="child1" kind="data"
content
  """
  First child
  """
end
end

snippet id="child2" kind="data"
content
  """
  Second child
  """
end
end
"#;

    let (mut store, instance) = compile_data_module(source);

    let get_outgoing_count = instance
        .get_typed_func::<i32, i32>(&mut store, "cov_get_outgoing_count")
        .expect("cov_get_outgoing_count");

    // parent (idx 0) should have 2 outgoing (contains child1, contains child2)
    let parent_out = get_outgoing_count.call(&mut store, 0).unwrap();
    assert_eq!(parent_out, 2, "Parent should have 2 outgoing edges");

    // child1 (idx 1) should have 1 outgoing (contained_by parent, auto-inverse)
    let child1_out = get_outgoing_count.call(&mut store, 1).unwrap();
    assert_eq!(child1_out, 1, "Child1 should have 1 outgoing edge (inverse)");

    // Verify we can read the relation entries
    let get_outgoing_rel = instance
        .get_typed_func::<(i32, i32), i64>(&mut store, "cov_get_outgoing_rel")
        .expect("cov_get_outgoing_rel");

    // Parent's first outgoing relation
    let rel0 = get_outgoing_rel.call(&mut store, (0, 0)).unwrap();
    assert_ne!(rel0, -1, "Should return valid relation, not -1");

    // Out of bounds should return -1
    let oob = get_outgoing_rel.call(&mut store, (0, 99)).unwrap();
    assert_eq!(oob, -1, "Out of bounds should return -1");
}

#[test]
fn test_gai_content_contains() {
    let source = r#"
snippet id="doc.auth" kind="data"
content
  """
  Authentication and authorization mechanisms for the API
  """
end
end

snippet id="doc.perf" kind="data"
content
  """
  Performance optimization and caching strategies
  """
end
end
"#;

    let (mut store, instance) = compile_data_module(source);

    let content_contains = instance
        .get_typed_func::<(i32, i32, i32), i32>(&mut store, "cov_content_contains")
        .expect("cov_content_contains");

    // Write search terms into memory
    let term = b"auth";
    let term_offset = 0x80000u32;
    {
        let memory = instance.get_memory(store.as_context_mut(), "memory")
            .expect("memory export");
        memory.data_mut(&mut store)[term_offset as usize..term_offset as usize + term.len()]
            .copy_from_slice(term);
    }

    let found = content_contains.call(&mut store, (0, term_offset as i32, term.len() as i32)).unwrap();
    assert_eq!(found, 1, "doc.auth should contain 'auth'");

    // Search for "auth" in doc.perf (idx 1) - should not match
    let not_found = content_contains.call(&mut store, (1, term_offset as i32, term.len() as i32)).unwrap();
    assert_eq!(not_found, 0, "doc.perf should not contain 'auth'");

    // Search for "caching" in doc.perf (idx 1)
    let term2 = b"caching";
    let term2_offset = 0x80100u32;
    {
        let memory = instance.get_memory(store.as_context_mut(), "memory")
            .expect("memory export");
        memory.data_mut(&mut store)[term2_offset as usize..term2_offset as usize + term2.len()]
            .copy_from_slice(term2);
    }

    let found2 = content_contains.call(&mut store, (1, term2_offset as i32, term2.len() as i32)).unwrap();
    assert_eq!(found2, 1, "doc.perf should contain 'caching'");
}

#[test]
fn test_gai_with_function_snippets() {
    // Test that data snippets and function snippets coexist
    let source = r#"
snippet id="kb.node1" kind="data"
content
  """
  Knowledge base node
  """
end
end

snippet id="math.double" kind="fn"
signature
  fn name="double"
    param name="x" type="Int"
    returns type="Int"
  end
end
body
  step id="s1" kind="compute"
    op=add
    input var="x"
    input var="x"
    as="result"
  end
  step id="s2" kind="return"
    from="result"
    as="_"
  end
end
end
"#;

    let (mut store, instance) = compile_data_module(source);

    // Function should work
    let double = instance
        .get_typed_func::<i64, i64>(&mut store, "double")
        .expect("double function should be exported");
    assert_eq!(double.call(&mut store, 21).unwrap(), 42);

    // GAI should also work
    // Note: DataGraph indexes all snippets (data + non-data) for relation resolution
    // So the function snippet is also counted as a node (with empty content)
    let node_count = instance
        .get_typed_func::<(), i32>(&mut store, "cov_node_count")
        .expect("cov_node_count");
    let count = node_count.call(&mut store, ()).unwrap();
    assert!(count >= 1, "Should have at least 1 node (data snippet)");
}

#[test]
fn test_gai_incoming_relations() {
    let source = r#"
snippet id="a" kind="data"
content
  """
  Node A
  """
end
relations
  rel to="b" type=describes
end
end

snippet id="b" kind="data"
content
  """
  Node B
  """
end
end
"#;

    let (mut store, instance) = compile_data_module(source);

    let get_incoming_count = instance
        .get_typed_func::<i32, i32>(&mut store, "cov_get_incoming_count")
        .expect("cov_get_incoming_count");

    // b (idx 1) should have 1 incoming edge (describes from a)
    let b_in = get_incoming_count.call(&mut store, 1).unwrap();
    assert_eq!(b_in, 1, "Node B should have 1 incoming edge");

    // a (idx 0) should have 1 incoming edge (described_by from b, auto-inverse)
    let a_in = get_incoming_count.call(&mut store, 0).unwrap();
    assert_eq!(a_in, 1, "Node A should have 1 incoming edge (inverse)");
}
