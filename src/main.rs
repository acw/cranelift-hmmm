use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::{types, AbiParam, Function, InstBuilder, Signature, UserFuncName};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::{isa, settings, Context};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{default_libcall_names, DataContext, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use target_lexicon::Triple;

fn main() -> Result<(), anyhow::Error> {
    let host = Triple::host();
    let isa = isa::lookup(host.clone())?.finish(settings::Flags::new(settings::builder()))?;
    let object_builder = ObjectBuilder::new(isa, "example", default_libcall_names())?;
    let mut object_module = ObjectModule::new(object_builder);

    // define the print function
    let string_param = AbiParam::new(types::I64);
    let int64_param = AbiParam::new(types::I64);
    let print_id = object_module.declare_function(
        "print",
        Linkage::Import,
        &Signature {
            params: vec![string_param, int64_param],
            returns: vec![],
            call_conv: CallConv::triple_default(&host),
        },
    )?;

    // define the string variable
    let string_global_id =
        object_module.declare_data("variable-name-x", Linkage::Local, false, false)?;
    let mut string_context = DataContext::new();
    string_context.define("x\0".to_owned().into_boxed_str().into_boxed_bytes());
    object_module.define_data(string_global_id, &string_context)?;
 
    // define gogogo
    let empty_signature = Signature {
        params: vec![],
        returns: vec![],
        call_conv: CallConv::triple_default(&host),
    };
    let gogogo_id = object_module.declare_function("gogogo", Linkage::Export, &empty_signature)?;
    let mut ctx = Context::new();
    ctx.func =
        Function::with_name_signature(UserFuncName::user(0, gogogo_id.as_u32()), empty_signature);

    // local versions of the object globals
    let string_local_id = object_module.declare_data_in_func(string_global_id, &mut ctx.func);
    let local_print_id = object_module.declare_func_in_func(print_id, &mut ctx.func);

    // set up the block
    let mut fctx = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fctx);
    let main_block = builder.create_block();
    builder.switch_to_block(main_block);

    // {
    //    x = 4;
    //    print("x", x);
    // }
    let var_x = Variable::new(0);
    builder.declare_var(var_x, types::I64);
    let val_x = builder.ins().iconst(types::I64, 4);
    builder.def_var(var_x, val_x);
    let var_name_str = builder.ins().symbol_value(types::I64, string_local_id);
    let val = builder.use_var(var_x);
    builder.ins().call(local_print_id, &[var_name_str, val]);

    // close it all up
    builder.ins().return_(&[]);
    builder.seal_block(main_block);
    builder.finalize();
    object_module.define_function(gogogo_id, &mut ctx)?;

    // write it out
    let bytes = object_module.finish().emit()?;
    std::fs::write("output.o", bytes)?;

    Ok(())
}
