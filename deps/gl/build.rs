extern crate gl_generator;

use gl_generator::{
    generators::{gen_enum_item, gen_parameters, gen_symbol_name, gen_types},
    Api, Fallbacks, Generator, Profile, Registry,
};
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;

#[allow(missing_copy_implementations)]
pub struct CustomGenerator;

impl Generator for CustomGenerator {
    fn write<W>(&self, registry: &Registry, dest: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        try!(write_header(dest));
        try!(write_metaloadfn(dest));
        try!(write_type_aliases(registry, dest));
        try!(write_enums(registry, dest));
        try!(write_gl_guard(dest));
        try!(write_fns(registry, dest));
        try!(write_fnptr_struct_def(dest));
        try!(write_ptrs(registry, dest));
        try!(write_fn_mods(registry, dest));
        try!(write_panicking_fns(registry, dest));
        try!(write_load_fn(registry, dest));
        Ok(())
    }
}

/// Creates a `__gl_imports` module which contains all the external symbols that we need for the
///  bindings.
fn write_header<W>(dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        r#"
        extern crate backtrace;
        mod __gl_imports {{
            pub use std::mem;
            pub use std::process;
            pub use std::os::raw;
            pub use std::ffi::CString;
        }}
    "#
    )
}

/// Creates the metaloadfn function for fallbacks
fn write_metaloadfn<W>(dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        r#"
        #[inline(never)]
        fn metaloadfn(loadfn: &mut FnMut(&'static str) -> *const __gl_imports::raw::c_void,
                      symbol: &'static str,
                      fallbacks: &[&'static str]) -> *const __gl_imports::raw::c_void {{
            let mut ptr = loadfn(symbol);
            if ptr.is_null() {{
                for &sym in fallbacks {{
                    ptr = loadfn(sym);
                    if !ptr.is_null() {{ break; }}
                }}
            }}
            ptr
        }}
    "#
    )
}

/// Creates a `types` module which contains all the type aliases.
///
/// See also `generators::gen_types`.
fn write_type_aliases<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    try!(writeln!(
        dest,
        r#"
        pub mod types {{
            #![allow(non_camel_case_types, non_snake_case, dead_code, missing_copy_implementations)]
    "#
    ));

    try!(gen_types(registry.api, dest));

    writeln!(
        dest,
        "
        }}
    "
    )
}

/// Creates all the `<enum>` elements at the root of the bindings.
fn write_enums<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    for enm in &registry.enums {
        try!(gen_enum_item(enm, "types::", dest));
    }

    Ok(())
}

