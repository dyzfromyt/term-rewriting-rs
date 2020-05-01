use super::{Atom, Operator, Rewrites, Rule, Signature, Term, Variable};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;
use std::fmt;

/// A first-order term rewriting system.
///
/// # Examples
///
/// ```
/// # use term_rewriting::{Signature, Rule, parse_rule, TRS, parse_trs};
/// let mut sig = Signature::default();
///
/// // Construct a TRS manually.
/// let r0 = parse_rule(&mut sig, "A(v0_) = A(B)").expect("parsed rule");
/// let r1 = parse_rule(&mut sig, "B = C | D").expect("parsed rule");
/// let r2 = parse_rule(&mut sig, "E(F) = G").expect("parsed rule");
///
/// let t = TRS::new(vec![r0, r1, r2]);
///
/// // Or, parse an entire TRS.
/// let t = parse_trs(
///     &mut sig,
///     "A(v0_) = A(B);
///      B = C | D;
///      E(F) = G;").expect("parsed TRS");
/// ```
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct TRS {
    pub(crate) is_deterministic: bool,
    pub rules: Vec<Rule>,
}
impl TRS {
    /// Constructs a [`Term Rewriting System`] from a list of [`Rule`]s.
    ///
    /// [`Rule`]: struct.Rule.html
    /// [`Term Rewriting System`]: https://en.wikipedia.ord/wiki/Rewriting#Term_rewriting_systems
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Rule, parse_trs, TRS};
    /// let mut sig = Signature::default();
    /// let trs = parse_trs(&mut sig, "A = B; C(v0_) = v0_; D(v1_) = D(E);").expect("parsed TRS");
    ///
    /// assert_eq!(trs.display(&sig), "A = B;\nC(v0_) = v0_;\nD(v1_) = D(E);");
    /// ```
    pub fn new(rules: Vec<Rule>) -> TRS {
        let mut trs = TRS {
            rules: vec![],
            is_deterministic: false,
        };
        trs.pushes(rules).ok();
        trs
    }
    /// Make the `TRS` [`deterministic`] and restrict it to be so until further notice.
    ///
    /// Return `true` if the `TRS` was changed, otherwise `false`.
    ///
    /// [`deterministic`]: http://en.wikipedia.org/wiki/Deterministic_system
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rand;
    /// # extern crate term_rewriting;
    /// # fn main(){
    /// # use term_rewriting::{Signature, Rule, parse_rule, TRS, parse_trs};
    /// # use rand::{thread_rng,Rng};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B | C;
    /// D = E;").expect("parse of A = B | C; D = E");
    /// let mut r = rand::thread_rng();
    ///
    /// let str_before = t.display(&sig);
    ///
    /// assert!(t.make_deterministic());
    ///
    /// assert_ne!(t.display(&sig), str_before);
    ///
    /// let str_before = t.display(&sig);
    /// let r = parse_rule(&mut sig, "C = B | D").expect("parse of C = B | D");
    ///
    /// if t.insert_idx(1, r.clone()).is_err() {
    ///     assert!(true);
    /// }
    ///
    /// assert_eq!(str_before, t.display(&sig));
    ///
    /// assert!((t.display(&sig) ==
    /// "A = B;
    /// D = E;") ||
    ///     (t.display(&sig) ==
    /// "A = C;
    /// D = E;"));
    /// # }
    /// ```
    pub fn make_deterministic(&mut self) -> bool {
        if !self.is_deterministic {
            for rule in self.rules.iter_mut() {
                rule.rhs.truncate(1);
            }
            self.is_deterministic = true;
            true
        } else {
            false
        }
    }
    /// Remove any [`determinism`] restriction the `TRS` might be under.
    ///
    /// Return `true` if the `TRS` was changed, otherwise `false`.
    ///
    /// See [`determinism`] for more information.
    ///
    /// [`Deterministic System`]: http://en.wikipedia.org/wiki/Deterministic_system
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rand;
    /// # extern crate term_rewriting;
    /// # fn main(){
    /// # use term_rewriting::{Signature, Rule, parse_rule, TRS, parse_trs, TRSError};
    /// # use rand::{thread_rng,Rng};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B | C;
    /// D = E;").expect("parse of A = B | C; D = E");
    /// let mut r = rand::thread_rng();
    ///
    /// t.make_deterministic();
    ///
    /// let str_before = t.display(&sig);
    /// let r = parse_rule(&mut sig, "C = B | D").expect("parse of C = B | D");
    /// assert!(t.insert_idx(1, r.clone()).is_err());
    /// assert_eq!(str_before, t.display(&sig));
    ///
    /// assert!(t.make_nondeterministic());
    ///
    /// t.insert_idx(1, r).expect("inserting C = B | D");
    ///
    /// assert!((t.display(&sig) ==
    /// "A = B;
    /// C = B | D;
    /// D = E;") ||
    ///     (t.display(&sig) ==
    /// "A = C;
    /// C = B | D;
    /// D = E;"));
    /// # }
    /// ```
    pub fn make_nondeterministic(&mut self) -> bool {
        let previous_state = self.is_deterministic;
        self.is_deterministic = false;
        previous_state
    }
    /// Report whether the `TRS` is currently deterministic.
    ///
    /// See [`Deterministic System`] for more information.
    ///
    /// [`Deterministic System`]: http://en.wikipedia.org/wiki/Deterministic_system
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate rand;
    /// # extern crate term_rewriting;
    /// # fn main(){
    /// # use term_rewriting::{Signature, Rule, parse_rule, TRS, parse_trs};
    /// # use rand::{thread_rng,Rng};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B | C;
    /// D = E;").expect("parse of A = B | C; D = E");
    /// let mut r = rand::thread_rng();
    ///
    /// assert!(!t.is_deterministic());
    ///
    /// t.make_deterministic();
    ///
    /// assert!(t.is_deterministic());
    /// # }
    /// ```
    pub fn is_deterministic(&self) -> bool {
        self.is_deterministic
    }
    /// The number of [`Rule`]s in the `TRS`.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// assert_eq!(t.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.rules.len()
    }
    /// Are there any [`Rule`]s in the `TRS`?
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// assert!(!t.is_empty());
    ///
    /// let t = parse_trs(&mut sig, "").expect("parse of blank string");
    ///
    /// assert!(t.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
    /// Return the number of total number of subterms across all [`Rule`]s in the `TRS`.
    ///
    /// See [`Term`] for more information.
    ///
    /// [`Term`]: struct.Term.html
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// assert_eq!(t.size(), 8);
    /// ```
    pub fn size(&self) -> usize {
        self.rules.iter().map(Rule::size).sum()
    }
    /// Serialize a `TRS`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    ///     "A = B;
    ///      C = D | E;
    ///      F(v0_) = G;").expect("parsed TRS");
    ///
    /// assert_eq!(t.display(&sig), "A = B;\nC = D | E;\nF(v0_) = G;");
    ///
    /// let trs = parse_trs(&mut sig,
    /// "A(v1_ v2_ v3_) = A(v1_ DECC(DECC(DIGIT(1) 0) 5) SUCC(SUCC(ZERO)));
    /// CONS(B CONS(C CONS(D NIL))) = CONS(C CONS(D NIL));
    /// B C D E = B C | D E;")
    ///     .expect("parsed trs");
    ///
    /// assert_eq!(trs.display(&sig),
    /// "A(v1_ v2_ v3_) = A(v1_ DECC(DECC(DIGIT(1) 0) 5) SUCC(SUCC(ZERO)));
    /// CONS(B CONS(C CONS(D NIL))) = CONS(C CONS(D NIL));
    /// .(.(.(B C) D) E) = .(B C) | .(D E);");
    /// ```
    pub fn display(&self, sig: &Signature) -> String {
        self.rules
            .iter()
            .map(|r| format!("{};", r.display(sig)))
            .join("\n")
    }
    /// A human-readable serialization of the `TRS`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, parse_trs};
    /// let mut sig = Signature::default();
    ///
    /// let trs = parse_trs(&mut sig,
    /// "A(v0_ v1_ v2_) = A(v0_ DECC(DECC(DIGIT(1) 0) 5) SUCC(SUCC(ZERO)));
    /// CONS(B CONS(C CONS(D NIL))) = CONS(C CONS(D NIL));
    /// B C D E = B C | D E;")
    ///     .expect("parsed TRS");
    ///
    /// assert_eq!(trs.pretty(&sig), "A(v0_, v1_, v2_) = A(v0_, 105, 2);\n[B, C, D] = [C, D];\nB C D E = B C | D E;");
    /// ```
    pub fn pretty(&self, sig: &Signature) -> String {
        self.rules
            .iter()
            .map(|r| format!("{};", r.pretty(sig)))
            .join("\n")
    }
    /// All the clauses in the `TRS`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F = G;").expect("parse of A = B; C = D | E; F = G;");
    ///
    /// let r0 = parse_rule(&mut sig, "A = B").expect("parse of A = B");
    /// let r1 = parse_rule(&mut sig, "C = D").expect("parse of C = D");
    /// let r2 = parse_rule(&mut sig, "C = E").expect("parse of C = E");
    /// let r3 = parse_rule(&mut sig, "F = G").expect("parse of F = G");
    ///
    /// assert_eq!(t.clauses(), vec![r0, r1, r2, r3]);
    /// ```
    pub fn clauses(&self) -> Vec<Rule> {
        self.rules.iter().flat_map(Rule::clauses).collect()
    }
    /// All the [`Operator`]s in the `TRS`.
    ///
    /// [`Operator`]: struct.Operator.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// let ops: Vec<String> = t.operators().iter().map(|o| o.display(&sig)).collect();
    ///
    /// assert_eq!(ops, vec!["A", "B", "C", "D", "E", "F", "G"]);
    /// ```
    pub fn operators(&self) -> Vec<Operator> {
        self.rules
            .iter()
            .flat_map(Rule::operators)
            .unique()
            .collect()
    }
    pub fn canonicalize(&mut self, map: &mut HashMap<usize, usize>) {
        self.rules
            .iter_mut()
            .for_each(|rule| rule.canonicalize(map));
    }
    /// Do two TRSs [`unify`]?
    ///
    /// [`unify`]: https://en.wikipedia.org/wiki/Unification_(computer_science)
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs};
    /// let mut sig = Signature::default();
    ///
    /// let t0 = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// let t1 = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(H) = G;").expect("parse of A = B; C = D | E; F(H) = G;");
    ///
    /// assert!(TRS::unifies(t0.clone(), t1));
    ///
    /// let t2 = parse_trs(&mut sig,
    /// "B = A;
    /// C = D | E;
    /// F(y_) = G;").expect("parse of A = B; C = D | E; F(y_) = G;");
    ///
    /// assert!(!TRS::unifies(t0, t2));
    /// ```
    pub fn unifies(trs1: TRS, trs2: TRS) -> bool {
        trs1.len() == trs2.len()
            && trs1
                .rules
                .iter()
                .zip(trs2.rules.iter())
                .all(|(r1, r2)| Rule::unify(r1, r2).is_some())
    }
    /// Does one TRS [`match`] another?
    ///
    /// See [`match`] for more information.
    ///
    /// [`Pattern Matching`]: https://en.wikipedia.org/wiki/Pattern_matching
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs};
    /// let mut sig = Signature::default();
    ///
    /// let t0 = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// let t1 = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(H) = G;").expect("parse of A = B; C = D | E; F(H) = G;");
    ///
    /// assert!(TRS::pmatches(t0.clone(), t1));
    ///
    /// let t2 = parse_trs(&mut sig,
    /// "B = A;
    /// C = D | E;
    /// F(y_) = G;").expect("parse of A = B; C = D | E; F(y_) = G;");
    ///
    /// assert!(!TRS::pmatches(t0.clone(), t2));
    ///
    /// let t3 = parse_trs(&mut sig,
    /// "A = B | C;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B | C; C = D | E; F(x_) = G");
    ///
    /// assert!(TRS::pmatches(t0.clone(), t3));
    ///
    /// let t4 = parse_trs(&mut sig,
    /// "A = B;
    /// C = D;
    /// D = E;").expect("parse of A = B; C = D; D = E;");
    ///
    /// assert!(!TRS::pmatches(t0, t4));
    /// ```
    pub fn pmatches(trs1: TRS, trs2: TRS) -> bool {
        trs1.len() == trs2.len()
            && trs1
                .rules
                .iter()
                .zip(trs2.rules.iter())
                .all(|(r1, r2)| Rule::pmatch(r1, r2).is_some())
    }
    /// Are two TRSs [`Alpha Equivalent`]?
    ///
    /// [`Alpha Equivalent`]: https://en.wikipedia.org/wiki/lambda_calculus#Alpha_equivalence
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs};
    /// let mut sig = Signature::default();
    ///
    /// let t0 = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// let t1 = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(H) = G;").expect("parse of A = B; C = D | E; F(H) = G;");
    ///
    /// assert!(!TRS::alphas(&t0, &t1));
    ///
    /// let t2 = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(y_) = G;").expect("parse of A = B; C = D | E; F(y_) = G;");
    ///
    /// assert!(TRS::alphas(&t0, &t2));
    /// ```
    pub fn alphas(trs1: &TRS, trs2: &TRS) -> bool {
        trs1.len() == trs2.len()
            && trs1
                .rules
                .iter()
                .zip(&trs2.rules[..])
                .all(|(r1, r2)| Rule::alpha(r1, r2).is_some())
    }
    /// Do two TRSs share the same shape?
    pub fn same_shape(t1: &TRS, t2: &TRS) -> bool {
        let mut omap = HashMap::new();
        let mut vmap = HashMap::new();
        TRS::same_shape_given(t1, t2, &mut omap, &mut vmap)
    }
    /// Do two rules share the same shape given some initial constraints?
    pub fn same_shape_given(
        t1: &TRS,
        t2: &TRS,
        ops: &mut HashMap<Operator, Operator>,
        vars: &mut HashMap<Variable, Variable>,
    ) -> bool {
        t1.len() == t2.len()
            && t1
                .rules
                .iter()
                .zip(&t2.rules)
                .all(|(r1, r2)| Rule::same_shape_given(r1, r2, ops, vars))
    }
    // Return rewrites modifying the entire term, if possible, else None.
    fn rewrite_head(&self, term: &Term) -> Option<Vec<Term>> {
        for rule in &self.rules {
            if let Some(ref sub) = Term::pmatch(&[(&rule.lhs, &term)]) {
                let items = rule.rhs.iter().map(|x| x.substitute(sub)).collect_vec();
                if !items.is_empty() {
                    return Some(items);
                }
            }
        }
        None
    }
    // Return rewrites modifying subterms, if possible, else None.
    fn rewrite_args(&self, term: &Term, strategy: Strategy, sig: &Signature) -> Option<Vec<Term>> {
        if let Term::Application { op, ref args } = *term {
            for (i, arg) in args.iter().enumerate() {
                let mut it = self.rewrite(arg, strategy, sig).peekable();
                if let Some(_) = it.peek() {
                    let res = it
                        .map(|x| {
                            let mut args = args.clone();
                            args[i] = x.clone();
                            Term::Application { op, args }
                        })
                        .collect();
                    return Some(res);
                }
            }
        }
        None
    }
    // performs all possible rewrites, else None.
    fn rewrite_all(&self, term: &Term) -> Option<Vec<Term>> {
        match term {
            Term::Variable(_) => None,
            Term::Application { ref args, .. } => {
                // rewrite head
                let mut rewrites = self.rewrite_head(term).unwrap_or_else(|| vec![]);
                // rewrite subterms
                for (i, arg) in args.iter().enumerate() {
                    for rewrite in self.rewrite_all(arg).unwrap_or_else(|| vec![]) {
                        rewrites.push(term.replace(&[i], rewrite).unwrap());
                    }
                }
                Some(rewrites)
            }
        }
    }
    // performs all possible rewrites, interpreting the term as a string
    fn rewrite_as_string(&self, term: &Term, sig: &Signature) -> Option<Vec<Term>> {
        let string = TRS::convert_term_to_string(term, sig)?;
        let mut rewrites = vec![];
        for rule in &self.rules {
            let pattern = TRS::convert_rule_to_strings(rule, sig)?;
            for breaks in TRS::gen_breaks(&pattern.0, string.len())?.iter() {
                if let Some(matches) = TRS::match_pattern(&pattern.0, &breaks[..], &string) {
                    for rhs in &pattern.1 {
                        let new_string = TRS::substitute_pattern(&rhs[..], &matches)?;
                        let new_term = TRS::convert_to_term(&new_string, sig)?;
                        rewrites.push(new_term)
                    }
                }
            }
        }
        Some(rewrites)
    }
    pub fn convert_list_to_string(term: &Term, sig: &mut Signature) -> Option<Vec<Atom>> {
        if term.as_guarded_application(sig, "NIL", 0).is_some() {
            Some(vec![])
        } else {
            let (_, args) = term.as_guarded_application(sig, ".", 2)?;
            let (_, inner_args) = args[0].as_guarded_application(sig, ".", 2)?;
            inner_args[0].as_guarded_application(sig, "CONS", 0)?;
            let mut string = vec![TRS::num_to_atom(&inner_args[1], sig)?];
            string.append(&mut TRS::convert_list_to_string(&args[1], sig)?);
            Some(string)
        }
    }
    fn digit_to_usize(term: &Term, sig: &Signature) -> Option<usize> {
        let (op, _) = term.as_application()?;
        match (op.name(sig), op.arity(sig)) {
            (Some(s), 0) if s == "0" => Some(0),
            (Some(s), 0) if s == "1" => Some(1),
            (Some(s), 0) if s == "2" => Some(2),
            (Some(s), 0) if s == "3" => Some(3),
            (Some(s), 0) if s == "4" => Some(4),
            (Some(s), 0) if s == "5" => Some(5),
            (Some(s), 0) if s == "6" => Some(6),
            (Some(s), 0) if s == "7" => Some(7),
            (Some(s), 0) if s == "8" => Some(8),
            (Some(s), 0) if s == "9" => Some(9),
            _ => None,
        }
    }
    fn num_to_usize(term: &Term, sig: &Signature) -> Option<usize> {
        let (_, args) = term.as_guarded_application(sig, ".", 2)?;
        if args[0].as_guarded_application(sig, "DIGIT", 0).is_some() {
            TRS::digit_to_usize(&args[1], sig)
        } else {
            let (_, inner_args) = args[0].as_guarded_application(sig, ".", 2)?;
            inner_args[0].as_guarded_application(sig, "DECC", 0)?;
            let sigs = TRS::num_to_usize(&inner_args[1], sig)?;
            let digit = TRS::digit_to_usize(&args[1], sig)?;
            Some(sigs * 10 + digit)
        }
    }
    fn num_to_atom(term: &Term, sig: &mut Signature) -> Option<Atom> {
        let n = TRS::num_to_usize(term, sig)?;
        if n < 100 {
            sig.operators()
                .iter()
                .find(|op| op.name(sig) == Some(&n.to_string()) && op.arity(sig) == 0)
                .map(|&op| Atom::from(op))
                .or_else(|| Some(Atom::from(sig.new_op(0, Some(n.to_string())))))
        } else {
            None
        }
    }
    fn convert_term_to_string(term: &Term, sig: &Signature) -> Option<Vec<Atom>> {
        match *term {
            Term::Variable(v) => Some(vec![Atom::Variable(v)]),
            Term::Application { op, ref args } => match (op.name(sig), op.arity(sig)) {
                (_, 0) => Some(vec![Atom::Operator(op)]),
                (Some(s), 2) if s == "." => {
                    let results = args
                        .iter()
                        .map(|a| TRS::convert_term_to_string(a, sig))
                        .collect_vec();
                    let mut string = vec![];
                    for result in results {
                        if let Some(mut chars) = result {
                            string.append(&mut chars);
                        } else {
                            return None;
                        }
                    }
                    Some(string)
                }
                _ => None,
            },
        }
    }
    fn convert_rule_to_strings(
        rule: &Rule,
        sig: &Signature,
    ) -> Option<(Vec<Atom>, Vec<Vec<Atom>>)> {
        let lhs = TRS::convert_term_to_string(&rule.lhs, sig)?;
        let rhs = rule
            .rhs
            .iter()
            .map(|r| TRS::convert_term_to_string(r, sig))
            .collect::<Option<Vec<_>>>()?;
        Some((lhs, rhs))
    }
    fn gen_breaks(pattern: &[Atom], n: usize) -> Option<Vec<Vec<usize>>> {
        let breaks = (0..=n)
            .combinations(pattern.len() - 1)
            .map(|mut x| {
                x.insert(0, 0);
                x.push(n);
                x
            })
            .filter(|x| TRS::valid_option(&pattern, &x))
            .collect_vec();
        Some(breaks)
    }
    fn valid_option(pattern: &[Atom], breaks: &[usize]) -> bool {
        for (i, atom) in pattern.iter().enumerate() {
            if let Atom::Operator(_) = atom {
                if breaks[i + 1] - breaks[i] != 1 {
                    return false;
                }
            }
        }
        true
    }
    fn match_pattern(
        pattern: &[Atom],
        breaks: &[usize],
        string: &[Atom],
    ) -> Option<HashMap<Variable, Vec<Atom>>> {
        let mut matches: HashMap<Variable, Vec<Atom>> = HashMap::new();

        for (i, &atom) in pattern.iter().enumerate() {
            match atom {
                Atom::Variable(v)
                    if matches.contains_key(&v)
                        && matches[&v] != string[breaks[i]..breaks[i + 1]].to_vec() =>
                {
                    return None
                }
                Atom::Operator(_) if string[breaks[i]..breaks[i + 1]] != [atom] => return None,
                _ => (),
            }

            if let Atom::Variable(v) = atom {
                matches
                    .entry(v)
                    .or_insert_with(|| string[breaks[i]..breaks[i + 1]].to_vec());
            }
        }
        Some(matches)
    }
    fn substitute_pattern(
        pattern: &[Atom],
        matches: &HashMap<Variable, Vec<Atom>>,
    ) -> Option<Vec<Atom>> {
        let mut string = vec![];
        for &atom in pattern.iter() {
            match atom {
                Atom::Variable(v) if matches.contains_key(&v) => {
                    string.append(&mut matches[&v].clone())
                }
                Atom::Operator(_) => string.push(atom),
                _ => return None,
            }
        }
        Some(string)
    }
    fn convert_to_term(string: &[Atom], sig: &Signature) -> Option<Term> {
        if string.is_empty() {
            return None;
        }
        let (mut term, bin_op_op) = match string[0] {
            Atom::Variable(v) => (
                Term::Variable(v),
                sig.operators()
                    .into_iter()
                    .find(|x| x.arity(sig) == 2 && x.name(sig) == Some(".")),
            ),
            Atom::Operator(op) => (
                Term::Application { op, args: vec![] },
                sig.operators()
                    .into_iter()
                    .find(|x| x.arity(sig) == 2 && x.name(sig) == Some(".")),
            ),
        };
        if let Some(bin_op) = bin_op_op {
            for character in string[1..].iter() {
                let subterm = match *character {
                    Atom::Variable(v) => Term::Variable(v),
                    Atom::Operator(op) => Term::Application { op, args: vec![] },
                };
                term = Term::Application {
                    op: bin_op,
                    args: vec![term, subterm],
                }
            }
            Some(term)
        } else {
            None
        }
    }
    /// madness: `p_string` treats two `Term`s as strings and computes a
    /// probabilistic edit distance between them.
    pub fn p_string(
        x: &Term,
        y: &Term,
        dist: PStringDist,
        t_max: usize,
        d_max: usize,
        sig: &Signature,
    ) -> Option<f64> {
        let sig = sig.clone();
        let x_string = TRS::convert_term_to_string(x, &sig)?;
        let y_string = TRS::convert_term_to_string(y, &sig)?;
        let p = PString::new(x_string, y_string, dist, &sig).compute(t_max, d_max);
        Some(p.ln())
    }
    /// madness: `p_list` treats two list `Term`s as strings and computes a
    /// probabilistic edit distance between them.
    pub fn p_list(
        x: &Term,
        y: &Term,
        dist: PStringDist,
        t_max: usize,
        d_max: usize,
        sig: &Signature,
    ) -> f64 {
        let mut sig = sig.clone();
        let x_string = TRS::convert_list_to_string(x, &mut sig);
        let y_string = TRS::convert_list_to_string(y, &mut sig);
        match (x_string, y_string) {
            (Some(x_string), Some(y_string)) => {
                let p = PString::new(x_string, y_string, dist, &sig).compute(t_max, d_max);
                p.ln()
            }
            _ => std::f64::NEG_INFINITY,
        }
    }

    /// Perform a single rewrite step.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Strategy, TRS, parse_trs, Term, parse_term};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// let term = parse_term(&mut sig, "J(F(C) K(C A))").expect("parse of J(F(C) K(C A))");
    ///
    /// let rewritten_terms: Vec<_> = t.rewrite(&term, Strategy::Normal, &sig).collect();
    /// assert_eq!(rewritten_terms.len(), 1);
    /// assert_eq!(rewritten_terms[0].display(&sig), "J(G K(C A))");
    ///
    /// let rewritten_terms: Vec<_> = t.rewrite(&term, Strategy::Eager, &sig).collect();
    /// assert_eq!(rewritten_terms.len(), 2);
    /// assert_eq!(rewritten_terms[0].display(&sig), "J(F(D) K(C A))");
    /// assert_eq!(rewritten_terms[1].display(&sig), "J(F(E) K(C A))");
    ///
    /// let rewritten_terms: Vec<_> = t.rewrite(&term, Strategy::All, &sig).collect();
    /// assert_eq!(rewritten_terms.len(), 6);
    /// assert_eq!(rewritten_terms[0].display(&sig), "J(G K(C A))");
    /// assert_eq!(rewritten_terms[1].display(&sig), "J(F(D) K(C A))");
    /// assert_eq!(rewritten_terms[2].display(&sig), "J(F(E) K(C A))");
    /// assert_eq!(rewritten_terms[3].display(&sig), "J(F(C) K(D A))");
    /// assert_eq!(rewritten_terms[4].display(&sig), "J(F(C) K(E A))");
    /// assert_eq!(rewritten_terms[5].display(&sig), "J(F(C) K(C B))");
    /// ```
    pub fn rewrite<'a>(
        &'a self,
        term: &'a Term,
        strategy: Strategy,
        sig: &'a Signature,
    ) -> TRSRewrites<'a> {
        TRSRewrites::new(self, term, strategy, sig)
    }
    /// Query a `TRS` for a [`Rule`] based on its left-hand-side; return both
    /// the [`Rule`] and its index if possible
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// let a = parse_term(&mut sig, "A").expect("parse of A");
    ///
    /// assert_eq!(t.get(&a).unwrap().1.display(&sig), "A = B");
    ///
    /// let c = parse_term(&mut sig, "C").expect("parse of C");
    ///
    /// assert_eq!(t.get(&c).unwrap().1.display(&sig), "C = D | E");
    /// ```
    pub fn get(&self, lhs: &Term) -> Option<(usize, Rule)> {
        for (idx, rule) in self.rules.iter().enumerate() {
            if Term::alpha(&[(lhs, &rule.lhs)]).is_some() {
                return Some((idx, rule.clone()));
            }
        }
        None
    }
    /// Query a `TRS` for a [`Rule`] based on its index; return the [`Rule`] if
    /// possible.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(x_) = G;").expect("parse of A = B; C = D | E; F(x_) = G;");
    ///
    /// assert_eq!(t.get_idx(0).unwrap().display(&sig), "A = B");
    ///
    /// assert_eq!(t.get_idx(1).unwrap().display(&sig), "C = D | E");
    /// ```
    pub fn get_idx(&self, idx: usize) -> Option<Rule> {
        if self.rules.len() > idx {
            Some(self.rules[idx].clone())
        } else {
            None
        }
    }
    /// Query a `TRS` for specific [`Rule`] clauses; return them if possible.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let t = parse_trs(&mut sig,
    /// "A(v0_ v1_) = B(v0_ v0_);
    /// D(v2_ v3_) = D(E F);").expect("parsed TRS");
    ///
    /// let r = parse_rule(&mut sig, "A(v4_ v5_) = B(v4_ v4_)").expect("parsed rule");
    ///
    /// assert_eq!(t.get_clause(&r).unwrap().1.display(&sig), "A(v0_ v1_) = B(v0_ v0_)");
    ///
    /// let r = parse_rule(&mut sig, "D(E E) = D(E F)").expect("parsed rule");
    ///
    /// assert!(t.get_clause(&r).is_none());
    /// ```
    pub fn get_clause(&self, rule: &Rule) -> Option<(usize, Rule)> {
        for (i, r) in self.rules.iter().enumerate() {
            if let Some(sub) = r.contains(rule) {
                return Some((i, rule.substitute(&sub)));
            }
        }
        None
    }
    /// Query a `TRS` for a [`Rule`] based on its left-hand-side; delete and
    /// return the [`Rule`] if it exists.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(v0_) = G;").expect("parsed TRS");
    ///
    /// let a = parse_term(&mut sig, "A").expect("parse of A");
    /// let c = parse_term(&mut sig, "C").expect("parse of C");
    ///
    /// assert_eq!(t.remove(&a).expect("removed A = B").display(&sig), "A = B");
    /// assert_eq!(t.remove(&c).expect("removed C = D").display(&sig), "C = D | E");
    /// assert_eq!(t.display(&sig), "F(v0_) = G;");
    /// ```
    pub fn remove(&mut self, lhs: &Term) -> Result<Rule, TRSError> {
        if let Some((idx, _)) = self.get(lhs) {
            Ok(self.rules.remove(idx))
        } else {
            Err(TRSError::NotInTRS)
        }
    }
    /// Query a `TRS` for a [`Rule`] based on its index; delete and return the
    /// [`Rule`] if it exists.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(v0_) = G;").expect("parsed TRS");
    ///
    /// assert_eq!(t.remove_idx(0).expect("removing A = B").display(&sig), "A = B");
    /// assert_eq!(t.remove_idx(0).expect("removing C = D").display(&sig), "C = D | E");
    /// assert_eq!(t.display(&sig), "F(v0_) = G;");
    /// ```
    pub fn remove_idx(&mut self, idx: usize) -> Result<Rule, TRSError> {
        if self.rules.len() > idx {
            Ok(self.rules.remove(idx))
        } else {
            Err(TRSError::InvalidIndex(idx, self.rules.len()))
        }
    }
    /// Query a `TRS` for a [`Rule`] based on its left-hand-side; delete and
    /// return the [`Rule`] if it exists.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(v0_) = G;").expect("parsed TRS");
    ///
    /// let r = parse_rule(&mut sig, "C = D").expect("parsed rule");
    ///
    /// assert_eq!(t.remove_clauses(&r).expect("removing C = D").display(&sig), "C = D");
    /// assert_eq!(t.display(&sig), "A = B;\nC = E;\nF(v0_) = G;");
    /// ```
    pub fn remove_clauses(&mut self, rule: &Rule) -> Result<Rule, TRSError> {
        self.rules
            .iter_mut()
            .filter_map(|r| r.discard(&rule))
            .next()
            .ok_or(TRSError::NotInTRS)
            .and_then(|discarded| {
                self.rules.retain(|rule| !rule.is_empty());
                Ok(discarded)
            })
    }
    /// Try to merge a [`Rule`] with an existing [`Rule`] or else insert it at index `i` in the `TRS` if possible.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(v0_) = G;").expect("parsed TRS");
    ///
    /// let r = parse_rule(&mut sig, "D = G").expect("parsed rule");
    ///
    /// t.insert(1, r).expect("inserting");
    ///
    /// assert_eq!(t.display(&sig), "A = B;\nD = G;\nC = D | E;\nF(v0_) = G;");
    /// ```
    pub fn insert(&mut self, idx: usize, rule: Rule) -> Result<&mut TRS, TRSError> {
        if self.insert_clauses(&rule).is_err() {
            self.insert_idx(idx, rule)
        } else {
            Ok(self)
        }
    }
    /// Insert a [`Rule`] at index `i` in the `TRS` if possible.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(v0_) = G;").expect("parsed TRS");
    ///
    /// let r = parse_rule(&mut sig, "D = G").expect("parsed rule");
    ///
    /// t.insert_idx(1, r).expect("inserting");
    ///
    /// assert_eq!(t.display(&sig), "A = B;\nD = G;\nC = D | E;\nF(v0_) = G;");
    /// ```
    pub fn insert_idx(&mut self, idx: usize, rule: Rule) -> Result<&mut TRS, TRSError> {
        if self.is_deterministic && rule.len() > 1 {
            return Err(TRSError::NondeterministicRule);
        } else if idx > self.rules.len() {
            return Err(TRSError::InvalidIndex(idx, self.rules.len()));
        } else if self.get(&rule.lhs).is_some() {
            return Err(TRSError::AlreadyInTRS);
        }
        self.rules.insert(idx, rule);
        Ok(self)
    }
    /// Inserts a series of [`Rule`]s into the `TRS` at the index provided if possible.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig, "A = B; C = D | E; F(v0_) = H;").expect("parsed TRS");
    ///
    /// let r0 = parse_rule(&mut sig, "G(v1_) = v1_").expect("parsed r0");
    /// let r1 = parse_rule(&mut sig, "B = C").expect("parsed r1");
    /// let r2 = parse_rule(&mut sig, "E = F | B").expect("parsed r2");
    ///
    /// t.inserts_idx(2, vec![r0, r1, r2]).expect("inserting");
    ///
    /// assert_eq!(t.display(&sig), "A = B;\nC = D | E;\nG(v1_) = v1_;\nB = C;\nE = F | B;\nF(v0_) = H;");
    /// ```
    pub fn inserts_idx(&mut self, idx: usize, rules: Vec<Rule>) -> Result<&mut TRS, TRSError> {
        for rule in rules.into_iter().rev() {
            self.insert_idx(idx, rule)?;
        }
        Ok(self)
    }
    /// Merge a [`Rule`] with an existing [`Rule`] in the `TRS` if possible.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(v0_) = G;").expect("parsed TRS");
    ///
    /// let r = parse_rule(&mut sig, "A = H").expect("parsed rule");
    ///
    /// let t = t.insert_clauses(&r).expect("inserting");
    ///
    /// assert_eq!(t.display(&sig), "A = B | H;\nC = D | E;\nF(v0_) = G;");
    /// ```
    pub fn insert_clauses(&mut self, rule: &Rule) -> Result<&mut TRS, TRSError> {
        if self.is_deterministic {
            Err(TRSError::NondeterministicRule)
        } else if let Some((idx, _)) = self.get(&rule.lhs) {
            self.rules[idx].merge(rule);
            Ok(self)
        } else {
            Err(TRSError::NotInTRS)
        }
    }
    /// Insert new [`Rule`] clauses if possible and move the entire [`Rule`] if
    /// necessary to be the first in the `TRS`.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(v0_) = G;").expect("parsed TRS");
    ///
    /// let r = parse_rule(&mut sig, "G(v1_) = v1_").expect("parsed rule");
    ///
    /// t.push(r).expect("inserting");
    ///
    /// assert_eq!(t.display(&sig), "G(v1_) = v1_;\nA = B;\nC = D | E;\nF(v0_) = G;");
    /// ```
    pub fn push(&mut self, rule: Rule) -> Result<&mut TRS, TRSError> {
        let lhs = rule.lhs.clone();
        self.insert(0, rule)?
            .get(&lhs)
            .ok_or(TRSError::NotInTRS)
            .and_then(move |(idx, _)| self.move_rule(idx, 0))
    }
    /// Inserts a series of [`Rule`]s at the beginning of the `TRS` if possible.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(v0_) = H;").expect("parsed TRS");
    ///
    /// let r0 = parse_rule(&mut sig, "G(v1_) = v1_").expect("parsed r0");
    /// let r1 = parse_rule(&mut sig, "B = C").expect("parsed r1");
    /// let r2 = parse_rule(&mut sig, "E = F | B").expect("parsed r2");
    ///
    /// t.pushes(vec![r0, r1, r2]).expect("inserting");
    ///
    /// assert_eq!(t.display(&sig),
    /// "G(v1_) = v1_;
    /// B = C;
    /// E = F | B;
    /// A = B;
    /// C = D | E;
    /// F(v0_) = H;");
    /// ```
    pub fn pushes(&mut self, rules: Vec<Rule>) -> Result<&mut TRS, TRSError> {
        for rule in rules.into_iter().rev() {
            self.push(rule)?;
        }
        Ok(self)
    }
    /// Move a [`Rule`] from index `i` to `j` if possible.
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig, "A = B; C = D | E; F(v0_) = G; H = I;").expect("parsed TRS");
    ///
    /// t.move_rule(0, 2).expect("moving rule from index 0 to index 2");
    ///
    /// assert_eq!(t.display(&sig), "C = D | E;\nF(v0_) = G;\nA = B;\nH = I;");
    /// ```
    pub fn move_rule(&mut self, i: usize, j: usize) -> Result<&mut TRS, TRSError> {
        if i != j {
            let rule = self.remove_idx(i)?;
            self.insert(j, rule)
        } else {
            Ok(self)
        }
    }
    /// Remove some [`Rule`] clauses while also inserting others if possible.
    ///
    /// The index `i` is used only in the case that the new clauses cannot be
    /// added to an existing [`Rule`].
    ///
    /// [`Rule`]: struct.Rule.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, TRS, parse_trs, Term, parse_term, Rule, parse_rule};
    /// let mut sig = Signature::default();
    ///
    /// let mut t = parse_trs(&mut sig,
    /// "A = B;
    /// C = D | E;
    /// F(v0_) = G;").expect("parsed TRS");
    ///
    /// let r = parse_rule(&mut sig, "C = D").expect("parse of C = D");
    /// let r_new = parse_rule(&mut sig, "C = A").expect("parse of C = A");
    /// t.replace(0, &r, r_new).expect("replaceing C = D with C = A");
    ///
    /// assert_eq!(t.display(&sig),
    /// "A = B;
    /// C = E | A;
    /// F(v0_) = G;");
    /// ```
    pub fn replace(&mut self, idx: usize, rule1: &Rule, rule2: Rule) -> Result<&mut TRS, TRSError> {
        self.remove_clauses(rule1)?;
        self.insert(idx, rule2)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Strategy {
    /// Perform only the leftmost-innermost rewrite
    Normal,
    /// Perform only the leftmost-innermost rewrite
    Eager,
    /// Perform all possible rewrites
    All,
    /// Rewrite term as a string (i.e. leaves only)
    String,
}
impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Strategy::Normal => write!(f, "Normal"),
            Strategy::Eager => write!(f, "Eager"),
            Strategy::All => write!(f, "All"),
            Strategy::String => write!(f, "String"),
        }
    }
}

