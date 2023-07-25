use std::fmt;
use std::rc::Rc;

use crate::{Analysis, EGraph, Id, Language, Pattern, SearchMatches, Subst, Var, ColorId, FromOp};
use std::fmt::{Formatter, Debug};
use std::marker::PhantomData;
use std::ops::Deref;
use itertools::Itertools;
use invariants::iassert;

/// A rewrite that searches for the lefthand side and applies the righthand side.
///
/// The [`rewrite!`] is the easiest way to create rewrites.
///
/// A [`Rewrite`] consists principally of a [`Searcher`] (the lefthand
/// side) and an [`Applier`] (the righthand side).
/// It additionally stores a name used to refer to the rewrite and a
/// long name used for debugging.
///
/// [`rewrite!`]: macro.rewrite.html
/// [`Searcher`]: trait.Searcher.html
/// [`Applier`]: trait.Applier.html
/// [`Condition`]: trait.Condition.html
/// [`ConditionalApplier`]: struct.ConditionalApplier.html
/// [`Rewrite`]: struct.Rewrite.html
/// [`Pattern`]: struct.Pattern.html
#[derive(Clone)]
#[non_exhaustive]
pub struct Rewrite<L: Language, N: Analysis<L>> {
    name: String,
    long_name: String,
    searcher: Rc<dyn Searcher<L, N>>,
    applier: Rc<dyn Applier<L, N>>,
}

impl<L, N> fmt::Debug for Rewrite<L, N>
    where
        L: Language + 'static,
        N: Analysis<L> + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DisplayAsDebug<T>(T);
        impl<T: fmt::Display> fmt::Debug for DisplayAsDebug<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        use std::any::Any;

        let mut d = f.debug_struct("Rewrite");
        d.field("name", &self.name)
            .field("long_name", &self.long_name);

        if let Some(pat) = <dyn Any>::downcast_ref::<Pattern<L>>(&self.searcher) {
            d.field("searcher", &DisplayAsDebug(pat));
        } else {
            d.field("searcher", &"<< searcher >>");
        }

        if let Some(pat) = <dyn Any>::downcast_ref::<Pattern<L>>(&self.applier) {
            d.field("applier", &DisplayAsDebug(pat));
        } else {
            d.field("applier", &"<< applier >>");
        }

        d.finish()
    }
}

impl<L: Language, N: Analysis<L>> Rewrite<L, N> {
    /// Returns the name of the rewrite.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the long name of the rewrite which should only be used for
    /// debugging and displaying.
    pub fn long_name(&self) -> &str {
        &self.long_name
    }
}

impl<L: Language + 'static, N: Analysis<L> + 'static> Rewrite<L, N> {
    /// Create a new [`Rewrite`]. You typically want to use the
    /// [`rewrite!`] macro instead.
    ///
    /// [`Rewrite`]: struct.Rewrite.html
    /// [`rewrite!`]: macro.rewrite.html
    pub fn old_new(
        name: impl Into<String>,
        long_name: impl Into<String>,
        searcher: impl Searcher<L, N> + 'static,
        applier: impl Applier<L, N> + 'static,
    ) -> Result<Self, String> {
        let name = name.into();
        let long_name = long_name.into();
        let searcher = Rc::new(searcher);
        let applier = Rc::new(applier);

        let bound_vars = searcher.vars();
        for v in applier.vars() {
            if !bound_vars.contains(&v) {
                return Err(format!("Rewrite {} refers to unbound var {}", name, v));
            }
        }

        Ok(Self {
            name,
            long_name,
            searcher,
            applier,
        })
    }

    /// Create a new [`Rewrite`]. You typically want to use the
    /// [`rewrite!`] macro instead.
    ///
    /// [`Rewrite`]: struct.Rewrite.html
    /// [`rewrite!`]: macro.rewrite.html
    pub fn new(
        name: impl Into<String>,
        // long_name: impl Into<String>,
        searcher: impl Searcher<L, N> + 'static,
        applier: impl Applier<L, N> + 'static,
    ) -> Result<Self, String> {
        let name = name.into();
        let long_name = name.clone();
        let searcher = Rc::new(searcher);
        let applier = Rc::new(applier);

        let bound_vars = searcher.vars();
        for v in applier.vars() {
            if !bound_vars.contains(&v) {
                return Err(format!("Rewrite {} refers to unbound var {}", name, v));
            }
        }

        Ok(Self {
            name,
            long_name,
            searcher,
            applier,
        })
    }

