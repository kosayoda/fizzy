use std::collections::HashMap;

use anyhow::Result;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::values::PointerValue;
use inkwell::IntPredicate;
use inkwell::{
    module::{Linkage, Module},
    AddressSpace,
};

use crate::Rules;

pub fn compile<'context>(
    context: &'context Context,
    module: &Module<'context>,
    builder: Builder,
    rules: Rules,
) -> Result<()> {
    // Create basic types
    let bool_type = context.bool_type();
    let i32_type = context.i32_type();
    let str_type = context.i8_type().ptr_type(AddressSpace::Generic);

    // Create main function type
    let main_fn_type = i32_type.fn_type(&[], false);
    let main_fn = module.add_function("main", main_fn_type, Some(Linkage::External));

    // Create printf
    let printf_type = i32_type.fn_type(&[str_type.into()], true);
    let printf = module
        .get_function("printf")
        .unwrap_or_else(|| module.add_function("printf", printf_type, Some(Linkage::External)));

    // Program entry block
    let main_bb = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(main_bb);

    // Initialize global strings
    let mut strings: HashMap<u32, PointerValue> = HashMap::new();
    for rule in &rules.rules {
        strings.insert(
            rule.divisor,
            builder
                .build_global_string_ptr(
                    &rule.literal,
                    &format!("{}.{}", rule.divisor, rule.literal),
                )
                .as_pointer_value(),
        );
    }

    // Main loop variables
    let loop_start = i32_type.const_int(rules.bounds.start.into(), true);
    let loop_end = i32_type.const_int(rules.bounds.end.into(), true);

    // Allocate remainder existence boolean on stack
    let _b = context.create_builder();
    let entry = main_fn.get_first_basic_block().unwrap();
    match entry.get_first_instruction() {
        Some(first_instr) => _b.position_before(&first_instr),
        None => _b.position_at_end(entry),
    }
    let rem_bool_alloca = _b.build_alloca(bool_type, "has_remainder");
    builder.build_store(rem_bool_alloca, bool_type.const_zero());

    // Allocate loop counter on stack
    let _b = context.create_builder();
    let entry = main_fn.get_first_basic_block().unwrap();
    match entry.get_first_instruction() {
        Some(first_instr) => _b.position_before(&first_instr),
        None => _b.position_at_end(entry),
    }
    let counter_alloca = _b.build_alloca(i32_type, "counter");
    builder.build_store(counter_alloca, loop_start);

    // Loop block
    let loop_bb = context.append_basic_block(main_fn, "loop");
    builder.build_unconditional_branch(loop_bb);
    builder.position_at_end(loop_bb);

    // Retrieve counter
    let curr = builder.build_load(counter_alloca, "curr_counter");
    let next = builder.build_int_add(
        curr.into_int_value(),
        i32_type.const_int(1, false),
        "next_counter",
    );
    builder.build_store(counter_alloca, next);

    // Add remainder checks
    let str_format = builder
        .build_global_string_ptr("%s", "string_format_str")
        .as_pointer_value();
    for rule in rules.rules {
        let divisor = i32_type.const_int(rule.divisor.into(), false);
        let is_divisible = builder.build_int_compare(
            IntPredicate::EQ,
            builder.build_int_unsigned_rem(curr.into_int_value(), divisor, &rule.literal),
            i32_type.const_zero(),
            &format!("{}.divisible_cond", rule.divisor),
        );

        let then_bb = context.append_basic_block(main_fn, &format!("{}.then", rule.divisor));
        let else_bb = context.append_basic_block(main_fn, &format!("{}.else", rule.divisor));

        builder.build_conditional_branch(is_divisible, then_bb, else_bb);

        // Then block, print fizz/buzz/bazz, else do nothing
        builder.position_at_end(then_bb);
        builder.build_call(
            printf,
            &[
                str_format.into(),
                (*strings.get(&rule.divisor).unwrap()).into(),
            ],
            &format!("{}", rule.divisor),
        );
        builder.build_store(rem_bool_alloca, bool_type.const_all_ones());
        builder.build_unconditional_branch(else_bb);
        builder.position_at_end(else_bb);
    }

    // See if we need to print the integer
    let newline = builder
        .build_global_string_ptr("\n", "newline_str")
        .as_pointer_value();
    let int_format = builder
        .build_global_string_ptr("%d", "int_format_str")
        .as_pointer_value();
    let rem_bool = builder.build_load(rem_bool_alloca, "load_rem_bool_alloca");
    let no_fizzbuzz_cond = builder.build_int_compare(
        IntPredicate::NE,
        rem_bool.into_int_value(),
        bool_type.const_all_ones(),
        "print_int_cond",
    );

    let then_bb = context.append_basic_block(main_fn, "print_int_then");
    let else_bb = context.append_basic_block(main_fn, "print_int_else");

    builder.build_conditional_branch(no_fizzbuzz_cond, then_bb, else_bb);

    // Then block, print integer, else do nothing
    builder.position_at_end(then_bb);
    builder.build_call(
        printf,
        &[int_format.into(), curr.into_int_value().into()],
        "print_int_call",
    );
    builder.build_unconditional_branch(else_bb);
    builder.position_at_end(else_bb);
    builder.build_call(printf, &[newline.into()], "print_newline");

    // Reset print integer check
    builder.build_store(rem_bool_alloca, bool_type.const_zero());

    // After loop block
    let end_cond = builder.build_int_compare(
        inkwell::IntPredicate::NE,
        curr.into_int_value(),
        loop_end,
        "loopcond",
    );
    let after_loop_bb = context.append_basic_block(main_fn, "afterloop");

    builder.build_conditional_branch(end_cond, loop_bb, after_loop_bb);
    builder.position_at_end(after_loop_bb);

    // Exit program
    let i32_zero = i32_type.const_int(0, false);
    builder.build_return(Some(&i32_zero));

    Ok(())
}
