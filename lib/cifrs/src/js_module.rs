use std::path::Path;
/// This contains a set of helper functions for traversing JS program modules
// TODO (Sean): I really dislike this implementation. There needs to be a better way
// SWC might have something built in for traversing their AST or could likely implement Visitor
// Can also look at OXC (https://github.com/web-infra-dev/oxc) as well instead of SWC
use std::sync::Arc;

use swc::{HandlerOpts, try_with_handler};
use swc_common::sync::Lrc;
use swc_common::{FileName, GLOBALS, SourceMap};
use swc_ecma_ast::Stmt::{Decl as DeclStmt, Expr as ExprStmt};
use swc_ecma_ast::{
    ArrayLit, AssignExpr, BlockStmt, CallExpr, Decl, EsVersion, Expr, ExprOrSpread, FnExpr,
    Function, KeyValueProp, Lit, Module, ModuleDecl, ModuleItem, ObjectLit, Program, Prop,
    PropOrSpread, Stmt, TplElement, VarDeclarator,
};
use swc_ecma_parser::{EsConfig, EsSyntax, Syntax};

use crate::CifrsResult;

// TODO: maybe create some traits for these
// - identification - is ident this name
// - property - get properties / get property by key or key/value
// - find methods?

// variable = expression

pub(crate) trait PropertyAccessor<'a> {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp>;

    fn get_property_with_value(
        &'a self,
        ident: &'static str,
        val: &'static str,
    ) -> Option<&'a KeyValueProp> {
        if let Some(prop) = self.get_property(ident) {
            if let Some(kv_val) = get_value_from_kv_as_string(prop) {
                if kv_val == val {
                    return Some(prop);
                }
            }
        }
        None
    }

    fn get_property_as_array(&'a self, ident: &'static str) -> Option<&'a ArrayLit> {
        self.get_property(ident)
            .and_then(|prop| prop.value.as_array())
    }

    fn get_property_as_obj(&'a self, ident: &'static str) -> Option<&'a ObjectLit> {
        self.get_property(ident)
            .and_then(|prop| prop.value.as_object())
    }

    fn get_property_as_string(&'a self, ident: &'static str) -> Option<String> {
        self.get_property(ident)
            .and_then(get_value_from_kv_as_string)
    }
}

impl<'a> PropertyAccessor<'a> for Module {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        self.body.iter().find_map(|item| item.get_property(ident))
    }
}

impl<'a> PropertyAccessor<'a> for ModuleItem {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        match self {
            ModuleItem::ModuleDecl(m) => m.get_property(ident),
            ModuleItem::Stmt(s) => s.get_property(ident),
        }
    }
}

impl<'a> PropertyAccessor<'a> for ModuleDecl {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        match self {
            // TODO: look at ExportDecl and ExportDefaultDecl
            ModuleDecl::ExportDefaultExpr(e) => e.expr.get_property(ident),
            _ => None,
        }
    }
}

impl<'a> PropertyAccessor<'a> for Stmt {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        match self {
            Stmt::Return(r) => r.arg.as_ref().and_then(|expr| expr.get_property(ident)),
            DeclStmt(d) => d.get_property(ident),
            Stmt::Expr(e) => e.expr.get_property(ident),
            _ => None,
        }
    }
}

impl<'a> PropertyAccessor<'a> for Decl {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        match self {
            // TODO: might need to do Decl::Fn
            Decl::Var(v) => v
                .decls
                .iter()
                .find_map(|declarator| declarator.get_property(ident)),
            _ => None,
        }
    }
}

impl<'a> PropertyAccessor<'a> for VarDeclarator {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        self.init.as_ref().and_then(|expr| expr.get_property(ident))
    }
}

impl<'a> PropertyAccessor<'a> for ExprOrSpread {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        self.expr.get_property(ident)
    }
}

impl<'a> PropertyAccessor<'a> for Expr {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        match self {
            Expr::Array(a) => a.get_property(ident),
            Expr::Assign(a) => a.get_property(ident),
            Expr::Call(c) => c.get_property(ident),
            Expr::Fn(f) => f.get_property(ident),
            Expr::Object(o) => o.get_property(ident),
            _ => None,
        }
    }
}

impl<'a> PropertyAccessor<'a> for AssignExpr {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        self.right.get_property(ident)
    }
}

impl<'a> PropertyAccessor<'a> for CallExpr {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        // TODO: this is only if we need to check ident
        if let Some(callee) = self.callee.as_expr() {
            let callee_prop = callee.get_property(ident);
            if callee_prop.is_some() {
                return callee_prop;
            }
        }

        self.args.iter().find_map(|expr| expr.get_property(ident))
    }
}

impl<'a> PropertyAccessor<'a> for FnExpr {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        self.function
            .body
            .as_ref()
            .and_then(|body| body.get_property(ident))
    }
}