    /// Call [`search`] on the [`Searcher`].
    ///
    /// [`Searcher`]: trait.Searcher.html
    /// [`search`]: trait.Searcher.html#method.search
    pub fn search(&self, egraph: &EGraph<L, N>) -> Option<SearchMatches> {
        self.searcher.search(egraph)
    }

    /// Call [`apply_matches`] on the [`Applier`].
    ///
    /// [`Applier`]: trait.Applier.html
    /// [`apply_matches`]: trait.Applier.html#method.apply_matches
    pub fn apply(&self, egraph: &mut EGraph<L, N>, matches: &Option<SearchMatches>) -> Vec<Id> {
        self.applier.apply_matches(egraph, matches)
    }

    /// This `run` is for testing use only. You should use things
    /// from the `egg::run` module
    #[cfg(test)]
    pub(crate) fn run(&self, egraph: &mut EGraph<L, N>) -> Vec<Id> {
        let start = instant::Instant::now();

        let matches = self.search(egraph);
        log::debug!("Found rewrite {} {} times", self.name, matches.as_ref().map_or(0, |m| m.len()));

        let ids = self.apply(egraph, &matches);
        let elapsed = start.elapsed();
        log::debug!(
            "Applied rewrite {} {} times in {}.{:03}",
            self.name,
            ids.len(),
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );

        egraph.rebuild();
        ids
    }
}

impl<L: Language, N: Analysis<L>> std::fmt::Display for Rewrite<L, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rewrite({}, {}, {})", self.name, self.searcher, self.applier)
    }
}

/// The lefthand side of a [`Rewrite`].
///
/// A [`Searcher`] is something that can search the egraph and find
/// matching substititions.
/// Right now the only significant [`Searcher`] is [`Pattern`].
///
/// [`Rewrite`]: struct.Rewrite.html
/// [`Searcher`]: trait.Searcher.html
/// [`Pattern`]: struct.Pattern.html
pub trait Searcher<L, N>: std::fmt::Display
    where
        L: Language,
        N: Analysis<L>,
{
    /// Search one eclass, returning None if no matches can be found.
    /// This should not return a SearchMatches with no substs.
    fn search_eclass(&self, egraph: &EGraph<L, N>, eclass: Id) -> Option<SearchMatches>;

    /// Search the whole [`EGraph`], returning a list of all the
    /// [`SearchMatches`] where something was found.
    /// This just calls [`search_eclass`] on each eclass.
    ///
    /// [`EGraph`]: struct.EGraph.html
    /// [`search_eclass`]: trait.Searcher.html#tymethod.search_eclass
    /// [`SearchMatches`]: struct.SearchMatches.html
    fn search(&self, egraph: &EGraph<L, N>) -> Option<SearchMatches> {
        let matches = egraph
            .classes()
            .filter_map(|e| self.search_eclass(egraph, e.id))
            .collect_vec();
        SearchMatches::merge_matches(matches)
    }

    /// Search one eclass with starting color
    /// This should also check all color-equal classes
    fn colored_search_eclass(&self, egraph: &EGraph<L, N>, eclass: Id, color: ColorId) -> Option<SearchMatches>;

    /// Returns a list of the variables bound by this Searcher
    fn vars(&self) -> Vec<Var>;

    /// Returns the number of matches in the e-graph
    fn n_matches(&self, egraph: &EGraph<L, N>) -> usize {
        self.search(egraph).map_or(0, |ms| ms.iter().map(|m| m.substs.len()).sum())
    }
}

