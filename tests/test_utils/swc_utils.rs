extern crate swc_common;
extern crate swc_ecma_ast;

use swc_common::DUMMY_SP;
use swc_ecma_ast::{BinExpr, BinaryOp, Expr, Ident, MemberExpr, MemberProp};

/// Generate an ident.
///
/// ```js
/// a
/// ```
pub fn create_ident(sym: &str) -> Ident {
    Ident {
        sym: sym.into(),
        optional: false,
        span: DUMMY_SP,
    }
}

/// Generate an ident expression.
///
/// ```js
/// a
/// ```
pub fn create_ident_expression(sym: &str) -> Expr {
    Expr::Ident(create_ident(sym))
}

/// Generate a binary expression.
///
/// ```js
/// a + b + c
/// a || b
/// ```
pub fn create_binary_expression(mut exprs: Vec<Expr>, op: BinaryOp) -> Expr {
    exprs.reverse();

    let mut left = None;

    while let Some(right_expr) = exprs.pop() {
        left = Some(if let Some(left_expr) = left {
            Expr::Bin(BinExpr {
                left: Box::new(left_expr),
                right: Box::new(right_expr),
                op,
                span: swc_common::DUMMY_SP,
            })
        } else {
            right_expr
        });
    }

    left.expect("expected one or more expressions")
}

/// Generate a member expression.
///
/// ```js
/// a.b
/// a
/// ```
pub fn create_member_expression(name: &str) -> Expr {
    let bytes = name.as_bytes();
    let mut index = 0;
    let mut start = 0;
    let mut parts = vec![];

    while index < bytes.len() {
        if bytes[index] == b'.' {
            parts.push(&name[start..index]);
            start = index + 1;
        }

        index += 1;
    }

    if parts.len() > 1 {
        let mut member = MemberExpr {
            obj: Box::new(create_ident_expression(parts[0])),
            prop: MemberProp::Ident(create_ident(parts[1])),
            span: swc_common::DUMMY_SP,
        };
        let mut index = 2;
        while index < parts.len() {
            member = MemberExpr {
                obj: Box::new(Expr::Member(member)),
                prop: MemberProp::Ident(create_ident(parts[1])),
                span: swc_common::DUMMY_SP,
            };
            index += 1;
        }
        Expr::Member(member)
    } else {
        create_ident_expression(name)
    }
}
