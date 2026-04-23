// tinyexpr-rs: C→Rust translation of tinyexpr
// Reference: ../tinyexpr-c/tinyexpr.c (do not modify)

// ── Native function pointer enums ─────────────────────────────────────────────

/// Pure native functions (no closure context), arity 0–7.
#[derive(Clone, Copy)]
pub enum NativeFn {
    F0(fn() -> f64),
    F1(fn(f64) -> f64),
    F2(fn(f64, f64) -> f64),
    F3(fn(f64, f64, f64) -> f64),
    F4(fn(f64, f64, f64, f64) -> f64),
    F5(fn(f64, f64, f64, f64, f64) -> f64),
    F6(fn(f64, f64, f64, f64, f64, f64) -> f64),
    F7(fn(f64, f64, f64, f64, f64, f64, f64) -> f64),
}

/// Closure functions — first arg is context pointer, arity 0–7.
#[derive(Clone, Copy)]
pub enum NativeClosure {
    C0(fn(*mut ()) -> f64),
    C1(fn(*mut (), f64) -> f64),
    C2(fn(*mut (), f64, f64) -> f64),
    C3(fn(*mut (), f64, f64, f64) -> f64),
    C4(fn(*mut (), f64, f64, f64, f64) -> f64),
    C5(fn(*mut (), f64, f64, f64, f64, f64) -> f64),
    C6(fn(*mut (), f64, f64, f64, f64, f64, f64) -> f64),
    C7(fn(*mut (), f64, f64, f64, f64, f64, f64, f64) -> f64),
}

// ── Arithmetic operators (named so parser can compare by pointer identity) ────

fn op_add(a: f64, b: f64) -> f64 {
    a + b
}
fn op_sub(a: f64, b: f64) -> f64 {
    a - b
}
fn op_mul(a: f64, b: f64) -> f64 {
    a * b
}
fn op_div(a: f64, b: f64) -> f64 {
    a / b
}
fn op_pow(a: f64, b: f64) -> f64 {
    a.powf(b)
}
fn op_fmod(a: f64, b: f64) -> f64 {
    a % b
}
fn op_negate(a: f64) -> f64 {
    -a
}
fn op_comma(_a: f64, b: f64) -> f64 {
    b
}

impl NativeFn {
    fn arity(self) -> usize {
        match self {
            NativeFn::F0(_) => 0,
            NativeFn::F1(_) => 1,
            NativeFn::F2(_) => 2,
            NativeFn::F3(_) => 3,
            NativeFn::F4(_) => 4,
            NativeFn::F5(_) => 5,
            NativeFn::F6(_) => 6,
            NativeFn::F7(_) => 7,
        }
    }
}

impl NativeClosure {
    fn arity(self) -> usize {
        match self {
            NativeClosure::C0(_) => 0,
            NativeClosure::C1(_) => 1,
            NativeClosure::C2(_) => 2,
            NativeClosure::C3(_) => 3,
            NativeClosure::C4(_) => 4,
            NativeClosure::C5(_) => 5,
            NativeClosure::C6(_) => 6,
            NativeClosure::C7(_) => 7,
        }
    }
}

// ── AST node ──────────────────────────────────────────────────────────────────

/// Expression tree node. Corresponds to C's `te_expr`.
///
/// # Safety
/// `Variable` holds a raw pointer that must remain valid for the lifetime of
/// the `Expr` tree. Closure `context` pointers carry the same requirement.
pub enum Expr {
    /// A literal floating-point constant.
    Constant(f64),

    /// A bound variable. The pointer must outlive this node (user's responsibility).
    Variable(*const f64),

    /// A pure or impure native function call.
    Function {
        pure: bool,
        func: NativeFn,
        args: Vec<Box<Expr>>,
    },

    /// A closure call — carries an opaque context pointer.
    Closure {
        pure: bool,
        func: NativeClosure,
        /// # Safety: must outlive the Expr tree.
        context: *mut (),
        args: Vec<Box<Expr>>,
    },
}

// ── Built-in math helpers ─────────────────────────────────────────────────────

fn te_fac(a: f64) -> f64 {
    if a < 0.0 {
        return f64::NAN;
    }
    if a > u32::MAX as f64 {
        return f64::INFINITY;
    }
    let ua = a as u64;
    let mut result: u64 = 1;
    for i in 1..=ua {
        if i > u64::MAX / result {
            return f64::INFINITY;
        }
        result *= i;
    }
    result as f64
}

