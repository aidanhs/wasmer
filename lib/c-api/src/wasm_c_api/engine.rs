use crate::error::{update_last_error, CApiError};
use cfg_if::cfg_if;
use std::sync::Arc;
use wasmer::Engine;
#[cfg(feature = "jit")]
use wasmer_engine_jit::JIT;
#[cfg(feature = "native")]
use wasmer_engine_native::Native;
#[cfg(feature = "object-file")]
use wasmer_engine_object_file::ObjectFile;

/// Kind of compilers that can be used by the engines.
///
/// This is a Wasmer-specific type with Wasmer-specific functions for
/// manipulating it.
#[cfg(feature = "compiler")]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum wasmer_compiler_t {
    /// Variant to represent the Cranelift compiler. See the
    /// [`wasmer_compiler_cranelift`] Rust crate.
    CRANELIFT = 0,

    /// Variant to represent the LLVM compiler. See the
    /// [`wasmer_compiler_llvm`] Rust crate.
    LLVM = 1,

    /// Variant to represent the Singlepass compiler. See the
    /// [`wasmer_compiler_singlepass`] Rust crate.
    SINGLEPASS = 2,
}

#[cfg(feature = "compiler")]
impl Default for wasmer_compiler_t {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(feature = "cranelift")] {
                Self::CRANELIFT
            } else if #[cfg(feature = "llvm")] {
                Self::LLVM
            } else if #[cfg(feature = "singlepass")] {
                Self::SINGLEPASS
            } else {
                compile_error!("Please enable one of the compiler backends")
            }
        }
    }
}

/// Kind of engines that can be used by the store.
///
/// This is a Wasmer-specific type with Wasmer-specific functions for
/// manipulating it.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum wasmer_engine_t {
    /// Variant to represent the JIT engine. See the
    /// [`wasmer_engine_jit`] Rust crate.
    JIT = 0,

    /// Variant to represent the Native engine. See the
    /// [`wasmer_engine_native`] Rust crate.
    NATIVE = 1,

    /// Variant to represent the Object File engine. See the
    /// [`wasmer_engine_object_file`] Rust crate.
    OBJECT_FILE = 2,
}

impl Default for wasmer_engine_t {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(feature = "jit")] {
                Self::JIT
            } else if #[cfg(feature = "native")] {
                Self::NATIVE
            } else if #[cfg(feature = "object-file")] {
                Self::OBJECT_FILE
            } else {
                compile_error!("Please enable one of the engines")
            }
        }
    }
}

/// A configuration holds the compiler and the engine used by the store.
///
/// cbindgen:ignore
#[derive(Debug, Default)]
#[repr(C)]
pub struct wasm_config_t {
    engine: wasmer_engine_t,
    #[cfg(feature = "compiler")]
    compiler: wasmer_compiler_t,
}

/// Create a new default Wasmer configuration.
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// # fn main() {
/// #    (assert_c! {
/// # #include "tests/wasmer_wasm.h"
/// #
/// int main() {
///     // Create the configuration.
///     wasm_config_t* config = wasm_config_new();
///
///     // Create the engine.
///     wasm_engine_t* engine = wasm_engine_new_with_config(config);
///
///     // Check we have an engine!
///     assert(engine);
///
///     // Free everything.
///     wasm_engine_delete(engine);
///
///     return 0;
/// }
/// #    })
/// #    .success();
/// # }
/// ```
///
/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wasm_config_new() -> Box<wasm_config_t> {
    Box::new(wasm_config_t::default())
}

/// Updates the configuration to specify a particular compiler to use.
///
/// This is a Wasmer-specific function.
///
/// # Example
///
/// ```rust,no_run
/// # use inline_c::assert_c;
/// # fn main() {
/// #    (assert_c! {
/// # #include "tests/wasmer_wasm.h"
/// #
/// int main() {
///     // Create the configuration.
///     wasm_config_t* config = wasm_config_new();
///
///     // Use the Cranelift compiler.
///     wasm_config_set_compiler(config, CRANELIFT);
///
///     // Create the engine.
///     wasm_engine_t* engine = wasm_engine_new_with_config(config);
///
///     // Check we have an engine!
///     assert(engine);
///
///     // Free everything.
///     wasm_engine_delete(engine);
///
///     return 0;
/// }
/// #    })
/// #    .success();
/// # }
/// ```
#[cfg(feature = "compiler")]
#[no_mangle]
pub extern "C" fn wasm_config_set_compiler(
    config: &mut wasm_config_t,
    compiler: wasmer_compiler_t,
) {
    config.compiler = compiler;
}