impl<'a> PropertyAccessor<'a> for BlockStmt {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        self.stmts.iter().find_map(|ps| ps.get_property(ident))
    }
}

impl<'a> PropertyAccessor<'a> for ArrayLit {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        // TODO: see how this performs against the for loop
        self.elems
            .iter()
            .filter_map(|e| e.as_ref())
            .find_map(|expr| expr.get_property(ident))
        // for elem in &self.elems {
        //     if let Some(exp) = elem {
        //         let prop = exp.get_property(ident);
        //         if prop.is_some() {
        //             return prop
        //         }
        //     }
        // }
        // None
    }
}

impl<'a> PropertyAccessor<'a> for ObjectLit {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        self.props.iter().find_map(|ps| ps.get_property(ident))
    }
}

impl<'a> PropertyAccessor<'a> for PropOrSpread {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        return match self {
            PropOrSpread::Spread(_) => None,
            PropOrSpread::Prop(p) => p.as_key_value().and_then(|kv| kv.get_property(ident)),
        };
    }
}

impl<'a> PropertyAccessor<'a> for KeyValueProp {
    fn get_property(&'a self, ident: &'static str) -> Option<&'a KeyValueProp> {
        if self
            .key
            .as_ident()
            .filter(|kv_ident| kv_ident.sym.as_ref() == ident)
            .is_some()
        {
            return Some(self);
        }

        self.value.get_property(ident)
    }
}

pub fn parse_js_module<P: AsRef<Path>>(filename: P, src: String) -> CifrsResult<Program> {
    let cm = Arc::<SourceMap>::default();
    let c = swc::Compiler::new(cm.clone());
    let output = GLOBALS.set(&Default::default(), || {
        try_with_handler(
            cm.clone(),
            HandlerOpts {
                ..Default::default()
            },
            |handler| {
                let fm = cm.new_source_file(Lrc::new(filename.as_ref().to_owned().into()), src);
                c.parse_js(
                    fm,
                    handler,
                    EsVersion::Es2020,
                    Syntax::Es(EsSyntax::default()),
                    swc::config::IsModule::Bool(true),
                    None,
                )
            },
        )
    })?;

    Ok(output)
}