#[derive(Debug, Clone)]
/// The error type for [`TRS`] manipulations.
///
/// [`TRS`]: struct.TRS.html
pub enum TRSError {
    /// Returned when requesting to edit a rule that is not in the TRS.
    ///
    /// See [`TRS::get`] for more information.
    ///
    /// [`TRS::get`]: struct.TRS.html#method.get
    NotInTRS,
    /// Returned when attempting to insert a rule into a TRS that already exists.
    ///
    /// See [`TRS::insert`] for more information.
    ///
    /// [`TRS::insert`]: struct.TRS.html#method.insert
    AlreadyInTRS,
    /// Returned when attempting to insert a rule with multiple RHSs into a deterministic TRS.
    ///
    /// See [`TRS::insert`] and [`TRS::make_deterministic`] for more information.
    ///
    /// [`TRS::insert`]: struct.TRS.html#method.insert
    /// [`TRS::make_deterministic`]: struct.TRS.html#method.make_deterministic
    NondeterministicRule,
    /// Returned when requesting the rule at an index that is out of the range of indicies for the TRS.
    ///
    /// See [`TRS::get_idx`] for more information.
    ///
    /// [`TRS::get_idx`]: struct.TRS.html#method.get_idx
    InvalidIndex(usize, usize),
}
impl fmt::Display for TRSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TRSError::NotInTRS => write!(f, "query rule not in TRS"),
            TRSError::AlreadyInTRS => write!(f, "pre-existing rule with same LHS in TRS"),
            TRSError::NondeterministicRule => {
                write!(f, "proposed rule is nondeterministic in deterministic TRS")
            }
            TRSError::InvalidIndex(length, max_length) => {
                write!(f, "index {} greater than max index {}", length, max_length)
            }
        }
    }
}
impl ::std::error::Error for TRSError {
    fn description(&self) -> &'static str {
        "TRS error"
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
/// Note: p_deletion + p_correct_sub + sum(all values of p_incorrect_sub) must equal 1.0.
pub struct PStringDist {
    /// The probability of a single insertion, used to compute the probability of `t` insertions as `(1-beta)(beta^t)`.
    pub beta: f64,
    /// The probability of any given symbol being the particular symbol inserted.
    pub p_insertion: f64,
    /// The probability of deleting a character.
    pub p_deletion: f64,
    /// The probability of substituting correctly.
    pub p_correct_sub: f64,
    /// A distribution over incorrect substitutions.
    pub p_incorrect_sub: PStringIncorrect,
}
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum PStringIncorrect {
    Constant(f64),
    Bounded {
        low: usize,
        high: usize,
        weight: f64,
    },
}

struct PString<'a> {
    cache: HashMap<(usize, usize, usize), f64>,
    m: usize,
    n: usize,
    x: Vec<Atom>,
    y: Vec<Atom>,
    dist: PStringDist,
    sig: &'a Signature,
}
impl<'a> PString<'a> {
    fn new(x: Vec<Atom>, y: Vec<Atom>, dist: PStringDist, sig: &'a Signature) -> PString<'a> {
        let cache = HashMap::new();
        let m = x.len();
        let n = y.len();
        PString {
            cache,
            m,
            n,
            x,
            y,
            dist,
            sig,
        }
    }
    pub fn compute(&mut self, t_max: usize, d_max: usize) -> f64 {
        // How few insertions can you do? n-m
        let t_start = self.n.saturating_sub(self.m);
        // How many insertions can you do? d_max-(m-n)
        let t_end = t_max.min((d_max + self.n).saturating_sub(self.m));
        (t_start..=t_end)
            .filter_map(|t| {
                // Insertions can't be deleted, so t must be smaller than n
                if t > self.n || self.n > self.m + t {
                    None
                } else {
                    let d = self.m + t - self.n;
                    let s = self.n - t;
                    Some(self.rho_t(t) * self.query((t, d, s)) * self.normalizer(t))
                }
            })
            .sum()
    }
    /// The probability of t insertions = (1-beta)*(beta^t)
    fn rho_t(&self, t: usize) -> f64 {
        (1.0 - self.dist.beta) * self.dist.beta.powi(t as i32)
    }
    fn normalizer(&self, t: usize) -> f64 {
        // m!t!/(m+t)! = min(m,t)!/(\prod_{i = max(m,t)..m+t} i)
        if self.m == 0 && t == 0 {
            1.0
        } else {
            let min_mt = t.min(self.m);
            let max_mt = t.max(self.m);
            let numerator: f64 = (1..=min_mt).product::<usize>() as f64;
            let denominator: f64 = (max_mt + 1..=(self.m + t)).product::<usize>() as f64;
            numerator / denominator
        }
    }
    fn p_sub(&self, x: &Atom, y: &Atom) -> f64 {
        if x == y {
            self.dist.p_correct_sub
        } else {
            match self.dist.p_incorrect_sub {
                PStringIncorrect::Constant(p) => p,
                PStringIncorrect::Bounded { low, high, weight } => {
                    let n_x = x.display(self.sig).parse::<usize>(); // 75
                    let n_y = y.display(self.sig).parse::<usize>(); // 81
                    match (n_x, n_y) {
                        (Ok(n_x), Ok(n_y)) => {
                            let range = high + 1 - low; // 100
                            let d_xy = if n_x > n_y { n_x - n_y } else { n_y - n_x }; // 6
                            let peak = if n_x == low || n_x == high {
                                range
                            } else {
                                (high + 1 - n_x).max(n_x + 1 - low)
                            }; // 76
                            let mass_y = peak - d_xy; // 70
                            let z = (1..peak).sum::<usize>()
                                + (1..peak).rev().take(high - n_x).sum::<usize>();
                            weight * (mass_y as f64) / (z as f64)
                        }
                        _ => 0.0,
                    }
                }
            }
        }
    }
    fn query(&mut self, key: (usize, usize, usize)) -> f64 {
        if self.cache.contains_key(&key) {
            return self.cache[&key];
        }
        let new_val = match key {
            (0, 0, 0) => 1.0,
            (t, 0, 0) if t > 0 => self.query((t - 1, 0, 0)) * self.dist.p_insertion,
            (0, d, 0) if d > 0 => self.query((0, d - 1, 0)) * self.dist.p_deletion,
            (0, 0, s) if s > 0 => {
                self.query((0, 0, s - 1)) * self.p_sub(&self.x[s - 1], &self.y[s - 1])
            }
            (t, d, 0) if t > 0 && d > 0 => {
                self.query((t - 1, d, 0)) * self.dist.p_insertion
                    + self.query((t, d - 1, 0)) * self.dist.p_deletion
            }
            (t, 0, s) if t > 0 && s > 0 => {
                self.query((t - 1, 0, s)) * self.dist.p_insertion
                    + self.query((t, 0, s - 1)) * self.p_sub(&self.x[s - 1], &self.y[s + t - 1])
            }
            (0, d, s) if d > 0 && s > 0 => {
                self.query((0, d - 1, s)) * self.dist.p_deletion
                    + self.query((0, d, s - 1)) * self.p_sub(&self.x[s + d - 1], &self.y[s - 1])
            }
            (t, d, s) if t > 0 && d > 0 && s > 0 => {
                self.query((t - 1, d, s)) * self.dist.p_insertion
                    + self.query((t, d - 1, s)) * self.dist.p_deletion
                    + self.query((t, d, s - 1)) * self.p_sub(&self.x[s + d - 1], &self.y[s + t - 1])
            }
            _ => 0.0,
        };
        self.cache.insert(key, new_val);
        new_val
    }
}

pub struct Normal<'a> {
    rewrites: NormalKind<'a>,
}

