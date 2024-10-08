use egg::{rewrite as rw, *};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

define_language! {
    #[derive(Serialize, Deserialize)]
    enum Lambda {
        Bool(bool),
        Num(i32),

        "var" = Var(Id),

        "+" = Add([Id; 2]),
        "==" = Eq([Id; 2]),

        "app" = App([Id; 2]),
        "lam" = Lambda([Id; 2]),
        "let" = Let([Id; 3]),
        "fix" = Fix([Id; 2]),

        "if" = If([Id; 3]),

        Symbol(egg::Symbol),
    }
}

impl Lambda {
    fn num(&self) -> Option<i32> {
        match self {
            Lambda::Num(n) => Some(*n),
            _ => None,
        }
    }
}

type EGraph = egg::EGraph<Lambda, LambdaAnalysis>;

#[derive(Default, Clone)]
struct LambdaAnalysis;

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    free: HashSet<Id>,
    constant: Option<Lambda>,
}

fn eval(egraph: &EGraph, enode: &Lambda) -> Option<Lambda> {
    let x = |i: &Id| egraph[*i].data.constant.clone();
    match enode {
        Lambda::Num(_) | Lambda::Bool(_) => Some(enode.clone()),
        Lambda::Add([a, b]) => Some(Lambda::Num(x(a)?.num()? + x(b)?.num()?)),
        Lambda::Eq([a, b]) => Some(Lambda::Bool(x(a)? == x(b)?)),
        _ => None,
    }
}

impl Analysis<Lambda> for LambdaAnalysis {
    type Data = Data;
    fn merge(&self, to: &mut Data, from: Data) -> bool {
        let before_len = to.free.len();
        // to.free.extend(from.free);
        to.free.retain(|i| from.free.contains(i));
        let did_change = before_len != to.free.len();
        if to.constant.is_none() && from.constant.is_some() {
            to.constant = from.constant;
            true
        } else {
            did_change
        }
    }

    fn make(egraph: &EGraph, enode: &Lambda) -> Data {
        let f = |i: &Id| egraph[*i].data.free.iter().cloned();
        let mut free = HashSet::default();
        match enode {
            Lambda::Var(v) => {
                free.insert(*v);
            }
            Lambda::Let([v, a, b]) => {
                free.extend(f(b));
                free.remove(v);
                free.extend(f(a));
            }
            Lambda::Lambda([v, a]) | Lambda::Fix([v, a]) => {
                free.extend(f(a));
                free.remove(v);
            }
            _ => enode.for_each(|c| free.extend(&egraph[c].data.free)),
        }
        let constant = eval(egraph, enode);
        Data { constant, free }
    }

    fn modify(egraph: &mut EGraph, id: Id) {
        if let Some(c) = egraph[id].data.constant.clone() {
            let const_id = egraph.add(c);
            egraph.union(id, const_id);
        }
    }
}

fn var(s: &str) -> Var {
    s.parse().unwrap()
}

struct IsConstApplier {
    v: Var,
}

impl Display for IsConstApplier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "is-const({})", self.v)
    }
}

impl Applier<Lambda, LambdaAnalysis> for IsConstApplier {
    fn apply_one(&self, egraph: &mut egg::EGraph<Lambda, LambdaAnalysis>, _eclass: Id, subst: &Subst) -> Vec<Id> {
        if egraph[subst[self.v]].data.constant.is_some() {
            vec![subst[self.v]]
        } else {
            vec![]
        }
    }
}

fn is_const(v: Var) -> IsConstApplier {
    IsConstApplier { v }
}

fn rules() -> Vec<Rewrite<Lambda, LambdaAnalysis>> {
    vec![
        // open term rules
        rw!("if-true";  "(if  true ?then ?else)" => "?then"),
        rw!("if-false"; "(if false ?then ?else)" => "?else"),
        multi_rewrite!("if-elim"; "?var1 = (if (== (var ?x) ?e) ?then ?else), ?var2 = (let ?x ?e ?then), ?var2 = (let ?x ?e ?else)" => "?var1 = ?else"),
        rw!("add-comm";  "(+ ?a ?b)"        => "(+ ?b ?a)"),
        rw!("add-assoc"; "(+ (+ ?a ?b) ?c)" => "(+ ?a (+ ?b ?c))"),
        rw!("eq-comm";   "(== ?a ?b)"        => "(== ?b ?a)"),
        // subst rules
        rw!("fix";      "(fix ?v ?e)"             => "(let ?v (fix ?v ?e) ?e)"),
        rw!("beta";     "(app (lam ?v ?body) ?e)" => "(let ?v ?e ?body)"),
        rw!("let-app";  "(let ?v ?e (app ?a ?b))" => "(app (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-add";  "(let ?v ?e (+   ?a ?b))" => "(+   (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-eq";   "(let ?v ?e (==   ?a ?b))" => "(==   (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-const";
            "(let ?v ?e ?c)" => { is_const(var("?c")) }),
        rw!("let-if";
            "(let ?v ?e (if ?cond ?then ?else))" =>
            "(if (let ?v ?e ?cond) (let ?v ?e ?then) (let ?v ?e ?else))"
        ),
        rw!("let-var-same"; "(let ?v1 ?e (var ?v1))" => "?e"),
        multi_rewrite!("let-var-diff"; "?x = (let ?v1 ?e (var ?v2)), ?v1 != ?v2" => "?x = (var ?v2)"),
        rw!("let-lam-same"; "(let ?v1 ?e (lam ?v1 ?body))" => "(lam ?v1 ?body)"),
        multi_rewrite!("let-lam-diff";
            "?root = (let ?v1 ?e (lam ?v2 ?body)), ?v1 != ?v2" =>
            { CaptureAvoid {
                root: var("?root"), fresh: var("?fresh"), v2: var("?v2"), e: var("?e"),
                if_not_free: "(lam ?v2 (let ?v1 ?e ?body))".parse().unwrap(),
                if_free: "(lam ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))".parse().unwrap(),
            }}),
    ]
}

