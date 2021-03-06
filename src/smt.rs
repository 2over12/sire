use crate::interpreter::{Expr, FuncDef, Ty, Value};
use rustc::mir::BinOp;

pub trait ToSmt {
    fn to_smt(&self) -> String;
}

impl ToSmt for FuncDef {
    fn to_smt(&self) -> String {
        let body = self.body.to_smt();
        let mut args_vec = match &self.ty {
            Ty::Func(vec) => vec.clone(),
            _ => unreachable!(),
        };

        let ret_ty = args_vec.remove(0).to_smt();

        let mut args = String::new();
        let mut args_with_ty = String::new();
        let mut args_ty = String::new();

        for (i, ty) in args_vec.iter().enumerate() {
            let smt_ty = ty.to_smt();
            args += &format!("x{} ", i + 1);
            args_with_ty += &format!("(x{} {}) ", i + 1, smt_ty);
            args_ty += &format!("{} ", smt_ty);
        }

        args.pop().unwrap();
        args_with_ty.pop().unwrap();
        args_ty.pop().unwrap();

        format!("(declare-fun {name} ({args_ty}) {ret_ty})\n(assert (forall ({args_with_ty}) (= ({name} {args}) {body})))\n", name = self.name, ret_ty = ret_ty, args = args, args_ty = args_ty, args_with_ty = args_with_ty, body = body)
    }
}

impl ToSmt for Ty {
    fn to_smt(&self) -> String {
        format!("(_ BitVec {})", self.size().unwrap())
    }
}

impl ToSmt for Value {
    fn to_smt(&self) -> String {
        match self {
            Value::Arg(n, _) => format!("x{}", n),
            Value::Const(b, ty) => match ty {
                Ty::Bool => format!("{}", *b != 0),
                ty => format!("(_ bv{} {})", b, ty.size().unwrap()),
            },
            Value::Function(n, _) => n.to_string(),
        }
    }
}

impl ToSmt for Expr {
    fn to_smt(&self) -> String {
        match self {
            Expr::Value(value) => value.to_smt(),
            Expr::BinaryOp(op, e1, e2) => {
                let smt_op = match e1.ty() {
                    Ty::Bool => match op {
                        BinOp::Eq => "=",
                        BinOp::Ne => "!=",
                        _ => unreachable!(),
                    },
                    Ty::Int(_) => match op {
                        BinOp::Add => "bvadd",
                        BinOp::Sub => "bvsub",
                        BinOp::Mul => "bvmul",
                        BinOp::Div => "bvsdiv",
                        BinOp::Rem => "bvsrem",
                        BinOp::Eq => "=",
                        BinOp::Lt => "bvslt",
                        BinOp::Le => "bvsle",
                        BinOp::Ne => "!=",
                        BinOp::Ge => "bvsge",
                        BinOp::Gt => "bvsgt",
                        _ => unreachable!(),
                    },
                    Ty::Uint(_) => match op {
                        BinOp::Add => "bvadd",
                        BinOp::Sub => "bvsub",
                        BinOp::Mul => "bvmul",
                        BinOp::Div => "bvudiv",
                        BinOp::Rem => "bvurem",
                        BinOp::Eq => "=",
                        BinOp::Lt => "bvult",
                        BinOp::Le => "bvule",
                        BinOp::Ne => "!=",
                        BinOp::Ge => "bvuge",
                        BinOp::Gt => "bvugt",
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                };
                format!("({} {} {})", smt_op, e1.to_smt(), e2.to_smt())
            }
            Expr::Apply(f, es) => format!(
                "({} {})",
                f.to_smt(),
                es.iter().map(|e| e.to_smt()).collect::<Vec<_>>().join(" ")
            ),
            Expr::Switch(val, cs, bs) => {
                println!("{:?}: {:?}", val, val.ty());
                if val.ty() == Ty::Bool {
                    format!(
                        "(ite {} {} {})",
                        val.to_smt(),
                        bs[1].to_smt(),
                        bs[0].to_smt()
                    )
                } else {
                    let mut cond = bs.last().unwrap().to_smt();
                    for i in (0..cs.len()).rev() {
                        cond = format!(
                            "(ite (= {} {}) {} {})",
                            val.to_smt(),
                            cs[i].to_smt(),
                            bs[i].to_smt(),
                            cond
                        );
                    }
                    cond
                }
            }
            _ => unimplemented!(),
        }
    }
}
