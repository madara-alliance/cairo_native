use cairo_lang_runner::{RunResultValue, SierraCasmRunner, StarknetState};
use cairo_lang_sierra::ProgramParser;
use cairo_native::{context::NativeContext, executor::JitNativeExecutor, OptLevel, Value};

#[test]
fn dict_entry_span_index_matches_vm() {
    // Each tuple element contains (Felt252DictEntry<u128>, u128). A stale felt
    // in the entry's layout doubles the array stride, selecting 33 instead of 22.
    // Keep this fixture in Sierra so running the regression needs no Cairo corelib.
    let program = ProgramParser::new()
        .parse(include_str!("../fixtures/dict_entry_span.sierra.txt"))
        .unwrap();
    let function = &program.funcs[0];
    let runner = SierraCasmRunner::new(
        program.clone(),
        Some(Default::default()),
        Default::default(),
        None,
    )
    .unwrap();
    let vm_result = runner
        .run_function_with_starknet_context(
            function,
            vec![],
            Some(1_000_000_000),
            StarknetState::default(),
        )
        .unwrap();
    assert_eq!(
        vm_result.value,
        RunResultValue::Success(vec![22_u128.into()])
    );

    for opt_level in [
        OptLevel::None,
        OptLevel::Less,
        OptLevel::Default,
        OptLevel::Aggressive,
    ] {
        let context = NativeContext::new();
        let module = context
            .compile(&program, false, Some(Default::default()), None)
            .unwrap();
        let executor = JitNativeExecutor::from_native_module(module, opt_level).unwrap();
        let result = executor
            .invoke_dynamic(&function.id, &[], Some(1_000_000_000))
            .unwrap();
        assert_eq!(result.return_value, Value::Uint128(22), "{opt_level:?}");
    }
}