fn te_ncr(n: f64, r: f64) -> f64 {
    if n < 0.0 || r < 0.0 || n < r {
        return f64::NAN;
    }
    if n > u32::MAX as f64 || r > u32::MAX as f64 {
        return f64::INFINITY;
    }
    let un = n as u64;
    let mut ur = r as u64;
    let mut result: u64 = 1;
    if ur > un / 2 {
        ur = un - ur;
    }
    for i in 1..=ur {
        if result > u64::MAX / (un - ur + i) {
            return f64::INFINITY;
        }
        result *= un - ur + i;
        result /= i;
    }
    result as f64
}

fn te_npr(n: f64, r: f64) -> f64 {
    te_ncr(n, r) * te_fac(r)
}

fn te_pi() -> f64 {
    std::f64::consts::PI
}

fn te_e() -> f64 {
    std::f64::consts::E
}

// ── Built-in function table ───────────────────────────────────────────────────

struct BuiltinEntry {
    name: &'static str,
    func: NativeFn,
    pure: bool,
}

/// Sorted alphabetically — required for binary search in find_builtin.
static BUILTINS: &[BuiltinEntry] = &[
    BuiltinEntry {
        name: "abs",
        func: NativeFn::F1(|a| a.abs()),
        pure: true,
    },
    BuiltinEntry {
        name: "acos",
        func: NativeFn::F1(|a| a.acos()),
        pure: true,
    },
    BuiltinEntry {
        name: "asin",
        func: NativeFn::F1(|a| a.asin()),
        pure: true,
    },
    BuiltinEntry {
        name: "atan",
        func: NativeFn::F1(|a| a.atan()),
        pure: true,
    },
    BuiltinEntry {
        name: "atan2",
        func: NativeFn::F2(|a, b| a.atan2(b)),
        pure: true,
    },
    BuiltinEntry {
        name: "ceil",
        func: NativeFn::F1(|a| a.ceil()),
        pure: true,
    },
    BuiltinEntry {
        name: "cos",
        func: NativeFn::F1(|a| a.cos()),
        pure: true,
    },
    BuiltinEntry {
        name: "cosh",
        func: NativeFn::F1(|a| a.cosh()),
        pure: true,
    },
    BuiltinEntry {
        name: "e",
        func: NativeFn::F0(te_e),
        pure: true,
    },
    BuiltinEntry {
        name: "exp",
        func: NativeFn::F1(|a| a.exp()),
        pure: true,
    },
    BuiltinEntry {
        name: "fac",
        func: NativeFn::F1(te_fac),
        pure: true,
    },
    BuiltinEntry {
        name: "floor",
        func: NativeFn::F1(|a| a.floor()),
        pure: true,
    },
    BuiltinEntry {
        name: "ln",
        func: NativeFn::F1(|a| a.ln()),
        pure: true,
    },
    BuiltinEntry {
        name: "log",
        func: NativeFn::F1(|a| a.log10()),
        pure: true,
    }, // C default: log10
    BuiltinEntry {
        name: "log10",
        func: NativeFn::F1(|a| a.log10()),
        pure: true,
    },
    BuiltinEntry {
        name: "ncr",
        func: NativeFn::F2(te_ncr),
        pure: true,
    },
    BuiltinEntry {
        name: "npr",
        func: NativeFn::F2(te_npr),
        pure: true,
    },
    BuiltinEntry {
        name: "pi",
        func: NativeFn::F0(te_pi),
        pure: true,
    },
    BuiltinEntry {
        name: "pow",
        func: NativeFn::F2(|a, b| a.powf(b)),
        pure: true,
    },
    BuiltinEntry {
        name: "sin",
        func: NativeFn::F1(|a| a.sin()),
        pure: true,
    },
    BuiltinEntry {
        name: "sinh",
        func: NativeFn::F1(|a| a.sinh()),
        pure: true,
    },
    BuiltinEntry {
        name: "sqrt",
        func: NativeFn::F1(|a| a.sqrt()),
        pure: true,
    },
    BuiltinEntry {
        name: "tan",
        func: NativeFn::F1(|a| a.tan()),
        pure: true,
    },
    BuiltinEntry {
        name: "tanh",
        func: NativeFn::F1(|a| a.tanh()),
        pure: true,
    },
];

