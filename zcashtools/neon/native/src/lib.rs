use neon::prelude::*;

use std::path::Path;

use neon_serde::ResultExt;
use zcash_hsmbuilder::errors::Error;
use zcash_hsmbuilder::*;

// reference
// https://neon-bindings.com/docs/primitives

//bridge stuff only

fn get_inittx_data(mut cx: FunctionContext) -> JsResult<JsValue> {
    // First get call arguments
    let arg0 = cx.argument::<JsValue>(0)?;
    let arg0_value: InitData = neon_serde::from_value(&mut cx, arg0).throw(&mut cx)?;
    let output = arg0_value.to_hsm_bytes();
    let js_value;
    if output.is_ok() {
        js_value = neon_serde::to_value(&mut cx, &output.unwrap()).throw(&mut cx)?;
        Ok(js_value)
    } else {
        cx.throw_error(output.err().unwrap().to_string())
    }
}

pub struct ZcashBuilderBridge {
    zcashbuilder: ZcashBuilder,
}

impl ZcashBuilderBridge {
    pub fn add_transparent_input(&mut self, t: TransparentInputBuilderInfo) -> Result<(), Error> {
        self.zcashbuilder.add_transparent_input(t)
    }

    pub fn add_transparent_output(
        &mut self,
        input: TransparentOutputBuilderInfo,
    ) -> Result<(), Error> {
        self.zcashbuilder.add_transparent_output(input)
    }

    pub fn add_sapling_spend(&mut self, input: SpendBuilderInfo) -> Result<(), Error> {
        self.zcashbuilder.add_sapling_spend(input)
    }

    pub fn add_sapling_output(&mut self, input: OutputBuilderInfo) -> Result<(), Error> {
        self.zcashbuilder.add_sapling_output(input)
    }

    pub fn build(&mut self, spendpath: &String, outputpath: &String) -> Result<Vec<u8>, Error> {
        let mut prover = txprover::LocalTxProver::new(Path::new(spendpath), Path::new(outputpath));
        self.zcashbuilder.build(&mut prover)
    }

    pub fn add_signatures(&mut self, input: TransactionSignatures) -> Result<(), Error> {
        self.zcashbuilder.add_signatures(input)
    }

    pub fn finalize(&mut self) -> Result<Vec<u8>, Error> {
        self.zcashbuilder.finalize_js()
    }
}

declare_types! {
    pub class JsZcashBuilder for ZcashBuilderBridge {
        init(mut cx) {
            let f = cx.argument::<JsNumber>(0)?.value();
            let b = ZcashBuilder::new(f as u64);
            Ok(ZcashBuilderBridge {
                zcashbuilder: b,
            })
        }

        method add_transparent_input(mut cx) {
            let arg0 = cx.argument::<JsValue>(0)?;
            let arg0_value :TransparentInputBuilderInfo = neon_serde::from_value(&mut cx, arg0).throw(&mut cx)?;
            let value;
            {
            let mut this = cx.this();
            let guard = cx.lock();
            let mut thishandler = this.borrow_mut(&guard);

            //grab input
            value = thishandler.add_transparent_input(arg0_value);
            }
            if value.is_ok(){
                Ok(cx.boolean(true).upcast())
            }else{
                cx.throw_error(value.err().unwrap().to_string())
            }
        }

        method add_transparent_output(mut cx) {
            let arg0 = cx.argument::<JsValue>(0)?;
            let arg0_value :TransparentOutputBuilderInfo = neon_serde::from_value(&mut cx, arg0).throw(&mut cx)?;
            let value;
            {
            let mut this = cx.this();
            let guard = cx.lock();
            let mut thishandler = this.borrow_mut(&guard);

            //grab input
            value = thishandler.add_transparent_output(arg0_value);
            }
            if value.is_ok(){
                Ok(cx.boolean(true).upcast())
            }else{
                cx.throw_error(value.err().unwrap().to_string())
            }
        }

        method add_sapling_spend(mut cx) {
            let arg0 = cx.argument::<JsValue>(0)?;
            let arg0_value :SpendBuilderInfo = neon_serde::from_value(&mut cx, arg0).throw(&mut cx)?;
            let value;
            {
            let mut this = cx.this();
            let guard = cx.lock();
            let mut thishandler = this.borrow_mut(&guard);

            //grab input
            value = thishandler.add_sapling_spend(arg0_value);
            }
            if value.is_ok(){
                Ok(cx.boolean(true).upcast())
            }else{
                cx.throw_error(value.err().unwrap().to_string())
            }
        }

        method add_sapling_output(mut cx) {
            let arg0 = cx.argument::<JsValue>(0)?;
            let arg0_value :OutputBuilderInfo = neon_serde::from_value(&mut cx, arg0).throw(&mut cx)?;
            let value;
            {
            let mut this = cx.this();
            let guard = cx.lock();
            let mut thishandler = this.borrow_mut(&guard);

            //grab input
            value = thishandler.add_sapling_output(arg0_value);
            }
            if value.is_ok(){
                Ok(cx.boolean(true).upcast())
            }else{
                cx.throw_error(value.err().unwrap().to_string())
            }
        }

        method build(mut cx){
            let spendpath: String = cx.argument::<JsString>(0)?.value();
            let outputpath: String = cx.argument::<JsString>(1)?.value();
            let value;
            {
            let mut this = cx.this();
            let guard = cx.lock();
            let mut thishandler = this.borrow_mut(&guard);

            //grab input
            value = thishandler.build(&spendpath, &outputpath);
            }
            if value.is_ok(){
                let js_value = neon_serde::to_value(&mut cx, &value.unwrap()).throw(&mut cx)?;
                Ok(js_value)
            }else{
                cx.throw_error(value.err().unwrap().to_string())
            }
        }

        method add_signatures(mut cx) {
            let arg0 = cx.argument::<JsValue>(0)?;
            let arg0_value :TransactionSignatures = neon_serde::from_value(&mut cx, arg0).throw(&mut cx)?;
            let value;
            {
            let mut this = cx.this();
            let guard = cx.lock();
            let mut thishandler = this.borrow_mut(&guard);

            //grab input
            value = thishandler.add_signatures(arg0_value);
            }
            if value.is_ok(){
                Ok(cx.boolean(true).upcast())
            }else{
                cx.throw_error(value.err().unwrap().to_string())
            }
        }

        method finalize(mut cx) {
            let value;
            {
            let mut this = cx.this();
            let guard = cx.lock();
            let mut thishandler = this.borrow_mut(&guard);
            //grab input
            value = thishandler.finalize();
            }
            if value.is_ok(){
                let js_value = neon_serde::to_value(&mut cx, &value.unwrap()).throw(&mut cx)?;
                Ok(js_value)
            }else{
                cx.throw_error(value.err().unwrap().to_string())
            }
        }
    }
}

register_module!(mut m, {
    m.export_class::<JsZcashBuilder>("zcashtools")?;
    m.export_function("get_inittx_data", get_inittx_data)?;
    Ok(())
});
