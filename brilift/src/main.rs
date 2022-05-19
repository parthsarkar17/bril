use bril_rs as bril;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift::codegen::{ir, isa, settings};
use cranelift::codegen::entity::EntityRef;
use cranelift::codegen::verifier::verify_function;
use std::collections::HashMap;

fn tr_type(typ: &bril::Type) -> ir::Type {
    match typ {
        bril::Type::Int => ir::types::I32,
        bril::Type::Bool => ir::types::B1,
    }
}

fn tr_sig(func: &bril::Function) -> ir::Signature {
    let mut sig = ir::Signature::new(isa::CallConv::SystemV);
    if let Some(ret) = &func.return_type {
        sig.returns.push(ir::AbiParam::new(tr_type(ret)));
    }
    for arg in &func.args {
        sig.params.push(ir::AbiParam::new(tr_type(&arg.arg_type)));
    }
    sig
}

fn all_vars(func: &bril::Function) -> HashMap<&String, &bril::Type> {
    func.instrs.iter().filter_map(|inst| {
        match inst {
            bril::Code::Instruction(op) => {
                match op {
                    bril::Instruction::Constant { dest, op: _, const_type: typ, value: _ } => {
                        Some((dest, typ))
                    },
                    bril::Instruction::Value { args: _, dest, funcs: _, labels: _, op: _, op_type: typ } => {
                        Some((dest, typ))
                    },
                    _ => None
                }
            },
            _ => None
        }
    }).collect()
}

fn compile_func(func: bril::Function) {
    // Build function signature.
    let sig = tr_sig(&func);
    
    // Create the function.
    // TODO Do something about the name.
    let mut fn_builder_ctx = FunctionBuilderContext::new();
    let mut cl_func = ir::Function::with_name_signature(ir::ExternalName::user(0, 0), sig);
    
    // Build the function body.
    {
        let mut builder = FunctionBuilder::new(&mut cl_func, &mut fn_builder_ctx);
        
        // Declare all variables.
        let vars = HashMap::<String, Variable>::new();
        for (i, (name, typ)) in all_vars(&func).iter().enumerate() {
            let var = Variable::new(i);
            builder.declare_var(var, tr_type(typ));
        }

        let block = builder.create_block();
        
        builder.finalize();
    }
    
    // Verify and print.
    let flags = settings::Flags::new(settings::builder());
    let res = verify_function(&cl_func, &flags);
    println!("{}", cl_func.display());
    if let Err(errors) = res {
        panic!("{}", errors);
    }
}

fn main() {
    // Load the Bril program from stdin.
    let prog = bril::load_program();
    
    // Cranelift builder context.

    for func in prog.functions {
        compile_func(func);
    }
}