struct CaptureAvoid {
    root: Var,
    fresh: Var,
    v2: Var,
    e: Var,
    if_not_free: Pattern<Lambda>,
    if_free: Pattern<Lambda>,
}

impl Applier<Lambda, LambdaAnalysis> for CaptureAvoid {
    fn apply_one(&self, egraph: &mut EGraph, _eclass: Id, subst: &Subst) -> Vec<Id> {
        let e = subst[self.e];
        let v2 = subst[self.v2];
        let v2_free_in_e = egraph[e].data.free.contains(&v2);
        let eclass = subst[self.root];
        if v2_free_in_e {
            let mut subst = subst.clone();
            let sym = Lambda::Symbol(format!("_{}", eclass).into());
            subst.insert(self.fresh, egraph.add(sym));
            self.if_free.apply_one(egraph, eclass, &subst)
        } else {
            self.if_not_free.apply_one(egraph, eclass, &subst)
        }
    }
}

impl Display for CaptureAvoid {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[cfg(test)]
egg::test_fn! {
    lambda_under, rules(),
    "(lam x (+ 4
               (app (lam y (var y))
                    4)))"
    =>
    // "(lam x (+ 4 (let y 4 (var y))))",
    // "(lam x (+ 4 4))",
    "(lam x 8))",
}

egg::test_fn! {
    lambda_let_simple, rules(),
    "(let x 0
     (let y 1
     (+ (var x) (var y))))"
    =>
    // "(let ?a 0
    //  (+ (var ?a) 1))",
    // "(+ 0 1)",
    "1",
}

egg::test_fn! {
    #[should_panic(expected = "Could not prove goal 0")]
    lambda_capture, rules(),
    "(let x 1 (lam x (var x)))" => "(lam x 1)"
}

egg::test_fn! {
    #[should_panic(expected = "Could not prove goal 0")]
    lambda_capture_free, rules(),
    "(let y (+ (var x) (var x)) (lam x (var y)))" => "(lam x (+ (var x) (var x)))"
}

egg::test_fn! {
    #[should_panic(expected = "Could not prove goal 0")]
    lambda_closure_not_seven, rules(),
    "(let five 5
     (let add-five (lam x (+ (var x) (var five)))
     (let five 6
     (app (var add-five) 1))))"
    =>
    "7"
}

egg::test_fn! {
    lambda_compose, rules(),
    "(let compose (lam f (lam g (lam x (app (var f)
                                       (app (var g) (var x))))))
     (let add1 (lam y (+ (var y) 1))
     (app (app (var compose) (var add1)) (var add1))))"
    =>
    "(lam ?x (+ 1
                (app (lam ?y (+ 1 (var ?y)))
                     (var ?x))))",
    "(lam ?x (+ (var ?x) 2))"
}

egg::test_fn! {
    lambda_if_simple, rules(),
    "(if (== 1 1) 7 9)" => "7"
}

egg::test_fn! {
    lambda_compose_many, rules(),
    "(let compose (lam f (lam g (lam x (app (var f)
                                       (app (var g) (var x))))))
     (let add1 (lam y (+ (var y) 1))
     (app (app (var compose) (var add1))
          (app (app (var compose) (var add1))
               (app (app (var compose) (var add1))
                    (app (app (var compose) (var add1))
                         (app (app (var compose) (var add1))
                              (app (app (var compose) (var add1))
                                   (var add1)))))))))"
    =>
    "(lam ?x (+ (var ?x) 7))"
}

egg::test_fn! {
    #[cfg_attr(feature = "upward-merging", ignore)]
    #[ignore]
    lambda_function_repeat, rules(),
    runner = Runner::default()
        .with_time_limit(std::time::Duration::from_secs(20))
        .with_node_limit(100_000)
        .with_iter_limit(60),
    "(let compose (lam f (lam g (lam x (app (var f)
                                       (app (var g) (var x))))))
     (let repeat (fix repeat (lam fun (lam n
        (if (== (var n) 0)
            (lam i (var i))
            (app (app (var compose) (var fun))
                 (app (app (var repeat)
                           (var fun))
                      (+ (var n) -1)))))))
     (let add1 (lam y (+ (var y) 1))
     (app (app (var repeat)
               (var add1))
          2))))"
    =>
    "(lam ?x (+ (var ?x) 2))"
}

egg::test_fn! {
    lambda_if, rules(),
    "(let zeroone (lam x
        (if (== (var x) 0)
            0
            1))
        (+ (app (var zeroone) 0)
        (app (var zeroone) 10)))"
    =>
    // "(+ (if false 0 1) (if true 0 1))",
    // "(+ 1 0)",
    "1",
}

egg::test_fn! {
    #[cfg_attr(feature = "upward-merging", ignore)]
    #[ignore]
    lambda_fib, rules(),
    runner = Runner::default()
        .with_iter_limit(60)
        .with_node_limit(50_000),
    "(let fib (fix fib (lam n
        (if (== (var n) 0)
            0
        (if (== (var n) 1)
            1
        (+ (app (var fib)
                (+ (var n) -1))
            (app (var fib)
                (+ (var n) -2)))))))
        (app (var fib) 4))"
    => "3"
}