/// The righthand side of a [`Rewrite`].
///
/// An [`Applier`] is anything that can do something with a
/// substitition ([`Subst`]). This allows you to implement rewrites
/// that determine when and how to respond to a match using custom
/// logic, including access to the [`Analysis`] data of an [`EClass`].
///
/// Notably, [`Pattern`] implements [`Applier`], which suffices in
/// most cases.
/// Additionally, `egg` provides [`ConditionalApplier`] to stack
/// [`Condition`]s onto an [`Applier`], which in many cases can save
/// you from having to implement your own applier.
///
/// # Example
/// ```
/// use egg::{rewrite as rw, *};
/// use std::fmt::{Display, Formatter};
///
/// define_language! {
///     enum Math {
///         Num(i32),
///         "+" = Add([Id; 2]),
///         "*" = Mul([Id; 2]),
///         Symbol(Symbol),
///     }
/// }
///
/// type EGraph = egg::EGraph<Math, MinSize>;
///
/// // Our metadata in this case will be size of the smallest
/// // represented expression in the eclass.
/// #[derive(Default, Clone)]
/// struct MinSize;
/// impl Analysis<Math> for MinSize {
///     type Data = usize;
///     fn merge(&self, to: &mut Self::Data, from: Self::Data) -> bool {
///         merge_if_different(to, (*to).min(from))
///     }
///     fn make(egraph: &EGraph, enode: &Math) -> Self::Data {
///         let get_size = |i: Id| egraph[i].data;
///         AstSize.cost(enode, get_size)
///     }
/// }
///
/// let rules = &[
///     rw!("commute-add"; "(+ ?a ?b)" => "(+ ?b ?a)"),
///     rw!("commute-mul"; "(* ?a ?b)" => "(* ?b ?a)"),
///     rw!("add-0"; "(+ ?a 0)" => "?a"),
///     rw!("mul-0"; "(* ?a 0)" => "0"),
///     rw!("mul-1"; "(* ?a 1)" => "?a"),
///     // the rewrite macro parses the rhs as a single token tree, so
///     // we wrap it in braces (parens work too).
///     rw!("funky"; "(+ ?a (* ?b ?c))" => { Funky {
///         a: "?a".parse().unwrap(),
///         b: "?b".parse().unwrap(),
///         c: "?c".parse().unwrap(),
///     }}),
/// ];
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// struct Funky {
///     a: Var,
///     b: Var,
///     c: Var,
/// }
///
/// impl std::fmt::Display for Funky {
///     fn fmt(&self,f: &mut Formatter<'_>) -> std::fmt::Result {
///         write!(f, "Funky")
///     }
/// }
///
/// impl Applier<Math, MinSize> for Funky {
///     fn apply_one(&self, egraph: &mut EGraph, matched_id: Id, subst: &Subst) -> Vec<Id> {
///         let a: Id = subst[self.a];
///         // In a custom Applier, you can inspect the analysis data,
///         // which is powerful combination!
///         let size_of_a = egraph[a].data;
///         if size_of_a > 50 {
///             println!("Too big! Not doing anything");
///             vec![]
///         } else {
///             // we're going to manually add:
///             // (+ (+ ?a 0) (* (+ ?b 0) (+ ?c 0)))
///             // to be unified with the original:
///             // (+    ?a    (*    ?b       ?c   ))
///             let b: Id = subst[self.b];
///             let c: Id = subst[self.c];
///             let zero = egraph.add(Math::Num(0));
///             let a0 = egraph.add(Math::Add([a, zero]));
///             let b0 = egraph.add(Math::Add([b, zero]));
///             let c0 = egraph.add(Math::Add([c, zero]));
///             let b0c0 = egraph.add(Math::Mul([b0, c0]));
///             let a0b0c0 = egraph.add(Math::Add([a0, b0c0]));
///             // NOTE: we just return the id according to what we
///             // want unified with matched_id. The `apply_matches`
///             // method actually does the union, _not_ `apply_one`.
///             vec![a0b0c0]
///         }
///     }
/// }
///
/// let start = "(+ x (* y z))".parse().unwrap();
/// let mut runner = Runner::default().with_expr(&start);
/// runner.egraph.rebuild();
/// runner.run(rules);
/// ```
/// [`Pattern`]: struct.Pattern.html
/// [`EClass`]: struct.EClass.html
/// [`Rewrite`]: struct.Rewrite.html
/// [`ConditionalApplier`]: struct.ConditionalApplier.html
/// [`Subst`]: struct.Subst.html
/// [`Applier`]: trait.Applier.html
/// [`Condition`]: trait.Condition.html
/// [`Analysis`]: trait.Analysis.html
pub trait Applier<L, N>: std::fmt::Display
    where
        L: Language + 'static,
        N: Analysis<L> +'static,
{
    /// Apply many substititions.
    ///
    /// This method should call [`apply_one`] for each match and then
    /// unify the results with the matched eclass.
    /// This should return a list of [`Id`]s where the union actually
    /// did something.
    ///
    /// The default implementation does this and should suffice for
    /// most use cases.
    ///
    /// [`Id`]: struct.Id.html
    /// [`apply_one`]: trait.Applier.html#method.apply_one
    fn apply_matches(&self, egraph: &mut EGraph<L, N>, matches: &Option<SearchMatches>) -> Vec<Id> {
        let mut added = vec![];
        if let Some(mat) = matches {
            for (eclass, substs) in mat.matches.iter() {
                for subst in substs {
                    let ids = self
                        .apply_one(egraph, *eclass, subst)
                        .into_iter()
                        .filter_map(|id| {
                            if !cfg!(feature = "colored") {
                                iassert!(subst.color().is_none());
                            }
                            let (to, did_something) =
                                egraph.opt_colored_union(subst.color(), id, *eclass);
                            if did_something {
                                Some(to)
                            } else {
                                None
                            }
                        });
                    added.extend(ids)
                }
            }
        }
        added
    }

    /// Apply a single substitition.
    ///
    /// An [`Applier`] should only add things to the egraph here,
    /// _not_ union them with the id `eclass`.
    /// That is the responsibility of the [`apply_matches`] method.
    /// The `eclass` parameter allows the implementer to inspect the
    /// eclass where the match was found if they need to.
    ///
    /// This should return a list of [`Id`]s of things you'd like to
    /// be unioned with `eclass`. There can be zero, one, or many.
    ///
    /// [`Applier`]: trait.Applier.html
    /// [`Id`]: struct.Id.html
    /// [`apply_matches`]: trait.Applier.html#method.apply_matches
    fn apply_one(&self, egraph: &mut EGraph<L, N>, eclass: Id, subst: &Subst) -> Vec<Id>;

    /// Returns a list of variables that this Applier assumes are bound.
    ///
    /// `egg` will check that the corresponding `Searcher` binds those
    /// variables.
    /// By default this return an empty `Vec`, which basically turns off the
    /// checking.
    fn vars(&self) -> Vec<Var> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use crate::{SymbolLang as S, *};
    use std::str::FromStr;
    use std::fmt::Formatter;

    type EGraph = crate::EGraph<S, ()>;

    #[test]
    fn conditional_rewrite() {
        crate::init_logger();
        let mut egraph = EGraph::default();

        let x = egraph.add(S::leaf("x"));
        let y = egraph.add(S::leaf("2"));
        let mul = egraph.add(S::new("*", vec![x, y]));

        let true_id = egraph.add(S::leaf("TRUE"));

        let mul_to_shift = multi_rewrite!(
            "mul_to_shift";
            "?x = (* ?a ?b), ?y = (is-power2 ?b), ?y = TRUE" => "?x =(>> ?a (log2 ?b))"
        );

        println!("rewrite shouldn't do anything yet");
        egraph.rebuild();
        let apps = mul_to_shift.run(&mut egraph);
        assert!(apps.is_empty());

        println!("Add the needed equality");
        let two_ispow2 = egraph.add(S::new("is-power2", vec![y]));
        egraph.union(two_ispow2, true_id);

        println!("Should fire now");
        egraph.rebuild();
        let apps = mul_to_shift.run(&mut egraph);
        // Can't check ids because multipattern
        assert!(apps.len() > 0);
    }

    #[test]
    fn fn_rewrite() {
        crate::init_logger();
        let mut egraph = EGraph::default();

        let start = RecExpr::from_str("(+ x y)").unwrap();
        let goal = RecExpr::from_str("xy").unwrap();

        let root = egraph.add_expr(&start);

        fn get(egraph: &EGraph, id: Id) -> Symbol {
            egraph[id].nodes[0].op
        }

        #[derive(Debug)]
        struct Appender;
        impl std::fmt::Display for Appender {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        impl Applier<SymbolLang, ()> for Appender {
            fn apply_one(&self, egraph: &mut EGraph, _eclass: Id, subst: &Subst) -> Vec<Id> {
                let a: Var = "?a".parse().unwrap();
                let b: Var = "?b".parse().unwrap();
                let a = get(&egraph, subst[a]);
                let b = get(&egraph, subst[b]);
                let s = format!("{}{}", a, b);
                vec![egraph.add(S::leaf(&s))]
            }
        }

        let fold_add = rewrite!(
            "fold_add"; "(+ ?a ?b)" => { Appender }
        );

        egraph.rebuild();
        fold_add.run(&mut egraph);
        assert_eq!(egraph.equivs(&start, &goal), vec![egraph.find(root)]);
    }
}