/// Binary search for a built-in by exact name. Mirrors C's find_builtin.
fn find_builtin(name: &str) -> Option<&'static BuiltinEntry> {
    BUILTINS
        .binary_search_by_key(&name, |e| e.name)
        .ok()
        .map(|i| &BUILTINS[i])
}

// ── User-facing variable/function binding ─────────────────────────────────────

/// One entry in the user-supplied variable/function table passed to `te_compile`.
/// Corresponds to C's `te_variable`.
pub enum TeVariable<'a> {
    Var {
        name: &'a str,
        /// Pointer to caller-owned f64.
        ptr: *const f64,
    },
    Func {
        name: &'a str,
        func: NativeFn,
        pure: bool,
    },
    Closure {
        name: &'a str,
        func: NativeClosure,
        /// Opaque context pointer, passed as first arg.
        context: *mut (),
        pure: bool,
    },
}

impl<'a> TeVariable<'a> {
    pub fn name(&self) -> &str {
        match self {
            TeVariable::Var { name, .. } => name,
            TeVariable::Func { name, .. } => name,
            TeVariable::Closure { name, .. } => name,
        }
    }
}

/// Linear search for a user-supplied variable/function by exact name.
/// Mirrors C's find_lookup.
fn find_lookup<'a>(vars: &'a [TeVariable<'a>], name: &str) -> Option<&'a TeVariable<'a>> {
    vars.iter().find(|v| v.name() == name)
}

// ── Lexer token ───────────────────────────────────────────────────────────────

/// Binary infix operator identity. Replaces C's function-pointer comparison
/// (e.g. `s->function == add`) with a safe, LTO-stable enum variant comparison.
#[derive(Clone, Copy, PartialEq, Eq)]
enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    FMod,
}

impl InfixOp {
    /// Convert to a `NativeFn` for building an `Expr::Function` node.
    fn as_native_fn(self) -> NativeFn {
        match self {
            InfixOp::Add => NativeFn::F2(op_add),
            InfixOp::Sub => NativeFn::F2(op_sub),
            InfixOp::Mul => NativeFn::F2(op_mul),
            InfixOp::Div => NativeFn::F2(op_div),
            InfixOp::Pow => NativeFn::F2(op_pow),
            InfixOp::FMod => NativeFn::F2(op_fmod),
        }
    }
}

/// Current lexer token. Replaces C's `int type + union` in `state`.
#[derive(Clone, Copy)]
enum Token {
    Null, // whitespace skip marker (loops in next_token)
    Error,
    End,
    Sep,   // ','
    Open,  // '('
    Close, // ')'
    Number(f64),
    Variable(*const f64),
    Infix(InfixOp), // binary infix operator — identified by enum, not fn ptr
    Function {
        pure: bool,
        func: NativeFn,
    },
    Closure {
        pure: bool,
        func: NativeClosure,
        context: *mut (),
    },
}

// ── Parser state ──────────────────────────────────────────────────────────────

struct State<'src, 'vars> {
    src: &'src str,
    pos: usize,
    token: Token,
    vars: &'vars [TeVariable<'vars>],
}

