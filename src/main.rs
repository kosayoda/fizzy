use anyhow::{anyhow, Result};
use inkwell::context::Context;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;

use fizzy::{compile, parse};

fn main() -> Result<()> {
    let src = r#"
        start: 1
        end: 100

        3: fizz
        5: buzz
    "#;

    let rules = parse(src);

    // Initialize codegen tools
    let context = Context::create();
    let module = context.create_module("fizzy");
    let builder = context.create_builder();

    compile(&context, &module, builder, rules)?;

    // Initialize target(s)
    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| anyhow!(format!("{:?}", e)))?;
    let target_triple = TargetMachine::get_default_triple();
    let cpu = TargetMachine::get_host_cpu_name().to_string();
    let features = TargetMachine::get_host_cpu_features().to_string();

    // Create target from detected triple
    let target = Target::from_triple(&target_triple).map_err(|e| anyhow!(format!("{:?}", e)))?;

    // Create machine from target
    let target_machine = target
        .create_target_machine(
            &target_triple,
            &cpu,
            &features,
            OptimizationLevel::Default,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| anyhow!("Unable to create target machine!"))?;

    // Convert module to machine code
    let output_filename = "output.o";
    target_machine
        .write_to_file(&module, FileType::Object, output_filename.as_ref())
        .map_err(|e| anyhow!(format!("{:?}", e)))?;

    Ok(())
}