enum NormalKind<'a> {
    None,
    Head(std::iter::Peekable<Rewrites<'a>>),
    Subterm(
        SmallVec<[(Operator, usize, &'a [Term]); 32]>,
        std::iter::Peekable<Rewrites<'a>>,
    ),
}

impl<'a> Normal<'a> {
    pub(crate) fn new(trs: &'a TRS, term: &'a Term) -> Normal<'a> {
        if let Term::Application { op, args } = term {
            // Try the head.
            for rule in &trs.rules {
                let mut it = rule.rewrite(term).peekable();
                if it.peek().is_some() {
                    return Normal {
                        rewrites: NormalKind::Head(it),
                    };
                }
            }
            // Try each arg.
            let mut stack: SmallVec<[(Operator, usize, &[Term]); 32]> =
                smallvec![(*op, 0, args.as_slice())];
            while let Some((op, arg, args)) = stack.pop() {
                match &args[arg] {
                    Term::Variable(_) => (),
                    Term::Application {
                        op: new_op,
                        args: new_args,
                    } => {
                        for rule in &trs.rules {
                            let mut it = rule.rewrite(&args[arg]).peekable();
                            if it.peek().is_some() {
                                stack.push((op, arg, args));
                                stack.reverse();
                                return Normal {
                                    rewrites: NormalKind::Subterm(stack, it),
                                };
                            }
                        }
                        if arg + 1 < args.len() {
                            stack.push((op, arg + 1, args));
                        }
                        if !new_args.is_empty() {
                            stack.push((*new_op, 0, new_args));
                        }
                    }
                }
            }
        }
        Normal {
            rewrites: NormalKind::None,
        }
    }
}

impl<'a> Iterator for Normal<'a> {
    type Item = Term;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.rewrites {
            NormalKind::None => None,
            NormalKind::Head(it) => it.next(),
            NormalKind::Subterm(stack, it) => it.next().map(|subterm| {
                stack.iter().fold(subterm, |subterm, &(op, arg, args)| {
                    let mut new_args = Vec::with_capacity(args.len());
                    new_args.extend_from_slice(&args[..arg]);
                    new_args.push(subterm);
                    new_args.extend_from_slice(&args[arg + 1..]);
                    Term::Application { op, args: new_args }
                })
            }),
        }
    }
}

