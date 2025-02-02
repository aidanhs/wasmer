use wasmer_vm::{
    ImportInitializerFuncPtr, VMExport, VMExportFunction, VMExportGlobal, VMExportMemory,
    VMExportTable,
};

use std::sync::Arc;

/// The value of an export passed from one instance to another.
#[derive(Debug, Clone)]
pub enum Export {
    /// A function export value.
    Function(ExportFunction),

    /// A table export value.
    Table(ExportTable),

    /// A memory export value.
    Memory(ExportMemory),

    /// A global export value.
    Global(ExportGlobal),
}

impl From<Export> for VMExport {
    fn from(other: Export) -> Self {
        match other {
            Export::Function(ExportFunction { vm_function, .. }) => VMExport::Function(vm_function),
            Export::Memory(ExportMemory { vm_memory }) => VMExport::Memory(vm_memory),
            Export::Table(ExportTable { vm_table }) => VMExport::Table(vm_table),
            Export::Global(ExportGlobal { vm_global }) => VMExport::Global(vm_global),
        }
    }
}

impl From<VMExport> for Export {
    fn from(other: VMExport) -> Self {
        match other {
            VMExport::Function(vm_function) => Export::Function(ExportFunction {
                vm_function,
                metadata: None,
            }),
            VMExport::Memory(vm_memory) => Export::Memory(ExportMemory { vm_memory }),
            VMExport::Table(vm_table) => Export::Table(ExportTable { vm_table }),
            VMExport::Global(vm_global) => Export::Global(ExportGlobal { vm_global }),
        }
    }
}

/// Extra metadata about `ExportFunction`s.
///
/// The metadata acts as a kind of manual virtual dispatch. We store the
/// user-supplied `WasmerEnv` as a void pointer and have methods on it
/// that have been adapted to accept a void pointer.
///
/// This struct owns the original `host_env`, thus when it gets dropped
/// it calls the `drop` function on it.
#[derive(Debug, PartialEq)]
pub struct ExportFunctionMetadata {
    /// This field is stored here to be accessible by `Drop`.
    ///
    /// At the time it was added, it's not accessed anywhere outside of
    /// the `Drop` implementation. This field is the "master copy" of the env,
    /// that is, the original env passed in by the user. Every time we create
    /// an `Instance` we clone this with the `host_env_clone_fn` field.
    ///
    /// Thus, we only bother to store the master copy at all here so that
    /// we can free it.
    ///
    /// See `wasmer_vm::export::VMExportFunction::vmctx` for the version of
    /// this pointer that is used by the VM when creating an `Instance`.
    pub(crate) host_env: *mut std::ffi::c_void,
    /// Function pointer to `WasmerEnv::init_with_instance(&mut self, instance: &Instance)`.
    ///
    /// This function is called to finish setting up the environment after
    /// we create the `api::Instance`.
    // This one is optional for now because dynamic host envs need the rest
    // of this without the init fn
    pub(crate) import_init_function_ptr: Option<ImportInitializerFuncPtr>,
    /// A function analogous to `Clone::clone` that returns a leaked `Box`.
    pub(crate) host_env_clone_fn: fn(*mut std::ffi::c_void) -> *mut std::ffi::c_void,
    /// The destructor to free the host environment.
    ///
    /// # Safety
    /// - This function should only be called in when properly synchronized.
    /// For example, in the `Drop` implementation of this type.
    pub(crate) host_env_drop_fn: unsafe fn(*mut std::ffi::c_void),
}

/// This can be `Send` because `host_env` comes from `WasmerEnv` which is
/// `Send`. Therefore all operations should work on any thread.
unsafe impl Send for ExportFunctionMetadata {}
/// This data may be shared across threads, `drop` is an unsafe function
/// pointer, so care must be taken when calling it.
unsafe impl Sync for ExportFunctionMetadata {}

impl ExportFunctionMetadata {
    /// Create an `ExportFunctionMetadata` type with information about
    /// the exported function.
    ///
    /// # Safety
    /// - the `host_env` must be `Send`.
    /// - all function pointers must work on any thread.
    pub unsafe fn new(
        host_env: *mut std::ffi::c_void,
        import_init_function_ptr: Option<ImportInitializerFuncPtr>,
        host_env_clone_fn: fn(*mut std::ffi::c_void) -> *mut std::ffi::c_void,
        host_env_drop_fn: fn(*mut std::ffi::c_void),
    ) -> Self {
        Self {
            host_env,
            import_init_function_ptr,
            host_env_clone_fn,
            host_env_drop_fn,
        }
    }
}

// We have to free `host_env` here because we always clone it before using it
// so all the `host_env`s freed at the `Instance` level won't touch the original.
impl Drop for ExportFunctionMetadata {
    fn drop(&mut self) {
        if !self.host_env.is_null() {
            // # Safety
            // - This is correct because we know no other references
            //   to this data can exist if we're dropping it.
            unsafe {
                (self.host_env_drop_fn)(self.host_env);
            }
        }
    }
}

/// A function export value with an extra function pointer to initialize
/// host environments.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportFunction {
    /// The VM function, containing most of the data.
    pub vm_function: VMExportFunction,
    /// Contains functions necessary to create and initialize host envs
    /// with each `Instance` as well as being responsible for the
    /// underlying memory of the host env.
    pub metadata: Option<Arc<ExportFunctionMetadata>>,
}

impl From<ExportFunction> for Export {
    fn from(func: ExportFunction) -> Self {
        Self::Function(func)
    }
}

/// A table export value.
#[derive(Debug, Clone)]
pub struct ExportTable {
    /// The VM table, containing info about the table.
    pub vm_table: VMExportTable,
}

impl From<ExportTable> for Export {
    fn from(table: ExportTable) -> Self {
        Self::Table(table)
    }
}

/// A memory export value.
#[derive(Debug, Clone)]
pub struct ExportMemory {
    /// The VM memory, containing info about the table.
    pub vm_memory: VMExportMemory,
}

impl From<ExportMemory> for Export {
    fn from(memory: ExportMemory) -> Self {
        Self::Memory(memory)
    }
}

/// A global export value.
#[derive(Debug, Clone)]
pub struct ExportGlobal {
    /// The VM global, containing info about the global.
    pub vm_global: VMExportGlobal,
}

impl From<ExportGlobal> for Export {
    fn from(global: ExportGlobal) -> Self {
        Self::Global(global)
    }
}
