use super::Signature;

/// A symbol for an unspecified term. Only carries meaning alongside a [`Signature`].
///
/// To construct a `Variable`, use [`Signature::new_var`]
///
/// [`Signature`]: struct.Signature.html
/// [`Signature::new_var`]: struct.Signature.html#method.new_var
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
    pub(crate) sig: Signature,
    pub(crate) id: usize,
}
impl Variable {
    /// Returns a `Variable`'s name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::Signature;
    /// let mut sig = Signature::default();
    /// let var = sig.new_var(Some("z".to_string()));
    ///
    /// assert_eq!(var.name(), Some("z".to_string()));
    /// ```
    pub fn name(&self) -> Option<String> {
        self.sig.sig.read().expect("poisoned signature").variables[self.id].clone()
    }
    /// Serialize a `Variable`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::Signature;
    /// let mut sig = Signature::default();
    /// let var = sig.new_var(Some("z".to_string()));
    ///
    /// assert_eq!(var.display(), "z_");
    /// ```
    pub fn display(&self) -> String {
        if let Some(ref name) = self.sig.sig.read().expect("poisoned signature").variables[self.id]
        {
            format!("{}_", name)
        } else {
            format!("var{}_", self.id)
        }
    }
}

/// A symbol with fixed arity. Only carries meaning alongside a [`Signature`].
///
/// To construct an `Operator`, use [`Signature::new_op`].
///
/// [`Signature`]: struct.Signature.html
/// [`Signature::new_op`]: struct.Signature.html#method.new_op
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Operator {
    pub(crate) sig: Signature,
    pub(crate) id: usize,
}
impl Operator {
    /// Returns an `Operator`'s arity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::Signature;
    /// let mut sig = Signature::default();
    /// let op = sig.new_op(2, Some("Z".to_string()));
    ///
    /// assert_eq!(op.arity(), 2);
    /// ```
    pub fn arity(&self) -> u32 {
        self.sig.sig.read().expect("poisoned signature").operators[self.id].0
    }
    /// Returns an `Operator`'s name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::Signature;
    /// let mut sig = Signature::default();
    /// let op = sig.new_op(2, Some("Z".to_string()));
    ///
    /// assert_eq!(op.name(), Some("Z".to_string()));
    /// ```
    pub fn name(&self) -> Option<String> {
        self.sig.sig.read().expect("poisoned signature").operators[self.id]
            .1
            .clone()
    }
    /// Serialize an `Operator`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::Signature;
    /// let mut sig = Signature::default();
    /// let op = sig.new_op(2, Some("Z".to_string()));
    ///
    /// assert_eq!(op.display(), "Z");
    /// ```
    pub fn display(&self) -> String {
        if let (_, Some(ref name)) =
            self.sig.sig.read().expect("poisoned signature").operators[self.id]
        {
            name.clone()
        } else {
            format!("op{}", self.id)
        }
    }
}

/// `Atom`s are the parts of a [`TRS`] that are not constructed from smaller parts: [`Variable`]s and [`Operator`]s.
///
/// [`TRS`]: struct.TRS.html
/// [`Variable`]: struct.Variable.html
/// [`Operator`]: struct.Operator.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    /// The [`Variable`] variant of an `Atom`.
    ///
    /// [`Variable`]: struct.Variable.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Atom};
    /// let mut sig = Signature::default();
    /// let x = sig.new_var(Some("x".to_string()));
    /// let atom = Atom::Variable(x);
    ///
    /// assert_eq!(atom.display(), "x_");
    /// ```
    Variable(Variable),
    /// The [`Operator`] variant of an `Atom`.
    ///
    /// [`Operator`]: struct.Operator.html
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Atom};
    /// let mut sig = Signature::default();
    /// let a = sig.new_op(0, Some("A".to_string()));
    /// let atom = Atom::Operator(a);
    ///
    /// assert_eq!(atom.display(), "A");
    /// ```
    Operator(Operator),
}
impl Atom {
    /// Serialize an `Atom`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use term_rewriting::{Signature, Atom};
    /// let mut sig = Signature::default();
    ///
    /// let a = sig.new_op(0, Some("A".to_string()));
    /// let atom = Atom::Operator(a);
    ///
    /// assert_eq!(atom.display(), "A");
    ///
    /// let x = sig.new_var(Some("x".to_string()));
    /// let atom = Atom::Variable(x);
    ///
    /// assert_eq!(atom.display(), "x_");
    /// ```
    pub fn display(&self) -> String {
        match *self {
            Atom::Variable(ref v) => v.display(),
            Atom::Operator(ref o) => o.display(),
        }
    }
}
impl From<Variable> for Atom {
    fn from(var: Variable) -> Atom {
        Atom::Variable(var)
    }
}
impl From<Operator> for Atom {
    fn from(op: Operator) -> Atom {
        Atom::Operator(op)
    }
}
