use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, FnArg, Ident, ItemFn, Pat};

struct CircuitFnArgs {
    input_bits: u64,
    naive_stack: bool,
}

impl syn::parse::Parse for CircuitFnArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut input_bits = None;
        let mut naive_stack = false;

        while !input.is_empty() {
            let name: syn::Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let value: syn::Lit = input.parse()?;

            match name.to_string().as_str() {
                "input_bits" => {
                      if let syn::Lit::Int(i) = value {
                        input_bits = Some(i.base10_parse()?);
                    } else {
                        return Err(syn::Error::new(name.span(), "`input_bits` must be an integer"));
                    }
                },
                "naive_stack" => {
                    if let syn::Lit::Bool(b) = value {
                        naive_stack = b.value;
                    } else {
                        return Err(syn::Error::new(name.span(), "`naive_stack` must be a bool (true/false)"));
                    }
                }
                unknown => return Err(syn::Error::new(
                    name.span(),
                    format!("unknown argument `{}`", unknown)
                )),
            }

            // consume optional trailing comma
            if input.peek(syn::Token![,]) {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        if input_bits.is_none() {
            println!("Warning using default 16 bit input wires, which might create a lot of padding gates. To set specific input bits of x length do, #[circuit_fn(bits = x)]");
        }

        Ok(CircuitFnArgs {
            input_bits: input_bits.unwrap_or(16),
            naive_stack
        })
    }
}

#[proc_macro_attribute]
pub fn circuit_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as CircuitFnArgs);
    let result = std::panic::catch_unwind(|| inner_circuit_fn(item, args.input_bits, args.naive_stack));
    match result {
        Ok(ts) => ts,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                format!("circuit_fn panicked: {}", s)
            } else if let Some(s) = e.downcast_ref::<String>() {
                format!("circuit_fn panicked: {}", s)
            } else {
                "circuit_fn panicked with unknown error".to_string()
            };
            syn::Error::new(proc_macro2::Span::call_site(), msg)
                .to_compile_error()
                .into()
        }
    }
}

fn inner_circuit_fn(item: TokenStream, bits: u64, naive_stack: bool) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let params = &input.sig.inputs;
    // ── Validate exactly 2 parameters ────────────────────────────────────────
    if params.len() != 2 {
        return syn::Error::new(
            fn_name.span(),
            format!(
                "#[circuit_fn] requires exactly 2 parameters (garbler_input, evaluator_input), got {}",
                params.len()
            ),
        )
        .to_compile_error()
        .into();
    }
    let ret_ty = &input.sig.output;
    if ret_ty == &syn::ReturnType::Default {
        return syn::Error::new(
            fn_name.span(),
            "#[circuit_fn] requires an explicit return type, e.g. `-> Vec<u8>`",
        )
        .to_compile_error()
        .into();
    }
    let fn_body = &input.block;

    let param_idents: Vec<Ident> = params
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pt) = arg {
                if let Pat::Ident(pi) = pt.pat.as_ref() {
                    return Some(pi.ident.clone());
                }
            }
            None
        })
        .collect();

    if param_idents.len() < 2 {
        return syn::Error::new(
            fn_name.span(),
            "#[circuit_fn] requires exactly 2 parameters: (garbler, evaluator)",
        )
        .to_compile_error()
        .into();
    }

    let g = &param_idents[0];
    let e = &param_idents[1];

    let builder_name = Ident::new(&format!("__circuit_{}", fn_name), Span::call_site());

    let circuit_body = lower_block(fn_body, naive_stack);
    let bits_fn_name = Ident::new(&format!("__circuit_{}_bits", fn_name), Span::call_site());

    // In inner_circuit_fn — the circuit twin
    quote! {
        #fn_vis fn #fn_name(#params) #ret_ty #fn_body
        #[doc(hidden)]
        #fn_vis fn #bits_fn_name() -> u64 {
            #bits
        }

    #[doc(hidden)]
    #fn_vis fn #builder_name(
        cb: &mut gc_machine::circuit_builder::CircuitBuilder,
        #g: Vec<gc_machine::circuit_builder::WireBuild>,
        #e: Vec<gc_machine::circuit_builder::WireBuild>,
    ) -> gc_machine::circuit_builder::BuildBlock {
        use gc_machine::circuit_builder::AsWires as _;
        #circuit_body
    }}
    .into()
}