/// Updates the configuration to specify a particular engine to use.
///
/// This is a Wasmer-specific function.
///
/// # Example
///
/// ```rust,no_run
/// # use inline_c::assert_c;
/// # fn main() {
/// #    (assert_c! {
/// # #include "tests/wasmer_wasm.h"
/// #
/// int main() {
///     // Create the configuration.
///     wasm_config_t* config = wasm_config_new();
///
///     // Use the JIT engine.
///     wasm_config_set_engine(config, JIT);
///
///     // Create the engine.
///     wasm_engine_t* engine = wasm_engine_new_with_config(config);
///
///     // Check we have an engine!
///     assert(engine);
///
///     // Free everything.
///     wasm_engine_delete(engine);
///
///     return 0;
/// }
/// #    })
/// #    .success();
/// # }
/// ```
#[no_mangle]
pub extern "C" fn wasm_config_set_engine(config: &mut wasm_config_t, engine: wasmer_engine_t) {
    config.engine = engine;
}

/// An engine is used by the store to drive the compilation and the
/// execution of a WebAssembly module.
///
/// cbindgen:ignore
#[repr(C)]
pub struct wasm_engine_t {
    pub(crate) inner: Arc<dyn Engine + Send + Sync>,
}

// Compiler JIT
#[cfg(feature = "compiler")]
use wasmer_compiler::CompilerConfig;
#[cfg(feature = "compiler")]
fn get_default_compiler_config() -> Box<dyn CompilerConfig> {
    cfg_if! {
        if #[cfg(feature = "cranelift")] {
            Box::new(wasmer_compiler_cranelift::Cranelift::default())
        } else if #[cfg(feature = "llvm")] {
            Box::new(wasmer_compiler_llvm::LLVM::default())
        } else if #[cfg(feature = "singlepass")] {
            Box::new(wasmer_compiler_singlepass::Singlepass::default())
        } else {
            compile_error!("Please enable one of the compiler backends")
        }
    }
}

cfg_if! {
    if #[cfg(all(feature = "jit", feature = "compiler"))] {
        /// Creates a new JIT engine with the default compiler.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let compiler_config: Box<dyn CompilerConfig> = get_default_compiler_config();
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(JIT::new(compiler_config).engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    } else if #[cfg(feature = "jit")] {
        /// Creates a new headless JIT engine.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(JIT::headless().engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    } else if #[cfg(all(feature = "native", feature = "compiler"))] {
        /// Creates a new native engine with the default compiler.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let mut compiler_config: Box<dyn CompilerConfig> = get_default_compiler_config();
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(Native::new(compiler_config).engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    } else if #[cfg(feature = "native")] {
        /// Creates a new headless native engine.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(Native::headless().engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    }
    // There are currently no uses of the object-file engine + compiler from the C API.
    // So if we get here, we default to headless mode regardless of if `compiler` is enabled.
    else if #[cfg(feature = "object-file")] {
        /// Creates a new headless object-file engine.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            let engine: Arc<dyn Engine + Send + Sync> = Arc::new(ObjectFile::headless().engine());
            Box::new(wasm_engine_t { inner: engine })
        }
    } else {
        /// Creates a new unknown engine, i.e. it will panic with an error message.
        ///
        /// # Example
        ///
        /// See [`wasm_engine_delete`].
        ///
        /// cbindgen:ignore
        #[no_mangle]
        pub extern "C" fn wasm_engine_new() -> Box<wasm_engine_t> {
            unimplemented!("No engine attached; You might want to recompile `wasmer_c_api` with for example `--feature jit`");
        }
    }
}

