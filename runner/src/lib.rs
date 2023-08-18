use std::{path::Path, rc::Rc, sync::Arc};

use deno_ast::{ParseParams, SourceTextInfo, MediaType, parse_module};
use deno_runtime::{
    deno_core::{error::AnyError, FsModuleLoader, ModuleSpecifier},
    deno_napi::v8::{self, DataError, GetPropertyNamesArgs, Global, Local, Promise, PromiseState},
    permissions::PermissionsContainer,
    worker::{MainWorker, WorkerOptions},
};
use module_loader::TsModuleLoader;
use thiserror::Error;

mod module_loader;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error(transparent)]
    AnyError(#[from] AnyError),
    #[error("Stage {stage} is not a function: {error}")]
    StageIsNotFunction {
        stage: String,
        #[source]
        error: Arc<DataError>,
    },
    #[error("No default export")]
    NoDefaultExport,
    #[error("Default export is not an object")]
    DefaultExportIsNotObject,
    // #[error("No stages property on default export")]
    // NoStages,
    // #[error("stages property is not an object")]
    // StagesIsNotObject,
}

#[allow(clippy::future_not_send)]
pub async fn run() -> Result<(), RunnerError> {
    let js_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("example.job.js");
    // let parse_options = ParseParams {
    //     specifier: "todo".into(),
    //     text_info: SourceTextInfo::from_string(std::fs::read_to_string(&js_path).unwrap()),
    //     media_type: MediaType::from_path(&js_path),
    //     capture_tokens: false,
    //     scope_analysis: true,
    //     maybe_syntax: None
    // };
    // let parsed = parse_module(parse_options).unwrap();
    let main_module = ModuleSpecifier::from_file_path(js_path).unwrap();
    let mut worker = MainWorker::bootstrap_from_options(
        main_module.clone(),
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(TsModuleLoader),
            extensions: vec![],
            ..Default::default()
        },
    );
    let main_module = worker.preload_main_module(&main_module).await?;
    worker.evaluate_module(main_module).await?;
    worker.run_event_loop(false).await?;
    let global = worker.js_runtime.get_module_namespace(main_module)?;

    let stages = {
        let scope = &mut worker.js_runtime.handle_scope();
        let global = global.open(scope);
        let default_str = v8::String::new(scope, "default").unwrap();
        let stages = global
            .get(scope, default_str.into())
            .ok_or(RunnerError::NoDefaultExport)?
            .to_object(scope)
            .ok_or(RunnerError::DefaultExportIsNotObject)?;
        // let params_str = v8::String::new(scope, "params").unwrap();
        // let params = job.get(scope, params_str.into()).unwrap();
        // let stages_str = v8::String::new(scope, "stages").unwrap();
        // let stages_str = stages_str.into();
        // if job.has(scope, stages_str) == Some(false) {
        //     Err(RunnerError::NoStages)?
        // }
        // let stages = job
        //     .get(scope, stages_str)
        //     .ok_or(RunnerError::NoStages)?;
        // dbg!(stages.to_rust_string_lossy(scope));
        // let stages = stages
        //     .to_object(scope)
        //     .ok_or(RunnerError::StagesIsNotObject)?;
        // let params: Local<Array> = params.try_into()?;
        // let mut res_params = Vec::<ParamData>::with_capacity(params.length() as usize);
        // let name_str = v8::String::new(scope, "name").unwrap();
        // let type_str = v8::String::new(scope, "type").unwrap();
        // for i in 0..params.length() {
        //     let param = params.get_index(scope, i).unwrap().to_object(scope).unwrap();
        //     let name = param.get(scope, name_str.into()).unwrap().to_rust_string_lossy(scope);
        //     let ty = param.get(scope, type_str.into()).unwrap().to_rust_string_lossy(scope);
        //     let ty = match ty.as_str() {
        //         "number" => ParamType::Number,
        //         "string" => ParamType::String,
        //         "object" => ParamType::Object,
        //         x => panic!("Unknown param type: {x}")
        //     };
        //     res_params.push(ParamData { name, ty })
        // }
        Global::new(scope, stages)
    };
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
    let stage_count = stage_names.length();
    let mut result = None;
    for i in 0..stage_count {
        let func_res = {
            let scope = &mut worker.js_runtime.handle_scope();
            let name = stage_names.get_index(scope, i).unwrap();
            let name_string = name.to_rust_string_lossy(scope);
            let func = func.get(scope, name).unwrap();
            let func: v8::Local<v8::Function> =
                func.try_into()
                    .map_err(|error| RunnerError::StageIsNotFunction {
                        stage: name_string,
                        error: Arc::new(error),
                    })?;
            let recv = Local::new(scope, &global);
            let res = func.call(scope, recv.into(), &[]).unwrap();
            Global::new(scope, res)
        };
        worker.js_runtime.run_event_loop(false).await?;
        let scope = &mut worker.js_runtime.handle_scope();
        let mut res = Local::new(scope, func_res);
        if res.is_promise() {
            let promise: Local<Promise> = res.try_into().unwrap();
            if promise.state() == PromiseState::Pending {
                panic!("Resulting promise is still pending")
            }
            if promise.has_handler() {
                panic!("Promise has handler")
            }
            res = promise.result(scope);
            // promise.
        }
        if i + 1 == stage_count {
            result = Some(Global::new(scope, res));
        }
    }
    if let Some(res) = result {
        let scope = &mut worker.js_runtime.handle_scope();
        let res = res.open(scope);
        println!("{}", res.to_rust_string_lossy(scope));
    }
    Ok(())
}