#[proc_macro]
pub fn circuit(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let tokens: Vec<_> = input2.into_iter().collect();

    let fn_name = match &tokens[..] {
        [proc_macro2::TokenTree::Ident(name), ..] => name.clone(),
        _ => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "circuit! expects: circuit! { add() }",
            )
            .to_compile_error()
            .into()
        }
    };

    let builder_name = Ident::new(&format!("__circuit_{}", fn_name), fn_name.span());
    let bits_fn_name = Ident::new(&format!("__circuit_{}_bits", fn_name), fn_name.span());

    quote! {{
        let mut __cb__ = CircuitBuilder::new();
        let (__g__, __e__) = __cb__.set_input_wires(#bits_fn_name());
        #builder_name(&mut __cb__, __g__, __e__);
        __cb__.get_circuit_build()
    }}
    .into()
}

// ── AST lowering — no macro_rules! needed ────────────────────────────────────

fn lower_block(block: &syn::Block, naive_stack: bool) -> proc_macro2::TokenStream {
    let mut out = proc_macro2::TokenStream::new();
    let last = block.stmts.len().saturating_sub(1);

    for (i, stmt) in block.stmts.iter().enumerate() {
        let is_last = i == last;
        match stmt {
            syn::Stmt::Local(local) => {
                let pat = &local.pat;
                if let Some(init) = &local.init {
                    let expr = lower_expr(&init.expr, naive_stack);
                    out.extend(quote! { let #pat = #expr; });
                }
            }
            syn::Stmt::Expr(expr, semi) => {
                let lowered = lower_expr(expr, naive_stack);
                if is_last && semi.is_none() {
                    out.extend(quote! { #lowered });
                } else {
                    out.extend(quote! { #lowered; });
                }
            }
            other => out.extend(quote! { #other }),
        }
    }
    out
}

fn lower_expr(expr: &syn::Expr, naive_stack: bool) -> proc_macro2::TokenStream {
    match expr {
        // variable
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Int(lit_int) => {
                quote! {
                    cb.build_variable({
                        use num_bigint::ToBigUint;
                        #lit_int.to_biguint().unwrap().to_bytes_le()
                    })
                }
            }
            _ => quote! { #expr_lit.clone() },
        },
        // addition
        syn::Expr::Binary(bin) if matches!(bin.op, syn::BinOp::Add(_)) => {
            let l = lower_expr(&bin.left, naive_stack);
            let r = lower_expr(&bin.right, naive_stack);
            quote! {{
            let __lhs__ = { #l };
            let __rhs__ = { #r };
            cb.build_adder(__lhs__.as_wires(), __rhs__.as_wires())
            }}
        }
        // multiplication
        syn::Expr::Binary(bin) if matches!(bin.op, syn::BinOp::Mul(_)) => {
            let l = lower_expr(&bin.left, naive_stack);
            let r = lower_expr(&bin.right, naive_stack);
            quote! {{
                let __lhs__ = { #l };
                let __rhs__ = { #r };
                cb.build_multiplier(__lhs__.as_wires(), __rhs__.as_wires())
            }}
        }
        // is_equal
        syn::Expr::Binary(bin) if matches!(bin.op, syn::BinOp::Eq(_)) => {
            let l = lower_expr(&bin.left, naive_stack);
            let r = lower_expr(&bin.right, naive_stack);
            quote! {{
                let __lhs__ = { #l };
                let __rhs__ = { #r };
                cb.build_is_equal(__lhs__.as_wires(), __rhs__.as_wires())
            }}
        }
        // if
        syn::Expr::If(expr_if) => {
            let cond = lower_expr(&expr_if.cond, naive_stack);
            let then_block = lower_block(&expr_if.then_branch, naive_stack);
            let else_block = expr_if
                .else_branch
                .as_ref()
                .map(|(_, e)| match e.as_ref() {
                    syn::Expr::Block(b) => lower_block(&b.block, naive_stack),
                    other => lower_expr(other, naive_stack),
                })
                .unwrap_or(quote! {
                    gc_machine::circuit_builder::BuildBlock { output: vec![], builds: vec![] }
                });

            let call = if naive_stack {
                quote! { cb.build_if(&__cond__, &__then__, &__else__) }
            } else {
                quote! { cb.build_stacked_if(&__cond__, &mut __else__, &mut __then__) }
            };

            quote! {{
                let __cond__: gc_machine::circuit_builder::WireBuild = { #cond }.as_wires()[0].clone();
                let mut __then__: gc_machine::circuit_builder::BuildBlock = { #then_block }.into();
                let mut __else__: gc_machine::circuit_builder::BuildBlock = { #else_block }.into();
                #call
            }}
        }
        syn::Expr::Paren(p) => lower_expr(&p.expr, naive_stack),
        syn::Expr::Block(b) => lower_block(&b.block, naive_stack),
        other => quote! { #other.clone() },
    }
}
