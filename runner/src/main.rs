use std::{path::Path, rc::Rc};

use deno_runtime::{
    deno_core::{error::AnyError, FsModuleLoader, ModuleSpecifier},
    deno_napi::v8::{self, GetPropertyNamesArgs, Global, Local, Array},
    permissions::PermissionsContainer,
    worker::{MainWorker, WorkerOptions},
};

#[tokio::main]
async fn main() -> Result<(), AnyError> {
    let js_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("example.job.js");
    println!("{js_path:?}");
    let main_module = ModuleSpecifier::from_file_path(js_path).unwrap();
    let mut worker = MainWorker::bootstrap_from_options(
        main_module.clone(),
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(FsModuleLoader),
            extensions: vec![],
            ..Default::default()
        },
    );
    let main_module = worker.preload_main_module(&main_module).await?;
    worker.evaluate_module(main_module).await?;
    worker.run_event_loop(false).await?;
    let global = worker.js_runtime.get_module_namespace(main_module)?;

    let (params, stages) = {
        let scope = &mut worker.js_runtime.handle_scope();
        let global = global.open(scope);
        let default_str = v8::String::new(scope, "default").unwrap();
        let job = global.get(scope, default_str.into()).unwrap().to_object(scope).unwrap();
        let params_str = v8::String::new(scope, "params").unwrap();
        let params = job.get(scope, params_str.into()).unwrap();
        let stages_str = v8::String::new(scope, "stages").unwrap();
        let stages = job
            .get(scope, stages_str.into())
            .unwrap()
            .to_object(scope)
            .unwrap();
        let params: Local<Array> = params.try_into()?;
        let mut res_params = Vec::<ParamData>::with_capacity(params.length() as usize);
        let name_str = v8::String::new(scope, "name").unwrap();
        let type_str = v8::String::new(scope, "type").unwrap();
        for i in 0..params.length() {
            let param = params.get_index(scope, i).unwrap().to_object(scope).unwrap();
            let name = param.get(scope, name_str.into()).unwrap().to_rust_string_lossy(scope);
            let ty = param.get(scope, type_str.into()).unwrap().to_rust_string_lossy(scope);
            let ty = match ty.as_str() {
                "number" => ParamType::Number,
                "string" => ParamType::String,
                "object" => ParamType::Object,
                x => panic!("Unknown param type: {x}")
            };
            res_params.push(ParamData { name, ty })
        }
        (res_params, Global::new(scope, stages))
    };
    println!("{params:?}");
    let stage_names = {
        let scope = &mut worker.js_runtime.handle_scope();
        let stages: &v8::Object = stages.open(scope);
        let stage_names = stages
            .get_property_names(scope, GetPropertyNamesArgs::default())
            .unwrap();
        Global::new(scope, stage_names)
    };
    let stage_names = stage_names.open(worker.js_runtime.v8_isolate());
    let func = stages.open(worker.js_runtime.v8_isolate());
    for i in 0..stage_names.length() {
        let func_res = {
            let scope = &mut worker.js_runtime.handle_scope();
            let name = stage_names.get_index(scope, i).unwrap();
            println!("Prop: {:?}", name.to_rust_string_lossy(scope));
            let func = func.get(scope, name).unwrap();
            let func: v8::Local<v8::Function> = func.try_into()?;
            let recv = Local::new(scope, &global);
            let res = func.call(scope, recv.into(), &[]).unwrap();
            Global::new(scope, res)
        };
        worker.js_runtime.run_event_loop(false).await?;
        let scope = &mut worker.js_runtime.handle_scope();
        println!("{}", func_res.open(scope).to_rust_string_lossy(scope));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParamData {
    name: String,
    ty: ParamType
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParamType {
    String,
    Number,
    Object
}
