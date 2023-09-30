use wasmtime::*;

fn main() -> wasmtime::Result<()> {
    // Enable fuel consumption
    let mut config = Config::new();
    config.consume_fuel(true);

    // Initialize the Wasmtime engine with the configured config
    // let engine = Engine::default();
    let engine = Engine::new(&config).unwrap();

    // Modules can be compiled through either the text or binary format
    let wat = r#"
        (module
            (import "host" "host_func" (func $host_hello (param i32)))

            (func (export "hello")
                i32.const 3
                call $host_hello)
        )
    "#;
    let module = Module::new(&engine, wat)?;

    // All wasm objects operate within the context of a "store". Each
    // `Store` has a type parameter to store host-specific data, which in
    // this case we're using `4` for.
    let mut store = Store::new(&engine, 4);

    // Inject fuel into the store
    let fuel_amount = 1000;
    store.add_fuel(fuel_amount)?;

    let host_func = Func::wrap(&mut store, |caller: Caller<'_, u32>, param: i32| {
        println!("Got {} from WebAssembly", param);
        println!("My host state is {}", caller.data());
    });

    // Instantiation of a module requires specifying its imports and then
    // afterwards we can fetch exports by name, as well as asserting the
    // type signature of the function with `get_typed_func`.
    let instance = Instance::new(&mut store, &module, &[host_func.into()])?;

    let hello = instance.get_typed_func::<(), ()>(&mut store, "hello")?;

    // And finally we can call the wasm!
    hello.call(&mut store, ())?;

    println!("Consumed fuel is {}", store.fuel_consumed().unwrap());

    linker()
}

fn linker() -> wasmtime::Result<()> {
    let engine = Engine::default();
    let wat = r#"
        (module
            (import "host" "host_func" (func $host_hello (param i32)))

            (func (export "hello")
                i32.const 3
                call $host_hello)
        )
    "#;
    let module = Module::new(&engine, wat)?;

    // Create a `Linker` and define our host function in it:
    let mut linker = Linker::new(&engine);
    linker.func_wrap("host", "host_func", |caller: Caller<'_, u32>, param: i32| {
        println!("Got {} from WebAssembly", param);
        println!("My host state is {}", caller.data());
    })?;

    // Use the `linker` to instantiate the module, which will automatically
    // resolve the imports of the module using name-based resolution.
    let mut store = Store::new(&engine, 4);
    let instance = linker.instantiate(&mut store, &module)?;
    let hello = instance.get_typed_func::<(), ()>(&mut store, "hello")?;
    hello.call(&mut store, ())?;

    Ok(())
}

// https://docs.rs/wasmtime/12.0.2/wasmtime/index.html

