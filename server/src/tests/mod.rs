use bevy::app::App;
use deno_core::{
    FsModuleLoader, JsRuntime, ModuleCore, ModuleSpecifier, PollEventLoopOptions, RuntimeOptions, M,
};
use std::rc::Rc;

#[test]
fn test1() {
    let _app = App::new();
}

#[tokio::test]
async fn test_deno() {
    let module_loader = FsModuleLoader {};

    let runtime_options = RuntimeOptions {
        is_main: true,
        inspector: false,
        module_loader: Some(Rc::new(module_loader)),
        ..RuntimeOptions::default()
    };
    println!("creating runtime");
    let mut runtime = JsRuntime::new(runtime_options);
    println!("runtime created");

    let code = ModuleCode::ensure_static_ascii(
        r#"
        import { delay } from 'https://deno.land/x/delay@v0.2.0/extension.ts';
        console.log('hello world');
        (async () => {
            await delay(1000);
            console.log("done");
        })();

    "#,
    );

    let spec = ModuleSpecifier::parse("internal://test.ts").unwrap();
    let module_id = runtime.load_main_module(&spec, Some(code)).await.unwrap();
    println!("module loaded");

    println!("running module");
    runtime.mod_evaluate(module_id).await.unwrap();
    println!("module run");

    println!("running event loop");
    runtime
        .run_event_loop(PollEventLoopOptions::default())
        .await
        .unwrap();
}
