// we're already nightly-only so might as well use unstable proc macro APIs.
#![feature(proc_macro_span)]

use std::error::Error;
use std::fs::DirBuilder;
use std::path::PathBuf;
use std::{env, process};

use litrs::StringLit;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn include_texture(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match include_texture_impl(input) {
        Ok(tokens) => tokens,
        Err(err) => {
            let err_str = err.to_string();
            quote! { compile_error!( #err_str ) }.into()
        }
    }
}

fn include_texture_impl(input: TokenStream) -> Result<TokenStream, Box<dyn Error>> {
    let tokens: Vec<_> = input.into_iter().collect();

    if tokens.len() != 1 {
        return Err(format!("expected exactly one input token, got {}", tokens.len()).into());
    }

    let texture_source_filename = &tokens[0];

    let string_lit = match StringLit::try_from(texture_source_filename) {
        Ok(lit) => lit,
        Err(err) => return Ok(err.to_compile_error()),
    };

    // The cwd can change depending on whether this is running in a doctest or not:
    // https://users.rust-lang.org/t/which-directory-does-a-proc-macro-run-from/71917
    //
    // But the span's `source_file()` seems to always be relative to the cwd.
    let cwd = env::current_dir()
        .map_err(|err| format!("unable to determine current directory: {err}"))?;

    let invoking_source_file = texture_source_filename.span().source_file().path();
    let Some(invoking_source_dir) = invoking_source_file.parent() else {
        return Ok(quote! {
            compile_error!(
                concat!(
                    "unable to find parent directory of current source file \"",
                    file!(),
                    "\""
                )
            )
        }
        .into());
    };

    // By joining these three pieces, we arrive at approximately the same behavior as `include_bytes!`
    let texture_source_file = cwd.join(invoking_source_dir).join(string_lit.value());

    #[cfg(not(win))]
    let texture_source_file = texture_source_file
        // This might be overkill, but it ensures we get a unique path if different
        // shaders with the same relative path are used within one program
        .canonicalize()
        .map_err(|err| format!("unable to resolve absolute path of texture source: {err}"))?;

    let texture_out_file: PathBuf = texture_source_file.with_extension("txbin");

    let out_dir = PathBuf::from(env!("OUT_DIR"));

    let out_path = out_dir.join(texture_out_file.components().skip(1).collect::<PathBuf>());
    // UNWRAP: we already canonicalized the source path, so it should have a parent.
    let out_parent = out_path.parent().unwrap();

    DirBuilder::new()
        .recursive(true)
        .create(out_parent)
        .map_err(|err| format!("unable to create output directory {out_parent:?}: {err}"))?;

    let devkitpro = PathBuf::from(env!("DEVKITPRO"));
    let tex3ds = devkitpro.join("tools/bin/tex3ds");

    let output = process::Command::new(&tex3ds)
        .arg("-z")
        .arg("none")
        .arg("-r")
        .arg("-o")
        .args([&out_path, &texture_source_file])
        .output()
        .map_err(|err| format!("unable to run {tex3ds:?}: {err}"))?;

    let error_code = match output.status.code() {
        Some(0) => None,
        code => Some(code.map_or_else(|| String::from("<unknown>"), |c| c.to_string())),
    };

    if let Some(code) = error_code {
        return Err(format!(
            "failed to compile texture: `tex3ds` exited with status {code}: {}",
            String::from_utf8_lossy(&output.stderr),
        )
        .into());
    }

    let bytes = &std::fs::read(&out_path)
        .map_err(|err| format!("unable to read output file {out_path:?}: {err}"))?[4..];

    let source_file_path = texture_source_file.to_string_lossy();

    let result = quote! {
        {
            // ensure the source is re-evaluted if the input file changes
            const _SOURCE: &[u8] = include_bytes! ( #source_file_path );

            // https://users.rust-lang.org/t/can-i-conveniently-compile-bytes-into-a-rust-program-with-a-specific-alignment/24049/2
            #[repr(C)]
            struct AlignedAsU32<Bytes: ?Sized> {
                _align: [u32; 0],
                bytes: Bytes,
            }

            // this assignment is made possible by CoerceUnsized
            const ALIGNED: &AlignedAsU32<[u8]> = &AlignedAsU32 {
                _align: [],
                // emits a token stream like `[10u8, 11u8, ... ]`
                bytes: [ #(#bytes),* ]
            };

            &ALIGNED.bytes
        }
    };

    Ok(result.into())
}