/// Creates the gl_guard function for opengl error checking
fn write_gl_guard<W>(dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        r#"
        unsafe fn gl_guard(fn_name: &str, params: &str) {{
            let err = __gl_imports::mem::transmute::<_, extern "system" fn() -> u32> (storage::GetError.f)();
            if err != self::NO_ERROR {{
                // Show generic info about the error
                println!("[OpenGL] error @ gl{{}}({{}})", fn_name, params);
                loop {{
                    // Gather OpenGL log length
                    let mut len: types::GLint = 0;
                    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, *mut types::GLint)>(storage::GetIntegerv.f)(self::DEBUG_NEXT_LOGGED_MESSAGE_LENGTH, &mut len as *mut types::GLint);
                    if len == 0 {{ break; }}

                    // Create string buffer
                    let blen = len as usize;
                    let mut buf: Vec<u8> = Vec::with_capacity(blen + 1);
                    buf.extend([b' '].iter().cycle().take(blen));
                    let buf = __gl_imports::CString::from_vec_unchecked(buf);

                    // Gather OpenGL log entry contents
                    let mut source: types::GLenum = 0; let mut ty: types::GLenum = 0; let mut id: types::GLuint = 0; let mut severity: types::GLenum = 0; let mut length: types::GLsizei = 0;
                    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint, types::GLsizei, *mut types::GLenum, *mut types::GLenum, *mut types::GLuint, *mut types::GLenum, *mut types::GLsizei, *mut types::GLchar) -> types::GLuint>(storage::GetDebugMessageLog.f)(1, len,
                        &mut source as *mut types::GLenum, &mut ty as *mut types::GLenum, &mut id as *mut types::GLuint, &mut severity as *mut types::GLenum, &mut length as *mut types::GLsizei, buf.as_ptr() as *mut types::GLchar);
                    let msg = buf.to_string_lossy().into_owned();

                    // Show current log entry
                    if ty == self::DEBUG_TYPE_ERROR {{
                        let source = match source {{
                            DEBUG_SOURCE_API             => "GL_DEBUG_SOURCE_API",
                            DEBUG_SOURCE_SHADER_COMPILER => "GL_DEBUG_SOURCE_SHADER_COMPILER",
                            DEBUG_SOURCE_WINDOW_SYSTEM   => "GL_DEBUG_SOURCE_WINDOW_SYSTEM",
                            DEBUG_SOURCE_THIRD_PARTY     => "GL_DEBUG_SOURCE_THIRD_PARTY",
                            DEBUG_SOURCE_APPLICATION     => "GL_DEBUG_SOURCE_APPLICATION",
                            DEBUG_SOURCE_OTHER           => "GL_DEBUG_SOURCE_OTHER",
                            _ => "???"
                        }};
                        let ty = match ty {{
                            DEBUG_TYPE_ERROR               => "GL_DEBUG_TYPE_ERROR",
                            DEBUG_TYPE_DEPRECATED_BEHAVIOR => "GL_DEBUG_TYPE_DEPRECATED_BEHAVIOR",
                            DEBUG_TYPE_UNDEFINED_BEHAVIOR  => "GL_DEBUG_TYPE_UNDEFINED_BEHAVIOR",
                            DEBUG_TYPE_PERFORMANCE         => "GL_DEBUG_TYPE_PERFORMANCE",
                            DEBUG_TYPE_PORTABILITY         => "GL_DEBUG_TYPE_PORTABILITY",
                            DEBUG_TYPE_MARKER              => "GL_DEBUG_TYPE_MARKER",
                            DEBUG_TYPE_PUSH_GROUP          => "GL_DEBUG_TYPE_PUSH_GROUP",
                            DEBUG_TYPE_POP_GROUP           => "GL_DEBUG_TYPE_POP_GROUP",
                            DEBUG_TYPE_OTHER               => "GL_DEBUG_TYPE_OTHER",
                            _ => "???"
                        }};
                        let severity = match severity {{
                            DEBUG_SEVERITY_HIGH         => "GL_DEBUG_SEVERITY_HIGH",
                            DEBUG_SEVERITY_MEDIUM       => "GL_DEBUG_SEVERITY_MEDIUM",
                            DEBUG_SEVERITY_LOW          => "GL_DEBUG_SEVERITY_LOW",
                            DEBUG_SEVERITY_NOTIFICATION => "GL_DEBUG_SEVERITY_NOTIFICATION",
                            _ => "???"
                        }};
                        println!("Type     : {{}}\nSource   : {{}}\nSeverity : {{}}\nMessage  : {{}}", ty, source, severity, msg);
                    }}
                }}

                let mut bt = String::new();
                let mut i = 0;
                backtrace::trace(|frame| {{
                    let ip = frame.ip();
                    let symbol_address = frame.symbol_address();
                    if symbol_address as usize == 0x0 {{
                        return true;
                    }}

                    // Resolve this instruction pointer to a symbol name
                    backtrace::resolve(ip, |symbol| {{
                        let filename = match symbol.filename() {{
                            Some(path) => {{
                                if path.is_absolute() {{
                                    format!("<external_path>/{{:?}}", path.file_name().unwrap())
                                }} else {{
                                    format!("{{:?}}", path)
                                }}
                            }},
                            None => "???".to_string()
                        }};
                        let lineno = match symbol.lineno() {{
                            Some(line) => line.to_string(),
                            None => "???".to_string()
                        }};
                        let name = match symbol.name() {{
                            Some(symbol_name) => format!("{{:?}}", symbol_name),
                            None => "???".to_string()
                        }};
                        let frame_info = format!(" #{{:<2}} {{:p}} {{:70}} {{}}:{{}}\n", i, symbol_address, name, filename, lineno);
                        bt.push_str(&frame_info);
                    }});

                    i += 1;
                    true // Keep going to the next frame
                }});
                println!("[Backtrace]\n{{}}", bt);
                __gl_imports::process::exit(-1);
            }}
        }}"#
    )
}

/// Creates the functions corresponding to the GL commands.
///
/// The function calls the corresponding function pointer stored in the `storage` module created
///  by `write_ptrs`.
fn write_fns<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    for cmd in &registry.cmds {
        if let Some(v) = registry.aliases.get(&cmd.proto.ident) {
            try!(writeln!(dest, "/// Fallbacks: {}", v.join(", ")));
        }

        let idents = gen_parameters(cmd, true, false);
        let typed_params = gen_parameters(cmd, false, true);
        let params = gen_parameters(cmd, true, true);

        let param_values = format!(
            "&format!(\"{}\" {})",
            (0..idents.len())
                .map(|_| "{:?}".to_string())
                .collect::<Vec<_>>()
                .join(", "),
            idents
                .iter()
                .zip(typed_params.iter())
                .map(|(name, ty)| if ty.contains("GLDEBUGPROC") {
                    format!(", \"<callback>\"")
                } else {
                    format!(", {}", name)
                }).collect::<Vec<_>>()
                .concat()
        );

        try!(writeln!(dest,
            "#[allow(non_snake_case, unused_variables, dead_code)] #[inline]
            pub unsafe fn {name}({params}) -> {return_suffix} {{
                let r = __gl_imports::mem::transmute::<_, extern \"system\" fn({typed_params}) -> {return_suffix}>\
                    (storage::{name}.f)({idents});
                    {guard}
                r
            }}",
            name = cmd.proto.ident,
            params = params.join(", "),
            typed_params = typed_params.join(", "),
            return_suffix = cmd.proto.ty,
            idents = idents.join(", "),
            guard = if cmd.proto.ident != "GetError" { format!("gl_guard(\"{}\", {});", cmd.proto.ident, param_values) } else { String::from("") }
        ));
    }

    Ok(())
}