/// Deletes an engine.
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// # fn main() {
/// #    (assert_c! {
/// # #include "tests/wasmer_wasm.h"
/// #
/// int main() {
///     // Create a default engine.
///     wasm_engine_t* engine = wasm_engine_new();
///
///     // Check we have an engine!
///     assert(engine);
///
///     // Free everything.
///     wasm_engine_delete(engine);
///
///     return 0;
/// }
/// #    })
/// #    .success();
/// # }
/// ```
///
/// cbindgen:ignore
#[no_mangle]
pub unsafe extern "C" fn wasm_engine_delete(_engine: Option<Box<wasm_engine_t>>) {}

/// Creates an engine with a particular configuration.
///
/// # Example
///
/// See [`wasm_config_new`].
///
/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn wasm_engine_new_with_config(
    config: Box<wasm_config_t>,
) -> Option<Box<wasm_engine_t>> {
    #[allow(dead_code)]
    fn return_with_error<M>(msg: M) -> Option<Box<wasm_engine_t>>
    where
        M: ToString,
    {
        update_last_error(CApiError {
            msg: msg.to_string(),
        });

        return None;
    };

    cfg_if! {
        if #[cfg(feature = "compiler")] {
            #[allow(unused_mut)]
            let mut compiler_config: Box<dyn CompilerConfig> = match config.compiler {
                wasmer_compiler_t::CRANELIFT => {
                    cfg_if! {
                        if #[cfg(feature = "cranelift")] {
                            Box::new(wasmer_compiler_cranelift::Cranelift::default())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `cranelift` feature.");
                        }
                    }
                },
                wasmer_compiler_t::LLVM => {
                    cfg_if! {
                        if #[cfg(feature = "llvm")] {
                            Box::new(wasmer_compiler_llvm::LLVM::default())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `llvm` feature.");
                        }
                    }
                },
                wasmer_compiler_t::SINGLEPASS => {
                    cfg_if! {
                        if #[cfg(feature = "singlepass")] {
                            Box::new(wasmer_compiler_singlepass::Singlepass::default())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `singlepass` feature.");
                        }
                    }
                },
            };

            let inner: Arc<dyn Engine + Send + Sync> = match config.engine {
                wasmer_engine_t::JIT => {
                    cfg_if! {
                        if #[cfg(feature = "jit")] {
                            Arc::new(JIT::new(compiler_config).engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `jit` feature.");
                        }
                    }
                },
                wasmer_engine_t::NATIVE => {
                    cfg_if! {
                        if #[cfg(feature = "native")] {
                            Arc::new(Native::new(compiler_config).engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `native` feature.");
                        }
                    }
                },
                wasmer_engine_t::OBJECT_FILE => {
                    cfg_if! {
                        // There are currently no uses of the object-file engine + compiler from the C API.
                        // So we run in headless mode.
                        if #[cfg(feature = "object-file")] {
                            Arc::new(ObjectFile::headless().engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `object-file` feature.");
                        }
                    }
                },
            };
            Some(Box::new(wasm_engine_t { inner }))
        } else {
            let inner: Arc<dyn Engine + Send + Sync> = match config.engine {
                wasmer_engine_t::JIT => {
                    cfg_if! {
                        if #[cfg(feature = "jit")] {
                            Arc::new(JIT::headless().engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `jit` feature.");
                        }
                    }
                },
                wasmer_engine_t::NATIVE => {
                    cfg_if! {
                        if #[cfg(feature = "native")] {
                            Arc::new(Native::headless().engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `native` feature.");
                        }
                    }
                },
                wasmer_engine_t::OBJECT_FILE => {
                    cfg_if! {
                        if #[cfg(feature = "object-file")] {
                            Arc::new(ObjectFile::headless().engine())
                        } else {
                            return return_with_error("Wasmer has not been compiled with the `object-file` feature.");
                        }
                    }
                },
            };
            Some(Box::new(wasm_engine_t { inner }))
        }
    }
}

#[cfg(test)]
mod tests {
    use inline_c::assert_c;

    #[test]
    fn test_engine_new() {
        (assert_c! {
            #include "tests/wasmer_wasm.h"

            int main() {
                wasm_engine_t* engine = wasm_engine_new();
                assert(engine);

                wasm_engine_delete(engine);

                return 0;
            }
        })
        .success();
    }
}