pub(crate) fn get_variable_declaration<'a>(
    program: &'a Program,
    ident: &'static str,
) -> Option<&'a VarDeclarator> {
    if let Some(module) = program.as_module() {
        for item in &module.body {
            if let Some(DeclStmt(decl)) = item.as_stmt() {
                if let Some(variable_decl) = decl.as_var() {
                    for declaration in &variable_decl.decls {
                        if let Some(decl_ident) = declaration.name.as_ident() {
                            if decl_ident.sym.as_ref() == ident {
                                return Some(declaration);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_call_expression<'a>(
    program: &'a Program,
    ident: &'static str,
) -> Option<&'a CallExpr> {
    if let Some(module) = program.as_module() {
        for item in &module.body {
            if let Some(ModuleDecl::ExportDefaultExpr(e)) = item.as_module_decl() {
                let expression = &*e.expr;
                if let Some(call) = expression.as_call() {
                    if let Some(callee) = call.callee.as_expr() {
                        if let Some(callee_ident) = callee.as_ident() {
                            if callee_ident.sym.as_ref() == ident {
                                return Some(call);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_assignment_function(program: &Program) -> Option<&Function> {
    if let Some(module) = program.as_module() {
        for item in &module.body {
            if let Some(ExprStmt(stmt)) = item.as_stmt() {
                let expression = &*stmt.expr;
                if let Some(assign) = expression.as_assign() {
                    let rhs = &*assign.right;
                    if let Some(func) = rhs.as_fn_expr() {
                        return Some(&*func.function);
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_assignment_obj(program: &Program) -> Option<&ObjectLit> {
    if let Some(module) = program.as_module() {
        for item in &module.body {
            if let Some(ExprStmt(stmt)) = item.as_stmt() {
                let expression = &*stmt.expr;
                if let Some(assign) = expression.as_assign() {
                    let rhs = &*assign.right;
                    let obj = match rhs {
                        Expr::Object(o) => Some(o),
                        Expr::Fn(f) => get_function_return_obj(&f.function),
                        _ => None,
                    };
                    if obj.is_some() {
                        return obj;
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_function_return_obj(func: &Function) -> Option<&ObjectLit> {
    if let Some(fn_body) = &func.body {
        for statement in &fn_body.stmts {
            if let Some(return_statement) = statement.as_return_stmt() {
                if let Some(return_statement_args) = &return_statement.arg {
                    let args = &**return_statement_args;
                    return args.as_object();
                }
            }
        }
    }

    None
}

pub(crate) fn get_call_string_property(call: &CallExpr, property: &'static str) -> Option<String> {
    for call_args in &call.args {
        let call_args_expression = &*call_args.expr;
        if let Some(obj) = call_args_expression.as_object() {
            return get_string_property_value(&obj.props, property);
        }
    }

    None
}

// TODO: can this be generic
// tuple enum variants (and also tuple structs) can be called like functions, so they implement Fn traits and can be passed where a function is expected.
// where F: 'static + Fn(String) -> Message
pub(crate) fn get_obj_property<'a>(
    obj: &'a ObjectLit,
    property_ident: &'static str,
) -> Option<&'a ObjectLit> {
    for prop in &obj.props {
        if let Some(p) = prop.as_prop() {
            if let Some(kv) = (*p).as_key_value() {
                if let Some(key_ident) = kv.key.as_ident() {
                    if key_ident.sym.as_ref() == property_ident {
                        return kv.value.as_object();
                    }
                }
            }
        }
    }

    None
}

pub(crate) fn get_array_property<'a>(
    obj: &'a ObjectLit,
    property_ident: &'static str,
) -> Option<&'a ArrayLit> {
    for prop in &obj.props {
        if let Some(p) = prop.as_prop() {
            if let Some(kv) = (*p).as_key_value() {
                if let Some(key_ident) = kv.key.as_ident() {
                    if key_ident.sym.as_ref() == property_ident {
                        return kv.value.as_array();
                    }
                }
            }
        }
    }

    None
}

pub(crate) fn find_array_element<'a>(
    arr: &'a ArrayLit,
    property_ident: &'static str,
    val: &'static str,
) -> Option<&'a ObjectLit> {
    for exp_or_spread in arr.elems.iter().flatten() {
        if let Some(o) = exp_or_spread.expr.as_object() {
            for op in &o.props {
                if let Some(op2) = op.as_prop() {
                    if let Some(kv) = op2.as_key_value() {
                        if let Some(ident) = kv.key.as_ident() {
                            if ident.sym.as_ref() == property_ident {
                                if let Some(kv_val) = get_value_from_kv_as_string(kv) {
                                    if kv_val == val {
                                        return Some(o);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn is_call_ident(call: &CallExpr, ident: &'static str) -> bool {
    if let Some(callee) = call.callee.as_expr() {
        if let Some(callee_ident) = callee.as_ident() {
            return callee_ident.sym.as_ref() == ident;
        }
    }
    false
}

// TODO: add method that just gets specific string via get_string_property_value
// TODO: could also do one that takes in a struct and a property
// could even go one further and do vec of object keys, with property, to do a recursive call to get to property
pub(crate) fn get_variable_properties<'a>(
    variable: &'a VarDeclarator,
    property: &'static str,
) -> Option<&'a Vec<PropOrSpread>> {
    if let Some(init_decl) = &variable.init {
        if let Some(init_decl_obj) = init_decl.as_object() {
            for prop_spread in &init_decl_obj.props {
                if let Some(prop) = prop_spread.as_prop() {
                    if let Some(kv) = (*prop).as_key_value() {
                        if let Some(ident) = kv.key.as_ident() {
                            if ident.sym.as_ref() == property {
                                if let Some(obj) = kv.value.as_object() {
                                    return Some(&obj.props);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn get_variable_property_as_string(
    variable: &VarDeclarator,
    property: &'static str,
) -> Option<String> {
    if let Some(init_decl) = &variable.init {
        if let Some(init_decl_obj) = init_decl.as_object() {
            return get_string_property_value(&init_decl_obj.props, property);
        }
    }
    None
}

pub(crate) fn get_string_property_value(
    properties: &Vec<PropOrSpread>,
    key: &'static str,
) -> Option<String> {
    for prop_spread in properties {
        if let Some(prop) = prop_spread.as_prop() {
            if let Some(kv) = (*prop).as_key_value() {
                if let Some(ident) = kv.key.as_ident() {
                    if ident.sym.as_ref() == key {
                        let val = get_value_from_kv_as_string(kv);
                        if val.is_some() {
                            return val;
                        }
                    }
                }
            }
        }
    }

    None
}

pub(crate) fn get_string_from_property(prop: &Prop) -> Option<String> {
    if let Some(kv) = (*prop).as_key_value() {
        return get_value_from_kv_as_string(kv);
    }

    None
}

pub(crate) fn get_value_from_kv_as_string(kv: &KeyValueProp) -> Option<String> {
    return match &*kv.value {
        Expr::Lit(l) => match l {
            Lit::Str(v) => Some(v.value.to_string()),
            Lit::Bool(v) => Some(v.value.to_string()),
            Lit::Null(_) => None,
            Lit::Num(v) => Some(v.to_string()),
            Lit::BigInt(v) => Some(v.value.to_string()),
            Lit::Regex(v) => Some(v.exp.to_string()),
            Lit::JSXText(v) => Some(v.value.to_string()),
        },
        Expr::Tpl(tpl) => {
            return if let Some(TplElement {
                cooked: Some(atom), ..
            }) = &tpl.quasis.first()
            {
                Some(atom.to_string())
            } else {
                None
            };
        }
        _ => None,
    };
}
