#![allow(clippy::redundant_closure_call)] //autogenerated parser code
use crate::parsing::ast::{EnumVariant, Expr, Stmt};
use crate::parsing::lexer::{Token, TokenKind};

macro_rules! t {
    ($e:pat) => {
        Token { kind: $e, .. }
    };
}

macro_rules! bin {
    ($op:expr, $a:expr, $b:expr) => {
        Expr::Binary($op, Box::new($a), Box::new($b))
    };
}

enum CallVariant {
    Normal(Vec<Expr>),
    Partial(Vec<Option<Expr>>),
    Property(Token),
    PropertyTest(Token),
}

enum AssignmentTarget {
    Variable(Token),
    Property(Expr),
}

peg::parser! {
    pub grammar program_parser() for [Token] {
        use TokenKind::*;
        pub rule program() -> Expr
            = block_expr()

        rule block() -> (Token, Token, Vec<Stmt>) =
            [bb@t!(BeginBlock)] [t!(LineEnd)]? s:stmt() ** [t!(LineEnd)] [t!(LineEnd)]? [be@t!(EndBlock)] {(bb, be, s)}


        rule stmt() -> Stmt =

             var_decl_stmt()
            / function_decl_stmt()
            / struct_decl_stmt()
            / enum_decl_stmt()
            / implementation_stmt()
            / assignment_stmt()
            / assert_stmt()
            / pass_stmt()
            / e:expr() {Stmt::Expression(e)}


        rule var_decl_stmt() -> Stmt =
            [t!(Var)] n:name() e:assignment_right_side()?
                {Stmt::VarDeclaration(n, e)}

        rule assignment_right_side() -> Expr =
            [t!(Equals)] e:expr() {e}

        rule function_decl_stmt() -> Stmt =
            [t!(Def)] n:name() args:maybe_arguments_and_equals() body:expr() {
                Stmt::FunctionDeclaration{name:n, args: args.0, vararg: args.1, body}
            }


        rule enum_decl_stmt() -> Stmt =
            [t!(Enum)] n:name() body:enum_body()? {
                Stmt::EnumDeclaration {
                    name: n,
                    variants: body.unwrap_or_default()
                 }
            }

        rule enum_body() -> Vec<EnumVariant> =
            [t!(Colon)] [bb@t!(BeginBlock)] [t!(LineEnd)]? v:enum_variant() ** [t!(LineEnd)] [t!(LineEnd)]? [be@t!(EndBlock)]{
                v
            }

        rule enum_variant() -> EnumVariant =
            b: struct_no_token() {
                EnumVariant {
                    name: b.0,
                    fields: b.1
                }
            }

        rule struct_decl_stmt() -> Stmt =
            [t!(Struct)] b:struct_no_token() {
                Stmt::StructDeclaration {
                    name: b.0,
                    fields: b.1
                }
            }

        rule struct_no_token() -> (Token, Vec<Token>) =
            n:name() body: struct_body()? {
                match body {
                    Some(entries) => {
                    (n, entries)

                    }
                    None => (n, vec![])
                }
            }



        rule struct_body() -> Vec<Token> =
            [t!(Colon)] n:struct_entry() {
                n
            }

            rule struct_entry() -> Vec<Token> =
                [bb@t!(BeginBlock)] [t!(LineEnd)]? n:name() ** [t!(LineEnd)] [t!(LineEnd)]? [be@t!(EndBlock)] {
                n
            }

        rule implementation_stmt() -> Stmt =
            [t!(Impl)] n: name() [t!(Colon)] methods: impl_block() {
                Stmt::ImplBlock {
                    name: n,
                    implementations: methods,
                }
            }

        rule impl_block() -> Vec<Stmt> =
            [bb@t!(BeginBlock)] [t!(LineEnd)]? m:function_decl_stmt() ** [t!(LineEnd)] [t!(LineEnd)]? [be@t!(EndBlock)] {
                m
            }

        rule paren_name_list() -> (Vec<Token>, Option<Token>) =

            [t!(LParen)] n:name()**[t!(Comma)] [t!(Comma)] v:vararg() [t!(Comma)]?  [t!(RParen)] {(n, Some(v))}
            /
            [t!(LParen)] n:name()**[t!(Comma)] [t!(Comma)]?  [t!(RParen)] {(n, None)}
            /
            [t!(LParen)] v:vararg() [t!(Comma)]? [t!(RParen)] {(vec![], Some(v))}

        rule vararg() -> Token =
            [t!(Star)] n: name() {n}

        rule maybe_arguments_and_equals() -> (Vec<Token>, Option<Token>) =
            n:paren_name_list() [t!(Equals)] {
                n
            }
            / [t!(Equals)] {(Vec::new(), None)}

        rule assignment_stmt() -> Stmt =
            target:assignment_target() [t!(Equals)] e:expr() {
                match target {
                    AssignmentTarget::Property(target) => {
                        Stmt::PropertyAssignment(
                            target, e)
                    }
                    AssignmentTarget::Variable(v) => {
                        Stmt::Assignment(v, e)
                    }
                }

                }

        rule assignment_target() -> AssignmentTarget =
        prop: property_access() {
            AssignmentTarget::Property(prop)
        }
        /

        n:name() {
            AssignmentTarget::Variable(n)
        }


        rule assert_stmt() -> Stmt =
            [a@t!(Assert)] e:expr() {Stmt::Assert(a, e)}

        rule if_expr() -> Expr =
            if_elif_else()/ if_elif() / if_then()

        rule if_elif() -> Expr =
            [t!(If)] cond:simple_expr() then:expr() [t!(LineEnd)]? elif:elif_body()+
                {
                    let mut last_if_cond = None;
                    for (cond, body) in elif.into_iter().rev() {
                        last_if_cond = Some(Box::new(Expr::If(Box::new(cond), Box::new(body), last_if_cond)));
                    }
                    Expr::If(Box::new(cond), Box::new(then), last_if_cond)
                }

        rule if_then() -> Expr =
            [t!(If)] cond:simple_expr() then:expr()
                {Expr::If(Box::new(cond), Box::new(then), None)}

        rule if_elif_else() -> Expr =
            [t!(If)] cond:simple_expr() then:expr() [t!(LineEnd)]? elif:elif_body()* [t!(Else)] else_body:expr()
                {
                    let mut last_if_cond = Some(Box::new(else_body));
                    for (cond, body) in elif.into_iter().rev() {
                        last_if_cond = Some(Box::new(Expr::If(Box::new(cond), Box::new(body), last_if_cond)));
                    }

                    Expr::If(Box::new(cond), Box::new(then), last_if_cond)}

        rule elif_body() -> (Expr, Expr) =
            [t!(ELif)] elif_cond:simple_expr() elif_body: expr() [t!(LineEnd)]? {
                (elif_cond, elif_body)
            }

        rule pass_stmt() -> Stmt =
            [t@t!(Pass)] {Stmt::Pass(t)}

        rule expr() -> Expr =
            block_expr() /
            if_expr() /
            simple_expr()

        rule block_expr() -> Expr =
            b:block() {Expr::Block(b.0, b.1, b.2)}

        rule simple_expr() -> Expr =
            arrow() /
            arithmetic()

        rule arrow() -> Expr =
            p:paren_name_list() [t@t!(Arrow)] b:simple_expr() {
            Expr::AnonFunction(p.0, p.1, t, Box::new(b))
        }

        rule arithmetic() -> Expr = precedence! {
            x: (@) [op@t!(Or)] y:@
                {bin!(op, x, y)}
            --
            x: (@) [op@t!(And)] y: @
                {bin!(op, x, y)}
            --
            [op@t!(Not)] x: @
                {
                    Expr::Unary(op, Box::new(x))
                }

            --
            x: (@) [op@t!(CompareEquals)] y:@
                {bin!(op, x, y)}
            x: (@) [op@t!(CompareNotEquals)] y:@
                {bin!(op, x, y)}
            --
            x: (@) [op@t!(CompareGreater)] y:@
                {bin!(op, x, y)}
            x: (@) [op@t!(CompareGreaterEqual)] y:@
                {bin!(op, x, y)}
            x: (@) [op@t!(CompareLess)] y:@
                {bin!(op, x, y)}
            x: (@) [op@t!(CompareLessEqual)] y:@
                {bin!(op, x, y)}

            --
            x: (@) [op@t!(Plus)] y:@
                {bin!(op, x, y)}
            x: (@) [op@t!(Minus)] y:@
                {bin!(op, x, y)}
            --
            x: (@) [op@t!(Star)] y:@
                {bin!(op, x, y)}
            x: (@) [op@t!(Slash)] y:@
                {bin!(op, x, y)}
            x: (@) [op@t!(Mod)] y:@
                {bin!(op, x, y)}
            --
            x:@ [op@t!(Power)] y:(@)
                {bin!(op, x, y)}
            --
            n:call() {n}
        }

        rule property_access() -> Expr =
            target: call() {?
                match target {
                    e @ Expr::PropertyAccess(..) => Ok(e),
                    _ => Err("property accesss or variable")
                }
            }

        rule call() -> Expr =
            target:term() calls:call_right_side()* {
                let mut res = target;
                for parens in calls {
                match parens {
                    CallVariant::Normal(args) => {
                        res = Expr::Call(Box::new(res), args)
                    }
                    CallVariant::Partial(args) => {
                        res = Expr::PartialCall(Box::new(res), args)
                    }
                    CallVariant::Property(prop) => {
                        res = Expr::PropertyAccess(Box::new(res), prop)
                    }

                    CallVariant::PropertyTest(prop) => {
                        res = Expr::PropertyTest(Box::new(res), prop)
                    }
                }
            }
                res
            }
            / term()

        rule call_right_side() -> CallVariant =
            call_property_access()
            / call_parens()

        rule call_parens() -> CallVariant =
            [t!(LParen)] args:simple_expr()**[t!(Comma)] [t!(Comma)]? [t!(RParen)] {CallVariant::Normal(args)}
        / [t!(LParen)] args:maybe_argument()**[t!(Comma)] [t!(Comma)]? [t!(RParen)] {
            CallVariant::Partial(args)
        }

        rule maybe_argument() -> Option<Expr> =
            e:simple_expr() {Some(e)}
        / [t!(Blank)] {None}

        rule call_property_access() -> CallVariant =
            [t!(Dot)] property_name: name() {CallVariant::Property(property_name)}
        /
            [t!(QuestionMark)] property_name: name() {CallVariant::PropertyTest(property_name)}

        rule term() -> Expr
            = [num@t!(Number(..))] {Expr::Number(num)}
            / t:name()
                {Expr::Name(t)}
            / [s@t!(ConstString(..))] {Expr::ConstString(s)}
            / [t!(LParen)] e:expr() [t!(RParen)] {e}



        rule name() -> Token
            = [t@Token{kind:TokenKind::Name(..), position:pos}] {t}
    }
}
