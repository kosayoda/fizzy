use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;
use inkwell::context::Context;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;

use fizzy::{compile, parse};

#[derive(Parser)]
#[clap(author, version, arg_required_else_help = true)]
struct Cli {
    /// The fizzylang source code.
    /// See: 'sample.fizz'
    source: PathBuf,
    /// The name of the object file to output.
    /// Defaults to 'output.o'
    #[clap(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    // CLI arguments
    let args = Cli::parse();
    let output_path = args.output.unwrap_or_else(|| PathBuf::from("output.o"));
    let src = std::fs::read_to_string(&args.source)?;

    println!("Compiling {:?}", &args.source);
    let rules = parse(&src);

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
    target_machine
        .write_to_file(&module, FileType::Object, output_path.as_ref())
        .map_err(|e| anyhow!(format!("{:?}", e)))?;

    println!("Fizzylang compiled to object file: {:?}", output_path);
    println!(
        "To produce an executable, compile the object file with any mainstream C compiler: gcc -o fizz {:?}",
        output_path
    );
    Ok(())
}