impl<'src, 'vars> State<'src, 'vars> {
    fn new(src: &'src str, vars: &'vars [TeVariable<'vars>]) -> Self {
        State {
            src,
            pos: 0,
            token: Token::Null,
            vars,
        }
    }

    /// 1-based byte offset of the current position, for error reporting.
    fn error_pos(&self) -> i32 {
        self.pos.max(1) as i32
    }
}

/// Scan a floating-point literal starting at `start`, returning (value, end_pos).
/// Mirrors C's `strtod` behaviour: digits, optional '.', optional 'e[+-]digits'.
fn scan_number(src: &str, start: usize) -> (f64, usize) {
    let b = src.as_bytes();
    let mut p = start;
    while p < b.len() && b[p].is_ascii_digit() {
        p += 1;
    }
    if p < b.len() && b[p] == b'.' {
        p += 1;
        while p < b.len() && b[p].is_ascii_digit() {
            p += 1;
        }
    }
    if p < b.len() && (b[p] == b'e' || b[p] == b'E') {
        let exp = p;
        p += 1;
        if p < b.len() && (b[p] == b'+' || b[p] == b'-') {
            p += 1;
        }
        if p < b.len() && b[p].is_ascii_digit() {
            while p < b.len() && b[p].is_ascii_digit() {
                p += 1;
            }
        } else {
            p = exp; // not a valid exponent — backtrack
        }
    }
    let v = src[start..p].parse::<f64>().unwrap_or(f64::NAN);
    (v, p)
}

/// Advance to the next token. Mirrors C's `next_token`.
fn next_token(s: &mut State) {
    loop {
        let b = s.src.as_bytes();
        if s.pos >= b.len() {
            s.token = Token::End;
            return;
        }
        let ch = b[s.pos];

        if ch.is_ascii_digit() || ch == b'.' {
            let (val, end) = scan_number(s.src, s.pos);
            s.pos = end;
            s.token = Token::Number(val);
            return;
        }

        if ch.is_ascii_alphabetic() {
            let start = s.pos;
            while s.pos < b.len() && (b[s.pos].is_ascii_alphanumeric() || b[s.pos] == b'_') {
                s.pos += 1;
            }
            let name = &s.src[start..s.pos];

            s.token = if let Some(var) = find_lookup(s.vars, name) {
                match var {
                    TeVariable::Var { ptr, .. } => Token::Variable(*ptr),
                    TeVariable::Func { func, pure, .. } => Token::Function {
                        pure: *pure,
                        func: *func,
                    },
                    TeVariable::Closure {
                        func,
                        context,
                        pure,
                        ..
                    } => Token::Closure {
                        pure: *pure,
                        func: *func,
                        context: *context,
                    },
                }
            } else if let Some(e) = find_builtin(name) {
                Token::Function {
                    pure: e.pure,
                    func: e.func,
                }
            } else {
                Token::Error
            };
            return;
        }

        s.pos += 1;
        s.token = match ch {
            b'+' => Token::Infix(InfixOp::Add),
            b'-' => Token::Infix(InfixOp::Sub),
            b'*' => Token::Infix(InfixOp::Mul),
            b'/' => Token::Infix(InfixOp::Div),
            b'^' => Token::Infix(InfixOp::Pow),
            b'%' => Token::Infix(InfixOp::FMod),
            b'(' => Token::Open,
            b')' => Token::Close,
            b',' => Token::Sep,
            b' ' | b'\t' | b'\n' | b'\r' => Token::Null, // whitespace — loop again
            _ => Token::Error,
        };
        if !matches!(s.token, Token::Null) {
            return;
        }
    }
}

// ── Parser helpers ────────────────────────────────────────────────────────────

fn token_arity(tok: Token) -> usize {
    match tok {
        Token::Function { func, .. } => func.arity(),
        Token::Closure { func, .. } => func.arity(),
        _ => 0,
    }
}

fn build_call(tok: Token, args: Vec<Box<Expr>>) -> Box<Expr> {
    match tok {
        Token::Function { pure, func } => Box::new(Expr::Function { pure, func, args }),
        Token::Closure {
            pure,
            func,
            context,
        } => Box::new(Expr::Closure {
            pure,
            func,
            context,
            args,
        }),
        _ => unreachable!(),
    }
}

// ── Recursive-descent parser ──────────────────────────────────────────────────
//
// Grammar (mirrors tinyexpr.c):
//   list   = expr {"," expr}
//   expr   = term {("+" | "-") term}
//   term   = factor {("*" | "/" | "%") factor}
//   factor = power {"^" power}
//   power  = {"-" | "+"} base
//   base   = NUMBER | VARIABLE | FN0 ["()"] | FN1 power
//            | FN2+ "(" expr {"," expr} ")" | "(" list ")"

fn parse_base(s: &mut State) -> Option<Box<Expr>> {
    match s.token {
        Token::Number(v) => {
            next_token(s);
            Some(Box::new(Expr::Constant(v)))
        }
        Token::Variable(ptr) => {
            next_token(s);
            Some(Box::new(Expr::Variable(ptr)))
        }
        Token::Function { .. } | Token::Closure { .. } => {
            let tok = s.token; // Copy
            let arity = token_arity(tok);
            next_token(s);
            match arity {
                0 => {
                    // optional empty parens: fn0 or fn0()
                    if matches!(s.token, Token::Open) {
                        next_token(s);
                        if !matches!(s.token, Token::Close) {
                            s.token = Token::Error;
                            return None;
                        }
                        next_token(s);
                    }
                    Some(build_call(tok, vec![]))
                }
                1 => {
                    // single arg, no parens required: fn1 <power>
                    let arg = parse_power(s)?;
                    Some(build_call(tok, vec![arg]))
                }
                _ => {
                    // arity ≥ 2: fn( expr, expr, … )
                    if !matches!(s.token, Token::Open) {
                        s.token = Token::Error;
                        return None;
                    }
                    let mut args: Vec<Box<Expr>> = Vec::with_capacity(arity);
                    let mut last_i = 0usize;
                    for i in 0..arity {
                        next_token(s);
                        args.push(parse_expr(s)?);
                        last_i = i;
                        if !matches!(s.token, Token::Sep) {
                            break; // no comma — must be the last arg
                        }
                    }
                    // mirrors C: `if (s->type != TOK_CLOSE || i != arity - 1)`
                    if !matches!(s.token, Token::Close) || last_i != arity - 1 {
                        s.token = Token::Error;
                        return None;
                    }
                    next_token(s);
                    Some(build_call(tok, args))
                }
            }
        }
        Token::Open => {
            next_token(s);
            let ret = parse_list(s)?;
            if !matches!(s.token, Token::Close) {
                s.token = Token::Error;
                return None;
            }
            next_token(s);
            Some(ret)
        }
        _ => {
            s.token = Token::Error;
            None
        }
    }
}

fn parse_power(s: &mut State) -> Option<Box<Expr>> {
    // collect unary sign: --x = x, -+x = -x
    let mut sign = 1i32;
    while let Token::Infix(op @ (InfixOp::Add | InfixOp::Sub)) = s.token {
        if op == InfixOp::Sub {
            sign = -sign;
        }
        next_token(s);
    }
    let base = parse_base(s)?;
    if sign == 1 {
        Some(base)
    } else {
        Some(Box::new(Expr::Function {
            pure: true,
            func: NativeFn::F1(op_negate),
            args: vec![base],
        }))
    }
}

fn parse_factor(s: &mut State) -> Option<Box<Expr>> {
    let mut ret = parse_power(s)?;
    while matches!(s.token, Token::Infix(InfixOp::Pow)) {
        let op = if let Token::Infix(op) = s.token {
            op
        } else {
            unreachable!()
        };
        next_token(s);
        let p = parse_power(s)?;
        ret = Box::new(Expr::Function {
            pure: true,
            func: op.as_native_fn(),
            args: vec![ret, p],
        });
    }
    Some(ret)
}

fn parse_term(s: &mut State) -> Option<Box<Expr>> {
    let mut ret = parse_factor(s)?;
    while let Token::Infix(op @ (InfixOp::Mul | InfixOp::Div | InfixOp::FMod)) = s.token {
        next_token(s);
        let f = parse_factor(s)?;
        ret = Box::new(Expr::Function {
            pure: true,
            func: op.as_native_fn(),
            args: vec![ret, f],
        });
    }
    Some(ret)
}

fn parse_expr(s: &mut State) -> Option<Box<Expr>> {
    let mut ret = parse_term(s)?;
    while let Token::Infix(op @ (InfixOp::Add | InfixOp::Sub)) = s.token {
        next_token(s);
        let te = parse_term(s)?;
        ret = Box::new(Expr::Function {
            pure: true,
            func: op.as_native_fn(),
            args: vec![ret, te],
        });
    }
    Some(ret)
}

fn parse_list(s: &mut State) -> Option<Box<Expr>> {
    let mut ret = parse_expr(s)?;
    while matches!(s.token, Token::Sep) {
        next_token(s);
        let e = parse_expr(s)?;
        ret = Box::new(Expr::Function {
            pure: true,
            func: NativeFn::F2(op_comma),
            args: vec![ret, e],
        });
    }
    Some(ret)
}

// ── Constant folding ──────────────────────────────────────────────────────────

/// Mirrors C's `optimize`: recursively fold pure functions with constant args.
fn optimize(n: &mut Box<Expr>) {
    let should_fold = match n.as_mut() {
        Expr::Constant(_) | Expr::Variable(_) => return,
        Expr::Function {
            pure: true, args, ..
        }
        | Expr::Closure {
            pure: true, args, ..
        } => {
            for arg in args.iter_mut() {
                optimize(arg);
            }
            args.iter().all(|a| matches!(a.as_ref(), Expr::Constant(_)))
        }
        _ => return, // impure — do not fold
    };
    if should_fold {
        let val = te_eval(&**n);
        **n = Expr::Constant(val);
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Evaluates `expression` and returns the result. Returns `NAN` on error.
/// Sets `*error` to the 1-based byte offset of the error, or 0 on success.
pub fn te_interp(expression: &str, error: &mut i32) -> f64 {
    match te_compile(expression, &[], error) {
        Some(expr) => te_eval(&expr),
        None => f64::NAN,
    }
}

/// Parses `expression` with optional variable bindings.
/// Returns `None` on parse error and sets `*error` to the error position (1-based).
pub fn te_compile<'a>(
    expression: &str,
    variables: &'a [TeVariable<'a>],
    error: &mut i32,
) -> Option<Box<Expr>> {
    let mut s = State::new(expression, variables);
    next_token(&mut s);

    let mut root = match parse_list(&mut s) {
        Some(r) => r,
        None => {
            *error = s.error_pos();
            return None;
        }
    };

    if !matches!(s.token, Token::End) {
        *error = s.error_pos();
        return None;
    }

    optimize(&mut root);
    *error = 0;
    Some(root)
}

/// Evaluates a compiled expression tree.
pub fn te_eval(n: &Expr) -> f64 {
    match n {
        Expr::Constant(v) => *v,
        Expr::Variable(ptr) => unsafe { **ptr },
        Expr::Function { func, args, .. } => eval_native_fn(*func, args),
        Expr::Closure {
            func,
            context,
            args,
            ..
        } => eval_closure(*func, *context, args),
    }
}

fn eval_native_fn(func: NativeFn, args: &[Box<Expr>]) -> f64 {
    let e = |i: usize| te_eval(&args[i]);
    match func {
        NativeFn::F0(f) => f(),
        NativeFn::F1(f) => f(e(0)),
        NativeFn::F2(f) => f(e(0), e(1)),
        NativeFn::F3(f) => f(e(0), e(1), e(2)),
        NativeFn::F4(f) => f(e(0), e(1), e(2), e(3)),
        NativeFn::F5(f) => f(e(0), e(1), e(2), e(3), e(4)),
        NativeFn::F6(f) => f(e(0), e(1), e(2), e(3), e(4), e(5)),
        NativeFn::F7(f) => f(e(0), e(1), e(2), e(3), e(4), e(5), e(6)),
    }
}

fn eval_closure(func: NativeClosure, ctx: *mut (), args: &[Box<Expr>]) -> f64 {
    let e = |i: usize| te_eval(&args[i]);
    match func {
        NativeClosure::C0(f) => f(ctx),
        NativeClosure::C1(f) => f(ctx, e(0)),
        NativeClosure::C2(f) => f(ctx, e(0), e(1)),
        NativeClosure::C3(f) => f(ctx, e(0), e(1), e(2)),
        NativeClosure::C4(f) => f(ctx, e(0), e(1), e(2), e(3)),
        NativeClosure::C5(f) => f(ctx, e(0), e(1), e(2), e(3), e(4)),
        NativeClosure::C6(f) => f(ctx, e(0), e(1), e(2), e(3), e(4), e(5)),
        NativeClosure::C7(f) => f(ctx, e(0), e(1), e(2), e(3), e(4), e(5), e(6)),
    }
}

// ── Smoke tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_eval() {
        let expr = Expr::Constant(42.0);
        assert_eq!(te_eval(&expr), 42.0);
    }

    #[test]
    fn variable_eval() {
        let x: f64 = 3.14;
        let expr = Expr::Variable(&x as *const f64);
        assert_eq!(te_eval(&expr), 3.14);
    }

    #[test]
    fn function1_eval() {
        let expr = Expr::Function {
            pure: true,
            func: NativeFn::F1(|a| a * 2.0),
            args: vec![Box::new(Expr::Constant(5.0))],
        };
        assert_eq!(te_eval(&expr), 10.0);
    }

    #[test]
    fn function2_eval() {
        let expr = Expr::Function {
            pure: true,
            func: NativeFn::F2(|a, b| a + b),
            args: vec![Box::new(Expr::Constant(3.0)), Box::new(Expr::Constant(4.0))],
        };
        assert_eq!(te_eval(&expr), 7.0);
    }

    #[test]
    fn builtin_lookup_found() {
        assert!(find_builtin("sin").is_some());
        assert!(find_builtin("atan2").is_some());
        assert!(find_builtin("pi").is_some());
        assert!(find_builtin("log").is_some());
    }

    #[test]
    fn builtin_lookup_not_found() {
        assert!(find_builtin("foo").is_none());
        assert!(find_builtin("sinn").is_none()); // prefix match must not succeed
        assert!(find_builtin("").is_none());
    }

    #[test]
    fn builtin_sin_eval() {
        let entry = find_builtin("sin").unwrap();
        let expr = Expr::Function {
            pure: entry.pure,
            func: entry.func,
            args: vec![Box::new(Expr::Constant(0.0))],
        };
        assert_eq!(te_eval(&expr), 0.0_f64.sin());
    }

    #[test]
    fn builtin_fac_values() {
        assert_eq!(te_fac(0.0), 1.0);
        assert_eq!(te_fac(5.0), 120.0);
        assert!(te_fac(-1.0).is_nan());
    }

    #[test]
    fn builtin_ncr_values() {
        assert_eq!(te_ncr(5.0, 2.0), 10.0);
        assert_eq!(te_ncr(0.0, 0.0), 1.0);
        assert!(te_ncr(-1.0, 1.0).is_nan());
    }

    #[test]
    fn user_lookup() {
        let x: f64 = 99.0;
        let vars = [TeVariable::Var {
            name: "x",
            ptr: &x as *const f64,
        }];
        assert!(find_lookup(&vars, "x").is_some());
        assert!(find_lookup(&vars, "y").is_none());
    }

    // ── integration: te_interp / te_compile ──────────────────────────────────

    fn interp(expr: &str) -> f64 {
        te_interp(expr, &mut 0i32)
    }

    #[test]
    fn interp_literal() {
        assert_eq!(interp("42"), 42.0);
        assert_eq!(interp("3.14"), 3.14);
    }

    #[test]
    fn interp_add_sub() {
        assert_eq!(interp("1+2"), 3.0);
        assert_eq!(interp("5-3"), 2.0);
        assert_eq!(interp("1+2+3"), 6.0);
    }

    #[test]
    fn interp_mul_div() {
        assert_eq!(interp("2*3"), 6.0);
        assert_eq!(interp("10/4"), 2.5);
    }

    #[test]
    fn interp_pow() {
        assert_eq!(interp("2^10"), 1024.0);
    }

    #[test]
    fn interp_precedence() {
        assert_eq!(interp("2+3*4"), 14.0);
        assert_eq!(interp("(2+3)*4"), 20.0);
    }

    #[test]
    fn interp_unary_minus() {
        assert_eq!(interp("-1"), -1.0);
        assert_eq!(interp("--1"), 1.0);
        assert_eq!(interp("-2^2"), 4.0); // C default: (-2)^2 — unary before base
    }

    #[test]
    fn interp_builtin_fn0() {
        let pi = interp("pi");
        assert!((pi - std::f64::consts::PI).abs() < 1e-14);
        let e = interp("e");
        assert!((e - std::f64::consts::E).abs() < 1e-14);
    }

    #[test]
    fn interp_builtin_fn1() {
        assert_eq!(interp("sin(0)"), 0.0);
        assert!((interp("sqrt(4)") - 2.0).abs() < 1e-14);
    }

    #[test]
    fn interp_builtin_fn2() {
        assert!((interp("pow(2,10)") - 1024.0).abs() < 1e-10);
        assert!((interp("atan2(1,1)") - std::f64::consts::FRAC_PI_4).abs() < 1e-14);
    }

    #[test]
    fn interp_whitespace() {
        assert_eq!(interp("  1 + 2 "), 3.0);
    }

    #[test]
    fn interp_error_incomplete() {
        let mut err = 0i32;
        let v = te_interp("1+", &mut err);
        assert!(v.is_nan());
        assert_ne!(err, 0);
    }

    #[test]
    fn interp_error_unknown_id() {
        let mut err = 0i32;
        let v = te_interp("foo", &mut err);
        assert!(v.is_nan());
        assert_ne!(err, 0);
    }

    #[test]
    fn compile_with_variable() {
        let x = 5.0f64;
        let vars = [TeVariable::Var {
            name: "x",
            ptr: &x as *const f64,
        }];
        let mut err = 0i32;
        let expr = te_compile("x*2+1", &vars, &mut err).unwrap();
        assert_eq!(err, 0);
        assert_eq!(te_eval(&expr), 11.0);
    }

    #[test]
    fn compile_constant_folded() {
        // After optimize(), sin(0) should be folded to Constant(0.0)
        let mut err = 0i32;
        let expr = te_compile("sin(0)", &[], &mut err).unwrap();
        assert_eq!(err, 0);
        assert!(matches!(*expr, Expr::Constant(_)));
        assert_eq!(te_eval(&expr), 0.0);
    }

    // ── next_token tests ──────────────────────────────────────────────────────

    fn tok(src: &str) -> Token {
        let mut s = State::new(src, &[]);
        next_token(&mut s);
        s.token
    }

    #[test]
    fn lex_end() {
        assert!(matches!(tok(""), Token::End));
        assert!(matches!(tok("   "), Token::End)); // whitespace-only
    }

    #[test]
    fn lex_number_integer() {
        assert!(matches!(tok("42"), Token::Number(v) if v == 42.0));
    }

    #[test]
    fn lex_number_decimal() {
        assert!(matches!(tok("3.14"), Token::Number(v) if (v - 3.14).abs() < 1e-10));
    }

    #[test]
    fn lex_number_exponent() {
        assert!(matches!(tok("1e3"), Token::Number(v) if v == 1000.0));
        assert!(matches!(tok("2.5e-1"), Token::Number(v) if (v - 0.25).abs() < 1e-10));
    }

    #[test]
    fn lex_number_dot_prefix() {
        assert!(matches!(tok(".5"), Token::Number(v) if v == 0.5));
    }

    #[test]
    fn lex_builtin_function() {
        assert!(matches!(
            tok("sin"),
            Token::Function {
                func: NativeFn::F1(_),
                ..
            }
        ));
        assert!(matches!(
            tok("atan2"),
            Token::Function {
                func: NativeFn::F2(_),
                ..
            }
        ));
        assert!(matches!(
            tok("pi"),
            Token::Function {
                func: NativeFn::F0(_),
                ..
            }
        ));
    }

    #[test]
    fn lex_unknown_identifier() {
        assert!(matches!(tok("foo"), Token::Error));
    }

    #[test]
    fn lex_infix_ops() {
        assert!(matches!(tok("+"), Token::Infix(InfixOp::Add)));
        assert!(matches!(tok("-"), Token::Infix(InfixOp::Sub)));
        assert!(matches!(tok("*"), Token::Infix(InfixOp::Mul)));
        assert!(matches!(tok("/"), Token::Infix(InfixOp::Div)));
        assert!(matches!(tok("^"), Token::Infix(InfixOp::Pow)));
        assert!(matches!(tok("%"), Token::Infix(InfixOp::FMod)));
    }

    #[test]
    fn lex_punctuation() {
        assert!(matches!(tok("("), Token::Open));
        assert!(matches!(tok(")"), Token::Close));
        assert!(matches!(tok(","), Token::Sep));
    }

    #[test]
    fn lex_whitespace_skip() {
        // leading whitespace is transparent
        assert!(matches!(tok("  42"), Token::Number(v) if v == 42.0));
        assert!(matches!(tok("\t+"), Token::Infix(InfixOp::Add)));
    }

    #[test]
    fn lex_error_char() {
        assert!(matches!(tok("?"), Token::Error));
        assert!(matches!(tok("@"), Token::Error));
    }

    #[test]
    fn lex_user_variable() {
        let x: f64 = 7.0;
        let vars = [TeVariable::Var {
            name: "x",
            ptr: &x as *const f64,
        }];
        let mut s = State::new("x", &vars);
        next_token(&mut s);
        assert!(matches!(s.token, Token::Variable(p) if unsafe { *p } == 7.0));
    }
}