pub struct TRSRewrites<'a>(TRSRewriteKind<'a>);

enum TRSRewriteKind<'a> {
    Normal(Normal<'a>),
    Eager(std::vec::IntoIter<Term>),
    All(std::vec::IntoIter<Term>),
    String(std::vec::IntoIter<Term>),
}

impl<'a> TRSRewrites<'a> {
    pub(crate) fn new(trs: &'a TRS, term: &'a Term, strategy: Strategy, sig: &Signature) -> Self {
        let kind = match strategy {
            Strategy::Normal => TRSRewriteKind::Normal(Normal::new(trs, term)),
            Strategy::Eager => TRSRewriteKind::Eager(
                trs.rewrite_args(term, strategy, sig)
                    .or_else(|| trs.rewrite_head(term))
                    .unwrap_or_else(|| vec![])
                    .into_iter(),
            ),
            Strategy::All => {
                TRSRewriteKind::All(trs.rewrite_all(term).unwrap_or_else(|| vec![]).into_iter())
            }
            Strategy::String => TRSRewriteKind::String(
                trs.rewrite_as_string(term, sig)
                    .unwrap_or_else(|| vec![])
                    .into_iter(),
            ),
        };
        TRSRewrites(kind)
    }
}

impl<'a> Iterator for TRSRewrites<'a> {
    type Item = Term;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            TRSRewriteKind::Normal(it) => it.next(),
            TRSRewriteKind::Eager(it) => it.next(),
            TRSRewriteKind::All(it) => it.next(),
            TRSRewriteKind::String(it) => it.next(),
        }
    }
}
