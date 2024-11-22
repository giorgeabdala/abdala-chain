use std::mem::MaybeUninit;

use wasmtime::{
    AsContext, AsContextMut, Caller, Config, Engine, Func, Instance, Memory, MemoryType, Module, OptLevel, Store,
};

// Código WebAssembly em formato de texto.
const WAT_CODE: &[u8] = include_bytes!("./wasm_runtime.wat");

#[derive(Clone, Debug)]
pub struct WasmCall {
    pub memory: Memory,
    web_assembly: Option<Instance>,
}

impl WasmCall {
    #[allow(invalid_value, clippy::missing_errors_doc)]
    /// Inicializa o estado, store e memória.
    pub fn new() -> anyhow::Result<Store<Self>> {
        let engine = Engine::default();
        let memory_type = MemoryType::new(2, Some(16));
        #[allow(clippy::uninit_assumed_init)]
        let state = Self {
            memory: unsafe { MaybeUninit::<Memory>::zeroed().assume_init() },
            web_assembly: None,
        };

        let mut store = Store::new(&engine, state);

        unsafe {
            let ptr = std::ptr::addr_of_mut!(store.data_mut().memory);
            ptr.write(Memory::new(&mut store, memory_type)?);
        }

        {
            let instance = WasmCall::init_instance(&engine, &mut store)?;
            store.data_mut().web_assembly = Some(instance);
        }

        Ok(store)
    }

    fn init_instance(engine: &Engine, store: &mut Store<Self>) -> anyhow::Result<Instance> {
        let module = Module::new(engine, WAT_CODE)?;

        #[allow(clippy::cast_possible_truncation)]
        let console_log_func =
            Func::wrap(&mut *store, |caller: Caller<'_, WasmCall>, offset: u32, len: u32| {
                let ctx = caller.as_context();
                let start = usize::try_from(offset).unwrap_or(usize::MAX);
                let end = start.saturating_add(len as usize);
                let Some(bytes) = ctx.data().memory.data(&ctx).get(start..end) else {
                    anyhow::bail!("out of bounds memory access");
                };
                let Ok(string) = std::str::from_utf8(bytes) else {
                    anyhow::bail!("invalid utf-8 string");
                };
                println!("{string}");
                Ok(())
            });

        let memory = store.data().memory;
        let imports = [memory.into(), console_log_func.into()];

        let instance = Instance::new(store, &module, &imports)?;

        Ok(instance)
    }

    pub fn add(&self, store: &mut Store<Self>, a: u32, b: u32) -> u32 {
        let instance = self.web_assembly.as_ref().unwrap();
        let run = instance.get_typed_func::<(u32, u32), u32>(&mut *store, "add").unwrap();
        run.call(store, (a, b)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let engine = Engine::default();
        let memory_type = MemoryType::new(2, Some(16));
        let store = WasmCall::new().unwrap();
        assert!(store.data().web_assembly.is_some());
    }

    #[test]
    fn test_add() {

        let mut store = WasmCall::new().unwrap();

        let result = {
            let state = store.data().clone();
            state.add(&mut store, 1, 7)
        };
        assert_eq!(result, 8);
    }

}