/// Creates a `FnPtr` structure which contains the store for a single binding.
fn write_fnptr_struct_def<W>(dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(dest,
             "
        #[allow(missing_copy_implementations)]
        pub struct FnPtr {{
            /// The function pointer that will be used when calling the function.
            f: *const __gl_imports::raw::c_void,
            /// True if the pointer points to a real function, false if points to a `panic!` fn.
            is_loaded: bool,
        }}
        impl FnPtr {{
            /// Creates a `FnPtr` from a load attempt.
            pub fn new(ptr: *const __gl_imports::raw::c_void) -> FnPtr {{
                if ptr.is_null() {{
                    FnPtr {{ f: missing_fn_panic as *const __gl_imports::raw::c_void, is_loaded: false }}
                }} else {{
                    FnPtr {{ f: ptr, is_loaded: true }}
                }}
            }}
        }}
    ")
}

/// Creates a `storage` module which contains a static `FnPtr` per GL command in the registry.
fn write_ptrs<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    try!(writeln!(
        dest,
        "mod storage {{
            #![allow(non_snake_case)]
            #![allow(non_upper_case_globals)]
            use super::__gl_imports::raw;
            use super::FnPtr;"
    ));

    for c in &registry.cmds {
        try!(writeln!(
            dest,
            "pub static mut {name}: FnPtr = FnPtr {{
                f: super::missing_fn_panic as *const raw::c_void,
                is_loaded: false
            }};",
            name = c.proto.ident
        ));
    }

    writeln!(dest, "}}")
}

/// Creates one module for each GL command.
///
/// Each module contains `is_loaded` and `load_with` which interact with the `storage` module
///  created by `write_ptrs`.
fn write_fn_mods<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    for c in &registry.cmds {
        let fallbacks = match registry.aliases.get(&c.proto.ident) {
            Some(v) => {
                let names = v
                    .iter()
                    .map(|name| format!("\"{}\"", gen_symbol_name(registry.api, &name[..])))
                    .collect::<Vec<_>>();
                format!("&[{}]", names.join(", "))
            }
            None => "&[]".to_string(),
        };
        let fnname = &c.proto.ident[..];
        let symbol = gen_symbol_name(registry.api, &c.proto.ident[..]);
        let symbol = &symbol[..];

        try!(writeln!(dest, r##"
            #[allow(non_snake_case)]
            pub mod {fnname} {{
                use super::{{storage, metaloadfn}};
                use super::__gl_imports::raw;
                use super::FnPtr;
                #[inline]
                #[allow(dead_code)]
                pub fn is_loaded() -> bool {{
                    unsafe {{ storage::{fnname}.is_loaded }}
                }}
                #[allow(dead_code)]
                pub fn load_with<F>(mut loadfn: F) where F: FnMut(&'static str) -> *const raw::c_void {{
                    unsafe {{
                        storage::{fnname} = FnPtr::new(metaloadfn(&mut loadfn, "{symbol}", {fallbacks}))
                    }}
                }}
            }}
        "##, fnname = fnname, fallbacks = fallbacks, symbol = symbol));
    }

    Ok(())
}

/// Creates a `missing_fn_panic` function.
///
/// This function is the mock that is called if the real function could not be called.
fn write_panicking_fns<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        "#[inline(never)]
        fn missing_fn_panic() -> ! {{
            panic!(\"{api} function was not loaded\")
        }}
        ",
        api = registry.api
    )
}

/// Creates the `load_with` function.
///
/// The function calls `load_with` in each module created by `write_fn_mods`.
fn write_load_fn<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    try!(writeln!(dest,
                  "
        /// Load each OpenGL symbol using a custom load function. This allows for the
        /// use of functions like `glfwGetProcAddress` or `SDL_GL_GetProcAddress`.
        /// ~~~ignore
        /// gl::load_with(|s| glfw.get_proc_address(s));
        /// ~~~
        #[allow(dead_code)]
        pub fn load_with<F>(mut loadfn: F) where F: FnMut(&'static str) -> *const __gl_imports::raw::c_void {{
    "));

    for c in &registry.cmds {
        try!(writeln!(
            dest,
            "{cmd_name}::load_with(&mut loadfn);",
            cmd_name = &c.proto.ident[..]
        ));
    }

    writeln!(
        dest,
        "
        }}
    "
    )
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&out_dir).join("bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(CustomGenerator, &mut file)
        .unwrap();
}
