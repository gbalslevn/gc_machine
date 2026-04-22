use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, FnArg, Ident, ItemFn, Pat};

#[proc_macro_attribute]
pub fn circuit_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let result = std::panic::catch_unwind(|| inner_circuit_fn(item));
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

fn inner_circuit_fn(item: TokenStream) -> TokenStream {
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

    let circuit_body = lower_block(fn_body);

    // In inner_circuit_fn — the circuit twin
    quote! {
    #fn_vis fn #fn_name(#params) #ret_ty #fn_body

    #[doc(hidden)]
    #fn_vis fn #builder_name(
        cb: &mut gc_machine::circuit_builder::CircuitBuilder,  
        #g: Vec<gc_machine::circuit_builder::WireBuild>,       
        #e: Vec<gc_machine::circuit_builder::WireBuild>,      
    ) -> Vec<gc_machine::circuit_builder::WireBuild> {         
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

    quote! {{
        let mut __cb__ = CircuitBuilder::new();
        let (__g__, __e__) = __cb__.set_input_wires(1);
        #builder_name(&mut __cb__, __g__, __e__);
        __cb__.get_circuit_build()
    }}
    .into()
}

// ── AST lowering — no macro_rules! needed ────────────────────────────────────

fn lower_block(block: &syn::Block) -> proc_macro2::TokenStream {
    let mut out = proc_macro2::TokenStream::new();
    let last = block.stmts.len().saturating_sub(1);

    for (i, stmt) in block.stmts.iter().enumerate() {
        let is_last = i == last;
        match stmt {
            syn::Stmt::Local(local) => {
                let pat = &local.pat;
                if let Some(init) = &local.init {
                    let expr = lower_expr(&init.expr);
                    out.extend(quote! { let #pat = #expr; });
                }
            }
            syn::Stmt::Expr(expr, semi) => {
                let lowered = lower_expr(expr);
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

fn lower_expr(expr: &syn::Expr) -> proc_macro2::TokenStream {
    match expr {
        syn::Expr::Binary(bin) if matches!(bin.op, syn::BinOp::Add(_)) => {
            let l = lower_expr(&bin.left);
            let r = lower_expr(&bin.right);
            quote! { cb.build_adder(&#l, &#r) }
        }
        syn::Expr::Binary(bin) if matches!(bin.op, syn::BinOp::Mul(_)) => {
            let l = lower_expr(&bin.left);
            let r = lower_expr(&bin.right);
            quote! { cb.build_multiplier(&#l, &#r) }
        }
        syn::Expr::Binary(bin) if matches!(bin.op, syn::BinOp::Eq(_)) => {
            let l = lower_expr(&bin.left);
            let r = lower_expr(&bin.right);
            quote! { vec![cb.build_is_equal(&#l, &#r)] }
        }
        syn::Expr::If(expr_if) => {
            let cond = lower_expr(&expr_if.cond);
            let then_block = lower_block(&expr_if.then_branch);
            let else_block = expr_if
                .else_branch
                .as_ref()
                .map(|(_, e)| match e.as_ref() {
                    syn::Expr::Block(b) => lower_block(&b.block),
                    other => lower_expr(other),
                })
                .unwrap_or(quote! { vec![] });
            quote! {{
                let __cond__     = #cond;
                let __then_out__ = { #then_block };
                let __else_out__ = { #else_block };
                cb.build_if(&__cond__[0], &__then_out__, &__else_out__)
            }}
        }
        syn::Expr::Paren(p) => lower_expr(&p.expr),
        syn::Expr::Block(b) => lower_block(&b.block),
        other => quote! { #other },
    }
}
